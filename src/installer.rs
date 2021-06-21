//! installer.rs
//!
//! Contains the main installer structure, as well as high-level means of controlling it.

use serde_json;

use std::fs::File;
use std::fs::OpenOptions;

use std::env;
use std::env::var;

use std::path::Path;
use std::path::PathBuf;

use std::sync::mpsc::Sender;

use std::io::copy;
use std::io::Cursor;

use std::process::Command;
use std::process::{exit, Stdio};

use crate::config::BaseAttributes;
use crate::config::Config;

use crate::sources::types::Version;

use crate::tasks::install::InstallTask;
use crate::tasks::uninstall::UninstallTask;
use crate::tasks::uninstall_global_shortcut::UninstallGlobalShortcutsTask;
use crate::tasks::DependencyTree;
use crate::tasks::TaskMessage;

use crate::logging::LoggingErrors;

use dirs::home_dir;

use std::fs::remove_file;

use crate::http;

use number_prefix::NumberPrefix::{self, Prefixed, Standalone};

use crate::native;

/// A message thrown during the installation of packages.
#[derive(Serialize)]
pub enum InstallMessage {
    Status(String, f64),
    PackageInstalled,
    Error(String),
    EOF,
}

/// Metadata about the current installation itself.
#[derive(Serialize, Deserialize, Clone)]
pub struct InstallationDatabase {
    pub packages: Vec<LocalInstallation>,
    pub shortcuts: Vec<String>,
}

impl InstallationDatabase {
    /// Creates a new, empty installation database.
    pub fn new() -> InstallationDatabase {
        InstallationDatabase {
            packages: Vec::new(),
            shortcuts: Vec::new(),
        }
    }
}

/// The installer framework contains metadata about packages, what is installable, what isn't,
/// etc.
pub struct InstallerFramework {
    pub base_attributes: BaseAttributes,
    pub config: Option<Config>,
    pub database: InstallationDatabase,
    pub install_path: Option<PathBuf>,
    pub preexisting_install: bool,
    pub is_launcher: bool,
    // If we just completed an uninstall, and we should clean up after ourselves.
    pub burn_after_exit: bool,
    pub launcher_path: Option<String>,
}

/// Contains basic properties on the status of the session. Subset of InstallationFramework.
#[derive(Serialize)]
pub struct InstallationStatus {
    pub database: InstallationDatabase,
    pub install_path: Option<String>,
    pub preexisting_install: bool,
    pub is_launcher: bool,
    pub launcher_path: Option<String>,
}

/// Tracks the state of a local installation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalInstallation {
    pub name: String,
    pub version: Version,
    /// Relative paths to generated files
    pub files: Vec<String>,
    /// Absolute paths to generated shortcut files
    pub shortcuts: Vec<String>,
}

macro_rules! declare_messenger_callback {
    ($target:expr) => {
        &|msg: &TaskMessage| match *msg {
            TaskMessage::DisplayMessage(msg, progress) => {
                if let Err(v) = $target.send(InstallMessage::Status(msg.to_string(), progress as _))
                {
                    error!("Failed to submit queue message: {:?}", v);
                }
            }
            TaskMessage::PackageInstalled => {
                if let Err(v) = $target.send(InstallMessage::PackageInstalled) {
                    error!("Failed to submit queue message: {:?}", v);
                }
            }
        }
    };
}

impl InstallerFramework {
    /// Returns a copy of the configuration.
    pub fn get_config(&self) -> Option<Config> {
        self.config.clone()
    }

    /// Returns the default install path.
    pub fn get_default_path(&self) -> Option<String> {
        let app_name = &self.base_attributes.name;

        let base_dir = match var("LOCALAPPDATA") {
            Ok(path) => PathBuf::from(path),
            Err(_) => home_dir()?,
        };

        let file_name = if cfg!(unix) {
            format!(".{}", app_name.to_ascii_lowercase())
        } else {
            app_name.to_string()
        };

        let file = base_dir.join(file_name);

        Some(file.to_str()?.to_owned())
    }

