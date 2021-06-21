//! Generates the global shortcut for this application.

use crate::installer::InstallerFramework;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskParamType;

use crate::logging::LoggingErrors;

use crate::native::create_shortcut;
use crate::tasks::save_database::SaveDatabaseTask;
use crate::tasks::TaskOrdering;

pub struct InstallGlobalShortcutsTask {}

impl Task for InstallGlobalShortcutsTask {
    fn execute(
        &mut self,
        _: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        messenger(&TaskMessage::DisplayMessage(
            "Generating global shortcut...",
            0.0,
        ));

        let path = context
            .install_path
            .as_ref()
            .log_expect("No install path specified");

        let starting_dir = path
            .to_str()
            .log_expect("Unable to build shortcut metadata (startingdir)");

        // Generate installer path
        let platform_extension = if cfg!(windows) {
            "maintenancetool.exe"
        } else {
            "maintenancetool"
        };

        let tool_path = path.join(platform_extension);
        let tool_path = tool_path
            .to_str()
            .log_expect("Unable to build shortcut metadata (tool)");

        let shortcut_file = create_shortcut(
            &format!("{} Maintenance Tool", context.base_attributes.name),
            &format!(
                "Launch the {} Maintenance Tool to update, modify and uninstall the application.",
                context.base_attributes.name
            ),
            tool_path,
            // TODO: Send by list
            "",
            &starting_dir,
            "",
        )?;

        if !shortcut_file.is_empty() {
            context.database.shortcuts.push(shortcut_file);
        }

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![TaskDependency::build(
            TaskOrdering::Post,
            Box::new(SaveDatabaseTask {}),
        )]
    }

    fn name(&self) -> String {
        "InstallGlobalShortcutsTask".to_string()
    }
}
