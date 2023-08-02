//! frontend/rest/services/verify_path.rs
//!
//! The /api/verify-path returns whether the path exists or not.

use crate::frontend::rest::services::Future;
use crate::frontend::rest::services::Request;
use crate::frontend::rest::services::Response;
use crate::frontend::rest::services::WebService;
use url::form_urlencoded;

use hyper::header::{ContentLength, ContentType};

use futures::future::Future as _;
use futures::stream::Stream;

use crate::logging::LoggingErrors;

use std::collections::HashMap;
use std::path::PathBuf;

/// Struct used by serde to send a JSON payload to the client containing an optional value.
#[derive(Serialize)]
struct VerifyResponse {
    exists: bool,
}

pub fn handle(_service: &WebService, req: Request) -> Future {
    Box::new(req.body().concat2().map(move |b| {
        let results = form_urlencoded::parse(b.as_ref())
            .into_owned()
            .collect::<HashMap<String, String>>();
        let mut exists = false;
        if let Some(path) = results.get("path") {
            let path = PathBuf::from(path);
            exists = path.is_dir();
        }

        let response = VerifyResponse { exists };

        let file = serde_json::to_string(&response)
            .log_expect("Failed to render JSON payload of default path object");

        Response::new()
            .with_header(ContentLength(file.len() as u64))
            .with_header(ContentType::json())
            .with_body(file)
    }))
}
