//! Downloads a package into memory.

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskDependency;
use tasks::TaskMessage;
use tasks::TaskOrdering;
use tasks::TaskParamType;

use tasks::resolver::ResolvePackageTask;

use http::stream_file;

use number_prefix::{decimal_prefix, Prefixed, Standalone};

use logging::LoggingErrors;

pub struct DownloadPackageTask {
    pub name: String,
}

impl Task for DownloadPackageTask {
    fn execute(
        &mut self,
        mut input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 1);

        let file = input.pop().log_expect("Should have input from resolver!");
        let (version, file) = match file {
            TaskParamType::File(v, f) => (v, f),
            _ => return Err("Unexpected param type to download package".to_string()),
        };

        // Check to see if this is the newest file available already
        for element in &context.database.packages {
            if element.name == self.name {
                if element.version == version {
                    info!("{:?} is already up to date.", self.name);
                    return Ok(TaskParamType::Break);
                }
                break;
            }
        }

        messenger(&TaskMessage::DisplayMessage(
            &format!("Downloading package {:?}...", self.name),
            0.0,
        ));

        let mut downloaded = 0;
        let mut data_storage: Vec<u8> = Vec::new();

        stream_file(&file.url, |data, size| {
            {
                data_storage.extend_from_slice(&data);
            }

            downloaded += data.len();

            let percentage = if size == 0 {
                0.0
            } else {
                (downloaded as f64) / (size as f64)
            };

            // Pretty print data volumes
            let pretty_current = match decimal_prefix(downloaded as f64) {
                Standalone(bytes) => format!("{} bytes", bytes),
                Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
            };
            let pretty_total = match decimal_prefix(size as f64) {
                Standalone(bytes) => format!("{} bytes", bytes),
                Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
            };

            messenger(&TaskMessage::DisplayMessage(
                &format!(
                    "Downloading {} ({} of {})...",
                    self.name, pretty_current, pretty_total
                ),
                percentage,
            ));
        })?;

        Ok(TaskParamType::FileContents(version, file, data_storage))
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![TaskDependency::build(
            TaskOrdering::Pre,
            Box::new(ResolvePackageTask {
                name: self.name.clone(),
            }),
        )]
    }

    fn name(&self) -> String {
        format!("DownloadPackageTask (for {:?})", self.name)
    }
}
