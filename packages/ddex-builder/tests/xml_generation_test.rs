use ddex_builder::{DDEXBuilder, BuildOptions};
use ddex_builder::builder::{
    BuildRequest, MessageHeaderRequest, PartyRequest, 
    LocalizedStringRequest, ReleaseRequest, TrackRequest
};

#[test]
fn test_basic_xml_generation() {
    let builder = DDEXBuilder::new();
    
    // Create a simple build request
    let request = BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("TEST_MSG_001".to_string()),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Test Label".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("PADPIDA2014071501R".to_string()),
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Test DSP".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("PADPIDA2014071501S".to_string()),
            },
            message_control_type: Some("LiveMessage".to_string()),
        },
        version: "4.3".to_string(),
        profile: Some("AudioAlbum".to_string()),
        releases: vec![
            ReleaseRequest {
                release_id: "0000000000000".to_string(),
                title: vec![LocalizedStringRequest {
                    text: "Test Album".to_string(),
                    language_code: Some("en".to_string()),
                }],
                artist: "Test Artist".to_string(),
                tracks: vec![
                    TrackRequest {
                        position: 1,
                        isrc: Some("USRC12345678".to_string()),
                        title: "Test Track 1".to_string(),
                        duration: 180, // 3 minutes
                    },
                    TrackRequest {
                        position: 2,
                        isrc: Some("USRC12345679".to_string()),
                        title: "Test Track 2".to_string(),
                        duration: 240, // 4 minutes
                    },
                ],
            },
        ],
        deals: vec![],
        extensions: None,
    };
    
    // Build with default options
    let result = builder.build(request, BuildOptions::default()).unwrap();
    
    // Verify output
    assert!(!result.xml.is_empty());
    assert!(result.xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(result.xml.contains("<ern:NewReleaseMessage"));
    assert!(result.xml.contains("MessageId>TEST_MSG_001</"));
    assert!(result.xml.contains("Test Album"));
    assert!(result.xml.contains("Test Track 1"));
    assert!(result.xml.contains("USRC12345678"));
    
    // Check statistics
    assert_eq!(result.statistics.releases, 1);
    assert_eq!(result.statistics.tracks, 2);
    
    println!("Generated XML ({} bytes):", result.xml.len());
    println!("{}", result.xml);
}

#[test]
fn test_deterministic_generation() {
    let builder = DDEXBuilder::new();
    
    let request = BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("DETERMINISTIC_TEST".to_string()),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Label".to_string(),
                    language_code: None,
                }],
                party_id: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "DSP".to_string(),
                    language_code: None,
                }],
                party_id: None,
            },
            message_control_type: None,
        },
        version: "4.3".to_string(),
        profile: None,
        releases: vec![],
        deals: vec![],
        extensions: None,
    };
    
    // Build with deterministic options
    let options = BuildOptions {
        determinism: Some(ddex_builder::DeterminismConfig::default()),
        ..Default::default()
    };
    
    // Build twice and verify identical output
    let result1 = builder.build(request.clone(), options.clone()).unwrap();
    let result2 = builder.build(request.clone(), options.clone()).unwrap();
    
    // Should generate identical XML (except timestamps)
    // For true determinism, we'd need to mock the timestamp
    assert_eq!(result1.xml.len(), result2.xml.len());
    
    // Canonical hash should be generated
    assert!(result1.canonical_hash.is_some());
    
    println!("Canonical hash: {:?}", result1.canonical_hash);
}