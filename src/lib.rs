use std::env;
use std::fs::{self, read_to_string, File};
use std::io;
use std::process::Command;
use std::io::BufReader;
use std::io::BufRead;

use glob::glob;
use shared_functions::read;

use self::error::Error;

mod cpu;
mod distro;
mod environment;
mod error;
mod packages;
mod shared_functions;
mod terminal;
mod uptime;

/// Obtain the temp of the CPU in Celsius, only tested on rpi, outputs to a Result<String>
pub fn temp() -> Result<String, Error> {
    Ok(format!("{}", read_to_string("/sys/class/thermal/thermal_zone0/temp")?.trim().parse::<f64>().unwrap() / 1000.0))
}

/// Obtain CPU model, outputs to a Result<String>
pub fn cpu() -> Result<String, Error> {
    let file = File::open("/proc/cpuinfo")?;
    let model = read_to_string("/sys/firmware/devicetree/base/model");
    fn info(file: File, line: usize) -> Result<String, Error> {
        let info = cpu::get(file, line);
        Ok(cpu::format(info).trim().to_string().replace("\n", ""))
    }
    if let Ok(model) = model {
        match model.starts_with("Raspberry") {
            true => info(file, 1),
            false => info(file, 4),
        }
    } else if shared_functions::exit_code() != 1 {
        info(file, 1)
    } else {
        info(file, 4)
    }
}

/// Obtain name of device, outputs to a string
pub fn device() -> Result<String, Error> {
    if shared_functions::exit_code() != 1 {
        let output_product = std::process::Command::new("sh")
            .args(&["-c", "getprop ro.product.name"])
            .output()
            .expect("");
        let product = String::from_utf8_lossy(&output_product.stdout).trim().to_string();
        let output_model = std::process::Command::new("sh")
            .args(&["-c", "getprop ro.product.model"])
            .output()
            .expect("");
        let model = String::from_utf8_lossy(&output_model.stdout).trim().to_string();
        let output_device = std::process::Command::new("sh")
            .args(&["-c", "getprop ro.product.device"])
            .output()
            .expect("");
        let device = String::from_utf8_lossy(&output_device.stdout).trim().to_string();
        let full = [
            product, 
            " ".to_string(), 
            model, 
            " (".to_string(), 
            device, 
            ")".to_string()
        ].concat();
        Ok(full)
    } else {
        let model = read_to_string("/sys/devices/virtual/dmi/id/product_name")
            .or_else(|_| read_to_string("/sys/firmware/devicetree/base/model"))?;
        Ok(model.trim().replace("\n", ""))
    }
}

/// Obtain the distro name, outputs to a string
pub fn distro() -> Result<String, Error> {
    if shared_functions::exit_code() != 1 {
        let output_distro = std::process::Command::new("sh")
            .args(&["-c", "getprop ro.build.version.release"])
            .output()
            .expect("");
        let mut distro = String::from_utf8_lossy(&output_distro.stdout).trim().to_string();
        distro = ["Android ".to_string(), distro].concat();
        let output_flavor = std::process::Command::new("sh")
            .args(&["-c", "getprop ro.build.flavor"])
            .output()
            .expect("");
        let flavor = String::from_utf8_lossy(&output_flavor.stdout).trim().to_string();
        let full = [distro, " (".to_string(), flavor, ")".to_string()].concat();
        Ok(full)

    } else {
        let distro = distro::dist("/bedrock/etc/os-release")
            .or_else(|_| distro::dist("/etc/os-release"))
            .or_else(|_| distro::dist("/usr/lib/os-release"))?;
        Ok(distro)
    }
}

/// Obtains the name of the user's DE or WM, outputs to a string
pub fn environment() -> Result<String, Error> {
    let de = environment::de().unwrap();
    if de == "N/A" {
        Ok(environment::wm().unwrap())
    } else {
        Ok(de)
    }
}

/// Obtain the contents of the env variable specified as an arg, outputs to a string
pub fn env(var: &str) -> Option<String> {
    if shared_functions::exit_code() != 1 {
        if var == "USER" {
            let output_user = std::process::Command::new("sh")
                .args(&["-c", "whoami"])
                .output()
                .expect("");
            Some(String::from_utf8_lossy(&output_user.stdout).trim().to_string())
        } else {
            Some(
                env::var(var)
                .unwrap_or_else(
                    |_| format!("N/A (could not read ${}, are you sure it's set?)", var)
                    )
                )
        }
    } else {
        Some(
            env::var(var)
            .unwrap_or_else(
                |_| format!("N/A (could not read ${}, are you sure it's set?)", var)
                )
            )
    }
}


