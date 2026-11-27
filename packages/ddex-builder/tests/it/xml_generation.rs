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
            release_id: Some("ALBUM_001".to_string()),
            release_reference: None, // Will be auto-generated
            title: vec![LocalizedStringRequest {
                text: "Linked Album".to_string(),
                language_code: Some("en".to_string()),
            }],
            artist: Some("Linked Artist".to_string()),
            label: None,        // Add this
            release_date: None, // Add this
            upc: None,          // Add this
            tracks: vec![
                TrackRequest {
                    track_id: "TRK_001".to_string(),
                    resource_reference: None, // Will be auto-generated
                    isrc: Some("USRC11111111".to_string()),
                    title: "First Linked Track".to_string(),
                    duration: "PT3M00S".to_string(),
                    artist: Some("Linked Artist".to_string()),
                    ..Default::default()
                },
                TrackRequest {
                    track_id: "TRK_002".to_string(),
                    resource_reference: None, // Will be auto-generated
                    isrc: Some("USRC22222222".to_string()),
                    title: "Second Linked Track".to_string(),
                    duration: "PT4M00S".to_string(),
                    artist: Some("Linked Artist".to_string()),
                    ..Default::default()
                },
            ],
            resource_references: None, // Add this,
            ..Default::default()
        }],
        deals: vec![],
        extensions: None,
        ..Default::default()
    };

    // Auto-link all references
    let link_report = linker.auto_link_request(&mut request).unwrap();

    println!("Linking Report:");
    println!("  Generated refs: {}", link_report.generated_refs);
    println!("  Linked resources: {}", link_report.linked_resources);

    // Debug: Print what references were generated
    println!("\nGenerated References:");
    println!(
        "  Release reference: {:?}",
        request.releases[0].release_reference
    );
    println!(
        "  Track 1 reference: {:?}",
        request.releases[0].tracks[0].resource_reference
    );
    println!(
        "  Track 2 reference: {:?}",
        request.releases[0].tracks[1].resource_reference
    );
    println!(
        "  Sender reference: {:?}",
        request.header.message_sender.party_reference
    );
    println!(
        "  Recipient reference: {:?}",
        request.header.message_recipient.party_reference
    );

    // Now build XML with linked references (plus the values ERN 4.x requires)
    let request = crate::ern4_support::ern4_test_data(request);
    let result = builder.build(request, BuildOptions::default()).unwrap();

    // Debug: Print the actual XML to see what we're getting
    println!("\nGenerated XML ({} bytes):", result.xml.len());
    println!("=====================================");
    println!("{}", result.xml);
    println!("=====================================\n");

    // Verify XML contains auto-generated references
    // Check for both with and without namespace prefix
    assert!(
        result.xml.contains("ReleaseReference>R1<")
            || result
                .xml
                .contains("<ReleaseReference>R1</ReleaseReference>"),
        "Could not find ReleaseReference R1 in XML. Check the actual output above."
    );

    assert!(
        result.xml.contains("ResourceReference>A1<")
            || result
                .xml
                .contains("<ResourceReference>A1</ResourceReference>"),
        "Could not find ResourceReference A1 in XML"
    );

    assert!(
        result.xml.contains("ResourceReference>A2<")
            || result
                .xml
                .contains("<ResourceReference>A2</ResourceReference>"),
        "Could not find ResourceReference A2 in XML"
    );

    assert!(
        result.xml.contains("PartyReference>P1<")
            || result.xml.contains("<PartyReference>P1</PartyReference>"),
        "Could not find PartyReference P1 (Sender) in XML"
    );

    assert!(
        result.xml.contains("PartyReference>P2<")
            || result.xml.contains("<PartyReference>P2</PartyReference>"),
        "Could not find PartyReference P2 (Recipient) in XML"
    );
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

/// The request below plus the values ERN 4.x requires (see `ern4_support::ern4_test_data`).
fn create_simple_request() -> BuildRequest {
    crate::ern4_support::ern4_test_data(create_simple_request_base())
}

fn create_simple_request_base() -> BuildRequest {
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
            release_id: Some("REL1".to_string()),
            release_reference: None,
            title: vec![],
            artist: Some("Artist".to_string()),
            label: None,        // Add this
            release_date: None, // Add this
            upc: None,          // Add this
            tracks: vec![TrackRequest {
                track_id: "TRK1".to_string(),
                resource_reference: None,
                isrc: Some("US123".to_string()),
                title: "Track".to_string(),
                duration: "PT3M".to_string(),
                artist: Some("Artist".to_string()),
                ..Default::default()
            }],
            resource_references: None,
            ..Default::default()
        }],
        deals: vec![],
        extensions: None,
        ..Default::default()
    }
}
