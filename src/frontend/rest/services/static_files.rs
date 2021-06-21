//! frontend/rest/services/static_files.rs
//!
//! The static files call returns static files embedded within the executable.
//!
//! e.g. index.html, main.js, ...

use crate::frontend::rest::assets;

use crate::frontend::rest::services::default_future;
use crate::frontend::rest::services::Future;
use crate::frontend::rest::services::Request;
use crate::frontend::rest::services::Response;
use crate::frontend::rest::services::WebService;

use hyper::header::{ContentLength, ContentType};
use hyper::StatusCode;

use crate::logging::LoggingErrors;

pub fn handle(_service: &WebService, req: Request) -> Future {
    // At this point, we have a web browser client. Search for a index page
    // if needed
    let mut path: String = req.path().to_owned();
    if path.ends_with('/') {
        path += "index.html";
    }

    default_future(match assets::file_from_string(&path) {
        Some((content_type, file)) => {
            let content_type = ContentType(
                content_type
                    .parse()
                    .log_expect("Failed to parse content type into correct representation"),
            );
            Response::new()
                .with_header(ContentLength(file.len() as u64))
                .with_header(content_type)
                .with_body(file)
        }
        None => Response::new().with_status(StatusCode::NotFound),
    })
}
