//! Installs a specific package.

use crate::installer::InstallerFramework;

use crate::tasks::download_pkg::DownloadPackageTask;
use crate::tasks::install_shortcuts::InstallShortcutsTask;
use crate::tasks::save_database::SaveDatabaseTask;
use crate::tasks::uninstall_pkg::UninstallPackageTask;
use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskOrdering;
use crate::tasks::TaskParamType;

use crate::config::PackageDescription;
use crate::installer::LocalInstallation;

use std::fs::create_dir_all;
use std::io::copy;

use crate::logging::LoggingErrors;

use crate::archives;

use std::fs::OpenOptions;
use std::path::Path;

pub struct InstallPackageTask {
    pub name: String,
}

impl Task for InstallPackageTask {
    fn execute(
        &mut self,
        mut input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        messenger(&TaskMessage::DisplayMessage(
            &format!("Installing package {:?}...", self.name),
            0.0,
        ));

        let path = context
            .install_path
            .as_ref()
            .log_expect("No install path specified");

        let mut installed_files = Vec::new();

        let mut metadata: Option<PackageDescription> = None;
        for description in &context
            .config
            .as_ref()
            .log_expect("Should have packages by now")
            .packages
        {
            if self.name == description.name {
                metadata = Some(description.clone());
                break;
            }
        }

        let package = match metadata {
            Some(v) => v,
            None => return Err(format!("Package {:?} could not be found.", self.name)),
        };

        // Grab data from the shortcut generator
        let shortcuts = input.pop().log_expect("Should have input from resolver!");
        let shortcuts = match shortcuts {
            TaskParamType::GeneratedShortcuts(files) => files,
            // If the resolver returned early, we need to unwind
            TaskParamType::Break => return Ok(TaskParamType::None),
            _ => return Err("Unexpected shortcuts param type to install package".to_string()),
        };

        // Ignore input from the uninstaller - no useful information passed
        input.pop();

        // Grab data from the resolver
        let data = input.pop().log_expect("Should have input from resolver!");
        let (version, file, data) = match data {
            TaskParamType::FileContents(version, file, data) => (version, file, data),
            _ => return Err("Unexpected file contents param type to install package".to_string()),
        };

        let mut archive = archives::read_archive(&file.name, data.as_slice())?;

        archive.for_each(&mut |i, archive_size, filename, mut file| {
            let string_name = filename
                .to_str()
                .ok_or("Unable to get str from file name")?
                .to_string();

            match &archive_size {
                Some(size) => {
                    messenger(&TaskMessage::DisplayMessage(
                        &format!("Extracting {} ({} of {})", string_name, i + 1, size),
                        (i as f64) / (*size as f64),
                    ));
                }
                _ => {
                    messenger(&TaskMessage::DisplayMessage(
                        &format!("Extracting {} ({} of ??)", string_name, i + 1),
                        0.0,
                    ));
                }
            }

            // Ensure that parent directories exist
            let mut parent_dir: &Path = &filename;
            while let Some(v) = parent_dir.parent() {
                parent_dir = v;

                let string_name = parent_dir
                    .to_str()
                    .ok_or("Unable to get str from file name")?
                    .to_string();

                if string_name.is_empty() {
                    continue;
                }

                if !installed_files.contains(&string_name) {
                    info!("Creating dir: {:?}", string_name);
                    installed_files.push(string_name);
                }

                match create_dir_all(path.join(&parent_dir)) {
                    Ok(v) => v,
                    Err(v) => return Err(format!("Unable to create dir: {:?}", v)),
                }
            }

            // Create target file
            let target_path = path.join(&filename);

            info!("Creating file: {:?}", string_name);

            if !installed_files.contains(&string_name) {
                installed_files.push(string_name);
            }

            let mut file_metadata = OpenOptions::new();
            file_metadata.write(true).create_new(true);

            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;

                file_metadata.mode(0o770);
            }

            let mut target_file = match file_metadata.open(target_path) {
                Ok(v) => v,
                Err(v) => return Err(format!("Unable to open file handle: {:?}", v)),
            };

            // Cross the streams
            match copy(&mut file, &mut target_file) {
                Ok(v) => v,
                Err(v) => return Err(format!("Unable to write to file: {:?}", v)),
            };

            Ok(())
        })?;

        // Save metadata about this package
        context.database.packages.push(LocalInstallation {
            name: package.name,
            version,
            shortcuts,
            files: installed_files,
        });

        messenger(&TaskMessage::PackageInstalled);

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![
            TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(DownloadPackageTask {
                    name: self.name.clone(),
                }),
            ),
            TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(UninstallPackageTask {
                    name: self.name.clone(),
                    optional: true,
                }),
            ),
            TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(InstallShortcutsTask {
                    name: self.name.clone(),
                }),
            ),
            TaskDependency::build(TaskOrdering::Post, Box::new(SaveDatabaseTask {})),
        ]
    }

    fn name(&self) -> String {
        format!("InstallPackageTask (for {:?})", self.name)
    }
}
