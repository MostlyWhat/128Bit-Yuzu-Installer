//! main.rs
//!
//! The main entrypoint for the application. Orchestrates the building of the installation
//! framework, and opens necessary HTTP servers/frontends.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(unsafe_code)]
#![deny(missing_docs)]

extern crate wry;

extern crate futures;
extern crate hyper;
extern crate url;

extern crate number_prefix;
extern crate reqwest;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

extern crate regex;
extern crate semver;

extern crate dirs;
extern crate tar;
extern crate xz2;
extern crate zip;

extern crate fern;
#[macro_use]
extern crate log;

extern crate chrono;

extern crate clap;
#[cfg(windows)]
extern crate widestring;
#[cfg(windows)]
extern crate winapi;

#[cfg(not(windows))]
extern crate slug;
#[cfg(not(windows))]
extern crate sysinfo;

extern crate jsonwebtoken as jwt;

extern crate base64;

mod archives;
mod config;
mod frontend;
mod http;
mod installer;
mod logging;
mod native;
mod self_update;
mod sources;
mod tasks;

use installer::InstallerFramework;

use logging::LoggingErrors;
use std::path::PathBuf;

use clap::App;
use clap::Arg;

use config::BaseAttributes;
use std::fs;
use std::process::{exit, Command, Stdio};

const RAW_CONFIG: &str = include_str!(concat!(env!("OUT_DIR"), "/bootstrap.toml"));

fn main() {
    let config = BaseAttributes::from_toml_str(RAW_CONFIG).expect("Config file could not be read");

    logging::setup_logger(format!("{}_installer.log", config.name))
        .expect("Unable to setup logging!");

    // Parse CLI arguments
    let app_name = config.name.clone();

    let app_about = format!("An interactive installer for {}", app_name);
    let app = App::new(format!("{} installer", app_name))
        .version(env!("CARGO_PKG_VERSION"))
        .about(app_about.as_ref())
        .arg(
            Arg::with_name("launcher")
                .long("launcher")
                .value_name("TARGET")
                .help("Launches the specified executable after checking for updates")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("swap")
                .long("swap")
                .value_name("TARGET")
                .help("Internal usage - swaps around a new installer executable")
                .takes_value(true),
        );

    let reinterpret_app = app.clone(); // In case a reparse is needed
    let mut matches = app.get_matches();

    info!("{} installer", app_name);

    // Handle self-updating if needed
    let current_exe = std::env::current_exe().log_expect("Current executable could not be found");
    let current_path = current_exe
        .parent()
        .log_expect("Parent directory of executable could not be found");

    // Handle self-updating if needed
    self_update::perform_swap(&current_exe, matches.value_of("swap"));
    if let Some(new_matches) = self_update::check_args(reinterpret_app, current_path) {
        matches = new_matches;
    }
    self_update::cleanup(current_path);

    // Load in metadata + setup the installer framework
    let mut fresh_install = false;
    let metadata_file = current_path.join("metadata.json");
    let mut framework = if metadata_file.exists() {
        info!("Using pre-existing metadata file: {:?}", metadata_file);
        InstallerFramework::new_with_db(config.clone(), current_path).unwrap_or_else(|e| {
            error!("Failed to load metadata: {:?}", e);
            warn!("Entering recovery mode");
            InstallerFramework::new_recovery_mode(config, current_path)
        })
    } else {
        info!("Starting fresh install");
        fresh_install = true;
        InstallerFramework::new(config)
    };

    // check for existing installs if we are running as a fresh install
    let installed_path = PathBuf::from(framework.get_default_path().unwrap());
    if fresh_install && installed_path.join("metadata.json").exists() {
        info!("Existing install detected! Copying Trying to launch this install instead");
        // Ignore the return value from this since it should exit the application if its successful
        let _ = replace_existing_install(&current_exe, &installed_path);
    }

    let is_launcher = if let Some(string) = matches.value_of("launcher") {
        framework.is_launcher = true;
        framework.launcher_path = Some(string.to_string());
        true
    } else {
        false
    };

    // Start up the UI
    frontend::launch(&app_name, is_launcher, framework);
}

fn replace_existing_install(current_exe: &PathBuf, installed_path: &PathBuf) -> Result<(), String> {
    // Generate installer path
    let platform_extension = if cfg!(windows) {
        "maintenancetool.exe"
    } else {
        "maintenancetool"
    };

    let new_tool = if cfg!(windows) {
        "maintenancetool_new.exe"
    } else {
        "maintenancetool_new"
    };

    if let Err(v) = fs::copy(current_exe, installed_path.join(new_tool)) {
        return Err(format!("Unable to copy installer binary: {:?}", v));
    }

    let existing = installed_path
        .join(platform_extension)
        .into_os_string()
        .into_string();
    let new = installed_path.join(new_tool).into_os_string().into_string();
    if existing.is_ok() && new.is_ok() {
        // Remove NTFS alternate stream which tells the operating system that the updater was downloaded from the internet
        if cfg!(windows) {
            let _ = fs::remove_file(
                installed_path.join("maintenancetool_new.exe:Zone.Identifier:$DATA"),
            );
        }
        info!("Launching {:?}", existing);
        let success = Command::new(new.unwrap())
            .arg("--swap")
            .arg(existing.unwrap())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if success.is_ok() {
            exit(0);
        } else {
            error!("Unable to start existing yuzu maintenance tool. Launching old one instead");
        }
    }

    Ok(())
}
