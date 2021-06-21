//! frontend/rest/services/install.rs
//!
//! The /api/install call installs a set of packages dictated by a POST request.

use crate::frontend::rest::services::stream_progress;
use crate::frontend::rest::services::Future;
use crate::frontend::rest::services::Request;
use crate::frontend::rest::services::WebService;

use crate::logging::LoggingErrors;

use crate::installer::InstallMessage;

use futures::future::Future as _;
use futures::stream::Stream;

use url::form_urlencoded;

use std::collections::HashMap;

pub fn handle(service: &WebService, req: Request) -> Future {
    let framework = service.framework.clone();

    Box::new(req.body().concat2().map(move |b| {
        let results = form_urlencoded::parse(b.as_ref())
            .into_owned()
            .collect::<HashMap<String, String>>();

        let mut to_install = Vec::new();
        let mut path: Option<String> = None;

        // Transform results into just an array of stuff to install
        for (key, value) in &results {
            if key == "path" {
                path = Some(value.to_owned());
                continue;
            }

            if value == "true" {
                to_install.push(key.to_owned());
            }
        }

        // The frontend always provides this
        let path =
            path.log_expect("No path specified by frontend when one should have already existed");

        stream_progress(move |sender| {
            let mut framework = framework
                .write()
                .log_expect("InstallerFramework has been dirtied");

            let new_install = !framework.preexisting_install;
            if new_install {
                framework.set_install_dir(&path);
            }

            if let Err(v) = framework.install(to_install, &sender, new_install) {
                error!("Install error occurred: {:?}", v);
                if let Err(v) = sender.send(InstallMessage::Error(v)) {
                    error!("Failed to send install error: {:?}", v);
                }
            }

            if let Err(v) = sender.send(InstallMessage::EOF) {
                error!("Failed to send EOF to client: {:?}", v);
            }
        })
    }))
}
