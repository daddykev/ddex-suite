use ddex_builder::presets::DdexVersion;
use ddex_builder::{Builder, BuildRequest, BuildOptions, ConversionOptions};
use ddex_builder::builder::{ReleaseRequest, SoundRecordingRequest, DealRequest};
use ddex_builder::presets::MessageProfile;
use ddex_builder::versions::{VersionConverter, VersionManager, ConversionResult};
use std::collections::HashMap;

#[tokio::test]
async fn test_version_round_trip_382_to_43() {
    let converter = VersionConverter::new();
    
    // Create ERN 3.8.2 compliant request
    let ern_382_request = create_ern_382_request();
    let mut builder = Builder::new();
    let ern_382_result = builder.build_internal(&ern_382_request)
        .expect("Failed to build ERN 3.8.2 release");
    
    // Convert 3.8.2 -> 4.3
    let conversion_options = ConversionOptions {
        allow_lossy: true,
        detailed_reports: true,
        preserve_unknown: false,
        add_metadata: true,
        preserve_comments: false,
        validation_level: ddex_builder::versions::ValidationLevel::Schema,
        custom_mappings: Default::default(),
    };
    
    let upgrade_result = converter.convert(
        &ern_382_result.xml,
        DdexVersion::Ern382,
        DdexVersion::Ern43,
        Some(conversion_options.clone())
    );
    
    match upgrade_result {
        ddex_builder::versions::ConverterResult::Success { xml: ern_43_xml, report } => {
            println!("Upgrade 3.8.2 -> 4.3: {} warnings, {} elements converted", 
                     report.warnings.len(), report.elements_converted);
            
            // Verify 4.3 XML contains expected elements
            assert!(ern_43_xml.contains("ern/43"));
            assert!(ern_43_xml.contains("MessageSchemaVersionId"));
            
            // Convert back 4.3 -> 3.8.2
            let downgrade_result = converter.convert(
                &ern_43_xml,
                DdexVersion::Ern43,
                DdexVersion::Ern382,
                Some(conversion_options.clone())
            );
            
            match downgrade_result {
                ddex_builder::versions::ConverterResult::Success { xml: back_to_382_xml, report: downgrade_report } => {
                    println!("Downgrade 4.3 -> 3.8.2: {} warnings, {} elements dropped", 
                             downgrade_report.warnings.len(), downgrade_report.elements_dropped);
                    
                    // Verify round-trip preserves core data
                    assert!(back_to_382_xml.contains("ern/382"));
                    assert!(back_to_382_xml.contains("Title"));
                    assert!(back_to_382_xml.contains("ISRC"));
                    
                    // Some data loss is expected in round-trip due to version differences
                    assert!(downgrade_report.elements_dropped > 0, "Should drop some 4.3-specific elements");
                    
                    // But core metadata should be preserved
                    assert!(back_to_382_xml.contains("Original Test Track")); // Title preserved
                    assert!(back_to_382_xml.contains("Original Artist")); // Artist preserved
                }
                ddex_builder::versions::ConverterResult::Failure { error, .. } => {
                    panic!("Failed to downgrade 4.3 -> 3.8.2: {}", error);
                }
            }
        }
        ddex_builder::versions::ConverterResult::Failure { error, .. } => {
            panic!("Failed to upgrade 3.8.2 -> 4.3: {}", error);
        }
    }
}

