//! Uninstalls a set of packages.

use crate::installer::InstallerFramework;

use crate::tasks::Task;
use crate::tasks::TaskParamType;

use crate::tasks::uninstall_pkg::UninstallPackageTask;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskOrdering;

pub struct UninstallTask {
    pub items: Vec<String>,
}

impl Task for UninstallTask {
    fn execute(
        &mut self,
        _: Vec<TaskParamType>,
        _: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        messenger(&TaskMessage::DisplayMessage("Wrapping up...", 0.0));
        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        let mut elements = Vec::new();

        for item in &self.items {
            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(UninstallPackageTask {
                    name: item.clone(),
                    optional: false,
                }),
            ));
        }

        elements
    }

    fn name(&self) -> String {
        "UninstallTask".to_string()
    }
}
