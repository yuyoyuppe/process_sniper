#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use figment::{
    providers::{Format, Serialized, Toml},
    value::{Dict, Map},
    Error, Figment, Metadata, Profile, Provider,
};
use serde::{Deserialize, Serialize};
use std::{thread, time};
use sysinfo::{Process, ProcessExt, Signal, System, SystemExt};

#[derive(Serialize, Deserialize)]
struct Config {
    blacklisted_processes: Vec<String>,
    refresh_interval: u64,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "processes - {:#?}; refresh time - {}s",
            self.blacklisted_processes, self.refresh_interval
        ))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            blacklisted_processes: Vec::new(),
            refresh_interval: 3u64,
        }
    }
}

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::default()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        Serialized::defaults(Config::default()).data()
    }
}

fn kill(process: &Process) {
    if !process.kill(Signal::Kill) {
        println!("Couldn't kill {}", process.name());
    }
}

fn main() -> Result<()> {
    let config: Config = Figment::from(Config::default())
        .merge(Toml::file("config.toml"))
        .extract()?;
    println!("Launched with {}", config);

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
}
