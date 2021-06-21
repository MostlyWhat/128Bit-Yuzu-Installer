//! Generates shortcuts for a specified file.

use crate::installer::InstallerFramework;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskParamType;

use crate::config::PackageDescription;

use crate::logging::LoggingErrors;

use crate::native::create_shortcut;

pub struct InstallShortcutsTask {
    pub name: String,
}

impl Task for InstallShortcutsTask {
    fn execute(
        &mut self,
        _: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        messenger(&TaskMessage::DisplayMessage(
            &format!("Generating shortcuts for package {:?}...", self.name),
            0.0,
        ));

        let path = context
            .install_path
            .as_ref()
            .log_expect("No install path specified");

        let starting_dir = path
            .to_str()
            .log_expect("Unable to build shortcut metadata (startingdir)");

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

        // Generate installer path
        let platform_extension = if cfg!(windows) {
            "maintenancetool.exe"
        } else {
            "maintenancetool"
        };

        for shortcut in package.shortcuts {
            let tool_path = path.join(platform_extension);
            let tool_path = tool_path
                .to_str()
                .log_expect("Unable to build shortcut metadata (tool)");

            let exe_path = path.join(shortcut.relative_path);
            let exe_path = exe_path
                .to_str()
                .log_expect("Unable to build shortcut metadata (exe)");

            installed_files.push(create_shortcut(
                &shortcut.name,
                &shortcut.description,
                tool_path,
                // TODO: Send by list
                &format!("--launcher \"{}\"", exe_path),
                &starting_dir,
                exe_path,
            )?);
        }

        Ok(TaskParamType::GeneratedShortcuts(installed_files))
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![]
    }

    fn name(&self) -> String {
        format!("InstallShortcutsTask (for {:?})", self.name)
    }
}
