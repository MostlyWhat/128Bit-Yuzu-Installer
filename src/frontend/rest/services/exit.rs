//! frontend/rest/services/exit.rs
//!
//! The /api/exit closes down the application.

use crate::frontend::rest::services::default_future;
use crate::frontend::rest::services::Future;
use crate::frontend::rest::services::Request;
use crate::frontend::rest::services::Response;
use crate::frontend::rest::services::WebService;

use hyper::header::ContentType;
use hyper::StatusCode;

use std::process::exit;

pub fn handle(service: &WebService, _req: Request) -> Future {
    match service.get_framework_write().shutdown() {
        Ok(_) => {
            exit(0);
        }
        Err(e) => {
            error!("Failed to complete framework shutdown: {:?}", e);

            default_future(
                Response::new()
                    .with_status(StatusCode::InternalServerError)
                    .with_header(ContentType::plaintext())
                    .with_body(format!("Failed to complete framework shutdown - {}", e)),
            )
        }
    }
}
