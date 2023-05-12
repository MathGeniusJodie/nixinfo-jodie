fork of nixinfo designed to be as fast as possible, using lessons learned from making tuxfetch
this fork will eventually be used in the rust rewrite of tuxfetch

### todo
- gpu fetch that doesn't rely on grep and lspci (should be 0.3ms)
- apk package fetch that doesn't rely on the very slow apk executable (should be 1ms for ~700 packages)
- use cpuid instruction on x86 (basically instant)
- use uname syscall for kernel version (basically instant)
- dpkg package fetch that doesn't rely on the executable
- more resilient wm fetch
- more resilient resolution fetch (fallback to framebuffer)

#original decription

## nixinfo
A lib crate for gathering system info such as cpu, distro, environment, kernel, etc in Rust.

To use: `nixinfo = "0.3.2"` in your `Cargo.toml`.

## Currently supported

- CPU model and temperature (Celsius)
  + `nixinfo::cpu()` -> `Result<String>`
  + `nixinfo::temp()` -> `Result<String>`
- Device name
  + `nixinfo::device()` -> `Result<String>`
- Distro name
  + `nixinfo::distro()` -> `Result<String>`
- Environment (e.g. DE or WM)
  + `nixinfo::environment()` -> `Result<String>`
- env variables
  + `nixinfo::env("VARIABLE")` -> `Option<String>`
- GPU info (requires `lspci` and `grep` to be installed for now until I find a pure rust solution)
  + `nixinfo::gpu()` -> `Result<String>`
- Hostname
  + `nixinfo::hostname()` -> `Result<String>`
- Kernel
  + `nixinfo::kernel()` -> `Result<String>`
- Total memory in MBs
  + `nixinfo::memory()` -> `Result<String>`
- Music info
  + Features for this:
    * `music_mpd` for music info from mpd
    * `music_playerctl` for music info from an MPRIS supporting program via `playerctl`
    * Enable neither of the features to get an N/A message
  + `nixinfo::music()` -> `String`
- Package counts (managers supported are apk, apt, dnf, dpkg, eopkg, pacman, pip, portage, rpm, and xbps)
  + `nixinfo::packages("manager")` -> `Result<String>`
- Terminal being used (unless tmux is used, in which case N/A will be outputted because reasons)
  + `nixnfo::terminal()` -> `Result<String>`
- Uptime of device
  + `nixinfo::uptime()` -> `Result<String>`

## TODO

- Obtain used memory in addition to total memory
- Support *BSD
