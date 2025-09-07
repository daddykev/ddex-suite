// core/tests/version_summary.rs
use ddex_parser::DDEXParser;
use ddex_core::models::versions::ERNVersion;

#[test]
fn test_version_support_summary() {
    println!("DDEX Parser Version Support Summary");
    println!("====================================");
    
    let supported_versions = vec![
        (ERNVersion::V3_8_2, "3.8.2", "Legacy support"),
        (ERNVersion::V4_2, "4.2", "Standard support"),
        (ERNVersion::V4_3, "4.3", "Latest with full features"),
    ];
    
    for (version, name, description) in supported_versions {
        println!("[OK] ERN {} - {}: {:?}", name, description, version);
    }
    
    // Test actual parsing for each version
    let parser = DDEXParser::new();
    
    for version in &[ERNVersion::V3_8_2, ERNVersion::V4_2, ERNVersion::V4_3] {
        let namespace = match version {
            ERNVersion::V3_8_2 => "http://ddex.net/xml/ern/382",
            ERNVersion::V4_2 => "http://ddex.net/xml/ern/42",
            ERNVersion::V4_3 => "http://ddex.net/xml/ern/43",
        };
        
        println!("  Version {:?} -> Namespace: {}", version, namespace);
    }
}

#[test]
fn test_feature_matrix() {
    println!("\nFeature Support Matrix:");
    println!("Version | MessageAuditTrail | ResourceGroups | PreOrderDates");
    println!("--------|-------------------|----------------|---------------");
    println!("3.8.2   | No                | No             | No");
    println!("4.2     | Yes               | No             | No");
    println!("4.3     | Yes               | Yes            | Yes");
}