//! frontend/rest/services/packages.rs
//!
//! The /api/packages call returns all the currently installed packages.

use frontend::rest::services::default_future;
use frontend::rest::services::encapsulate_json;
use frontend::rest::services::Future;
use frontend::rest::services::Request;
use frontend::rest::services::Response;
use frontend::rest::services::WebService;

use hyper::header::{ContentLength, ContentType};

use logging::LoggingErrors;

pub fn handle(service: &WebService, _req: Request) -> Future {
    let framework = service.get_framework_read();

    let file = encapsulate_json(
        "packages",
        &serde_json::to_string(&framework.database)
            .log_expect("Failed to render JSON representation of database"),
    );

    default_future(
        Response::new()
            .with_header(ContentLength(file.len() as u64))
            .with_header(ContentType::json())
            .with_body(file),
    )
}
