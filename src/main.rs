#![cfg(windows)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use libc::{sighandler_t, signal, SIGABRT, SIGINT, SIGTERM};
use serde::Deserialize;
use std::{ptr, thread, time};
use sysinfo::{Process, ProcessExt, Signal, System, SystemExt};
use winapi::um::winuser::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_CLOSE, WM_QUIT,
};

#[derive(Deserialize)]
struct Config {
    #[serde(default = "default_processes")]
    blacklisted_processes: Vec<String>,
    #[serde(default = "default_interval")]
    refresh_interval: u64,
}

fn default_interval() -> u64 {
    3
}

fn default_processes() -> Vec<String> {
    vec!["software_reporter_tool".into()]
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "processes - {:#?}; refresh time - {}s",
            self.blacklisted_processes, self.refresh_interval
        ))
    }
}

fn kill(process: &Process) {
    if !process.kill(Signal::Kill) {
        println!("Couldn't kill {}", process.name());
    }
}

fn run_message_loop() -> ! {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        loop {
            if GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                if msg.message == WM_CLOSE || msg.message == WM_QUIT {
                    exit(0, 0);
                }
                DispatchMessageW(&msg);
            } else {
                exit(0, 0);
            }
        }
    }
}

extern "C" fn exit(_signum: i32, _subcode: i32) -> ! {
    println!("Bye!");
    std::process::exit(0);
}

fn register_signal_handlers() {
    for signum in [SIGABRT, SIGINT, SIGTERM] {
        unsafe {
            signal(signum, exit as sighandler_t);
        }
    }
}

fn load_config() -> Config {
    let file = std::fs::read_to_string("config.toml").unwrap_or_default();
    toml::from_str(&file).unwrap_or(Config {
        refresh_interval: default_interval(),
        blacklisted_processes: default_processes(),
    })
}

fn main() {
    register_signal_handlers();

    let config: Config = load_config();
    println!("Launched with {}", config);

    std::thread::spawn(move || {
        let mut sys = System::new_all();
        loop {
            config
                .blacklisted_processes
                .iter()
                .flat_map(|proc_name| sys.process_by_name(proc_name))
                .for_each(kill);
            thread::sleep(time::Duration::from_secs(config.refresh_interval));
            sys.refresh_processes();
        }
    });

    // Allows Windows to understand that we're SIGTERM-friendly.
    // TODO: looks like we might need to create an actual window https://github.com/pachi/rust_winapi_examples/blob/master/src/bin/02_window.rs#L67
    run_message_loop()
}
