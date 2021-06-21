//! frontend/rest/services/dark_mode.rs
//!
//! This call returns if dark mode is enabled on the system currently.

use crate::frontend::rest::services::default_future;
use crate::frontend::rest::services::Future;
use crate::frontend::rest::services::Request;
use crate::frontend::rest::services::Response;
use crate::frontend::rest::services::WebService;

use hyper::header::{ContentLength, ContentType};

use crate::logging::LoggingErrors;

use crate::native::is_dark_mode_active;

pub fn handle(_service: &WebService, _req: Request) -> Future {
    let file = serde_json::to_string(&is_dark_mode_active())
        .log_expect("Failed to render JSON payload of installation status object");

    default_future(
        Response::new()
            .with_header(ContentLength(file.len() as u64))
            .with_header(ContentType::json())
            .with_body(file),
    )
}
