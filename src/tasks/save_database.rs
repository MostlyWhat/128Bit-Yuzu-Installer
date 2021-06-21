//! Saves the main database into the installation directory.

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskDependency;
use tasks::TaskMessage;
use tasks::TaskParamType;

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
