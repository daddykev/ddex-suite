// core/src/models/versions/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::detector::ERNVersion;
    use crate::models::versions::VersionDifferences;
    
    #[test]
    fn test_version_features() {
        let v382 = VersionDifferences::for_version(ERNVersion::V3_8_2);
        assert!(!v382.features.supports_message_audit_trail);
        assert!(!v382.features.supports_technical_instantiation);
        
        let v42 = VersionDifferences::for_version(ERNVersion::V4_2);
        assert!(v42.features.supports_message_audit_trail);
        assert!(v42.features.supports_technical_instantiation);
        assert!(!v42.features.supports_resource_group);
        
        let v43 = VersionDifferences::for_version(ERNVersion::V4_3);
        assert!(v43.features.supports_resource_group);
        assert!(v43.features.supports_chapter_information);
    }
    
    #[test]
    fn test_namespace_detection() {
        let namespaces = vec![
            ("http://ddex.net/xml/ern/382", ERNVersion::V3_8_2),
            ("http://ddex.net/xml/ern/42", ERNVersion::V4_2),
            ("http://ddex.net/xml/ern/43", ERNVersion::V4_3),
        ];
        
        for (ns, expected) in namespaces {
            // Test namespace to version mapping
            let version = detect_version_from_namespace(ns);
            assert_eq!(version, expected);
        }
    }
    
    fn detect_version_from_namespace(ns: &str) -> ERNVersion {
        if ns.contains("382") || ns.contains("3.8.2") {
            ERNVersion::V3_8_2
        } else if ns.contains("/42") || ns.contains("4.2") {
            ERNVersion::V4_2
        } else if ns.contains("/43") || ns.contains("4.3") {
            ERNVersion::V4_3
        } else {
            ERNVersion::V4_3 // Default to latest
        }
    }
}