//! Overall hierarchy for installing a installation of the application.

use installer::InstallerFramework;

use tasks::ensure_only_instance::EnsureOnlyInstanceTask;
use tasks::install_dir::VerifyInstallDirTask;
use tasks::install_global_shortcut::InstallGlobalShortcutsTask;
use tasks::install_pkg::InstallPackageTask;
use tasks::save_executable::SaveExecutableTask;
use tasks::uninstall_pkg::UninstallPackageTask;

use tasks::Task;
use tasks::TaskDependency;
use tasks::TaskMessage;
use tasks::TaskOrdering;
use tasks::TaskParamType;

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

        for item in &self.uninstall_items {
            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(UninstallPackageTask {
                    name: item.clone(),
                    optional: false,
                }),
            ));
        }

        for item in &self.items {
            elements.push(TaskDependency::build(
                TaskOrdering::Pre,
                Box::new(InstallPackageTask { name: item.clone() }),
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
