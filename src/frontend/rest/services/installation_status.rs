//! frontend/rest/services/installation_status.rs
//!
//! The /api/installation-status call returns metadata relating to the current status of
//! the installation.
//!
//! e.g. if the application is in maintenance mode

use frontend::rest::services::default_future;
use frontend::rest::services::Future;
use frontend::rest::services::Request;
use frontend::rest::services::Response;
use frontend::rest::services::WebService;

use hyper::header::{ContentLength, ContentType};

use logging::LoggingErrors;

pub fn handle(service: &WebService, _req: Request) -> Future {
    let framework = service.get_framework_read();

    let response = framework.get_installation_status();

    let file = serde_json::to_string(&response)
        .log_expect("Failed to render JSON payload of installation status object");

    default_future(
        Response::new()
            .with_header(ContentLength(file.len() as u64))
            .with_header(ContentType::json())
            .with_body(file),
    )
}
