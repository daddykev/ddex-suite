#[cfg(test)]
mod tests {
    use crate::parser::detector::VersionDetector;
    use crate::parser::security::SecurityConfig;
    use ddex_core::models::versions::ERNVersion;
    use std::io::Cursor;

    #[test]
    fn test_version_detection_43() {
        let xml = r#"<?xml version="1.0"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
</ern:NewReleaseMessage>"#;

        let version = VersionDetector::detect(Cursor::new(xml)).unwrap();
        assert_eq!(version, ERNVersion::V4_3);
    }

    #[test]
    fn test_version_detection_42() {
        let xml = r#"<?xml version="1.0"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/42">
</NewReleaseMessage>"#;

        let version = VersionDetector::detect(Cursor::new(xml)).unwrap();
        assert_eq!(version, ERNVersion::V4_2);
    }

    #[test]
    fn test_version_detection_382() {
        let xml = r#"<?xml version="1.0"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/382" MessageSchemaVersionId="ern/382">
</ern:NewReleaseMessage>"#;

        let version = VersionDetector::detect(Cursor::new(xml)).unwrap();
        assert_eq!(version, ERNVersion::V3_8_2);
    }

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();
        assert_eq!(config.max_element_depth, 100);
    }
}
