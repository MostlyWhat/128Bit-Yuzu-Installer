//! frontend/rest/services/browser.rs
//!
//! Launches the user's web browser on request from the frontend.

use crate::frontend::rest::services::Future as InternalFuture;
use crate::frontend::rest::services::{Request, Response, WebService};
use crate::logging::LoggingErrors;
use futures::{Future, Stream};
use hyper::header::ContentType;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenRequest {
    url: String,
}

pub fn handle(_service: &WebService, req: Request) -> InternalFuture {
    Box::new(req.body().concat2().map(move |body| {
        let req: OpenRequest = serde_json::from_slice(&body).log_expect("Malformed request");
        if webbrowser::open(&req.url).is_ok() {
            Response::new()
                .with_status(hyper::Ok)
                .with_header(ContentType::json())
                .with_body("{}")
        } else {
            Response::new()
                .with_status(hyper::BadRequest)
                .with_header(ContentType::json())
                .with_body("{}")
        }
    }))
}
