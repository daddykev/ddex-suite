use ddex_core::models::versions::ERNVersion;

pub trait ERNVersionExt {
    fn namespace_uri(&self) -> &str;
}

impl ERNVersionExt for ERNVersion {
    fn namespace_uri(&self) -> &str {
        match self {
            ERNVersion::V3_8_2 => "http://ddex.net/xml/ern/382",
            ERNVersion::V4_2 => "http://ddex.net/xml/ern/42",
            ERNVersion::V4_3 => "http://ddex.net/xml/ern/43",
        }
    }
}
