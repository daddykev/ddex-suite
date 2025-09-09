//! Golden file tests for DDEX Builder

use ddex_builder::{DDEXBuilder, BuildRequest, BuildOptions};
use ddex_builder::builder::{
    IdStrategy, ReleaseRequest, TrackRequest, LocalizedStringRequest,
    MessageHeaderRequest, PartyRequest
};
use insta::assert_snapshot;

#[test]
fn test_audio_album_golden() {
    let builder = DDEXBuilder::new();
    
    let request = BuildRequest {
        header: MessageHeaderRequest {
            message_id: None,
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Test Sender".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Test Recipient".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_control_type: Some("LiveMessage".to_string()),
            message_created_date_time: Some("2025-01-01T00:00:00Z".to_string()),  // Fixed timestamp
        },
        version: "4.3".to_string(),
        profile: Some("AudioAlbum".to_string()),
        releases: vec![
            ReleaseRequest {
                release_id: "REL001".to_string(),
                release_reference: Some("R1".to_string()),
                title: vec![LocalizedStringRequest {
                    text: "Test Album".to_string(),
                    language_code: Some("en".to_string()),
                }],
                artist: "Test Artist".to_string(),
                label: Some("Test Label".to_string()),
                release_date: Some("2024-01-01".to_string()),
                upc: Some("123456789014".to_string()),
                tracks: vec![
                    TrackRequest {
                        track_id: "TRK001".to_string(),
                        resource_reference: Some("A1".to_string()),
                        isrc: "USRC12345678".to_string(),
                        title: "Track One".to_string(),
                        duration: "PT3M45S".to_string(),
                        artist: "Test Artist".to_string(),
                    },
                    TrackRequest {
                        track_id: "TRK002".to_string(),
                        resource_reference: Some("A2".to_string()),
                        isrc: "USRC12345679".to_string(),
                        title: "Track Two".to_string(),
                        duration: "PT4M20S".to_string(),
                        artist: "Test Artist feat. Guest".to_string(),
                    },
                ],
                resource_references: None,
            },
        ],
        deals: vec![],
        extensions: None,
    };
    
    let options = BuildOptions {
        determinism: Some(ddex_builder::DeterminismConfig::default()),
        preflight_level: ddex_builder::preflight::PreflightLevel::Warn,
        id_strategy: IdStrategy::StableHash,
        stable_hash_config: None,
    };
    
    let result = builder.build(request, options).unwrap();
    
    // Assert against golden file
    assert_snapshot!("audio_album_output", result.xml);
    assert!(result.errors.is_empty());
    assert!(result.canonical_hash.is_some());
}

#[test]
fn test_deterministic_generation() {
    let builder = DDEXBuilder::new();
    
    let request = create_test_request();
    let options = BuildOptions {
        determinism: Some(ddex_builder::DeterminismConfig {
            canon_mode: ddex_builder::determinism::CanonMode::DbC14n,
            sort_strategy: ddex_builder::determinism::SortStrategy::Canonical,
            output_mode: ddex_builder::determinism::OutputMode::DbC14n,
            ..Default::default()
        }),
        preflight_level: ddex_builder::preflight::PreflightLevel::Strict,
        id_strategy: IdStrategy::StableHash,
        stable_hash_config: None,
    };
    
    // Generate multiple times
    let result1 = builder.build(request.clone(), options.clone()).unwrap();
    let result2 = builder.build(request.clone(), options.clone()).unwrap();
    let result3 = builder.build(request, options).unwrap();
    
    // All should be identical
    assert_eq!(result1.xml, result2.xml);
    assert_eq!(result2.xml, result3.xml);
    assert_eq!(result1.canonical_hash, result2.canonical_hash);
    assert_eq!(result2.canonical_hash, result3.canonical_hash);
}

#[test]
fn test_preflight_validation() {
    let builder = DDEXBuilder::new();
    
    // Invalid ISRC
    let mut request = create_test_request();
    request.releases[0].tracks[0].isrc = "INVALID".to_string();
    
    let options = BuildOptions {
        determinism: None,
        preflight_level: ddex_builder::preflight::PreflightLevel::Strict,
        id_strategy: IdStrategy::UUID,
        stable_hash_config: None,
    };
    
    let result = builder.build(request, options);
    assert!(result.is_err());
}

#[test]
fn test_stable_hash_ids() {
    use ddex_builder::id_generator::{StableHashGenerator, StableHashConfig, HashAlgorithm};
    
    let config = StableHashConfig {
        recipe: "v1".to_string(),
        algorithm: HashAlgorithm::Blake3,
        use_cache: true,
        salt: Some("test_salt".to_string()),
    };
    
    let mut generator = StableHashGenerator::new(config);
    
    // Generate IDs
    let id1 = generator.generate_release_id(
        "123456789012",
        "Album",
        &["USRC12345678".to_string()],
        &["US".to_string()],
    ).unwrap();
    
    // Same inputs should produce same ID
    let id2 = generator.generate_release_id(
        "123456789012",
        "Album",
        &["USRC12345678".to_string()],
        &["US".to_string()],
    ).unwrap();
    
    assert_eq!(id1, id2);
    assert!(id1.starts_with("B3:"));
}

#[test]
fn test_profile_validation() {
    use ddex_builder::preflight::{PreflightValidator, ValidationConfig};
    
    let config = ValidationConfig {
        level: ddex_builder::preflight::PreflightLevel::Strict,
        validate_identifiers: true,
        validate_checksums: true,
        check_required_fields: true,
        validate_dates: true,
        validate_references: true,
        profile: Some("AudioAlbum".to_string()),
    };
    
    let validator = PreflightValidator::new(config);
    
    // Single track should warn for AudioAlbum
    let mut request = create_test_request();
    request.releases[0].tracks = vec![request.releases[0].tracks[0].clone()];
    
    let result = validator.validate(&request).unwrap();
    assert!(!result.warnings.is_empty());
    assert!(result.warnings[0].code == "ALBUM_TRACK_COUNT");
}

fn create_test_request() -> BuildRequest {
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: None,
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Test Sender".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Test Recipient".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_control_type: Some("LiveMessage".to_string()),
            message_created_date_time: Some("2025-01-01T00:00:00Z".to_string()),  // Fixed timestamp
        },
        version: "4.3".to_string(),
        profile: Some("AudioAlbum".to_string()),
        releases: vec![
            ReleaseRequest {
                release_id: "REL001".to_string(),
                release_reference: Some("R1".to_string()),
                title: vec![LocalizedStringRequest {
                    text: "Test Release".to_string(),
                    language_code: Some("en".to_string()),
                }],
                artist: "Test Artist".to_string(),
                label: Some("Test Label".to_string()),
                release_date: Some("2024-01-01".to_string()),
                upc: Some("123456789014".to_string()),
                tracks: vec![
                    TrackRequest {
                        track_id: "TRK001".to_string(),
                        resource_reference: Some("A1".to_string()),
                        isrc: "USRC12345678".to_string(),
                        title: "Test Track".to_string(),
                        duration: "PT3M30S".to_string(),
                        artist: "Test Artist".to_string(),
                    },
                    TrackRequest {
                        track_id: "TRK002".to_string(),
                        resource_reference: Some("A2".to_string()),
                        isrc: "USRC12345679".to_string(),
                        title: "Another Track".to_string(),
                        duration: "PT4M00S".to_string(),
                        artist: "Test Artist".to_string(),
                    },
                ],
                resource_references: None,
            },
        ],
        deals: vec![],
        extensions: None,
    }
}