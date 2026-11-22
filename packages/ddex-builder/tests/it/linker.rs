use ddex_builder::builder::{
    BuildRequest, LocalizedStringRequest, MessageHeaderRequest, PartyRequest, ReleaseRequest,
    TrackRequest,
};
use ddex_builder::linker::{LinkingError, ReferenceStyle};
use ddex_builder::{EntityType, LinkerConfig, ReferenceLinker};

#[test]
fn test_reference_generator_sequential() {
    let mut linker = ReferenceLinker::new();

    // Generate references in sequence
    let resource_ref1 = linker.generate_reference(EntityType::Resource);
    let resource_ref2 = linker.generate_reference(EntityType::Resource);
    let release_ref1 = linker.generate_reference(EntityType::Release);
    let resource_ref3 = linker.generate_reference(EntityType::Resource);
    let release_ref2 = linker.generate_reference(EntityType::Release);
    let party_ref1 = linker.generate_reference(EntityType::Party);

    // Verify sequential numbering per entity type
    assert_eq!(resource_ref1, "A1");
    assert_eq!(resource_ref2, "A2");
    assert_eq!(resource_ref3, "A3");
    assert_eq!(release_ref1, "R1");
    assert_eq!(release_ref2, "R2");
    assert_eq!(party_ref1, "P1");
}

#[test]
fn test_reference_generator_deterministic() {
    // Create two linkers and generate same sequence
    let mut linker1 = ReferenceLinker::new();
    let mut linker2 = ReferenceLinker::new();

    let refs1: Vec<String> = (0..10)
        .map(|i| {
            if i % 2 == 0 {
                linker1.generate_reference(EntityType::Resource)
            } else {
                linker1.generate_reference(EntityType::Release)
            }
        })
        .collect();

    let refs2: Vec<String> = (0..10)
        .map(|i| {
            if i % 2 == 0 {
                linker2.generate_reference(EntityType::Resource)
            } else {
                linker2.generate_reference(EntityType::Release)
            }
        })
        .collect();

    // Should produce identical sequences
    assert_eq!(refs1, refs2);
}

#[test]
fn test_entity_registration() {
    let mut linker = ReferenceLinker::new();

    // Register entities
    linker.register_entity(
        EntityType::Resource,
        "track_001".to_string(),
        "A1".to_string(),
    );
    linker.register_entity(
        EntityType::Resource,
        "track_002".to_string(),
        "A2".to_string(),
    );
    linker.register_entity(
        EntityType::Release,
        "release_001".to_string(),
        "R1".to_string(),
    );

    // Verify we can retrieve them
    let all_refs = linker.get_all_references();

    assert_eq!(all_refs[&EntityType::Resource].len(), 2);
    assert_eq!(all_refs[&EntityType::Release].len(), 1);
    assert_eq!(all_refs[&EntityType::Resource]["track_001"], "A1");
    assert_eq!(all_refs[&EntityType::Resource]["track_002"], "A2");
    assert_eq!(all_refs[&EntityType::Release]["release_001"], "R1");
}

#[test]
fn test_release_resource_linking() {
    let mut linker = ReferenceLinker::new();

    // Register some resources first
    linker.register_entity(
        EntityType::Resource,
        "track_001".to_string(),
        "A1".to_string(),
    );
    linker.register_entity(
        EntityType::Resource,
        "track_002".to_string(),
        "A2".to_string(),
    );
    linker.register_entity(
        EntityType::Resource,
        "track_003".to_string(),
        "A3".to_string(),
    );

    // Link release to resources
    let resource_ids = vec![
        "track_001".to_string(),
        "track_002".to_string(),
        "track_003".to_string(),
    ];
    let references = linker
        .link_release_to_resources("release_001", &resource_ids)
        .unwrap();

    assert_eq!(references.len(), 3);
    assert_eq!(references[0].resource_reference, "A1");
    assert_eq!(references[0].sequence_number, 1);
    assert_eq!(references[1].resource_reference, "A2");
    assert_eq!(references[1].sequence_number, 2);
    assert_eq!(references[2].resource_reference, "A3");
    assert_eq!(references[2].sequence_number, 3);
}

#[test]
fn test_link_unknown_resource_error() {
    let mut linker = ReferenceLinker::new();

    // Try to link to non-existent resource
    let resource_ids = vec!["unknown_track".to_string()];
    let result = linker.link_release_to_resources("release_001", &resource_ids);

    assert!(result.is_err());
    match result.unwrap_err() {
        LinkingError::UnknownResource(id) => assert_eq!(id, "unknown_track"),
        _ => panic!("Expected UnknownResource error"),
    }
}

