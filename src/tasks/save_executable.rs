//! Saves the installer executable into the install directory.

use crate::installer::InstallerFramework;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskParamType;

use std::fs::File;
use std::fs::OpenOptions;

use std::io::copy;

use std::env::current_exe;

use crate::logging::LoggingErrors;

pub struct SaveExecutableTask {}

impl Task for SaveExecutableTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);
        messenger(&TaskMessage::DisplayMessage(
            "Copying installer binary...",
            0.0,
        ));

        let path = context
            .install_path
            .as_ref()
            .log_expect("No install path specified");

        let current_app = match current_exe() {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to locate installer binary: {:?}", v)),
        };

        let mut current_app_file = match File::open(current_app) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to open installer binary: {:?}", v)),
        };

        let platform_extension = if cfg!(windows) {
            "maintenancetool.exe"
        } else {
            "maintenancetool"
        };

        let new_app = path.join(platform_extension);

        let mut file_metadata = OpenOptions::new();
        file_metadata.write(true).create_new(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;

            file_metadata.mode(0o770);
        }

        let mut new_app_file = match file_metadata.open(new_app) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to open installer binary: {:?}", v)),
        };

        if let Err(v) = copy(&mut current_app_file, &mut new_app_file) {
            return Err(format!("Unable to copy installer binary: {:?}", v));
        }

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![]
    }

    fn name(&self) -> String {
        "SaveExecutableTask".to_string()
    }
}
