//! Verifies properties about the installation directory.

use crate::installer::InstallerFramework;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskParamType;

use std::fs::create_dir_all;
use std::fs::read_dir;

use crate::logging::LoggingErrors;

pub struct VerifyInstallDirTask {
    pub clean_install: bool,
}

impl Task for VerifyInstallDirTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);
        messenger(&TaskMessage::DisplayMessage(
            "Polling installation directory...",
            0.0,
        ));

        let path = context
            .install_path
            .as_ref()
            .log_expect("No install path specified");

        if !path.exists() {
            create_dir_all(&path)
                .map_err(|x| format!("Failed to create install directory: {:?}", x))?;
        }

        if self.clean_install {
            let paths = read_dir(&path)
                .map_err(|x| format!("Failed to read install destination: {:?}", x))?;

            if paths.count() != 0 {
                return Err(format!("Install destination ({:?}) is not empty.", path));
            }
        }

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![]
    }

    fn name(&self) -> String {
        format!(
            "VerifyInstallDirTask (with clean-install = {})",
            self.clean_install
        )
    }
}
