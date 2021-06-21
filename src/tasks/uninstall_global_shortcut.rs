//! Uninstalls a specific package.

use crate::installer::InstallerFramework;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskParamType;

use crate::tasks::save_database::SaveDatabaseTask;
use crate::tasks::TaskOrdering;
use std::fs::remove_file;

pub struct UninstallGlobalShortcutsTask {}

impl Task for UninstallGlobalShortcutsTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);

        messenger(&TaskMessage::DisplayMessage(
            "Uninstalling global shortcut...",
            0.0,
        ));

        while let Some(file) = context.database.shortcuts.pop() {
            info!("Deleting shortcut {:?}", file);
            remove_file(file).map_err(|x| format!("Unable to delete global shortcut: {:?}", x))?;
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
        "UninstallGlobalShortcutsTask".to_string()
    }
}
