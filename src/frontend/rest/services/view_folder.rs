//! frontend/rest/services/view_folder.rs
//!
//! The /api/view-local-folder returns whether the path exists or not.
//! Side-effect: will open the folder in the default file manager if it exists.

use super::default_future;
use crate::frontend::rest::services::Future;
use crate::frontend::rest::services::Request;
use crate::frontend::rest::services::Response;
use crate::frontend::rest::services::WebService;

use hyper::header::{ContentLength, ContentType};

use crate::logging::LoggingErrors;
use crate::native::open_in_shell;

pub fn handle(service: &WebService, _: Request) -> Future {
    let framework = service.get_framework_read();
    let mut response = false;
    let path = framework.install_path.clone();
    if let Some(path) = path {
        response = true;
        open_in_shell(path.as_path());
    }

    let file = serde_json::to_string(&response)
        .log_expect("Failed to render JSON payload of installation status object");

    default_future(
        Response::new()
            .with_header(ContentLength(file.len() as u64))
            .with_header(ContentType::json())
            .with_body(file),
    )
}