#[tokio::test]
async fn test_version_round_trip_42_to_43() {
    let converter = VersionConverter::new();
    
    // Create ERN 4.2 compliant request
    let ern_42_request = create_ern_42_request();
    let mut builder = Builder::new();
    let ern_42_result = builder.build_internal(&ern_42_request)
        .expect("Failed to build ERN 4.2 release");
    
    let conversion_options = ConversionOptions::default();
    
    // Convert 4.2 -> 4.3
    let upgrade_result = converter.convert(
        &ern_42_result.xml,
        DdexVersion::Ern42,
        DdexVersion::Ern43,
        Some(conversion_options.clone())
    );
    
    match upgrade_result {
        ddex_builder::versions::ConverterResult::Success { xml: ern_43_xml, report } => {
            println!("Upgrade 4.2 -> 4.3: {} warnings, {} elements added", 
                     report.warnings.len(), report.elements_added);
            
            // Verify 4.3 features are available
            assert!(ern_43_xml.contains("ern/43"));
            
            // Convert back 4.3 -> 4.2
            let downgrade_result = converter.convert(
                &ern_43_xml,
                DdexVersion::Ern43,
                DdexVersion::Ern42,
                Some(conversion_options)
            );
            
            match downgrade_result {
                ddex_builder::versions::ConverterResult::Success { xml: back_to_42_xml, report: downgrade_report } => {
                    println!("Downgrade 4.3 -> 4.2: {} warnings", downgrade_report.warnings.len());
                    
                    // Should maintain high fidelity between 4.2 and 4.3
                    assert!(back_to_42_xml.contains("ern/42"));
                    assert!(back_to_42_xml.contains("Enhanced Test Track"));
                    assert!(back_to_42_xml.contains("BitRate"));
                    
                    // High fidelity round-trip should have minimal data loss
                    assert!(downgrade_report.elements_dropped <= 2, "Minimal data loss expected");
                }
                ddex_builder::versions::ConverterResult::Failure { error, .. } => {
                    panic!("Failed to downgrade 4.3 -> 4.2: {}", error);
                }
            }
        }
        ddex_builder::versions::ConverterResult::Failure { error, .. } => {
            panic!("Failed to upgrade 4.2 -> 4.3: {}", error);
        }
    }
}

#[tokio::test]
async fn test_version_conversion_warnings() {
    let converter = VersionConverter::new();
    
    // Create 4.3 release with advanced features
    let advanced_43_request = create_advanced_ern_43_request();
    let mut builder = Builder::new();
    let ern_43_result = builder.build_internal(&advanced_43_request)
        .expect("Failed to build advanced ERN 4.3 release");
    
    // Convert to older version (should generate warnings)
    let conversion_result = converter.convert(
        &ern_43_result.xml,
        DdexVersion::Ern43,
        DdexVersion::Ern382,
        Some(ConversionOptions::default())
    );
    
    match conversion_result {
        ddex_builder::versions::ConverterResult::Success { xml: _, report } => {
            // Should generate multiple warnings for unsupported features
            assert!(!report.warnings.is_empty(), "Should generate conversion warnings");
            
            // Check for specific warning types
            let element_dropped_warnings: Vec<_> = report.warnings.iter()
                .filter(|w| matches!(w.warning_type, ddex_builder::versions::ConversionWarningType::ElementDropped))
                .collect();
            
            let element_renamed_warnings: Vec<_> = report.warnings.iter()
                .filter(|w| matches!(w.warning_type, ddex_builder::versions::ConversionWarningType::ElementRenamed))
                .collect();
            
            println!("Conversion warnings: {} dropped, {} renamed", 
                     element_dropped_warnings.len(), element_renamed_warnings.len());
            
            assert!(!element_dropped_warnings.is_empty(), "Should drop unsupported 4.3 elements");
            assert!(report.elements_dropped > 0, "Should count dropped elements");
        }
        ddex_builder::versions::ConverterResult::Failure { error, .. } => {
            panic!("Conversion should succeed with warnings, not fail: {}", error);
        }
    }
}

