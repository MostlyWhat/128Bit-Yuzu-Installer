//! patreon.rs
//!
//! Contains the yuzu-emu core API implementation of a release source.

use http::build_client;
use reqwest::header::USER_AGENT;
use reqwest::StatusCode;
use sources::types::*;

pub struct PatreonReleases {}

/// The configuration for this release.
#[derive(Serialize, Deserialize)]
struct PatreonConfig {
    repo: String,
}

impl PatreonReleases {
    pub fn new() -> Self {
        PatreonReleases {}
    }
}

impl ReleaseSource for PatreonReleases {
    fn get_current_releases(&self, _config: &TomlValue) -> Result<Vec<Release>, String> {
        let config: PatreonConfig = match _config.clone().try_into() {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to parse release config: {:?}", v)),
        };

        let mut results: Vec<Release> = Vec::new();

        // Build the HTTP client up
        let client = build_client()?;
        let mut response = client
            .get(&format!(
                "https://api.yuzu-emu.org/downloads/{}/",
                config.repo
            ))
            .header(USER_AGENT, "liftinstall (j-selby)")
            .send()
            .map_err(|x| format!("Error while sending HTTP request: {:?}", x))?;

        match response.status() {
            StatusCode::OK => {}
            StatusCode::FORBIDDEN => {
                return Err("You are not eligible to download this release".to_string());
            }
            _ => {
                return Err(format!("Bad status code: {:?}.", response.status()));
            }
        }

        let body = response
            .text()
            .map_err(|x| format!("Failed to decode HTTP response body: {:?}", x))?;

        let result: serde_json::Value = serde_json::from_str(&body)
            .map_err(|x| format!("Failed to parse response: {:?}", x))?;

        // Parse JSON from server
        let mut files = Vec::new();

        let id: u64 = match result["version"].as_u64() {
            Some(v) => v,
            None => return Err("JSON payload missing information about ID".to_string()),
        };

        let downloads = match result["files"].as_array() {
            Some(v) => v,
            None => return Err("JSON payload not an array".to_string()),
        };

        for file in downloads.iter() {
            let string = match file["name"].as_str() {
                Some(v) => v,
                None => {
                    return Err("JSON payload missing information about release name".to_string());
                }
            };

            let url = match file["url"].as_str() {
                Some(v) => v,
                None => {
                    return Err("JSON payload missing information about release URL".to_string());
                }
            };

            files.push(File {
                name: string.to_string(),
                url: url.to_string(),
                requires_authorization: true,
            });
        }

        results.push(Release {
            version: Version::new_number(id),
            files,
        });
        Ok(results)
    }
}
