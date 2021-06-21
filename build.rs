#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
extern crate cc;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

extern crate which;

use std::env;
use std::path::PathBuf;

use std::fs::copy;
use std::fs::File;

use std::io::Read;
use std::process::Command;

use std::env::consts::OS;

/// Describes the application itself.
#[derive(Debug, Deserialize)]
pub struct BaseAttributes {
    pub name: String,
    pub target_url: String,
}

#[cfg(windows)]
fn handle_binary(config: &BaseAttributes) {
    let mut res = winres::WindowsResource::new();
    res.set_icon("ui/public/favicon.ico");
    res.set(
        "FileDescription",
        &format!("Interactive installer for {}", config.name),
    );
    res.set("ProductName", &format!("{} installer", config.name));
    res.set(
        "OriginalFilename",
        &format!("{}_installer.exe", config.name),
    );
    res.compile().expect("Failed to generate metadata");

    cc::Build::new()
        .cpp(true)
        .define("_WIN32_WINNT", Some("0x0600"))
        .define("WINVER", Some("0x0600"))
        .file("src/native/interop.cpp")
        .compile("interop");
}

#[cfg(not(windows))]
fn handle_binary(_config: &BaseAttributes) {}

fn main() {
    let output_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let current_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ui_dir = current_dir.join("ui");

    let os = OS.to_lowercase();

    // Find target config
    let target_config = PathBuf::from(format!("bootstrap.{}.toml", os));

    if !target_config.exists() {
        panic!(
            "There is no config file specified for the platform: {:?}. \
             Create a file named \"bootstrap.{}.toml\" in the root directory.",
            os, os
        );
    }

    // Read in the config for our own purposes
    let file_contents = {
        let mut file = File::open(&target_config).expect("Unable to open config file");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .expect("Unable to read config file contents");
        buf
    };

    let config: BaseAttributes =
        toml::from_slice(&file_contents).expect("Unable to parse config file");
    handle_binary(&config);

    // Copy for the main build
    copy(&target_config, output_dir.join("bootstrap.toml")).expect("Unable to copy config file");

    let yarn_binary =
        which::which("yarn").expect("Failed to find yarn - please go ahead and install it!");

    // Build and deploy frontend files
    Command::new(&yarn_binary)
        .arg("--version")
        .spawn()
        .expect("Yarn could not be launched");
    Command::new(&yarn_binary)
        .arg("--cwd")
        .arg(ui_dir.to_str().expect("Unable to covert path"))
        .spawn()
        .unwrap()
        .wait()
        .expect("Unable to install Node.JS dependencies using Yarn");
    let return_code = Command::new(&yarn_binary)
        .args(&[
            "--cwd",
            ui_dir.to_str().expect("Unable to covert path"),
            "run",
            "build",
            "--dest",
            output_dir
                .join("static")
                .to_str()
                .expect("Unable to convert path"),
        ])
        .status()
        .expect("Unable to build frontend assets using Webpack");
    assert!(return_code.success());
}
