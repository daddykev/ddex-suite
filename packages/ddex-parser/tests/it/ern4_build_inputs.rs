//! ERN 4.x values the builder needs to rebuild a parsed message (parse -> build round
//! trip): Genre, ParentalWarningType, ReleaseType/@UserDefinedValue, ReleaseLabelReference,
//! resource Type, resource ProprietaryIds, TrackRelease Genre, the root attributes and
//! xs:date deal start dates. Values are read from the official DDEX samples.

use ddex_core::models::common::IdentifierType;
use ddex_core::models::graph::ERNMessage;
use ddex_parser::DDEXParser;
use std::io::Cursor;

fn sample_text(dir: &str, name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/../../examples/{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        dir,
        name
    ))
    .expect("read sample")
}

fn graph(xml: &str) -> ERNMessage {
    DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .expect("parse")
        .graph
}

fn root_attr(graph: &ERNMessage, name: &str) -> Option<String> {
    graph
        .attributes
        .as_ref()?
        .get_str(name)
        .map(|v| v.to_xml_value())
}

#[test]
fn release_genre_parental_warning_and_label_reference() {
    for dir in ["Samples42", "Samples43"] {
        let g = graph(&sample_text(dir, "1 Audio.xml"));
        let release = &g.releases[0];
        assert_eq!(release.genre.len(), 1, "{dir}");
        assert_eq!(release.genre[0].genre_text, "J-Pop", "{dir}");
        assert_eq!(release.parental_warning_type, ["NoAdviceAvailable"], "{dir}");
        assert_eq!(release.release_label_reference, ["PEMI"], "{dir}");
        // The label reference resolves to a PartyList party with a name
        let label = g
            .parties
            .iter()
            .find(|p| p.party_reference.as_deref() == Some("PEMI"))
            .expect("label party");
        assert!(!label.party_name.is_empty(), "{dir}");
    }
}

#[test]
fn resource_type_and_parental_warning() {
    let g = graph(&sample_text("Samples43", "1 Audio.xml"));
    let a1 = g
        .resources
        .iter()
        .find(|r| r.resource_reference == "A1")
        .unwrap();
    assert_eq!(a1.type_value.as_deref(), Some("MusicalWorkSoundRecording"));
    assert_eq!(a1.type_user_defined_value, None);
    assert_eq!(a1.parental_warning_type, ["NoAdviceAvailable"]);
}

#[test]
fn track_release_genre() {
    let g = graph(&sample_text("Samples43", "1 Audio.xml"));
    let tr = &g.track_releases[0];
    assert_eq!(tr.release_reference, "R1");
    assert_eq!(tr.release_label_reference, ["PEMI"]);
    assert_eq!(
        tr.genre.iter().map(|g| g.genre_text.as_str()).collect::<Vec<_>>(),
        ["J-Pop"]
    );
}

#[test]
fn root_language_and_avs_version() {
    let g = graph(&sample_text("Samples43", "1 Audio.xml"));
    assert_eq!(root_attr(&g, "LanguageAndScriptCode").as_deref(), Some("en"));
    assert_eq!(root_attr(&g, "AvsVersionId").as_deref(), Some("3"));
    // Namespace declarations are not attributes
    assert_eq!(root_attr(&g, "xmlns:ern"), None);
}

#[test]
fn user_defined_values_are_kept() {
    let xml = sample_text("Samples43", "1 Audio.xml")
        .replacen(
            "<ReleaseType>Album</ReleaseType>",
            r#"<ReleaseType Namespace="PADPIDA2013042401U" UserDefinedValue="Box &amp; Book">UserDefined</ReleaseType>"#,
            1,
        )
        .replacen(
            "<Type>MusicalWorkSoundRecording</Type>",
            r#"<Type UserDefinedValue="Stem">UserDefined</Type>"#,
            1,
        );
    let g = graph(&xml);
    assert_eq!(
        g.releases[0].release_type_user_defined_value.as_deref(),
        Some("Box & Book")
    );
    let a1 = &g.resources[0];
    assert_eq!(a1.type_value.as_deref(), Some("UserDefined"));
    assert_eq!(a1.type_user_defined_value.as_deref(), Some("Stem"));
}

#[test]
fn resource_proprietary_ids() {
    // 4.3 sample 7: resources carry ProprietaryIds in ResourceId
    let g = graph(&sample_text("Samples43", "7 LongformMusicalWorkVideo.xml"));
    let ids: Vec<_> = g
        .resources
        .iter()
        .flat_map(|r| &r.resource_id)
        .filter(|id| matches!(id.id_type, IdentifierType::Proprietary))
        .collect();
    assert!(
        ids.iter()
            .any(|id| id.value == "12UMGIM13040"
                && id.namespace.as_deref() == Some("PADPIDA2012031302R")),
        "{ids:?}"
    );
}

#[test]
fn xs_date_deal_start_dates_are_parsed() {
    let g = graph(&sample_text("Samples43", "1 Audio.xml"));
    let start = g.deals[0].deal_terms.start_date.expect("start date");
    assert_eq!(start.format("%Y-%m-%d").to_string(), "2004-04-01");
}