/// Obtain the name of the GPU, outputs to a string

fn open_pci_ids() -> Option<File> {
    [
        "/usr/share/hwdata/pci.ids",
        "/usr/share/misc/pci.ids",
        "/usr/share/pci.ids",
    ]
    .iter()
    .find_map(|path| File::open(path).ok())
}

fn lines_find(
    reader: &mut BufReader<File>,
    f: &dyn Fn(&Vec<u8>) -> bool,
) -> Result<Option<Vec<u8>>, Error> {
    let mut line = Vec::new();
    loop {
        line.clear();
        reader.read_until(b'\n', &mut line)?;
        if line.is_empty() {
            break;
        }
        if f(&line) {
            return Ok(Some(line));
        }
    }
    Ok(None)
}

pub fn gpu() -> Result<String, Error> {
    let mut vendor = std::fs::read("/sys/class/drm/card0/device/vendor")?;
    let mut device = std::fs::read("/sys/class/drm/card0/device/device")?;

    if let Some(file) = open_pci_ids() {
        let mut reader = BufReader::new(file);

        if let Some(line) = lines_find(&mut reader, &|line| line.starts_with(&vendor[2..6]))? {
            vendor = line[6..].to_vec();
        }

        if let Some(line) = lines_find(&mut reader, &|line| {
            !line[0] == b'\t' || device.get(2..6) == line.get(1..5)
        })? {
            if line[0] == b'\t' {
                device = line[7..].to_vec();
            }
        }
    }
    //remove newlines
    vendor.pop();
    device.pop();
    let mut result = vendor;
    result.push(b' ');
    result.append(&mut device);
    Ok(String::from_utf8_lossy(&result).to_string())
}

/// Obtain the hostname, outputs to a Result<String>
pub fn hostname() -> Result<String, Error> {
    if shared_functions::exit_code() != 1 {
        let output_hostname = std::process::Command::new("sh")
            .args(&["-c", "hostname"])
            .output()
            .expect("");
        Ok(String::from_utf8_lossy(&output_hostname.stdout).trim().to_string())
    } else {
        Ok(read_to_string("/etc/hostname")?.trim().to_string())
    }
}

/// Obtain the kernel version, outputs to a Result<String>
pub fn kernel() -> Result<String, Error> {
    Ok(read_to_string("/proc/sys/kernel/osrelease")?
        .trim()
        .to_string()
        .replace("\n", ""))
}

/// Obtain total memory in MBs, outputs to a Result<String>
pub fn memory() -> Result<String, Error> {
    const MEMTOTAL: &str = "MemTotal";
    const MEMINFO: &str = "/proc/meminfo";
    const ERROR_01: &str = "no MemTotal line found in /proc/meminfo!";
    const ERROR_02: &str = "No memoryinfo in MemTotal line!";
    const UNIT: [&str; 5] = ["kB", "MB", "GB", "TB", "PB"];
    const SEPARATOR_COLON: &str = ":";
    const EMPTY_STRING: &str = "";

    const DIVISOR_U64: u64 = 1024;
    const UNIT_MB: &str = "MB";

    pub trait ToIoResult<T> {
        fn to_io_result(self) -> io::Result<T>;
    }

    impl<T, E: ToString> ToIoResult<T> for Result<T, E> {
        fn to_io_result(self) -> io::Result<T> {
            match self {
                Ok(x) => Ok(x),
                Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
            }
        }
    }

    let meminfo = fs::read_to_string(MEMINFO)?;
    for line in meminfo.lines() {
        if line.starts_with(MEMTOTAL) {
            let mut rsplit = line.rsplit(SEPARATOR_COLON);
            let size = match rsplit.next() {
                Some(x) => x
                    .replace(UNIT[0], EMPTY_STRING)
                    .trim()
                    .parse::<u64>()
                    .to_io_result()?,
                None => Err(io::Error::new(io::ErrorKind::Other, ERROR_02))?,
            };
            return Ok(format!("{} {}", (size / DIVISOR_U64), UNIT_MB));
        }
    }
    Err(io::Error::new(io::ErrorKind::Other, ERROR_01))?
}

