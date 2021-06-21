//! frontend/mod.rs
//!
//! Provides the frontend interface, including HTTP server.

use std::sync::{Arc, RwLock};

use crate::installer::InstallerFramework;
use crate::logging::LoggingErrors;

pub mod rest;
mod ui;

/// Launches the main web server + UI. Returns when the framework has been consumed + web UI closed.
pub fn launch(app_name: &str, is_launcher: bool, framework: InstallerFramework) {
    let framework = Arc::new(RwLock::new(framework));

    let (servers, address) = rest::server::spawn_servers(framework.clone());

    ui::start_ui(app_name, &address, is_launcher);

    // Explicitly hint that we want the servers instance until here.
    drop(servers);

    framework
        .write()
        .log_expect("Failed to write to framework to finalize")
        .shutdown()
        .log_expect("Failed to finalize framework");
}
