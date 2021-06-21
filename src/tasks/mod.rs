//! Contains a framework for the processing of discrete Tasks, as well as
//! various implementations of it for installer-related tasks.

use std::fmt;
use std::fmt::Display;

use crate::installer::InstallerFramework;

use crate::sources::types::File;
use crate::sources::types::Version;

pub mod download_pkg;
pub mod ensure_only_instance;
pub mod install;
pub mod install_dir;
pub mod install_global_shortcut;
pub mod install_pkg;
pub mod install_shortcuts;
pub mod resolver;
pub mod save_database;
pub mod save_executable;
pub mod uninstall;
pub mod uninstall_global_shortcut;
pub mod uninstall_pkg;
pub mod uninstall_shortcuts;

/// An abstraction over the various parameters that can be passed around.
pub enum TaskParamType {
    None,
    /// Metadata about a file
    File(Version, File),
    /// Downloaded contents of a file
    FileContents(Version, File, Vec<u8>),
    /// List of shortcuts that have been generated
    GeneratedShortcuts(Vec<String>),
    /// Tells the runtime to break parsing other dependencies
    Break,
}

/// Specifies the relative ordering of a dependency with it's parent.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum TaskOrdering {
    /// This task should occur before the main process
    Pre,
    /// This task should occur after the main process. These have their results discarded.
    Post,
}

/// A dependency of a task with various properties.
pub struct TaskDependency {
    ordering: TaskOrdering,
    task: Box<dyn Task>,
}

impl TaskDependency {
    /// Builds a new dependency from the specified task.
    pub fn build(ordering: TaskOrdering, task: Box<dyn Task>) -> TaskDependency {
        TaskDependency { ordering, task }
    }
}

/// A message from a task.
pub enum TaskMessage<'a> {
    DisplayMessage(&'a str, f64),
    PackageInstalled,
}

/// A Task is a small, async task conforming to a fixed set of inputs/outputs.
pub trait Task {
    /// Executes this individual task, evaluating to the given Output result.
    ///
    /// Each dependency is given an indice in the inputted vector.
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String>;

    /// Returns a vector containing all dependencies that need to be executed
    /// before this task can function.
    fn dependencies(&self) -> Vec<TaskDependency>;

    /// Returns a short name used for formatting the dependency tree.
    fn name(&self) -> String;
}

/// The dependency tree allows for smart iteration on a Task struct.
pub struct DependencyTree {
    task: Box<dyn Task>,
    dependencies: Vec<(TaskOrdering, DependencyTree)>,
}

impl DependencyTree {
    /// Renders the dependency tree into a user-presentable string.
    fn render(&self) -> String {
        let mut buf = self.task.name();

        buf += "\n";

        for i in 0..self.dependencies.len() {
            let (order, dependency) = &(self.dependencies[i]);
            let dependencies = dependency.render();
            let dependencies = format!("{:?} {}", order, dependencies.trim());

            if i + 1 == self.dependencies.len() {
                buf += "└── ";
                buf += &dependencies.replace("\n", "\n    ");
            } else {
                buf += "├── ";
                buf += &dependencies.replace("\n", "\n│   ");
                buf += "\n";
            }
        }

        buf
    }

    /// Executes this pipeline.
    pub fn execute(
        &mut self,
        context: &mut InstallerFramework,
        messenger: &dyn Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        let total_tasks = (self.dependencies.len() + 1) as f64;

        let mut inputs = Vec::<TaskParamType>::with_capacity(self.dependencies.len());

        let mut count = 0;

        for (ordering, i) in &mut self.dependencies {
            if ordering != &TaskOrdering::Pre {
                continue;
            }

            let result = i.execute(context, &|msg: &TaskMessage| match *msg {
                TaskMessage::DisplayMessage(msg, progress) => {
                    messenger(&TaskMessage::DisplayMessage(
                        msg,
                        progress / total_tasks + (1.0 / total_tasks) * f64::from(count),
                    ))
                }
                _ => messenger(msg),
            })?;

            // Check to see if we skip matching other dependencies
            let do_break = match &result {
                TaskParamType::Break => true,
                _ => false,
            };

            inputs.push(result);
            count += 1;

            if do_break {
                break;
            }
        }

        let task_result = self
            .task
            .execute(inputs, context, &|msg: &TaskMessage| match *msg {
                TaskMessage::DisplayMessage(msg, progress) => {
                    messenger(&TaskMessage::DisplayMessage(
                        msg,
                        progress / total_tasks + (1.0 / total_tasks) * f64::from(count),
                    ))
                }
                _ => messenger(msg),
            })?;

        if let TaskParamType::Break = task_result {
            // We are done here
            return Ok(TaskParamType::Break);
        }

        for (ordering, i) in &mut self.dependencies {
            if ordering != &TaskOrdering::Post {
                continue;
            }

            let result = i.execute(context, &|msg: &TaskMessage| match *msg {
                TaskMessage::DisplayMessage(msg, progress) => {
                    messenger(&TaskMessage::DisplayMessage(
                        msg,
                        progress / total_tasks + (1.0 / total_tasks) * f64::from(count),
                    ))
                }
                _ => messenger(msg),
            })?;

            // Check to see if we skip matching other dependencies
            let do_break = match &result {
                TaskParamType::Break => true,
                _ => false,
            };

            count += 1;

            if do_break {
                break;
            }
        }

        Ok(task_result)
    }

    /// Builds a new pipeline from the specified task, iterating on dependencies.
    pub fn build(task: Box<dyn Task>) -> DependencyTree {
        let dependencies = task
            .dependencies()
            .into_iter()
            .map(|x| (x.ordering, DependencyTree::build(x.task)))
            .collect();

        DependencyTree { task, dependencies }
    }
}

impl Display for DependencyTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.render())
    }
}
