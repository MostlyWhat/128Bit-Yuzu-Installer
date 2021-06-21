//! self_update.rs
//!
//! Handles different components of self-updating.

use std::fs::{remove_file, File};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::{thread, time};

use clap::{App, ArgMatches};

use crate::logging::LoggingErrors;

/// Swaps around the main executable if needed.
pub fn perform_swap(current_exe: &PathBuf, to_path: Option<&str>) {
    // Check to see if we are currently in a self-update
    if let Some(to_path) = to_path {
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
                        Err::<(), _>(e).log_expect("Copying new binary failed");
                    }
                }
            }
        }

        Command::new(to_path)
            .spawn()
            .log_expect("Unable to start child process");

        exit(0);
    }
}

pub fn check_args<'a>(app: App<'a, '_>, current_path: &Path) -> Option<ArgMatches<'a>> {
    // If we just finished a update, we need to inject our previous command line arguments
    let args_file = current_path.join("args.json");

    if args_file.exists() {
        let database: Vec<String> = {
            let metadata_file =
                File::open(&args_file).log_expect("Unable to open args file handle");

            serde_json::from_reader(metadata_file).log_expect("Unable to read metadata file")
        };

        let matches = app.get_matches_from(database);

        info!("Parsed command line arguments from original instance");
        remove_file(args_file).log_expect("Unable to clean up args file");

        Some(matches)
    } else {
        None
    }
}

pub fn cleanup(current_path: &Path) {
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
}
