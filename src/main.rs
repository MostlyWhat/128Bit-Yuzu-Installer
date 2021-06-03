//! main.rs
//!
//! The main entrypoint for the application. Orchestrates the building of the installation
//! framework, and opens necessary HTTP servers/frontends.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(unsafe_code)]
#![deny(missing_docs)]

#[cfg(windows)]
extern crate nfd;

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
extern crate xz_decom;
extern crate zip;
extern crate flate2;

extern crate fern;
#[macro_use]
extern crate log;

extern crate chrono;

extern crate clap;

#[cfg(windows)]
extern crate winapi;

mod archives;
mod assets;
mod config;
mod http;
mod installer;
mod logging;
mod native;
mod rest;
mod sources;
mod tasks;

use web_view::*;

use installer::InstallerFramework;

#[cfg(windows)]
use nfd::Response;

use rest::WebServer;

use std::net::TcpListener;
use std::net::ToSocketAddrs;

use std::sync::Arc;
use std::sync::RwLock;

use std::path::PathBuf;

use std::process::exit;
use std::process::Command;
use std::{thread, time};

use std::fs::remove_file;
use std::fs::File;

use logging::LoggingErrors;

use clap::App;
use clap::Arg;
use log::Level;

use config::BaseAttributes;

static RAW_CONFIG: &'static str = include_str!(concat!(env!("OUT_DIR"), "/bootstrap.toml"));

#[derive(Deserialize, Debug)]
enum CallbackType {
    SelectInstallDir { callback_name: String },
    Log { msg: String, kind: String },
}

fn main() {
    let config = BaseAttributes::from_toml_str(RAW_CONFIG).expect("Config file could not be read");

    logging::setup_logger(format!("{}_installer.log", config.name))
        .expect("Unable to setup logging!");

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
        ).arg(
            Arg::with_name("swap")
                .long("swap")
                .value_name("TARGET")
                .help("Internal usage - swaps around a new installer executable")
                .takes_value(true),
        );

    let reinterpret_app = app.clone(); // In case a reparse is needed
    let mut matches = app.get_matches();

    info!("{} installer", app_name);

    let current_exe = std::env::current_exe().log_expect("Current executable could not be found");
    let current_path = current_exe
        .parent()
        .log_expect("Parent directory of executable could not be found");

    // Check to see if we are currently in a self-update
    if let Some(to_path) = matches.value_of("swap") {
        let to_path = PathBuf::from(to_path);

        // Sleep a little bit to allow Windows to close the previous file handle
        thread::sleep(time::Duration::from_millis(3000));

        info!(
            "Swapping installer from {} to {}",
            current_exe.display(),
            to_path.display()
        );

        // Attempt it a few times because Windows can hold a lock
        for i in 1..=5 {
            let swap_result = if cfg!(windows) {
                use std::fs::copy;

                copy(&current_exe, &to_path).map(|_x| ())
            } else {
                use std::fs::rename;

                rename(&current_exe, &to_path)
            };

            match swap_result {
                Ok(_) => break,
                Err(e) => {
                    if i < 5 {
                        info!("Copy attempt failed: {:?}, retrying in 3 seconds.", e);
                        thread::sleep(time::Duration::from_millis(3000));
                    } else {
                        let _: () = Err(e).log_expect("Copying new binary failed");
                    }
                }
            }
        }

        Command::new(to_path)
            .spawn()
            .log_expect("Unable to start child process");

        exit(0);
    }

    // If we just finished a update, we need to inject our previous command line arguments
    let args_file = current_path.join("args.json");

    if args_file.exists() {
        let database: Vec<String> = {
            let metadata_file =
                File::open(&args_file).log_expect("Unable to open args file handle");

            serde_json::from_reader(metadata_file).log_expect("Unable to read metadata file")
        };

        matches = reinterpret_app.get_matches_from(database);

        info!("Parsed command line arguments from original instance");
        remove_file(args_file).log_expect("Unable to clean up args file");
    }

    // Cleanup any remaining new maintenance tool instances if they exist
    if cfg!(windows) {
        let updater_executable = current_path.join("maintenancetool_new.exe");

        if updater_executable.exists() {
            // Sleep a little bit to allow Windows to close the previous file handle
            thread::sleep(time::Duration::from_millis(3000));

            // Attempt it a few times because Windows can hold a lock
            for i in 1..=5 {
                let swap_result = remove_file(&updater_executable);
                match swap_result {
                    Ok(_) => break,
                    Err(e) => {
                        if i < 5 {
                            info!("Cleanup attempt failed: {:?}, retrying in 3 seconds.", e);
                            thread::sleep(time::Duration::from_millis(3000));
                        } else {
                            warn!("Deleting temp binary failed after 5 attempts: {:?}", e);
                        }
                    }
                }
            }
        }
    }

    // Load in metadata as to learn about the environment
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

    // Firstly, allocate us an epidermal port
    let target_port = {
        let listener = TcpListener::bind("127.0.0.1:0")
            .log_expect("At least one local address should be free");
        listener
            .local_addr()
            .log_expect("Should be able to pull address from listener")
            .port()
    };

    // Now, iterate over all ports
    let addresses = "localhost:0"
        .to_socket_addrs()
        .log_expect("No localhost address found");

    let mut servers = Vec::new();
    let mut http_address = None;

    let framework = Arc::new(RwLock::new(framework));

    // Startup HTTP server for handling the web view
    for mut address in addresses {
        address.set_port(target_port);

        let server = WebServer::with_addr(framework.clone(), address)
            .log_expect("Failed to bind to address");

        info!("Server: {:?}", address);

        http_address = Some(address);

        servers.push(server);
    }

    let http_address = http_address.log_expect("No HTTP address found");

    let http_address = format!("http://localhost:{}", http_address.port());

    // Init the web view
    let size = if is_launcher { (600, 300) } else { (1024, 500) };

    let resizable = false;
    let debug = true;

    run(
        &format!("{} Installer", app_name),
        Content::Url(http_address),
        Some(size),
        resizable,
        debug,
        |_| {},
        |wv, msg, _| {
            let command: CallbackType =
                serde_json::from_str(msg).log_expect(&format!("Unable to parse string: {:?}", msg));

            debug!("Incoming payload: {:?}", command);

            match command {
                CallbackType::SelectInstallDir { callback_name } => {
                    #[cfg(windows)]
                    let result = match nfd::open_pick_folder(None)
                        .log_expect("Unable to open folder dialog")
                    {
                        Response::Okay(v) => v,
                        _ => return,
                    };

                    #[cfg(not(windows))]
                    let result =
                        wv.dialog(Dialog::ChooseDirectory, "Select a install directory...", "");

                    if !result.is_empty() {
                        let result = serde_json::to_string(&result)
                            .log_expect("Unable to serialize response");
                        let command = format!("{}({});", callback_name, result);
                        debug!("Injecting response: {}", command);
                        wv.eval(&command);
                    }
                }
                CallbackType::Log { msg, kind } => {
                    let kind = match kind.as_ref() {
                        "info" | "log" => Level::Info,
                        "warn" => Level::Warn,
                        "error" => Level::Error,
                        _ => Level::Error,
                    };

                    log!(target: "liftinstall::frontend-js", kind, "{}", msg);
                }
            }
        },
        (),
    );
}
