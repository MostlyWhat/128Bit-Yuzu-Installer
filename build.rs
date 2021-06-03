extern crate walkdir;

#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
extern crate cc;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use walkdir::WalkDir;

use std::env;
use std::path::PathBuf;

use std::fs::copy;
use std::fs::create_dir_all;
use std::fs::File;

use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

use std::env::consts::OS;

const FILES_TO_PREPROCESS: &'static [&'static str] = &["helpers.js", "views.js"];

/// Describes the application itself.
#[derive(Debug, Deserialize)]
pub struct BaseAttributes {
    pub name: String,
    pub target_url: String,
}

#[cfg(windows)]
fn handle_binary(config: &BaseAttributes) {
    let mut res = winres::WindowsResource::new();
    res.set_icon("static/favicon.ico");
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
        .file("src/native/interop.cpp")
        .compile("interop");
}

#[cfg(not(windows))]
fn handle_binary(_config: &BaseAttributes) {}

fn main() {
    let output_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

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

    // Copy files from static/ to build dir
    for entry in WalkDir::new("static") {
        let entry = entry.expect("Unable to read output directory");

        let output_file = output_dir.join(entry.path());

        if entry.path().is_dir() {
            create_dir_all(output_file).expect("Unable to create dir");
        } else {
            let filename = entry
                .path()
                .file_name()
                .expect("Unable to parse filename")
                .to_str()
                .expect("Unable to convert to string");

            if FILES_TO_PREPROCESS.contains(&filename) {
                // Do basic preprocessing - transcribe template string
                let source = BufReader::new(File::open(entry.path()).expect("Unable to copy file"));
                let mut target = File::create(output_file).expect("Unable to copy file");

                let mut is_template_string = false;

                for line in source.lines() {
                    let line = line.expect("Unable to read line from JS file");

                    let mut is_break = false;
                    let mut is_quote = false;

                    let mut output_line = String::new();

                    if is_template_string {
                        output_line += "\"";
                    }

                    for c in line.chars() {
                        if c == '\\' {
                            is_break = true;
                            output_line.push('\\');
                            continue;
                        }

                        if (c == '\"' || c == '\'') && !is_break && !is_template_string {
                            is_quote = !is_quote;
                        }

                        if c == '`' && !is_break && !is_quote {
                            output_line += "\"";
                            is_template_string = !is_template_string;
                            continue;
                        }

                        if c == '"' && !is_break && is_template_string {
                            output_line += "\\\"";
                            continue;
                        }

                        is_break = false;
                        output_line.push(c);
                    }

                    if is_template_string {
                        output_line += "\" +";
                    }

                    output_line.push('\n');

                    target
                        .write(output_line.as_bytes())
                        .expect("Unable to write line");
                }
            } else {
                copy(entry.path(), output_file).expect("Unable to copy file");
            }
        }
    }
}
