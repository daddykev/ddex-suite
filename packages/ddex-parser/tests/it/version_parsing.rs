//! Parsing one minimal message per supported ERN version, plus graph/flat consistency.
//! (Absorbs the former integration_test, version_migration, phase2_complete and
//! model_consistency files, which parsed the same three messages.)

use crate::common::{parse_fixture, read_fixture};
use ddex_core::models::versions::ERNVersion;
use ddex_parser::DDEXParser;
use std::io::Cursor;

#[test]
fn parses_each_supported_version() {
    for (name, expected) in [
        ("ern_381.xml", ERNVersion::V3_8_1),
        ("ern382_basic.xml", ERNVersion::V3_8_2),
        ("ern42_basic.xml", ERNVersion::V4_2),
        ("ern43_basic.xml", ERNVersion::V4_3),
    ] {
        let parsed = parse_fixture(name);
        assert_eq!(parsed.graph.version, expected, "{name}");
        assert_eq!(parsed.flat.releases.len(), 1, "{name}: one release");
    }
}

#[test]
fn reuses_one_parser_across_versions() {
    let mut parser = DDEXParser::new();
    for name in ["ern382_basic.xml", "ern42_basic.xml", "ern43_basic.xml"] {
        let xml = read_fixture(name);
        parser
            .parse(Cursor::new(xml.as_bytes()))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}

#[test]
fn ern_43_header_and_release_fields() {
    let parsed = parse_fixture("ern43_basic.xml");
    assert_eq!(parsed.flat.message_id, "MSG_TV7HAUF8KQ");
    assert_eq!(parsed.flat.sender.id, "PADPIDA2014120301");
    assert_eq!(parsed.flat.sender.name.as_deref(), Some("Test Sender"));
    assert_eq!(parsed.flat.recipient.id, "PADPIDA2014120302");
    assert_eq!(parsed.flat.releases.len(), 1);
    assert_eq!(parsed.graph.deals.len(), 1);
}

#[test]
fn graph_and_flat_models_agree() {
    let parsed = parse_fixture("ern43_basic.xml");
    assert!(!parsed.graph.resources.is_empty());
    assert!(!parsed.flat.releases.is_empty());
    // flat.version uses the enum spelling (e.g. "V4_3"), which the bindings expose as-is.
    assert_eq!(format!("{:?}", parsed.graph.version), parsed.flat.version);
}

#[test]
fn graph_model_survives_a_serde_json_round_trip() {
    let parsed = parse_fixture("ern43_basic.xml");
    let json = serde_json::to_string(&parsed.graph).unwrap();
    let back: ddex_core::models::graph::ERNMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
}
