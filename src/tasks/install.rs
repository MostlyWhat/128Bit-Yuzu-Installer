//! Overall hierarchy for installing a installation of the application.

use crate::installer::InstallerFramework;

use crate::tasks::ensure_only_instance::EnsureOnlyInstanceTask;
use crate::tasks::install_dir::VerifyInstallDirTask;
use crate::tasks::install_global_shortcut::InstallGlobalShortcutsTask;
use crate::tasks::install_pkg::InstallPackageTask;
use crate::tasks::save_executable::SaveExecutableTask;
use crate::tasks::uninstall_pkg::UninstallPackageTask;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskOrdering;
use crate::tasks::TaskParamType;

pub struct InstallTask {
    pub items: Vec<String>,
    pub uninstall_items: Vec<String>,
    pub fresh_install: bool,
}

impl Task for InstallTask {
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

        elements.push(TaskDependency::build(
            TaskOrdering::Pre,
            Box::new(EnsureOnlyInstanceTask {}),
        ));

        elements.push(TaskDependency::build(
            TaskOrdering::Pre,
            Box::new(VerifyInstallDirTask {
                clean_install: self.fresh_install,
            }),
        ));

        for item in &self.items {
            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(InstallPackageTask { name: item.clone() }),
            ));
        }

        for item in &self.uninstall_items {
            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(UninstallPackageTask {
                    name: item.clone(),
                    optional: false,
                }),
            ));
        }

        if self.fresh_install {
            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(SaveExecutableTask {}),
            ));

            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(InstallGlobalShortcutsTask {}),
            ));
        }

        elements
    }

    fn name(&self) -> String {
        "InstallTask".to_string()
    }
}
