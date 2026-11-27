//! DDEX ERN versions known to the builder.
//!
//! [`DdexVersion`] names an ERN version (it is also exported at the crate root as
//! `ddex_builder::DdexVersion`); the build itself targets the version string in
//! `BuildRequest::version`. Before 0.6.0 this module also had a version converter, a version
//! detector and compatibility tables. The converter only swapped the namespace URI and renamed
//! one element while reporting success, and the detector scored invented element lists, so
//! they were removed.

use serde::{Deserialize, Serialize};

/// A DDEX ERN version (also exported at the crate root).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DdexVersion {
    /// ERN 3.8.2
    #[serde(rename = "ERN/3.8.2")]
    Ern382,
    /// ERN 4.2
    #[serde(rename = "ERN/4.2")]
    Ern42,
    /// ERN 4.3
    #[serde(rename = "ERN/4.3")]
    Ern43,
    /// ERN 4.1
    #[serde(rename = "ERN/4.1")]
    Ern41,
}

impl std::fmt::Display for DdexVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DdexVersion::Ern382 => write!(f, "ERN/3.8.2"),
            DdexVersion::Ern42 => write!(f, "ERN/4.2"),
            DdexVersion::Ern43 => write!(f, "ERN/4.3"),
            DdexVersion::Ern41 => write!(f, "ERN/4.1"),
        }
    }
}

/// Version helpers
pub mod utils {
    use super::DdexVersion;

    /// The versions the builder writes schema-valid output for (ERN 3.8.1 output is the
    /// 3.8.2 document in the 3.8.1 namespace and has no `DdexVersion` value)
    pub fn supported_versions() -> Vec<DdexVersion> {
        vec![DdexVersion::Ern382, DdexVersion::Ern42, DdexVersion::Ern43]
    }
}
