//! Contains functions to help with logging.

use chrono;
use fern;
use log;

use std::fmt::Debug;
use std::io;

pub fn setup_logger(file_name: String) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(io::stdout())
        .chain(fern::log_file(file_name)?)
        .apply()?;
    Ok(())
}

/// An additional wrapper usable on Result/Option types to add logging to the regular
/// panic route.
pub trait LoggingErrors<T>
where
    Self: Sized,
{
    /// Unwraps this object. See `unwrap()`.
    #[inline]
    fn log_unwrap(self) -> T {
        self.log_expect("Failed to unwrap")
    }

    /// Unwraps this object, with a specified error message on failure. See `expect()`.
    fn log_expect(self, msg: &str) -> T;
}

impl<T, E: Debug> LoggingErrors<T> for Result<T, E> {
    #[inline]
    fn log_expect(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(v) => {
                error!("Fatal error: {}: {:?}", msg, v);
                panic!("Expectation failed");
            }
        }
    }
}

impl<T> LoggingErrors<T> for Option<T> {
    #[inline]
    fn log_expect(self, msg: &str) -> T {
        match self {
            Some(v) => v,
            None => {
                error!("Fatal error: {}", msg);
                panic!("Expectation failed");
            }
        }
    }
}
