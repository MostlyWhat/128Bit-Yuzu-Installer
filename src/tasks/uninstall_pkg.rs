//! Uninstalls a specific package.

use crate::installer::InstallerFramework;

use crate::tasks::save_database::SaveDatabaseTask;
use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskOrdering;
use crate::tasks::TaskParamType;

use crate::installer::LocalInstallation;

use std::fs::remove_dir;
use std::fs::remove_file;

use crate::logging::LoggingErrors;
use crate::tasks::uninstall_shortcuts::UninstallShortcutsTask;

pub struct UninstallPackageTask {
    pub name: String,
    pub optional: bool,
}

impl Task for UninstallPackageTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 1);

        let path = context
            .install_path
            .as_ref()
            .log_expect("No install path specified");

        let mut metadata: Option<LocalInstallation> = None;
        for i in 0..context.database.packages.len() {
            if self.name == context.database.packages[i].name {
                metadata = Some(context.database.packages.remove(i));
                break;
            }
        }

        let package = match metadata {
            Some(v) => v,
            None => {
                if self.optional {
                    return Ok(TaskParamType::None);
                }

                return Err(format!(
                    "Package {:?} could not be found for uninstall.",
                    self.name
                ));
            }
        };

        messenger(&TaskMessage::DisplayMessage(
            &format!("Uninstalling package {:?}...", self.name),
            0.0,
        ));

        let mut directories = Vec::new();

        let max = package.files.len();
        for (i, file) in package.files.iter().enumerate() {
            let name = file.clone();
            let file = path.join(file);
            info!("Deleting {:?}", file);

            messenger(&TaskMessage::DisplayMessage(
                &format!("Deleting {} ({} of {})", name, i + 1, max),
                (i as f64) / (max as f64),
            ));

            let result = if file.is_dir() {
                // we don't delete directory just yet
                directories.push(file);
                Ok(())
            } else {
                remove_file(file)
            };

            if let Err(v) = result {
                error!("Failed to delete file: {:?}", v);
            }
        }

        // sort directories by reverse depth order
        directories.sort_by(|a, b| {
            let depth_a = a.components().fold(0usize, |acc, _| acc + 1);
            let depth_b = b.components().fold(0usize, |acc, _| acc + 1);
            depth_b.cmp(&depth_a)
        });
        for i in directories.iter() {
            info!("Deleting directory: {:?}", i);
            remove_dir(i).ok();
        }

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![
            TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(UninstallShortcutsTask {
                    name: self.name.clone(),
                    optional: self.optional,
                }),
            ),
            TaskDependency::build(TaskOrdering::Post, Box::new(SaveDatabaseTask {})),
        ]
    }

    fn name(&self) -> String {
        format!(
            "UninstallPackageTask (for {:?}, optional = {})",
            self.name, self.optional
        )
    }
}
