//! frontend/rest/services/attributes.rs
//!
//! The /api/attr call returns an executable script containing session variables.

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
        "base_attributes",
        &framework
            .base_attributes
            .to_json_str()
            .log_expect("Failed to render JSON representation of config"),
    );

    default_future(
        Response::new()
            .with_header(ContentLength(file.len() as u64))
            .with_header(ContentType::json())
            .with_body(file),
    )
}
