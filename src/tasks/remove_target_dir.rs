//! remove the whole target directory from the existence

use crate::installer::InstallerFramework;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskParamType;

pub struct RemoveTargetDirTask {}

impl Task for RemoveTargetDirTask {
    fn execute(
        &mut self,
        _: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        messenger(&TaskMessage::DisplayMessage(
            "Removing previous install...",
            0.1,
        ));
        // erase the database as well
        context.database.packages = Vec::new();
        if let Some(path) = context.install_path.as_ref() {
            let entries = std::fs::read_dir(path)
                .map_err(|e| format!("Error reading {}: {}", path.to_string_lossy(), e))?;
            // remove everything under the path
            if !context.preexisting_install {
                std::fs::remove_dir_all(path)
                    .map_err(|e| format!("Error removing {}: {}", path.to_string_lossy(), e))?;
                return Ok(TaskParamType::None);
            }
            // remove everything except the maintenancetool if repairing
            for entry in entries {
                let path = entry
                    .map_err(|e| format!("Error reading file: {}", e))?
                    .path();
                if let Some(filename) = path.file_name() {
                    if filename.to_string_lossy().starts_with("maintenancetool") {
                        continue;
                    }
                }
                if path.is_dir() {
                    std::fs::remove_dir_all(&path)
                        .map_err(|e| format!("Error removing {}: {}", path.to_string_lossy(), e))?;
                } else {
                    std::fs::remove_file(&path)
                        .map_err(|e| format!("Error removing {}: {}", path.to_string_lossy(), e))?;
                }
            }
        }

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![]
    }

    fn name(&self) -> String {
        "RemoveTargetDirTask".to_string()
    }
}
