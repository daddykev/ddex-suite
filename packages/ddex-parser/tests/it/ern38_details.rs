//! ERN 3.8.2 details read by the 0.4.6 parser (fixture `ern382_details.xml`): ReleaseId
//! ProprietaryId in document order, values from ReleaseDetailsByTerritory blocks with their
//! territories (default: the Worldwide block), Release-level P/C lines and dates, the
//! ResourceGroup tree, SoundRecordingDetailsByTerritory artists, label, genre, parental
//! warning and P line, MessageControlType and deal EndDate. `scripts/ern38_population.py`
//! checks the same fields against lxml for the YouTube 3.8.2 samples.

use ddex_core::models::common::IdentifierType;
use ddex_core::models::graph::MessageControlType;
use ddex_parser::DDEXParser;
use std::io::Cursor;

#[test]
fn ern382_details_fixture() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/ern382_details.xml"
    );
    let xml = std::fs::read_to_string(path).unwrap();
    let msg = DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .unwrap();
    let (graph, flat) = (&msg.graph, &msg.flat);

    assert_eq!(
        graph.message_header.message_control_type,
        Some(MessageControlType::TestMessage)
    );
    let release = &graph.releases[0];
    let ids: Vec<_> = release
        .release_id
        .iter()
        .map(|i| (i.id_type.clone(), i.value.as_str()))
        .collect();
    assert_eq!(
        ids,
        [
            (IdentifierType::UPC, "123456789012"),
            (IdentifierType::Proprietary, "REL-0001")
        ]
    );
    // Both territory blocks are kept, each value with its block's territories.
    let labels: Vec<_> = release
        .label_name
        .iter()
        .map(|l| (l.name.as_str(), l.scope.territory_codes.clone()))
        .collect();
    assert_eq!(
        labels,
        [
            ("Details Records Japan", vec!["JP".to_string()]),
            ("Details Records", vec!["Worldwide".to_string()])
        ]
    );
    assert_eq!(release.genre.len(), 2);
    assert_eq!(
        release.original_release_date[0].release_event_type,
        "GlobalOriginalReleaseDate"
    );
    assert_eq!(
        release.original_release_date[0].date.as_deref(),
        Some("2016")
    );
    assert_eq!(
        release.original_release_date[0].event_date, None,
        "partial date not completed"
    );
    let group = release.resource_group.as_ref().expect("ResourceGroup");
    assert_eq!(group.resource_groups[0].sequence_number, Some(1));
    assert_eq!(group.resource_groups[0].resource_group_type, None);
    assert_eq!(group.resource_groups[0].content_items.len(), 2);

    let r = &flat.releases[0];
    assert_eq!(r.label_name.as_deref(), Some("Details Records"));
    assert_eq!(r.genre.as_deref(), Some("Pop"));
    assert_eq!(r.parental_warning.as_deref(), Some("Explicit"));
    assert_eq!(r.p_line.as_ref().and_then(|c| c.year), Some(2024));
    assert_eq!(
        r.c_line.as_ref().map(|c| c.text.as_str()),
        Some("(C) 2024 Details Records")
    );
    assert_eq!(
        r.release_date.map(|d| d.to_rfc3339()).as_deref(),
        Some("2024-09-20T00:00:00+00:00")
    );
    assert_eq!(r.release_dates.len(), 2);

    let t = &r.tracks[0];
    assert_eq!(
        t.display_artist.as_deref(),
        Some("Details Artist feat. Guest Artist")
    );
    let roles: Vec<_> = t
        .artists
        .iter()
        .map(|a| (a.name.as_str(), a.role.as_str()))
        .collect();
    assert_eq!(
        roles,
        [
            ("Details Artist", "MainArtist"),
            ("Guest Artist", "FeaturedArtist")
        ]
    );
    assert_eq!((t.track_number, t.disc_number), (Some(1), None));
    assert_eq!(t.group_path.len(), 1);
    assert_eq!(t.label_name.as_deref(), Some("Details Records"));
    assert_eq!(
        (t.genre.as_deref(), t.sub_genre.as_deref()),
        (Some("Pop"), Some("Synth-pop"))
    );
    assert_eq!(t.parental_warning.as_deref(), Some("Explicit"));
    assert_eq!(
        t.p_line.as_ref().map(|c| c.text.as_str()),
        Some("(P) 2024 Details Records")
    );
    assert_eq!(r.tracks[1].p_line, None);
    assert!(flat.deals[0].validity.end.is_some());
}
