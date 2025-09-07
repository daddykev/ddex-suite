use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ERNVersion {
    V3_8_2,  // Changed from ERN382
    V4_2,    // Changed from ERN42
    V4_3,    // Changed from ERN43
}

impl ERNVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            ERNVersion::V3_8_2 => "3.8.2",
            ERNVersion::V4_2 => "4.2",
            ERNVersion::V4_3 => "4.3",
        }
    }
    
    pub fn namespace(&self) -> &'static str {
        match self {
            ERNVersion::V3_8_2 => "http://ddex.net/xml/ern/382",
            ERNVersion::V4_2 => "http://ddex.net/xml/ern/42",
            ERNVersion::V4_3 => "http://ddex.net/xml/ern/43",
        }
    }
}

impl std::fmt::Display for ERNVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERN {}", self.as_str())
    }
}
