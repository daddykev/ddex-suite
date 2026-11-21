//! Regression tests for issue #1 and community PRs #2-#4:
//! optional DealReference, resources without ReferenceTitle, SoundRecordings without
//! Duration, ERN 3.8.1, and deals linked to their releases.
//!
//! Absent optional data must be reported as `None`, never as generated IDs, IDs used as
//! titles, or zero durations.

use ddex_core::models::flat::ParsedERNMessage;
use ddex_parser::parser::stream::StreamingParser;
use ddex_parser::{DDEXParser, ERNVersion};
use std::io::Cursor;
use std::time::Duration;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {path}: {e}"))
}

fn parse(name: &str) -> ParsedERNMessage {
    let xml = fixture(name);
    DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .unwrap_or_else(|e| panic!("parsing {name}: {e}"))
}

#[test]
fn deal_without_deal_reference_parses_with_no_id() {
    let message = parse("deal_without_reference.xml");

    assert_eq!(message.flat.deals.len(), 1);
    let deal = &message.flat.deals[0];
    assert_eq!(deal.deal_id, None, "absent DealReference must not be invented");
    assert_eq!(deal.releases, vec!["R1".to_string()]);
    assert_eq!(deal.territories.included, vec!["Worldwide".to_string()]);
}

#[test]
fn deals_are_linked_to_their_releases() {
    let message = parse("deals_linked_to_releases.xml");
    let deals = &message.flat.deals;

    // One Deal per <Deal> element, not one per <ReleaseDeal>
    assert_eq!(deals.len(), 3);

    assert_eq!(deals[0].deal_id.as_deref(), Some("D1"));
    assert_eq!(deals[0].releases, vec!["R1".to_string(), "R2".to_string()]);
    assert_eq!(deals[0].territories.included, vec!["US".to_string()]);

    // Second Deal in the same ReleaseDeal: own terms, same release links, no DealReference
    assert_eq!(deals[1].deal_id, None);
    assert_eq!(deals[1].releases, vec!["R1".to_string(), "R2".to_string()]);
    assert_eq!(deals[1].territories.included, vec!["GB".to_string()]);

    assert_eq!(deals[2].deal_id.as_deref(), Some("D3"));
    assert_eq!(deals[2].releases, vec!["R2".to_string()]);
}

#[test]
fn streamed_deals_are_linked_to_their_releases() {
    let xml = fixture("deals_linked_to_releases.xml");
    let mut parser = StreamingParser::new(Cursor::new(xml.as_bytes()), ERNVersion::V4_3);

    let deals: Vec<_> = parser
        .stream_deals()
        .collect::<Result<_, _>>()
        .expect("streaming deals");

    assert_eq!(deals.len(), 3, "every <Deal> must come through the stream");
    assert!(
        deals.iter().all(|d| !d.deal_release_reference.is_empty()),
        "every streamed deal must be linked to a release"
    );
    assert_eq!(deals[1].deal_reference, None);
    assert_eq!(deals[2].deal_release_reference, vec!["R2".to_string()]);
}

#[test]
fn streamed_deal_is_complete_per_schema() {
    use ddex_parser::streaming::ParsedElement;

    let xml = fixture("deal_without_reference.xml");
    let mut parser = StreamingParser::new(Cursor::new(xml.as_bytes()), ERNVersion::V4_3);
    let deal = parser.stream_deals().next().expect("a deal").expect("valid deal");

    // DealReference is optional; a deal linked to its release is complete
    assert_eq!(deal.deal_reference, None);
    assert!(ParsedElement::Deal(deal).is_complete());
}

#[test]
fn image_without_reference_title_has_no_title_and_is_not_a_track() {
    let message = parse("image_without_reference_title.xml");

    let image = message.flat.resources.get("A2").expect("image resource A2");
    assert_eq!(image.resource_type, "Image");
    assert_eq!(image.title, None, "the resource reference must not be used as a title");

    let tracks = &message.flat.releases[0].tracks;
    assert_eq!(tracks.len(), 1, "images are not tracks");
    assert_eq!(tracks[0].track_id, "A1");
    assert_eq!(tracks[0].position, 1, "skipped image must not shift track positions");
    assert_eq!(tracks[0].title.as_deref(), Some("Example Track"));
    assert_eq!(tracks[0].duration, Some(Duration::from_secs(200)));
    assert_eq!(tracks[0].duration_formatted.as_deref(), Some("3:20"));
}

#[test]
fn sound_recording_without_duration_has_no_duration() {
    let message = parse("sound_recording_without_duration.xml");

    let track = &message.flat.releases[0].tracks[0];
    assert_eq!(track.title.as_deref(), Some("Track Without Duration"));
    assert_eq!(track.duration, None, "absent Duration must not become zero");
    assert_eq!(track.duration_formatted, None, "absent Duration must not become \"0:00\"");

    let resource = message.flat.resources.get("A1").expect("resource A1");
    assert_eq!(resource.duration, None);
}

#[test]
fn ern_381_is_reported_as_381() {
    let xml = fixture("ern_381.xml");

    let mut reader = Cursor::new(xml.as_bytes());
    let detected = ddex_parser::parser::detector::VersionDetector::detect(&mut reader).unwrap();
    assert_eq!(detected, ERNVersion::V3_8_1);

    let message = parse("ern_381.xml");
    assert_eq!(message.graph.version, ERNVersion::V3_8_1);
    assert_eq!(message.graph.version.namespace(), "http://ddex.net/xml/ern/381");

    // Parsed with the 3.8.2 logic
    let track = &message.flat.releases[0].tracks[0];
    assert_eq!(track.title.as_deref(), Some("ERN 3.8.1 Track"));
    assert_eq!(track.duration, Some(Duration::from_secs(125)));
}

#[test]
fn ern_4x_titles_come_from_display_title_text() {
    let message = parse("deals_linked_to_releases.xml");
    let titles: Vec<_> = message
        .flat
        .releases
        .iter()
        .map(|r| r.default_title.as_deref())
        .collect();
    assert_eq!(titles, vec![Some("Linked Release One"), Some("Linked Release Two")]);
}