#[tokio::test]
async fn test_version_detection() {
    let version_manager = VersionManager::new();
    let mut builder = Builder::new();
    
    // Test ERN 3.8.2 detection
    let ern_382_request = create_ern_382_request();
    let ern_382_result = builder.build_internal(&ern_382_request)
        .expect("Failed to build ERN 3.8.2 for detection");
    
    let detected_382 = version_manager.detect_version(&ern_382_result.xml)
        .expect("Failed to detect ERN 3.8.2");
    assert_eq!(detected_382.detected_version, DdexVersion::Ern382);
    assert!(detected_382.confidence > 0.8, "Should have high confidence");
    
    // Test ERN 4.2 detection  
    let ern_42_request = create_ern_42_request();
    let ern_42_result = builder.build_internal(&ern_42_request)
        .expect("Failed to build ERN 4.2 for detection");
    
    let detected_42 = version_manager.detect_version(&ern_42_result.xml)
        .expect("Failed to detect ERN 4.2");
    assert_eq!(detected_42.detected_version, DdexVersion::Ern42);
    assert!(detected_42.confidence > 0.8, "Should have high confidence");
    
    // Test ERN 4.3 detection
    let ern_43_request = create_ern_43_request();
    let ern_43_result = builder.build_internal(&ern_43_request)
        .expect("Failed to build ERN 4.3 for detection");
    
    let detected_43 = version_manager.detect_version(&ern_43_result.xml)
        .expect("Failed to detect ERN 4.3");
    assert_eq!(detected_43.detected_version, DdexVersion::Ern43);
    assert!(detected_43.confidence > 0.8, "Should have high confidence");
    
    println!("Version detection: 3.8.2={:.2}, 4.2={:.2}, 4.3={:.2}", 
             detected_382.confidence, detected_42.confidence, detected_43.confidence);
}

#[tokio::test]
async fn test_conversion_path_finding() {
    let converter = VersionConverter::new();
    
    // Test direct conversions
    assert!(converter.can_convert(DdexVersion::Ern382, DdexVersion::Ern42));
    assert!(converter.can_convert(DdexVersion::Ern42, DdexVersion::Ern43));
    assert!(converter.can_convert(DdexVersion::Ern43, DdexVersion::Ern42));
    assert!(converter.can_convert(DdexVersion::Ern42, DdexVersion::Ern382));
    
    // Test multi-step conversions
    assert!(converter.can_convert(DdexVersion::Ern382, DdexVersion::Ern43));
    assert!(converter.can_convert(DdexVersion::Ern43, DdexVersion::Ern382));
    
    let supported_conversions = converter.get_supported_conversions();
    println!("Supported conversions: {:?}", supported_conversions);
    
    assert!(supported_conversions.len() >= 4, "Should support core conversion pairs");
}

#[tokio::test]
async fn test_namespace_migration() {
    let converter = VersionConverter::new();
    
    // Create simple XML with old namespace
    let old_namespace_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/382" MessageSchemaVersionId="ern/382">
    <MessageHeader>
        <MessageId>TEST001</MessageId>
    </MessageHeader>
    <ReleaseList>
        <Release>
            <ReleaseId>REL001</ReleaseId>
            <Title>Test Release</Title>
        </Release>
    </ReleaseList>
</NewReleaseMessage>"#;
    
    // Convert namespace from 3.8.2 to 4.3
    let conversion_result = converter.convert(
        old_namespace_xml,
        DdexVersion::Ern382,
        DdexVersion::Ern43,
        Some(ConversionOptions::default())
    );
    
    match conversion_result {
        ddex_builder::versions::ConverterResult::Success { xml, .. } => {
            // Should update namespace
            assert!(xml.contains("http://ddex.net/xml/ern/43"));
            assert!(xml.contains("ern/43"));
            
            // Should preserve content
            assert!(xml.contains("TEST001"));
            assert!(xml.contains("REL001"));
            assert!(xml.contains("Test Release"));
            
            println!("Namespace migration successful");
        }
        ddex_builder::versions::ConverterResult::Failure { error, .. } => {
            panic!("Namespace migration failed: {}", error);
        }
    }
}

