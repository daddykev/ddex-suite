//! Real round trip through both crates, for ERN 4.2 and 4.3: BuildRequest -> ddex-builder
//! XML (schema-valid) -> ddex-parser -> BuildRequest -> XML again (byte-identical).

use crate::ern4_support::{
    assert_xsd_valid, build, deal, header, parse, request_from_parsed, track, VERSIONS,
};
use ddex_builder::builder::{ReleaseIdRequest, ReleaseRequest, TrackRequest};
use ddex_builder::BuildRequest;

fn round_trip_track(n: u32, isrc: &str, title: &str, duration: &str) -> TrackRequest {
    TrackRequest {
        isrc: isrc.to_string(),
        duration: duration.to_string(),
        artist: "Round Trip Artist".to_string(),
        ..track(n, title)
    }
}

/// A request with every value pinned (MessageId, created time, references), so the
/// output is a pure function of the request.
fn original_request(version: &str) -> BuildRequest {
    let mut header = header("RT-MSG-001");
    header.message_sender.party_name[0].text = "Round Trip Label".to_string();
    header.message_recipient.party_name[0].text = "Round Trip DSP".to_string();
    // Not read back by the parser, so not part of a byte-identical rebuild
    header.message_control_type = None;
    BuildRequest {
        header,
        version: version.to_string(),
        profile: None,
        releases: vec![ReleaseRequest {
            release_reference: Some("R1".to_string()),
            title: crate::ern4_support::text("Round Trip Album"),
            artist: "Round Trip Artist".to_string(),
            label: Some("Round Trip Records".to_string()),
            tracks: vec![
                round_trip_track(1, "USRT12400001", "Track One", "PT3M42S"),
                round_trip_track(2, "USRT12400002", "Track Two", "PT4M5S"),
            ],
            release_type: Some("Album".into()),
            genre: Some("Pop".to_string()),
            parental_warning_type: Some("NotExplicit".to_string()),
            release_ids: Some(ReleaseIdRequest {
                grid: Some("A1-ABCDE-0000000001-M".to_string()),
                icpn: Some("123456789012".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }],
        deals: vec![deal(&["R1"], "SubscriptionModel", "OnDemandStream")],
        extensions: None,
        language_and_script_code: Some("en".to_string()),
        avs_version_id: (version == "4.3").then(|| "12".to_string()),
    }
}

#[test]
fn parser_reads_back_what_the_builder_wrote() {
    for version in VERSIONS {
        let xml = build(original_request(version));
        assert_xsd_valid(&xml, version, "round-trip request");
        let parsed = parse(&xml);
        let flat = &parsed.flat;

        assert_eq!(flat.message_id, "RT-MSG-001", "{version}");
        assert_eq!(flat.sender.id, "PADPIDA0000000001", "{version}");
        assert_eq!(flat.sender.name.as_deref(), Some("Round Trip Label"), "{version}");
        assert_eq!(flat.recipient.id, "PADPIDA0000000002", "{version}");
        assert_eq!(flat.releases.len(), 1, "{version}");

        let release = &flat.releases[0];
        assert_eq!(release.default_title.as_deref(), Some("Round Trip Album"), "{version}");
        assert_eq!(release.display_artist.as_deref(), Some("Round Trip Artist"), "{version}");
        assert_eq!(release.release_type, "Album", "{version}");
        assert_eq!(release.genre.as_deref(), Some("Pop"), "{version}");
        assert_eq!(release.identifiers.upc.as_deref(), Some("123456789012"), "{version}");

        let tracks: Vec<_> = release
            .tracks
            .iter()
            .map(|t| {
                (
                    t.isrc.as_deref(),
                    t.title.as_deref(),
                    t.duration.map(|d| d.as_secs()),
                    t.position,
                )
            })
            .collect();
        assert_eq!(
            tracks,
            [
                (Some("USRT12400001"), Some("Track One"), Some(222), 1),
                (Some("USRT12400002"), Some("Track Two"), Some(245), 2),
            ],
            "{version}"
        );
        assert_eq!(flat.deals.len(), 1, "{version}");
        assert_eq!(flat.deals[0].releases, ["R1"], "{version}");
    }
}

#[test]
fn rebuilding_from_parsed_output_is_byte_identical() {
    for version in VERSIONS {
        let first = build(original_request(version));
        let rebuilt_request = request_from_parsed(&parse(&first), version);
        let second = build(rebuilt_request);
        assert_eq!(
            first, second,
            "ERN {version}: build -> parse -> build must reproduce the same bytes"
        );
    }
}
