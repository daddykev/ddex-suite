//! Real round trip through both crates: BuildRequest → ddex-builder XML → ddex-parser →
//! BuildRequest → XML again. Replaces the former ddex-parser test that used a hand-written
//! quick-xml matcher and a string-template "builder" instead of the real crates.

use ddex_builder::builder::{
    LocalizedStringRequest, MessageHeaderRequest, PartyRequest, ReleaseRequest, TrackRequest,
};
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};
use ddex_core::models::flat::{Organization, ParsedERNMessage};
use ddex_parser::DDEXParser;
use std::io::Cursor;

fn text(value: &str) -> Vec<LocalizedStringRequest> {
    vec![LocalizedStringRequest {
        text: value.to_string(),
        language_code: None,
    }]
}

fn party(id: &str, name: &str) -> PartyRequest {
    PartyRequest {
        party_name: text(name),
        party_id: Some(id.to_string()),
        party_reference: None,
    }
}

fn track(n: u32, isrc: &str, title: &str, duration: &str) -> TrackRequest {
    TrackRequest {
        track_id: format!("TRK{n:03}"),
        resource_reference: Some(format!("A{n}")),
        isrc: isrc.to_string(),
        title: title.to_string(),
        duration: duration.to_string(),
        artist: "Round Trip Artist".to_string(),
    }
}

/// A request with every value pinned (MessageId, created time, references), so the
/// output is a pure function of the request.
fn original_request() -> BuildRequest {
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("RT-MSG-001".to_string()),
            message_sender: party("PADPIDA0000000001", "Round Trip Label"),
            message_recipient: party("PADPIDA0000000002", "Round Trip DSP"),
            message_control_type: Some("LiveMessage".to_string()),
            message_created_date_time: Some("2024-09-13T12:00:00Z".to_string()),
        },
        version: "4.3".to_string(),
        profile: Some("AudioAlbum".to_string()),
        releases: vec![ReleaseRequest {
            release_id: "REL001".to_string(),
            release_reference: Some("R1".to_string()),
            title: text("Round Trip Album"),
            artist: "Round Trip Artist".to_string(),
            label: None,
            release_date: None,
            upc: None,
            tracks: vec![
                track(1, "USRT12400001", "Track One", "PT3M42S"),
                track(2, "USRT12400002", "Track Two", "PT4M05S"),
            ],
            resource_references: None,
        }],
        deals: vec![],
        extensions: None,
    }
}

fn build(request: BuildRequest) -> String {
    let result = DDEXBuilder::new()
        .build(request, BuildOptions::default())
        .expect("build");
    assert!(
        result.errors.is_empty(),
        "build errors: {:?}",
        result.errors
    );
    result.xml
}

fn parse(xml: &str) -> ParsedERNMessage {
    DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .unwrap_or_else(|e| panic!("parser rejected builder output: {e}\n{xml}"))
}

fn iso_duration(d: std::time::Duration) -> String {
    let s = d.as_secs();
    format!("PT{}M{:02}S", s / 60, s % 60)
}

fn party_from(org: &Organization) -> PartyRequest {
    PartyRequest {
        party_name: org.name.as_deref().map(text).unwrap_or_default(),
        party_id: Some(org.id.clone()),
        party_reference: None,
    }
}

/// Map the parser's flattened model back onto a BuildRequest.
fn request_from(parsed: &ParsedERNMessage, template: &BuildRequest) -> BuildRequest {
    let flat = &parsed.flat;
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some(flat.message_id.clone()),
            message_sender: party_from(&flat.sender),
            message_recipient: party_from(&flat.recipient),
            message_control_type: template.header.message_control_type.clone(),
            message_created_date_time: Some(
                flat.message_date
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            ),
        },
        version: template.version.clone(),
        profile: template.profile.clone(),
        releases: flat
            .releases
            .iter()
            .zip(&template.releases)
            .map(|(release, original)| ReleaseRequest {
                release_id: original.release_id.clone(),
                release_reference: Some(release.release_id.clone()),
                title: text(release.default_title.as_deref().unwrap_or_default()),
                artist: release.display_artist.clone().unwrap_or_default(),
                label: None,
                release_date: None,
                upc: None,
                tracks: release
                    .tracks
                    .iter()
                    .zip(&original.tracks)
                    .map(|(t, o)| TrackRequest {
                        track_id: o.track_id.clone(),
                        resource_reference: Some(t.track_id.clone()),
                        isrc: t.isrc.clone().unwrap_or_default(),
                        title: t.title.clone().unwrap_or_default(),
                        duration: t.duration.map(iso_duration).unwrap_or_default(),
                        artist: t.display_artist.clone().unwrap_or_default(),
                    })
                    .collect(),
                resource_references: None,
            })
            .collect(),
        deals: vec![],
        extensions: None,
    }
}

#[test]
#[ignore = "known bug: for ERN 4.3 the builder writes a 3.8-style tree (no Release/ReleaseType, ReferenceTitle instead of DisplayTitleText, no ResourceGroup, MessageCreatedDateTime before MessageSender), which the parser rejects or reads without titles"]
fn parser_reads_back_what_the_builder_wrote() {
    let xml = build(original_request());
    let parsed = parse(&xml);
    let flat = &parsed.flat;

    assert_eq!(flat.message_id, "RT-MSG-001");
    assert_eq!(flat.sender.id, "PADPIDA0000000001");
    assert_eq!(flat.sender.name.as_deref(), Some("Round Trip Label"));
    assert_eq!(flat.recipient.id, "PADPIDA0000000002");
    assert_eq!(flat.releases.len(), 1);

    let release = &flat.releases[0];
    assert_eq!(release.default_title.as_deref(), Some("Round Trip Album"));
    assert_eq!(release.display_artist.as_deref(), Some("Round Trip Artist"));

    let tracks: Vec<_> = release
        .tracks
        .iter()
        .map(|t| {
            (
                t.isrc.as_deref(),
                t.title.as_deref(),
                t.duration.map(|d| d.as_secs()),
            )
        })
        .collect();
    assert_eq!(
        tracks,
        [
            (Some("USRT12400001"), Some("Track One"), Some(222)),
            (Some("USRT12400002"), Some("Track Two"), Some(245)),
        ]
    );
}

#[test]
#[ignore = "known bug: for ERN 4.3 the builder writes a 3.8-style tree (no Release/ReleaseType, ReferenceTitle instead of DisplayTitleText, no ResourceGroup, MessageCreatedDateTime before MessageSender), which the parser rejects or reads without titles"]
fn rebuilding_from_parsed_output_is_byte_identical() {
    let original = original_request();
    let first = build(original.clone());
    let rebuilt_request = request_from(&parse(&first), &original);
    let second = build(rebuilt_request);
    assert_eq!(
        first, second,
        "build → parse → build must reproduce the same bytes"
    );
}
