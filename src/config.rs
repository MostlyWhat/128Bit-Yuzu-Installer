//! config.rs
//!
//! Contains Config structures, as well as means of serialising them.

use toml;
use toml::de::Error as TomlError;

use serde_json::{self, Error as SerdeError};

use crate::sources::get_by_name;
use crate::sources::types::Release;

/// Description of the source of a package.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageSource {
    pub name: String,
    #[serde(rename = "match")]
    pub match_regex: String,
    pub config: toml::Value,
}

/// Describes if/how a shortcut should be built for a package.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageShortcut {
    pub name: String,
    pub relative_path: String,
    pub description: String,
}

/// Describes a overview of a individual package.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageDescription {
    pub name: String,
    pub description: String,
    pub default: Option<bool>,
    pub source: PackageSource,
    #[serde(default)]
    pub shortcuts: Vec<PackageShortcut>,
}

/// Describes the application itself.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BaseAttributes {
    pub name: String,
    pub target_url: String,
}

impl BaseAttributes {
    /// Serialises as a JSON string.
    pub fn to_json_str(&self) -> Result<String, SerdeError> {
        serde_json::to_string(self)
    }

    /// Builds a configuration from a specified TOML string.
    pub fn from_toml_str(contents: &str) -> Result<Self, TomlError> {
        toml::from_str(contents)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub installing_message: String,
    /// URL to a new updater, if required
    #[serde(default)]
    pub new_tool: Option<String>,
    pub packages: Vec<PackageDescription>,
    #[serde(default)]
    pub hide_advanced: bool,
}

impl Config {
    /// Serialises as a JSON string.
    pub fn to_json_str(&self) -> Result<String, SerdeError> {
        serde_json::to_string(self)
    }

    /// Builds a configuration from a specified TOML string.
    pub fn from_toml_str(contents: &str) -> Result<Self, TomlError> {
        toml::from_str(contents)
    }
}

impl PackageSource {
    /// Fetches releases for a given package
    pub fn get_current_releases(&self) -> Result<Vec<Release>, String> {
        let package_handler = match get_by_name(&self.name) {
            Some(v) => v,
            _ => return Err(format!("Handler {} not found", self.name)),
        };

        package_handler.get_current_releases(&self.config)
    }
}
