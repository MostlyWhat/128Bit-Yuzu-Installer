//! frontend/rest/services/config.rs
//!
//! The /api/config call returns the current installer framework configuration.
//!
//! This endpoint should be usable directly from a <script> tag during loading.

use crate::frontend::rest::services::Future;
use crate::frontend::rest::services::Request;
use crate::frontend::rest::services::Response;
use crate::frontend::rest::services::WebService;

use hyper::header::{ContentLength, ContentType};

use crate::logging::LoggingErrors;

use crate::config::Config;

use crate::http::build_async_client;

use futures::stream::Stream;
use futures::Future as _;

pub fn handle(service: &WebService, _req: Request) -> Future {
    let framework_url = {
        service
            .get_framework_read()
            .base_attributes
            .target_url
            .clone()
    };

    info!("Downloading configuration from {:?}...", framework_url);

    let framework = service.framework.clone();

    // Hyper doesn't allow for clients to do sync network operations in a async future.
    // This smallish pipeline joins the two together.
    Box::new(
        build_async_client()
            .log_expect("Failed to build async client")
            .get(&framework_url)
            .send()
            .map_err(|x| {
                error!("HTTP error while downloading configuration file: {:?}", x);
                hyper::Error::Incomplete
            })
            .and_then(|x| {
                x.into_body().concat2().map_err(|x| {
                    error!("HTTP error while parsing configuration file: {:?}", x);
                    hyper::Error::Incomplete
                })
            })
            .and_then(move |x| {
                let x = String::from_utf8(x.to_vec()).map_err(|x| {
                    error!("UTF-8 error while parsing configuration file: {:?}", x);
                    hyper::Error::Incomplete
                })?;

                let config = Config::from_toml_str(&x).map_err(|x| {
                    error!("Serde error while parsing configuration file: {:?}", x);
                    hyper::Error::Incomplete
                })?;

                let mut framework = framework
                    .write()
                    .log_expect("Failed to get write lock for framework");

                framework.config = Some(config);

                info!("Configuration file downloaded successfully.");

                let file = framework
                    .get_config()
                    .log_expect("Config should be loaded by now")
                    .to_json_str()
                    .log_expect("Failed to render JSON representation of config");

                Ok(Response::new()
                    .with_header(ContentLength(file.len() as u64))
                    .with_header(ContentType::json())
                    .with_body(file))
            }),
    )
}