// Music info
/// Connects to mpd, and obtains music info in the format "artist - album (date) - title", outputs to a String
#[cfg(feature = "music_mpd")]
pub fn music() -> Result<String, Box<dyn std::error::Error>> {
    let mut c = mpd::Client::connect("127.0.0.1:6600")?;
    let song = c.currentsong().unwrap().unwrap();
    let na = "N/A".to_string();
    let tit = song.title.as_ref().unwrap();
    let art = song.tags.get("Artist").unwrap_or(&na);
    let alb = song.tags.get("Album").unwrap_or(&na);
    let dat = song.tags.get("Date").unwrap_or(&na);
    Ok(format!("{} - {} ({}) - {}", art, alb, dat, tit))
}

#[cfg(feature = "music_playerctl")]
/// Gets music info from `playerctl` in the format "artist - album - title", outputs to a String
pub fn music() -> Result<String, Box<dyn std::error::Error>> {
    let child = std::process::Command::new("playerctl")
        .args(&["metadata", "-f", "{{artist}} - {{album}} - {{title}}"])
        .output();
    let output;
    if child.is_ok() {
        output = String::from_utf8_lossy(&child.unwrap().stdout).to_string();
    } else {
        output = String::from("N/A (failed to collect output from `playerctl`)");
    }
    Ok(output)
}

/// If neither `music_mpd` nor `music_playerctl` is used.
#[cfg(not(feature = "music_mpd"))]
#[cfg(not(feature = "music_playerctl"))]
pub fn music() -> String {
    "N/A (music feature must be used to pull in the mpd dependency)".to_string()
}

/// Obtain list of packages based on what manager is given as an arg, outputs to a string
pub fn packages(manager: &str) -> Result<String, Error> {
    match manager {
        "apk" => {
            let output = Command::new("apk").arg("info").output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "apt" | "dpkg" => {
            let output = Command::new("dpkg").args(&["--get-selections"]).output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "dnf" => {
            let output = Command::new("dnf").args(&["list", "installed"]).output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "eopkg" => {
            let output = Command::new("eopkg").arg("list-installed").output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "flatpak" => {
            let output = Command::new("flatpak").args(&["list"]).output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "pacman" => {
            let output = Command::new("pacman").args(&["-Q", "-q"]).output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "pip" => {
            let output = Command::new("pip").arg("list").output()?;
            Ok(format!("{}", packages::count(output) - 2)) // -2 to deal with header lines in output
        }
        "portage" => {
            let content = read(File::open("/var/lib/portage/world").unwrap()).unwrap();
            let file_vector: Vec<&str> = content.split('\n').collect();

            let mut list: Vec<String> = Vec::new();
            for entry in glob("/var/db/pkg/*/*/").expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => list.push(path.display().to_string()),
                    Err(e) => println!("{:?}", e),
                }
            }

            Ok(format!(
                "{} (explicit), {} (total)",
                file_vector.iter().count() - 1,
                list.iter().count()
            ))
        }
        "rpm" => {
            let output = Command::new("rpm").args(&["-q", "-a"]).output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "xbps" => {
            let output = Command::new("xbps-query").arg("-l").output()?;
            Ok(format!("{}", packages::count(output)))
        }
        _ => Ok(format!(
            "N/A ({} is not supported, please file a bug to get it added!)",
            manager
        )),
    }
}

/// Obtain the name of the terminal being used, outputs to a Result<String>
pub fn terminal() -> Result<String, Error> {
    let id = std::process::id();
    let path = format!("/proc/{}/status", id);
    let process_id = terminal::ppid(File::open(path)?).trim().replace("\n", "");
    let process_name = terminal::name(process_id.clone()).trim().replace("\n", "");
    let info = terminal::info(process_name, process_id).unwrap();
    if info == "systemd" || info.is_empty() {
        Ok("N/A (could not determine the terminal, this could be an issue of using tmux)".to_string())
    } else {
        Ok(info)
    }
}

/// Obtains the current uptime of the system, outputs to a Result<String>
pub fn uptime() -> Result<String, Error> {
    let raw_uptime = read_to_string("/proc/uptime")?;
    let uptime_vec: Vec<&str> = raw_uptime.split('.').collect();
    let uptime = uptime_vec[0].parse::<i64>().unwrap();
    let (days, hours, minutes) = uptime::duration(uptime);
    Ok(format!("{} {} {}", days, hours, minutes).trim().to_string())
}
