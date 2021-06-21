//! frontend/rest/services/uninstall.rs
//!
//! The /api/uninstall call uninstalls all packages.

use frontend::rest::services::default_future;
use frontend::rest::services::stream_progress;
use frontend::rest::services::Future;
use frontend::rest::services::Request;
use frontend::rest::services::WebService;

use logging::LoggingErrors;

use installer::InstallMessage;

pub fn handle(service: &WebService, _req: Request) -> Future {
    let framework = service.framework.clone();

    default_future(stream_progress(move |sender| {
        let mut framework = framework
            .write()
            .log_expect("InstallerFramework has been dirtied");

        if let Err(v) = framework.uninstall(&sender) {
            error!("Uninstall error occurred: {:?}", v);
            if let Err(v) = sender.send(InstallMessage::Error(v)) {
                error!("Failed to send uninstall error: {:?}", v);
            };
        }

        if let Err(v) = sender.send(InstallMessage::EOF) {
            error!("Failed to send EOF to client: {:?}", v);
        }
    }))
}