    /// Sends a request for something to be installed.
    /// items: Array of named packages to be installed/kept
    /// messages: Channel used to send progress messages
    /// fresh_install: If the install directory must be empty
    pub fn install(
        &mut self,
        items: Vec<String>,
        messages: &Sender<InstallMessage>,
        fresh_install: bool,
    ) -> Result<(), String> {
        info!(
            "Framework: Installing {:?} to {:?}",
            items,
            self.install_path
                .clone()
                .log_expect("Install directory not initialised")
        );

        // Calculate packages to *uninstall*
        let mut uninstall_items = Vec::new();
        if !fresh_install {
            for package in &self.database.packages {
                if !items.contains(&package.name) {
                    uninstall_items.push(package.name.clone());
                }
            }

            info!(
                "Framework: Uninstalling {:?} additionally.",
                uninstall_items
            );
        }

        let task = Box::new(InstallTask {
            items,
            uninstall_items,
            fresh_install,
        });

        let mut tree = DependencyTree::build(task);

        info!("Dependency tree:\n{}", tree);

        tree.execute(self, declare_messenger_callback!(messages))
            .map(|_x| ())
    }

    /// Sends a request for everything to be uninstalled.
    pub fn uninstall(&mut self, messages: &Sender<InstallMessage>) -> Result<(), String> {
        let items: Vec<String> = self
            .database
            .packages
            .iter()
            .map(|x| x.name.clone())
            .collect();

        let task = Box::new(UninstallTask { items });

        let mut tree = DependencyTree::build(task);

        info!("Dependency tree:\n{}", tree);

        tree.execute(self, declare_messenger_callback!(messages))
            .map(|_x| ())?;

        // Uninstall shortcuts
        let task = Box::new(UninstallGlobalShortcutsTask {});

        let mut tree = DependencyTree::build(task);

        tree.execute(self, declare_messenger_callback!(messages))
            .map(|_x| ())?;

        // Delete the metadata file
        let path = self
            .install_path
            .as_ref()
            .log_expect("No install path specified");

        remove_file(path.join("metadata.json"))
            .map_err(|x| format!("Failed to delete metadata: {:?}", x))?;

        // Logging will have to be done later
        self.burn_after_exit = true;

        Ok(())
    }

    /// Verifies that the config has all requirements met (no need to update the
    /// updater, for example). This will terminate if this is the case after applying
    /// the correct actions.
    pub fn update_updater(&mut self, messages: &Sender<InstallMessage>) -> Result<(), String> {
        let tool = self
            .config
            .as_ref()
            .log_expect("Config should exist by now")
            .new_tool
            .as_ref()
            .log_expect("Frontend asked for updater update when one doesn't exist");

        let mut downloaded = 0;
        let mut data_storage: Vec<u8> = Vec::new();

        http::stream_file(tool, |data, size| {
            {
                data_storage.extend_from_slice(&data);
            }

            downloaded += data.len();

            let percentage = if size == 0 {
                0.0
            } else {
                (downloaded as f64) / (size as f64)
            };

            // Pretty print data volumes
            let pretty_current = match NumberPrefix::decimal(downloaded as f64) {
                Standalone(bytes) => format!("{} bytes", bytes),
                Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
            };
            let pretty_total = match NumberPrefix::decimal(size as f64) {
                Standalone(bytes) => format!("{} bytes", bytes),
                Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
            };

            if let Err(v) = messages.send(InstallMessage::Status(
                format!(
                    "Downloading self-update ({} of {})...",
                    pretty_current, pretty_total
                ),
                percentage as _,
            )) {
                error!("Failed to submit queue message: {:?}", v);
            }
        })?;

        info!("Launching new updater...");

        // Save to file in current dir
        let current_exe = env::current_exe().log_expect("Current executable could not be found");
        let path = current_exe
            .parent()
            .log_expect("Parent directory of executable could not be found");

        let platform_extension = if cfg!(windows) {
            "maintenancetool_new.exe"
        } else {
            "maintenancetool_new"
        };

        let new_app = path.join(platform_extension);

        let mut file_metadata = OpenOptions::new();
        file_metadata.write(true).create(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;

            file_metadata.mode(0o770);
        }

        {
            let mut new_app_file = match file_metadata.open(&new_app) {
                Ok(v) => v,
                Err(v) => return Err(format!("Unable to open installer binary: {:?}", v)),
            };

            if let Err(v) = copy(&mut Cursor::new(data_storage), &mut new_app_file) {
                return Err(format!("Unable to copy installer binary: {:?}", v));
            }
        }

        // Save current command line arguments
        let args_file = path.join("args.json");
        let args: Vec<String> = env::args_os()
            .map(|x| {
                x.to_str()
                    .log_expect("Unable to convert argument to String")
                    .to_string()
            })
            .collect();

        {
            let new_app_file = match File::create(&args_file) {
                Ok(v) => v,
                Err(v) => return Err(format!("Unable to open args file: {:?}", v)),
            };

            serde_json::to_writer(new_app_file, &args).log_expect("Unable to write args");
        }

        let current_exe = env::current_exe().log_expect("Current executable could not be found");

        // Launch this new process
        Command::new(new_app)
            .arg("--swap")
            .arg(current_exe)
            .spawn()
            .log_expect("Unable to start child process");

        exit(0);
    }

