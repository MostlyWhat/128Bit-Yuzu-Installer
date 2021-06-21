//! frontend/rest/services/mod.rs
//!
//! Provides all services used by the REST server.

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::installer::{InstallMessage, InstallerFramework};

use hyper::server::Service;
use hyper::{Method, StatusCode};

use crate::logging::LoggingErrors;

use std::sync::mpsc::{channel, Sender};

use std::thread;

use hyper::header::ContentType;

use futures::future::Future as _;
use futures::sink::Sink;

mod attributes;
mod config;
mod dark_mode;
mod default_path;
mod exit;
mod install;
mod installation_status;
mod packages;
mod static_files;
mod uninstall;
mod update_updater;

/// Expected incoming Request format from Hyper.
pub type Request = hyper::server::Request;

/// Completed response type returned by the server.
pub type Response = hyper::server::Response;

/// Error type returned by the server.
pub type Error = hyper::Error;

/// The return type used by function calls to the web server.
pub type Future = Box<dyn futures::Future<Item = Response, Error = Error>>;

/// If advanced functionality is not needed, return a default instant future.
pub fn default_future(response: Response) -> Future {
    Box::new(futures::future::ok(response))
}

/// Encapsulates JSON as a injectable Javascript script.
pub fn encapsulate_json(field_name: &str, json: &str) -> String {
    format!("var {} = {};", field_name, json)
}

/// Streams messages from a specified task to the client in a thread.
pub fn stream_progress<F: 'static>(function: F) -> Response
where
    F: FnOnce(Sender<InstallMessage>) -> () + Send,
{
    let (sender, receiver) = channel();
    let (tx, rx) = hyper::Body::pair();

    // Startup a thread to do this operation for us
    thread::spawn(move || function(sender));

    // Spawn a thread for transforming messages to chunk messages
    thread::spawn(move || {
        let mut tx = tx;
        loop {
            let response = receiver
                .recv()
                .log_expect("Failed to receive message from runner thread");

            if let InstallMessage::EOF = response {
                break;
            }

            let mut response = serde_json::to_string(&response)
                .log_expect("Failed to render JSON logging response payload");
            response.push('\n');
            tx = tx
                .send(Ok(response.into_bytes().into()))
                .wait()
                .log_expect("Failed to write JSON response payload to client");
        }
    });

    Response::new()
        .with_header(ContentType::plaintext())
        .with_body(rx)
}

/// Holds internal state for a single Hyper instance. Multiple will exist.
pub struct WebService {
    framework: Arc<RwLock<InstallerFramework>>,
}

impl WebService {
    /// Returns an immutable reference to the framework. May block.
    pub fn get_framework_read(&self) -> RwLockReadGuard<InstallerFramework> {
        self.framework
            .read()
            .log_expect("InstallerFramework has been dirtied")
    }

    /// Returns an immutable reference to the framework. May block.
    pub fn get_framework_write(&self) -> RwLockWriteGuard<InstallerFramework> {
        self.framework
            .write()
            .log_expect("InstallerFramework has been dirtied")
    }

    /// Creates a new WebService instance. Multiple are likely going to exist at once,
    /// so create a lock to hold this.
    pub fn new(framework: Arc<RwLock<InstallerFramework>>) -> WebService {
        WebService { framework }
    }
}

impl Service for WebService {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = Future;

    fn call(&self, req: Self::Request) -> Self::Future {
        let method = req.method().clone();
        let path = req.path().to_string();

        match (method, path.as_str()) {
            (Method::Get, "/api/attrs") => attributes::handle(self, req),
            (Method::Get, "/api/config") => config::handle(self, req),
            (Method::Get, "/api/dark-mode") => dark_mode::handle(self, req),
            (Method::Get, "/api/default-path") => default_path::handle(self, req),
            (Method::Get, "/api/exit") => exit::handle(self, req),
            (Method::Get, "/api/packages") => packages::handle(self, req),
            (Method::Get, "/api/installation-status") => installation_status::handle(self, req),
            (Method::Post, "/api/start-install") => install::handle(self, req),
            (Method::Post, "/api/uninstall") => uninstall::handle(self, req),
            (Method::Post, "/api/update-updater") => update_updater::handle(self, req),
            (Method::Get, _) => static_files::handle(self, req),
            e => {
                info!("Returned 404 for {:?}", e);
                default_future(Response::new().with_status(StatusCode::NotFound))
            }
        }
    }
}
