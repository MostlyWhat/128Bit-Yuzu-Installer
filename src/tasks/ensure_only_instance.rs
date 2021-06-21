//! Verifies that this is the only running instance of the installer, and that no application is running.

use crate::installer::InstallerFramework;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskParamType;

<<<<<<< Updated upstream
use native::Process;
use native::get_process_names;
=======
use crate::native::get_process_names;
use crate::native::Process;
>>>>>>> Stashed changes

use std::process;

pub struct EnsureOnlyInstanceTask {}

impl Task for EnsureOnlyInstanceTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        _messenger: &Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);

        let current_pid =  process::id() as usize;
        for Process { pid, name } in get_process_names() {
            if pid == current_pid {
                continue;
            }

            let exe = name;

            if exe.ends_with("maintenancetool.exe") || exe.ends_with("maintenancetool") {
                return Err(format!("Maintenance tool is already running!"));
            }

            for package in &context.database.packages {
                for file in &package.files {
                    if exe.ends_with(file) {
                        return Err(format!("The installed application is currently running!"));
                    }
                }
            }
        }

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![]
    }

    fn name(&self) -> String {
        format!("EnsureOnlyInstanceTask")
    }
}
