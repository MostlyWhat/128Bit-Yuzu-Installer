//! Configures lift to launch the new package on fresh install after its closed
//! If theres multiple launchable packages, then choose the first listed in config
//! If there are multiple shortcuts for the first package, then launch the first.

use crate::installer::InstallerFramework;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskParamType;

use crate::config::PackageDescription;

use crate::logging::LoggingErrors;

pub struct LaunchOnExitTask {}

impl Task for LaunchOnExitTask {
    fn execute(
        &mut self,
        _: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        _: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        let pkg = &context.database.packages.first();
        if pkg.is_none() {
            return Ok(TaskParamType::None);
        }
        let pkg = pkg.unwrap();

        // look up the first shortcut for the first listed package in the database
        let path = context
            .install_path
            .as_ref()
            .log_expect("No install path specified");

        let mut metadata: Option<PackageDescription> = None;
        for description in &context
            .config
            .as_ref()
            .log_expect("Should have packages by now")
            .packages
        {
            if pkg.name == description.name {
                metadata = Some(description.clone());
                break;
            }
        }

        let package_desc = match metadata {
            Some(v) => v,
            // Package metadata is missing. Dunno what went wrong but we can skip this then
            None => return Ok(TaskParamType::None),
        };

        let shortcut = package_desc.shortcuts.first();

        // copy the path to the actual exe into launcher_path so it'll load it on exit
        context.launcher_path = shortcut.map(|s| {
            path.join(s.relative_path.clone())
                .to_str()
                .map(|t| t.to_string())
                .unwrap()
        });

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![]
    }

    fn name(&self) -> String {
        "LaunchOnExitTask".to_string()
    }
}
