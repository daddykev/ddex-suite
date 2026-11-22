use ddex_builder::builder::{
    BuildRequest, LocalizedStringRequest, MessageHeaderRequest, PartyRequest, ReleaseRequest,
    TrackRequest,
};
use ddex_builder::{BuildOptions, DDEXBuilder, ReferenceLinker};

#[test]
fn test_linker_with_xml_generation() {
    let builder = DDEXBuilder::new();
    let mut linker = ReferenceLinker::new();

    // Create request without references
    let mut request = BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("LINK_TEST_001".to_string()),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Auto Link Label".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("LABEL_123".to_string()),
                party_reference: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Auto Link DSP".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("DSP_456".to_string()),
                party_reference: None,
            },
            message_control_type: Some("LiveMessage".to_string()),
            message_created_date_time: None, // Add to existing MessageHeaderRequest structs
        },
        version: "4.3".to_string(),
        profile: Some("AudioAlbum".to_string()),
        releases: vec![ReleaseRequest {
            release_id: "ALBUM_001".to_string(),
            release_reference: None, // Will be auto-generated
            title: vec![LocalizedStringRequest {
                text: "Linked Album".to_string(),
                language_code: Some("en".to_string()),
            }],
            artist: "Linked Artist".to_string(),
            label: None,        // Add this
            release_date: None, // Add this
            upc: None,          // Add this
            tracks: vec![
                TrackRequest {
                    track_id: "TRK_001".to_string(),
                    resource_reference: None, // Will be auto-generated
                    isrc: "USRC11111111".to_string(),
                    title: "First Linked Track".to_string(),
                    duration: "PT3M00S".to_string(),
                    artist: "Linked Artist".to_string(),
                },
                TrackRequest {
                    track_id: "TRK_002".to_string(),
                    resource_reference: None, // Will be auto-generated
                    isrc: "USRC22222222".to_string(),
                    title: "Second Linked Track".to_string(),
                    duration: "PT4M00S".to_string(),
                    artist: "Linked Artist".to_string(),
                },
            ],
            resource_references: None, // Will be auto-generated
        }],
        deals: vec![],
        extensions: None,
    };

    // Auto-link all references
    let link_report = linker.auto_link_request(&mut request).unwrap();

    println!("Linking Report:");
    println!("  Generated refs: {}", link_report.generated_refs);
    println!("  Linked resources: {}", link_report.linked_resources);
    println!("  Validation passed: {}", link_report.validation_passed);

    // Now build XML with linked references
    let result = builder.build(request, BuildOptions::default()).unwrap();

    // Verify XML contains auto-generated references
    assert!(result
        .xml
        .contains("<ReleaseReference>R1</ReleaseReference>"));
    assert!(result
        .xml
        .contains("<ResourceReference>A1</ResourceReference>"));
    assert!(result
        .xml
        .contains("<ResourceReference>A2</ResourceReference>"));
    assert!(result.xml.contains("<PartyReference>P1</PartyReference>")); // Sender
    assert!(result.xml.contains("<PartyReference>P2</PartyReference>")); // Recipient

    println!("\nGenerated XML with auto-linked references:");
    println!("{}", result.xml);
}

#[test]
fn test_deterministic_linking() {
    let mut linker1 = ReferenceLinker::new();
    let mut linker2 = ReferenceLinker::new();

    let mut request1 = create_simple_request();
    let mut request2 = create_simple_request();

    // Auto-link both requests
    linker1.auto_link_request(&mut request1).unwrap();
    linker2.auto_link_request(&mut request2).unwrap();

    // Should produce identical references
    assert_eq!(
        request1.releases[0].release_reference,
        request2.releases[0].release_reference
    );
    assert_eq!(
        request1.releases[0].tracks[0].resource_reference,
        request2.releases[0].tracks[0].resource_reference
    );
}

fn create_simple_request() -> BuildRequest {
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("TEST".to_string()),
            message_sender: PartyRequest {
                party_name: vec![],
                party_id: Some("S1".to_string()),
                party_reference: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![],
                party_id: Some("R1".to_string()),
                party_reference: None,
            },
            message_control_type: None,
            message_created_date_time: None, // Add to existing MessageHeaderRequest structs
        },
        version: "4.3".to_string(),
        profile: None,
        releases: vec![ReleaseRequest {
            release_id: "REL1".to_string(),
            release_reference: None,
            title: vec![],
            artist: "Artist".to_string(),
            label: None,        // Add this
            release_date: None, // Add this
            upc: None,          // Add this
            tracks: vec![TrackRequest {
                track_id: "TRK1".to_string(),
                resource_reference: None,
                isrc: "US123".to_string(),
                title: "Track".to_string(),
                duration: "PT3M".to_string(),
                artist: "Artist".to_string(),
            }],
            resource_references: None,
        }],
        deals: vec![],
        extensions: None,
    }
}
