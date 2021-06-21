//! main.rs
//!
//! The main entrypoint for the application. Orchestrates the building of the installation
//! framework, and opens necessary HTTP servers/frontends.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(unsafe_code)]
#![deny(missing_docs)]

extern crate web_view;

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

use clap::App;
use clap::Arg;

use config::BaseAttributes;

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

    self_update::perform_swap(&current_exe, matches.value_of("swap"));
    if let Some(new_matches) = self_update::check_args(reinterpret_app, current_path) {
        matches = new_matches;
    }
    self_update::cleanup(current_path);

    // Load in metadata + setup the installer framework
    let metadata_file = current_path.join("metadata.json");
    let mut framework = if metadata_file.exists() {
        info!("Using pre-existing metadata file: {:?}", metadata_file);
        InstallerFramework::new_with_db(config, current_path).log_expect("Unable to parse metadata")
    } else {
        info!("Starting fresh install");
        InstallerFramework::new(config)
    };

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
