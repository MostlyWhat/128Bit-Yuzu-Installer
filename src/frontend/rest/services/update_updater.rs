//! frontend/rest/services/update_updater.rs
//!
//! The /api/update-updater call attempts to update the currently running updater.

use crate::frontend::rest::services::default_future;
use crate::frontend::rest::services::stream_progress;
use crate::frontend::rest::services::Future;
use crate::frontend::rest::services::Request;
use crate::frontend::rest::services::WebService;

use crate::logging::LoggingErrors;

use crate::installer::InstallMessage;

pub fn handle(service: &WebService, _req: Request) -> Future {
    let framework = service.framework.clone();

    default_future(stream_progress(move |sender| {
        let mut framework = framework
            .write()
            .log_expect("InstallerFramework has been dirtied");

        if let Err(v) = framework.update_updater(&sender) {
            error!("Self-update error occurred: {:?}", v);
            if let Err(v) = sender.send(InstallMessage::Error(v)) {
                error!("Failed to send self-update error: {:?}", v);
            };
        }

        if let Err(v) = sender.send(InstallMessage::EOF) {
            error!("Failed to send EOF to client: {:?}", v);
        }
    }))
}
