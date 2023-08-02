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
        let mut force_install = false;
        let mut install_desktop_shortcut = false;

        // Transform results into just an array of stuff to install
        for (key, value) in &results {
            if key == "path" {
                path = Some(value.to_owned());
                continue;
            } else if key == "installDesktopShortcut" {
                info!("Found installDesktopShortcut {:?}", value);
                install_desktop_shortcut = value == "true";
                continue;
            }

            if key == "mode" && value == "force" {
                force_install = true;
                continue;
            }

            if value == "true" {
                to_install.push(key.to_owned());
            }
        }

        if !install_desktop_shortcut {
            let framework_ref = framework
                .read()
                .log_expect("InstallerFramework has been dirtied");
            install_desktop_shortcut = framework_ref.preexisting_install
                && framework_ref
                    .database
                    .packages
                    .first()
                    .and_then(|x| Some(x.shortcuts.len() > 1))
                    .unwrap_or(false);
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

            if let Err(v) = framework.install(
                to_install,
                &sender,
                new_install,
                install_desktop_shortcut,
                force_install,
            ) {
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
