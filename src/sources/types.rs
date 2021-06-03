//! types.rs
//!
//! Contains traits that should be implemented by sources, as well as corresponding types.

use std::cmp::Ordering;

pub use semver::Version as SemverVersion;

pub use toml::value::Value as TomlValue;

/// The version of an application.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Version {
    Semver(SemverVersion),
    Integer(u64),
}

// All versions are needed, even if a source is disabled at compile-time.
#[allow(dead_code)]
impl Version {
    /// Coarses versions into semver versions. This will use a integer version as the major
    /// field if required.
    fn coarse_into_semver(&self) -> SemverVersion {
        match *self {
            Version::Semver(ref version) => version.to_owned(),
            Version::Integer(ref version) => {
                SemverVersion::from((version.to_owned(), 0 as u64, 0 as u64))
            }
        }
    }

    /// Returns a new Version, backed by semver.
    pub fn new_semver(version: SemverVersion) -> Version {
        Version::Semver(version)
    }

    /// Returns a new Version, backed by a integer.
    pub fn new_number(version: u64) -> Version {
        Version::Integer(version)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        match *self {
            Version::Semver(ref version) => match *other {
                Version::Semver(ref other_version) => Some(version.cmp(other_version)),
                _ => None,
            },
            Version::Integer(ref num) => match *other {
                Version::Integer(ref other_num) => Some(num.cmp(other_num)),
                _ => None,
            },
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.coarse_into_semver().cmp(&other.coarse_into_semver())
    }
}

/// A individual file in a release.
#[derive(Debug)]
pub struct File {
    pub name: String,
    pub url: String,
}

impl File {}

/// A individual release of an application.
#[derive(Debug)]
pub struct Release {
    pub version: Version,
    pub files: Vec<File>,
}

/// A source of releases.
pub trait ReleaseSource {
    /// Gets a list of the available releases from this source. Should cache internally
    /// if possible using a mutex.
    fn get_current_releases(&self, config: &TomlValue) -> Result<Vec<Release>, String>;
}
