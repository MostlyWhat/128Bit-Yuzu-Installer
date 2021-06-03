//! rest.rs
//!
//! Provides a HTTP/REST server for both frontend<->backend communication, as well
//! as talking to external applications.

use serde_json;

use futures::future;
use futures::Future;
use futures::Sink;
use futures::Stream;

use hyper::header::{ContentLength, ContentType};
use hyper::server::{Http, Request, Response, Service};
use hyper::{self, Error as HyperError, Get, Post, StatusCode};

use url::form_urlencoded;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::exit;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread::{self, JoinHandle};

use assets;

use installer::InstallMessage;
use installer::InstallerFramework;

use logging::LoggingErrors;

use http;

use config::Config;

use native;

#[derive(Serialize)]
struct FileSelection {
    path: Option<String>,
}

/// Acts as a communication mechanism between the Hyper WebService and the rest of the
/// application.
pub struct WebServer {
    _handle: JoinHandle<()>,
}

impl WebServer {
    /// Creates a new web server with the specified address.
    pub fn with_addr(
        framework: Arc<RwLock<InstallerFramework>>,
        addr: SocketAddr,
    ) -> Result<Self, HyperError> {
        let handle = thread::spawn(move || {
            let server = Http::new()
                .bind(&addr, move || {
                    Ok(WebService {
                        framework: framework.clone(),
                    })
                }).log_expect("Failed to bind to port");

            server.run().log_expect("Failed to run HTTP server");
        });

        Ok(WebServer { _handle: handle })
    }
}

/// Holds internal state for Hyper
struct WebService {
    framework: Arc<RwLock<InstallerFramework>>,
}