#[test]
fn test_auto_linking_complete_request() {
    let mut linker = ReferenceLinker::new();

    // Create a complete build request
    let mut request = create_test_build_request();

    // Auto-link the request
    let report = linker.auto_link_request(&mut request).unwrap();

    // Verify report
    assert!(report.generated_refs > 0);
    assert!(report.linked_resources > 0);
    assert!(report.validation_passed);

    // Verify references were added to the request
    let release = &request.releases[0];
    assert!(release.release_reference.is_some());
    assert_eq!(release.release_reference.as_ref().unwrap(), "R1");

    // Verify tracks got references
    assert!(release.tracks[0].resource_reference.is_some());
    assert_eq!(release.tracks[0].resource_reference.as_ref().unwrap(), "A1");
    assert!(release.tracks[1].resource_reference.is_some());
    assert_eq!(release.tracks[1].resource_reference.as_ref().unwrap(), "A2");

    // Verify resource references were added to release
    assert!(release.resource_references.is_some());
    let resource_refs = release.resource_references.as_ref().unwrap();
    assert_eq!(resource_refs.len(), 2);
    assert!(resource_refs.contains(&"A1".to_string()));
    assert!(resource_refs.contains(&"A2".to_string()));

    // Verify parties got references
    assert!(request.header.message_sender.party_reference.is_some());
    assert_eq!(
        request
            .header
            .message_sender
            .party_reference
            .as_ref()
            .unwrap(),
        "P1"
    );
    assert!(request.header.message_recipient.party_reference.is_some());
    assert_eq!(
        request
            .header
            .message_recipient
            .party_reference
            .as_ref()
            .unwrap(),
        "P2"
    );
}

#[test]
fn test_validation_passes_with_valid_references() {
    let mut linker = ReferenceLinker::new();

    // Register entities with proper relationships
    linker.register_entity(
        EntityType::Resource,
        "track_001".to_string(),
        "A1".to_string(),
    );
    linker.register_entity(
        EntityType::Release,
        "release_001".to_string(),
        "R1".to_string(),
    );

    // Validate - should pass
    let validation_result = linker.validate_references();
    assert!(validation_result.is_ok());
}

#[test]
fn test_custom_reference_style() {
    let config = LinkerConfig {
        reference_style: ReferenceStyle::Prefixed {
            separator: "_".to_string(),
        },
        auto_link: true,
        validate_on_build: true,
        strict: false,
    };

    let mut linker = ReferenceLinker::with_config(config);

    let ref1 = linker.generate_reference(EntityType::Resource);
    let ref2 = linker.generate_reference(EntityType::Release);

    assert_eq!(ref1, "A_1");
    assert_eq!(ref2, "R_1");
}

// Helper function to create a test build request
fn create_test_build_request() -> BuildRequest {
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("TEST_MSG_123".to_string()),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Test Label".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("SENDER_001".to_string()),
                party_reference: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Test DSP".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("RECIPIENT_001".to_string()),
                party_reference: None,
            },
            message_control_type: Some("LiveMessage".to_string()),
            message_created_date_time: None, // Add to existing MessageHeaderRequest structs
        },
        version: "4.3".to_string(),
        profile: Some("AudioAlbum".to_string()),
        releases: vec![ReleaseRequest {
            release_id: "REL_001".to_string(),
            release_reference: None,
            title: vec![LocalizedStringRequest {
                text: "Test Album".to_string(),
                language_code: Some("en".to_string()),
            }],
            artist: "Test Artist".to_string(),
            label: None,        // Add this
            release_date: None, // Add this
            upc: None,          // Add this
            tracks: vec![
                TrackRequest {
                    track_id: "TRACK_001".to_string(),
                    resource_reference: None,
                    isrc: "USRC12345678".to_string(),
                    title: "Track 1".to_string(),
                    duration: "PT3M30S".to_string(),
                    artist: "Test Artist".to_string(),
                },
                TrackRequest {
                    track_id: "TRACK_002".to_string(),
                    resource_reference: None,
                    isrc: "USRC12345679".to_string(),
                    title: "Track 2".to_string(),
                    duration: "PT4M15S".to_string(),
                    artist: "Test Artist".to_string(),
                },
            ],
            resource_references: None,
        }],
        deals: vec![],
        extensions: None,
    }
}
