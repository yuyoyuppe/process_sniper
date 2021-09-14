# What is it?
__Problem__: Modern <s>spyware</s> software is smart enough to hog pc's cpu/hdd only when a user is idle. It can also circumvent some security measures for its telemetry processes.

__Solution__: process_sniper - lightweight utility which could be run in the background to kill those processes as they're spawned.
# How to use
## Build

```cmd
set RUSTFLAGS=-C target-cpu=native
cargo build --release
```
## Example `config.toml`
```toml
blacklisted_processes = ["software_reporter_tool"]
refresh_interval = 3
```


# Alternatives

- Windows security policies
- Firewall software
- etc.