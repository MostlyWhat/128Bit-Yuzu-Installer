//! frontend/rest/server.rs
//!
//! Contains the over-arching server object + methods to manipulate it.

use crate::frontend::rest::services::WebService;

use crate::installer::InstallerFramework;

use crate::logging::LoggingErrors;

use hyper::server::Http;

use std::sync::{Arc, RwLock};

use std::net::{SocketAddr, TcpListener, ToSocketAddrs};

use std::thread;
use std::thread::JoinHandle;

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
    ) -> Result<Self, hyper::Error> {
        let handle = thread::spawn(move || {
            let server = Http::new()
                .bind(&addr, move || Ok(WebService::new(framework.clone())))
                .log_expect("Failed to bind to port");

            server.run().log_expect("Failed to run HTTP server");
        });

        Ok(WebServer { _handle: handle })
    }
}

/// Spawns a server instance on all local interfaces.
///
/// Returns server instances + http address of service running.
pub fn spawn_servers(framework: Arc<RwLock<InstallerFramework>>) -> (Vec<WebServer>, String) {
    // Firstly, allocate us an epidermal port
    let target_port = {
        let listener = TcpListener::bind("127.0.0.1:0")
            .log_expect("At least one local address should be free");
        listener
            .local_addr()
            .log_expect("Should be able to pull address from listener")
            .port()
    };

    // Now, iterate over all ports
    let addresses = "localhost:0"
        .to_socket_addrs()
        .log_expect("No localhost address found");

    let mut instances = Vec::with_capacity(addresses.len());
    let mut http_address = None;

    // Startup HTTP server for handling the web view
    for mut address in addresses {
        address.set_port(target_port);

        let server = WebServer::with_addr(framework.clone(), address)
            .log_expect("Failed to bind to address");

        info!("Spawning server instance @ {:?}", address);

        http_address = Some(address);

        instances.push(server);
    }

    let http_address = http_address.log_expect("No HTTP address found");

    (
        instances,
        format!("http://localhost:{}", http_address.port()),
    )
}
