//! Saves the main database into the installation directory.

use crate::installer::InstallerFramework;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskParamType;

pub struct SaveDatabaseTask {}

impl Task for SaveDatabaseTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);
        messenger(&TaskMessage::DisplayMessage(
            "Saving application database...",
            0.0,
        ));

        context.save_database()?;

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![]
    }

    fn name(&self) -> String {
        "SaveDatabaseTask".to_string()
    }
}
