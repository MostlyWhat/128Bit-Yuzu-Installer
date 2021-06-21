//! Resolves package names into a metadata + version object.

use std::env::consts::OS;

use crate::installer::InstallerFramework;

use crate::tasks::Task;
use crate::tasks::TaskDependency;
use crate::tasks::TaskMessage;
use crate::tasks::TaskParamType;

use crate::config::PackageDescription;

use regex::Regex;

use crate::logging::LoggingErrors;

pub struct ResolvePackageTask {
    pub name: String,
}

impl Task for ResolvePackageTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);
        let mut metadata: Option<PackageDescription> = None;
        for description in &context
            .config
            .as_ref()
            .log_expect("Should have packages by now")
            .packages
        {
            if self.name == description.name {
                metadata = Some(description.clone());
                break;
            }
        }

        let package = match metadata {
            Some(v) => v,
            None => return Err(format!("Package {:?} could not be found.", self.name)),
        };

        messenger(&TaskMessage::DisplayMessage(
            &format!(
                "Polling {} for latest version of {:?}...",
                package.source.name, package.name
            ),
            0.0,
        ));

        let results = package.source.get_current_releases()?;

        messenger(&TaskMessage::DisplayMessage(
            &format!("Resolving dependency for {:?}...", package.name),
            0.5,
        ));

        let filtered_regex = package.source.match_regex.replace("#PLATFORM#", OS);
        let regex = match Regex::new(&filtered_regex) {
            Ok(v) => v,
            Err(v) => return Err(format!("An error occurred while compiling regex: {:?}", v)),
        };

        // Find the latest release in here
        let latest_result = results
            .into_iter()
            .filter(|f| f.files.iter().filter(|x| regex.is_match(&x.name)).count() > 0)
            .max_by_key(|f| f.version.clone());

        let latest_result = match latest_result {
            Some(v) => v,
            None => return Err("No release with correct file found".to_string()),
        };

        let latest_version = latest_result.version.clone();

        // Find the matching file in here
        let latest_file = latest_result
            .files
            .into_iter()
            .find(|x| regex.is_match(&x.name))
            .log_expect("Searched file should have existed, but didn't");

        info!("Selected file: {:?}", latest_file);

        Ok(TaskParamType::File(latest_version, latest_file))
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![]
    }

    fn name(&self) -> String {
        format!("ResolvePackageTask (for {:?})", self.name)
    }
}