impl Service for WebService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    /// HTTP request handler
    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(future::ok(match (req.method(), req.path()) {
            // This endpoint should be usable directly from a <script> tag during loading.
            (&Get, "/api/attrs") => {
                let framework = self
                    .framework
                    .read()
                    .log_expect("InstallerFramework has been dirtied");

                let file = encapsulate_json(
                    "base_attributes",
                    &framework
                        .base_attributes
                        .to_json_str()
                        .log_expect("Failed to render JSON representation of config"),
                );

                Response::<hyper::Body>::new()
                    .with_header(ContentLength(file.len() as u64))
                    .with_header(ContentType::json())
                    .with_body(file)
            }
            // Returns the web config loaded
            (&Get, "/api/config") => {
                let mut framework = self
                    .framework
                    .write()
                    .log_expect("InstallerFramework has been dirtied");

                info!(
                    "Downloading configuration from {:?}...",
                    framework.base_attributes.target_url
                );

                match http::download_text(&framework.base_attributes.target_url)
                    .map(|x| Config::from_toml_str(&x))
                {
                    Ok(Ok(config)) => {
                        framework.config = Some(config.clone());

                        info!("Configuration file downloaded successfully.");

                        let file = framework
                            .get_config()
                            .log_expect("Config should be loaded by now")
                            .to_json_str()
                            .log_expect("Failed to render JSON representation of config");

                        Response::<hyper::Body>::new()
                            .with_header(ContentLength(file.len() as u64))
                            .with_header(ContentType::json())
                            .with_body(file)
                    }
                    Ok(Err(v)) => {
                        error!("Bad configuration file: {:?}", v);

                        Response::<hyper::Body>::new()
                            .with_status(StatusCode::ServiceUnavailable)
                            .with_header(ContentType::plaintext())
                            .with_body("Bad HTTP response")
                    }
                    Err(v) => {
                        error!(
                            "General connectivity error while downloading config: {:?}",
                            v
                        );

                        Response::<hyper::Body>::new()
                            .with_status(StatusCode::ServiceUnavailable)
                            .with_header(ContentLength(v.len() as u64))
                            .with_header(ContentType::plaintext())
                            .with_body(v)
                    }
                }
            }
            // This endpoint should be usable directly from a <script> tag during loading.
            (&Get, "/api/packages") => {
                let framework = self
                    .framework
                    .read()
                    .log_expect("InstallerFramework has been dirtied");

                let file = encapsulate_json(
                    "packages",
                    &serde_json::to_string(&framework.database)
                        .log_expect("Failed to render JSON representation of database"),
                );

                Response::<hyper::Body>::new()
                    .with_header(ContentLength(file.len() as u64))
                    .with_header(ContentType::json())
                    .with_body(file)
            }
            // Returns the default path for a installation
            (&Get, "/api/default-path") => {
                let framework = self
                    .framework
                    .read()
                    .log_expect("InstallerFramework has been dirtied");
                let path = framework.get_default_path();

                let response = FileSelection { path };

                let file = serde_json::to_string(&response)
                    .log_expect("Failed to render JSON payload of default path object");

                Response::<hyper::Body>::new()
                    .with_header(ContentLength(file.len() as u64))
                    .with_header(ContentType::json())
                    .with_body(file)
            }
            // Immediately exits the application
            (&Get, "/api/exit") => {
                let framework = self
                    .framework
                    .read()
                    .log_expect("InstallerFramework has been dirtied");

                if let Some(ref v) = framework.launcher_path {
                    Command::new(v)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .log_expect("Unable to start child process");
                }

                if framework.burn_after_exit {
                    native::burn_on_exit(&framework.base_attributes.name);
                }

                exit(0);
            }
            // Gets properties such as if the application is in maintenance mode
            (&Get, "/api/installation-status") => {
                let framework = self
                    .framework
                    .read()
                    .log_expect("InstallerFramework has been dirtied");

                let response = framework.get_installation_status();

                let file = serde_json::to_string(&response)
                    .log_expect("Failed to render JSON payload of installation status object");

                Response::<hyper::Body>::new()
                    .with_header(ContentLength(file.len() as u64))
                    .with_header(ContentType::json())
                    .with_body(file)
            }
            // Streams the installation of a particular set of packages
            (&Post, "/api/uninstall") => {
                // We need to bit of pipelining to get this to work
                let framework = self.framework.clone();

                return Box::new(req.body().concat2().map(move |_b| {
                    let (sender, receiver) = channel();
                    let (tx, rx) = hyper::Body::pair();

                    // Startup a thread to do this operation for us
                    thread::spawn(move || {
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
                    });

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

                    Response::<hyper::Body>::new()
                        //.with_header(ContentLength(file.len() as u64))
                        .with_header(ContentType::plaintext())
                        .with_body(rx)
                }));
            }
            // Updates the installer
            (&Post, "/api/update-updater") => {
                // We need to bit of pipelining to get this to work
                let framework = self.framework.clone();

                return Box::new(req.body().concat2().map(move |_b| {
                    let (sender, receiver) = channel();
                    let (tx, rx) = hyper::Body::pair();

                    // Startup a thread to do this operation for us
                    thread::spawn(move || {
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
                    });

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

                    Response::<hyper::Body>::new()
                        //.with_header(ContentLength(file.len() as u64))
                        .with_header(ContentType::plaintext())
                        .with_body(rx)
                }));
            }
            // Streams the installation of a particular set of packages
            (&Post, "/api/start-install") => {
                // We need to bit of pipelining to get this to work
                let framework = self.framework.clone();

                return Box::new(req.body().concat2().map(move |b| {
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
                    let path = path.log_expect(
                        "No path specified by frontend when one should have already existed",
                    );

                    let (sender, receiver) = channel();
                    let (tx, rx) = hyper::Body::pair();

                    // Startup a thread to do this operation for us
                    thread::spawn(move || {
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
                    });

                    // Spawn a thread for transforming messages to chunk messages
                    thread::spawn(move || {
                        let mut tx = tx;
                        loop {
                            let mut panic_after_finish = false;

                            let response = match receiver
                                .recv() {
                                Ok(v) => v,
                                Err(v) => {
                                    error!("Queue message failed: {:?}", v);
                                    panic_after_finish = true;
                                    InstallMessage::Error("Internal error".to_string())
                                }
                            };

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

                            if panic_after_finish {
                                panic!("Failed to read from queue (flushed error message successfully)");
                            }
                        }
                    });

                    Response::<hyper::Body>::new()
                        //.with_header(ContentLength(file.len() as u64))
                        .with_header(ContentType::plaintext())
                        .with_body(rx)
                }));
            }

            // Static file handler
            (&Get, _) => {
                // At this point, we have a web browser client. Search for a index page
                // if needed
                let mut path: String = req.path().to_owned();
                if path.ends_with('/') {
                    path += "index.html";
                }

                match assets::file_from_string(&path) {
                    Some((content_type, file)) => {
                        let content_type = ContentType(content_type.parse().log_expect(
                            "Failed to parse content type into correct representation",
                        ));
                        Response::<hyper::Body>::new()
                            .with_header(ContentLength(file.len() as u64))
                            .with_header(content_type)
                            .with_body(file)
                    }
                    None => Response::new().with_status(StatusCode::NotFound),
                }
            }
            // Fallthrough for POST/PUT/CONNECT/...
            _ => Response::new().with_status(StatusCode::NotFound),
        }))
    }
}

/// Encapsulates JSON as a injectable Javascript script.
fn encapsulate_json(field_name: &str, json: &str) -> String {
    format!("var {} = {};", field_name, json)
}
