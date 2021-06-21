//! frontend/rest/services/default_path.rs
//!
//! The /api/default-path returns the default path for the application to install into.

use crate::frontend::rest::services::default_future;
use crate::frontend::rest::services::Future;
use crate::frontend::rest::services::Request;
use crate::frontend::rest::services::Response;
use crate::frontend::rest::services::WebService;

use hyper::header::{ContentLength, ContentType};

use crate::logging::LoggingErrors;

/// Struct used by serde to send a JSON payload to the client containing an optional value.
#[derive(Serialize)]
struct FileSelection {
    path: Option<String>,
}

pub fn handle(service: &WebService, _req: Request) -> Future {
    let path = { service.get_framework_read().get_default_path() };

    let response = FileSelection { path };

    let file = serde_json::to_string(&response)
        .log_expect("Failed to render JSON payload of default path object");

    default_future(
        Response::new()
            .with_header(ContentLength(file.len() as u64))
            .with_header(ContentType::json())
            .with_body(file),
    )
}