    /// Saves the applications database.
    pub fn save_database(&self) -> Result<(), String> {
        // We have to have a install path for us to be able to do anything
        let path = match self.install_path.clone() {
            Some(v) => v,
            None => return Err("No install directory for installer".to_string()),
        };

        let metadata_path = path.join("metadata.json");
        let metadata_file = match File::create(metadata_path) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to open file handle: {:?}", v)),
        };

        match serde_json::to_writer(metadata_file, &self.database) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to write to file: {:?}", v)),
        };

        Ok(())
    }

    /// Configures this installer to install to the specified location.
    /// If there was a currently configured install path, this will be left as-is.
    pub fn set_install_dir(&mut self, dir: &str) {
        self.install_path = Some(Path::new(dir).to_owned());
    }

    /// Returns metadata on the current status of the installation.
    pub fn get_installation_status(&self) -> InstallationStatus {
        InstallationStatus {
            database: self.database.clone(),
            install_path: match self.install_path.clone() {
                Some(v) => Some(v.display().to_string()),
                None => None,
            },
            preexisting_install: self.preexisting_install,
            is_launcher: self.is_launcher,
            launcher_path: self.launcher_path.clone(),
        }
    }

    /// Shuts down the installer instance.
    pub fn shutdown(&mut self) -> Result<(), String> {
        info!("Shutting down installer framework...");

        if let Some(ref v) = self.launcher_path.take() {
            info!("Launching {:?}", v);

            Command::new(v)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|x| format!("Unable to start application: {:?}", x))?;
        }

        if self.burn_after_exit {
            info!("Requesting that self be deleted after exit.");
            native::burn_on_exit(&self.base_attributes.name);
            self.burn_after_exit = false;
        }

        Ok(())
    }

    /// Creates a new instance of the Installer Framework with a specified Config.
    pub fn new(attrs: BaseAttributes) -> Self {
        InstallerFramework {
            base_attributes: attrs,
            config: None,
            database: InstallationDatabase::new(),
            install_path: None,
            preexisting_install: false,
            is_launcher: false,
            burn_after_exit: false,
            launcher_path: None,
        }
    }

    /// Creates a new instance of the Installer Framework with a specified Config, managing
    /// a pre-existing installation.
    pub fn new_with_db(attrs: BaseAttributes, install_path: &Path) -> Result<Self, String> {
        let path = install_path.to_owned();
        let metadata_path = path.join("metadata.json");
        let metadata_file = match File::open(metadata_path) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to open file handle: {:?}", v)),
        };

        let database: InstallationDatabase = match serde_json::from_reader(metadata_file) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to read metadata file: {:?}", v)),
        };

        Ok(InstallerFramework {
            base_attributes: attrs,
            config: None,
            database,
            install_path: Some(path),
            preexisting_install: true,
            is_launcher: false,
            burn_after_exit: false,
            launcher_path: None,
        })
    }
}