#[tokio::test]
async fn test_element_mapping_conversion() {
    let converter = VersionConverter::new();
    
    // Create XML with elements that get renamed between versions
    let source_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/382" MessageSchemaVersionId="ern/382">
    <ResourceList>
        <SoundRecording>
            <TechnicalSoundRecordingDetails>
                <BitRate>1411</BitRate>
                <Codec>FLAC</Codec>
            </TechnicalSoundRecordingDetails>
        </SoundRecording>
    </ResourceList>
</NewReleaseMessage>"#;
    
    // Convert and check element remapping
    let conversion_result = converter.convert(
        source_xml,
        DdexVersion::Ern382,
        DdexVersion::Ern42,
        Some(ConversionOptions::default())
    );
    
    match conversion_result {
        ddex_builder::versions::ConverterResult::Success { xml, report } => {
            // TechnicalSoundRecordingDetails -> TechnicalDetails
            assert!(xml.contains("TechnicalDetails"));
            assert!(!xml.contains("TechnicalSoundRecordingDetails"));
            
            // Should report element renaming
            let rename_warnings: Vec<_> = report.warnings.iter()
                .filter(|w| matches!(w.warning_type, ddex_builder::versions::ConversionWarningType::ElementRenamed))
                .collect();
            
            assert!(!rename_warnings.is_empty(), "Should report element renaming");
            
            println!("Element mapping successful with {} rename warnings", rename_warnings.len());
        }
        ddex_builder::versions::ConverterResult::Failure { error, .. } => {
            panic!("Element mapping failed: {}", error);
        }
    }
}

#[tokio::test]
async fn test_conversion_fidelity_metrics() {
    let converter = VersionConverter::new();
    
    // Create comprehensive test release
    let comprehensive_request = create_comprehensive_test_request();
    let mut builder = Builder::new();
    let original_result = builder.build_internal(&comprehensive_request)
        .expect("Failed to build comprehensive test release");
    
    // Test conversion fidelity across different version paths
    let conversion_paths = vec![
        (DdexVersion::Ern43, DdexVersion::Ern42),
        (DdexVersion::Ern42, DdexVersion::Ern382),
        (DdexVersion::Ern43, DdexVersion::Ern382),
    ];
    
    for (from_version, to_version) in conversion_paths {
        println!("Testing fidelity: {:?} -> {:?}", from_version, to_version);
        
        let conversion_result = converter.convert(
            &original_result.xml,
            from_version,
            to_version,
            Some(ConversionOptions::default())
        );
        
        match conversion_result {
            ddex_builder::versions::ConverterResult::Success { xml: _, report } => {
                let total_elements = report.elements_converted + report.elements_dropped + report.elements_added;
                let fidelity_percentage = if total_elements > 0 {
                    (report.elements_converted as f64 / total_elements as f64) * 100.0
                } else {
                    100.0
                };
                
                println!("  Fidelity: {:.1}% (converted={}, dropped={}, added={})", 
                         fidelity_percentage, report.elements_converted, 
                         report.elements_dropped, report.elements_added);
                
                // Assert minimum fidelity thresholds
                match (from_version, to_version) {
                    (DdexVersion::Ern43, DdexVersion::Ern42) => {
                        assert!(fidelity_percentage >= 90.0, "4.3->4.2 should have high fidelity");
                    }
                    (DdexVersion::Ern42, DdexVersion::Ern382) => {
                        assert!(fidelity_percentage >= 75.0, "4.2->3.8.2 should have reasonable fidelity");
                    }
                    (DdexVersion::Ern43, DdexVersion::Ern382) => {
                        assert!(fidelity_percentage >= 65.0, "4.3->3.8.2 should have acceptable fidelity");
                    }
                    _ => {}
                }
            }
            ddex_builder::versions::ConverterResult::Failure { error, .. } => {
                panic!("Conversion failed for {:?} -> {:?}: {}", from_version, to_version, error);
            }
        }
    }
}

