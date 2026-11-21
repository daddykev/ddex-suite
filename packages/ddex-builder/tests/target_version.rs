//! The builder must emit the namespace of the requested ERN version (including 3.8.1)
//! and reject versions it cannot target instead of writing a made-up namespace.

use ddex_builder::builder::{
    parse_target_version, LocalizedStringRequest, MessageHeaderRequest, PartyRequest,
    ReleaseRequest, TrackRequest,
};
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};
use ddex_core::models::versions::ERNVersion;

fn party(name: &str) -> PartyRequest {
    PartyRequest {
        party_name: vec![LocalizedStringRequest {
            text: name.to_string(),
            language_code: None,
        }],
        party_id: None,
        party_reference: None,
    }
}

fn request(version: &str) -> BuildRequest {
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("MSG-VERSION-TEST".to_string()),
            message_sender: party("Example Sender"),
            message_recipient: party("Example Recipient"),
            message_control_type: Some("LiveMessage".to_string()),
            message_created_date_time: Some("2026-01-01T00:00:00Z".to_string()),
        },
        version: version.to_string(),
        profile: Some("AudioSingle".to_string()),
        releases: vec![ReleaseRequest {
            release_id: "REL001".to_string(),
            release_reference: Some("R1".to_string()),
            title: vec![LocalizedStringRequest {
                text: "Version Test".to_string(),
                language_code: None,
            }],
            artist: "Example Artist".to_string(),
            label: None,
            release_date: None,
            upc: None,
            tracks: vec![TrackRequest {
                track_id: "TRK001".to_string(),
                resource_reference: Some("A1".to_string()),
                isrc: "ZZ0000000001".to_string(),
                title: "Version Test Track".to_string(),
                duration: "PT3M0S".to_string(),
                artist: "Example Artist".to_string(),
            }],
            resource_references: None,
        }],
        deals: vec![],
        extensions: None,
    }
}

#[test]
fn accepted_version_spellings() {
    for (input, expected) in [
        ("3.8.1", ERNVersion::V3_8_1),
        ("381", ERNVersion::V3_8_1),
        ("ern/381", ERNVersion::V3_8_1),
        ("3.8.2", ERNVersion::V3_8_2),
        ("4.2", ERNVersion::V4_2),
        ("ern/43", ERNVersion::V4_3),
    ] {
        assert_eq!(
            parse_target_version(input).unwrap(),
            expected,
            "input {input}"
        );
    }
}

#[test]
fn builds_ern_381_with_381_namespace() {
    let result = DDEXBuilder::new()
        .build(request("3.8.1"), BuildOptions::default())
        .expect("3.8.1 is a supported target");

    assert!(
        result.xml.contains("http://ddex.net/xml/ern/381"),
        "expected the ern/381 namespace in:\n{}",
        result.xml
    );
    assert!(
        !result.xml.contains("http://ddex.net/xml/ern/382"),
        "a 3.8.1 build must not silently emit 3.8.2"
    );
}

#[test]
fn rejects_unsupported_version() {
    let err = DDEXBuilder::new()
        .build(request("9.9"), BuildOptions::default())
        .expect_err("unsupported versions must fail");
    assert!(
        err.to_string().contains("unsupported ERN version '9.9'"),
        "{err}"
    );
}
