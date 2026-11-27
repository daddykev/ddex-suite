//! Builder output tests: repeatability, preflight validation and profiles. (The ignored
//! golden-file snapshot test was removed in 0.6.0; see KNOWN_ISSUES.md "Parked".)

use ddex_builder::builder::{
    LocalizedStringRequest, MessageHeaderRequest, PartyRequest, ReleaseRequest, TrackRequest,
};
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};

#[test]
fn test_deterministic_generation() {
    let builder = DDEXBuilder::new();

    let request = create_test_request();
    let options = BuildOptions {
        output: ddex_builder::OutputConfig {
            canon_mode: ddex_builder::output::CanonMode::DbC14n,
            ..Default::default()
        },
        preflight_level: ddex_builder::preflight::PreflightLevel::Strict,
        ..BuildOptions::default()
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
    request.releases[0].tracks[0].isrc = Some("INVALID".to_string());

    let options = BuildOptions {
        preflight_level: ddex_builder::preflight::PreflightLevel::Strict,
        ..BuildOptions::default()
    };

    let result = builder.build(request, options);
    assert!(result.is_err());
}

#[test]
fn test_profile_validation() {
    use ddex_builder::preflight::{PreflightValidator, ValidationConfig};

    let config = ValidationConfig {
        level: ddex_builder::preflight::PreflightLevel::Strict,
        validate_identifiers: true,
        check_required_fields: true,
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

/// The request below plus the values ERN 4.x requires (see `ern4_support::ern4_test_data`).
fn create_test_request() -> BuildRequest {
    crate::ern4_support::ern4_test_data(create_test_request_base())
}

fn create_test_request_base() -> BuildRequest {
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: "MSG-GOLDEN-001".to_string(),
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
            message_created_date_time: "2025-01-01T00:00:00Z".to_string(),
        },
        version: "4.3".to_string(),
        profile: Some("AudioAlbum".to_string()),
        releases: vec![ReleaseRequest {
            release_id: Some("REL001".to_string()),
            release_reference: Some("R1".to_string()),
            title: vec![LocalizedStringRequest {
                text: "Test Release".to_string(),
                language_code: Some("en".to_string()),
            }],
            artist: Some("Test Artist".to_string()),
            label: Some("Test Label".to_string()),
            release_date: Some("2024-01-01".to_string()),
            upc: Some("123456789014".to_string()),
            tracks: vec![
                TrackRequest {
                    track_id: "TRK001".to_string(),
                    resource_reference: Some("A1".to_string()),
                    isrc: Some("USRC12345678".to_string()),
                    title: "Test Track".to_string(),
                    duration: "PT3M30S".to_string(),
                    artist: Some("Test Artist".to_string()),
                    ..Default::default()
                },
                TrackRequest {
                    track_id: "TRK002".to_string(),
                    resource_reference: Some("A2".to_string()),
                    isrc: Some("USRC12345679".to_string()),
                    title: "Another Track".to_string(),
                    duration: "PT4M00S".to_string(),
                    artist: Some("Test Artist".to_string()),
                    ..Default::default()
                },
            ],
            resource_references: None,
            ..Default::default()
        }],
        deals: vec![],
        extensions: None,
        ..Default::default()
    }
}