fn create_ern_382_request() -> BuildRequest {
    BuildRequest {
        message_id: "ERN382_TEST_001".to_string(),
        version: Some(DdexVersion::Ern382),
        profile: Some(MessageProfile::AudioSingle),
        sender: "TestSender382".to_string(),
        recipient: "TestRecipient382".to_string(),
        release: ReleaseRequest {
            release_id: "REL382001".to_string(),
            title: "Original Test Track".to_string(),
            display_artist: "Original Artist".to_string(),
            label_name: Some("Legacy Records".to_string()),
            release_date: "2024-01-01".to_string(),
            original_release_date: Some("2024-01-01".to_string()),
            genre: Some("Pop".to_string()),
            pline: Some("℗ 2024 Legacy Records".to_string()),
            cline: Some("© 2024 Legacy Records".to_string()),
            upc: Some("123456789382".to_string()),
            grid: None,
            icpn: None,
            catalog_number: Some("LR382001".to_string()),
            release_type: Some("Single".to_string()),
        },
        resources: ddex_builder::builder::ResourcesRequest {
            sound_recordings: vec![
                SoundRecordingRequest {
                    resource_reference: "R1".to_string(),
                    resource_id: "SR382001".to_string(),
                    title: "Original Test Track".to_string(),
                    display_artist: "Original Artist".to_string(),
                    isrc: Some("ORIG17603820".to_string()),
                    duration: Some("PT210S".to_string()), // Simple format for 3.8.2
                    technical_details: {
                        let mut details = HashMap::new();
                        details.insert("FileName".to_string(), "track382.wav".to_string());
                        details.insert("Codec".to_string(), "PCM".to_string());
                        // No BitRate or advanced features in 3.8.2
                        details
                    },
                },
            ],
            image_resources: vec![],
            video_resources: vec![],
        },
        deals: vec![
            DealRequest {
                deal_id: "DEAL382001".to_string(),
                commercial_model_type: None, // Not supported in 3.8.2
                usage_type: "Stream".to_string(),
                territory_code: "US".to_string(), // Limited territory support
                start_date: Some("2024-01-01".to_string()),
                end_date: None,
                price: Some("9.99".to_string()),
                currency: Some("USD".to_string()),
                resources: vec!["R1".to_string()],
            },
        ],
        metadata: HashMap::new(),
        options: BuildOptions::default(),
    }
}

fn create_ern_42_request() -> BuildRequest {
    BuildRequest {
        message_id: "ERN42_TEST_001".to_string(),
        version: Some(DdexVersion::Ern42),
        profile: Some(MessageProfile::AudioSingle),
        sender: "TestSender42".to_string(),
        recipient: "TestRecipient42".to_string(),
        release: ReleaseRequest {
            release_id: "REL42001".to_string(),
            title: "Enhanced Test Track".to_string(),
            display_artist: "Enhanced Artist".to_string(),
            label_name: Some("Modern Records".to_string()),
            release_date: "2024-01-01".to_string(),
            original_release_date: Some("2024-01-01".to_string()),
            genre: Some("Electronic".to_string()),
            pline: Some("℗ 2024 Modern Records".to_string()),
            cline: Some("© 2024 Modern Records".to_string()),
            upc: Some("123456789420".to_string()),
            grid: None,
            icpn: None,
            catalog_number: Some("MR42001".to_string()),
            release_type: Some("Single".to_string()),
        },
        resources: ddex_builder::builder::ResourcesRequest {
            sound_recordings: vec![
                SoundRecordingRequest {
                    resource_reference: "R1".to_string(),
                    resource_id: "SR42001".to_string(),
                    title: "Enhanced Test Track".to_string(),
                    display_artist: "Enhanced Artist".to_string(),
                    isrc: Some("ENHA17604200".to_string()),
                    duration: Some("PT3M30S".to_string()),
                    technical_details: {
                        let mut details = HashMap::new();
                        details.insert("FileName".to_string(), "track42.flac".to_string());
                        details.insert("Codec".to_string(), "FLAC".to_string());
                        details.insert("BitRate".to_string(), "1411".to_string());
                        details.insert("SampleRate".to_string(), "44100".to_string());
                        details
                    },
                },
            ],
            image_resources: vec![],
            video_resources: vec![],
        },
        deals: vec![
            DealRequest {
                deal_id: "DEAL42001".to_string(),
                commercial_model_type: Some("SubscriptionModel".to_string()),
                usage_type: "Stream".to_string(),
                territory_code: "Worldwide".to_string(),
                start_date: Some("2024-01-01".to_string()),
                end_date: None,
                price: Some("9.99".to_string()),
                currency: Some("USD".to_string()),
                resources: vec!["R1".to_string()],
            },
        ],
        metadata: HashMap::new(),
        options: BuildOptions::default(),
    }
}

