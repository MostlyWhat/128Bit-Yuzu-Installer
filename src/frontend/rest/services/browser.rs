//! frontend/rest/services/browser.rs
//!
//! Launches the user's web browser on request from the frontend.

use frontend::rest::services::Future as InternalFuture;
use frontend::rest::services::{Request, Response, WebService};
use futures::{Future, Stream};
use hyper::header::ContentType;
use logging::LoggingErrors;
use std::collections::HashMap;
use url::form_urlencoded;

pub fn handle(_service: &WebService, req: Request) -> InternalFuture {
    Box::new(req.body().concat2().map(move |body| {
        let req = form_urlencoded::parse(body.as_ref())
            .into_owned()
            .collect::<HashMap<String, String>>();
        if webbrowser::open(req.get("url").log_expect("No URL to launch")).is_ok() {
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
