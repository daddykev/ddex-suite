use ddex_core::models::versions::ERNVersion;
// core/src/parser/detector.rs


pub struct VersionDetector;

impl VersionDetector {
    pub fn detect<R: std::io::Read>(reader: R) -> crate::error::Result<ERNVersion> {
        let mut buf = Vec::new();
        let mut reader = std::io::BufReader::new(reader);
        use std::io::Read;
        reader.read_to_end(&mut buf)?;
        
        let xml_str = String::from_utf8_lossy(&buf);
        
        // Check for version in namespace
        if xml_str.contains("http://ddex.net/xml/ern/382") {
            Ok(ERNVersion::V3_8_2)
        } else if xml_str.contains("http://ddex.net/xml/ern/42") {
            Ok(ERNVersion::V4_2)
        } else if xml_str.contains("http://ddex.net/xml/ern/43") {
            Ok(ERNVersion::V4_3)
        } else {
            // Default to latest
            Ok(ERNVersion::V4_3)
        }
    }
}