fn create_ern_43_request() -> BuildRequest {
    BuildRequest {
        message_id: "ERN43_TEST_001".to_string(),
        version: Some(DdexVersion::Ern43),
        profile: Some(MessageProfile::AudioSingle),
        sender: "TestSender43".to_string(),
        recipient: "TestRecipient43".to_string(),
        release: ReleaseRequest {
            release_id: "REL43001".to_string(),
            title: "Advanced Test Track".to_string(),
            display_artist: "Advanced Artist".to_string(),
            label_name: Some("Future Records".to_string()),
            release_date: "2024-01-01".to_string(),
            original_release_date: Some("2024-01-01".to_string()),
            genre: Some("Synthwave".to_string()),
            pline: Some("℗ 2024 Future Records".to_string()),
            cline: Some("© 2024 Future Records".to_string()),
            upc: Some("123456789430".to_string()),
            grid: None,
            icpn: None,
            catalog_number: Some("FR43001".to_string()),
            release_type: Some("Single".to_string()),
        },
        resources: ddex_builder::builder::ResourcesRequest {
            sound_recordings: vec![
                SoundRecordingRequest {
                    resource_reference: "R1".to_string(),
                    resource_id: "SR43001".to_string(),
                    title: "Advanced Test Track".to_string(),
                    display_artist: "Advanced Artist".to_string(),
                    isrc: Some("ADVN17604300".to_string()),
                    duration: Some("PT3M45.500S".to_string()), // High precision
                    technical_details: {
                        let mut details = HashMap::new();
                        details.insert("FileName".to_string(), "track43.flac".to_string());
                        details.insert("Codec".to_string(), "FLAC".to_string());
                        details.insert("BitRate".to_string(), "2822".to_string()); // Hi-res
                        details.insert("SampleRate".to_string(), "96000".to_string()); // Hi-res
                        details.insert("BitsPerSample".to_string(), "24".to_string());
                        details.insert("HashSum".to_string(), "sha256:abcd1234efgh5678ijklmnop9012qrst3456uvwx7890yzab".to_string());
                        details.insert("HashAlgorithm".to_string(), "SHA-256".to_string());
                        details
                    },
                },
            ],
            image_resources: vec![],
            video_resources: vec![],
        },
        deals: vec![
            DealRequest {
                deal_id: "DEAL43001".to_string(),
                commercial_model_type: Some("SubscriptionModel".to_string()),
                usage_type: "Stream".to_string(),
                territory_code: "Worldwide".to_string(),
                start_date: Some("2024-01-01".to_string()),
                end_date: None,
                price: Some("9.99".to_string()),
                currency: Some("USD".to_string()),
                resources: vec!["R1".to_string()],
            },
        ],
        metadata: HashMap::new(),
        options: BuildOptions::default(),
    }
}

fn create_advanced_ern_43_request() -> BuildRequest {
    let mut request = create_ern_43_request();
    
    // Add advanced ERN 4.3 features
    request.message_id = "ERN43_ADVANCED_001".to_string();
    request.release.title = "Advanced ERN 4.3 Features Test".to_string();
    
    // Add advanced technical details that won't convert to older versions
    request.resources.sound_recordings[0].technical_details.insert(
        "ProprietaryId".to_string(), "ADV43_PROP_001".to_string()
    );
    request.resources.sound_recordings[0].technical_details.insert(
        "EncodingProfile".to_string(), "HiRes24_96".to_string()
    );
    
    request
}

fn create_comprehensive_test_request() -> BuildRequest {
    create_advanced_ern_43_request()
}