//! The WASM binding returns `serde_wasm_bindgen::to_value(&ParsedERNMessage)`, so the 0.5.0
//! fields reach JavaScript through the models' `Serialize` impls. This checks that shape
//! (via serde_json, same impls): the new fields are present, and absent values serialize
//! as null / [] rather than being invented or dropped.

use ddex_parser::DDEXParser;
use serde_json::{json, Value};
use std::io::Cursor;

fn serialized(fixture: &str) -> Value {
    let path = format!("{}/tests/fixtures/{fixture}", env!("CARGO_MANIFEST_DIR"));
    let xml = std::fs::read_to_string(path).unwrap();
    let msg = DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .unwrap();
    serde_json::to_value(&msg).unwrap()
}

#[test]
fn new_fields_are_serialized() {
    let v = serialized("ern382_details.xml");
    let flat = &v["flat"];
    assert_eq!(flat["message_control_type"], "TestMessage");
    let release = &flat["releases"][0];
    assert_eq!(release["label_name"], "Details Records");
    assert_eq!(
        release["identifiers"]["proprietary"][0]["value"],
        "REL-0001"
    );
    assert_eq!(
        release["release_dates"][1],
        json!({"date": "2024-09-20", "is_approximate": null, "territory_codes": ["Worldwide"]})
    );
    assert_eq!(release["original_release_dates"][0]["date"], "2016");
    assert_eq!(release["original_release_date"], Value::Null);
    assert_eq!(release["tracks"][0]["artists"][1]["role"], "FeaturedArtist");
    assert_eq!(flat["deals"][0]["validity"]["end"], "2025-09-19T00:00:00Z");
    // graph: every P line with its territory scope
    assert_eq!(
        v["graph"]["releases"][0]["p_line"][0]["scope"]["territory_codes"],
        json!(["Worldwide"])
    );
    assert_eq!(
        v["graph"]["message_header"]["message_control_type"],
        "TestMessage"
    );
}

#[test]
fn absent_values_serialize_as_null_or_empty() {
    let v = serialized("ern43_basic.xml");
    let flat = &v["flat"];
    assert_eq!(flat["message_control_type"], Value::Null);
    let release = &flat["releases"][0];
    for key in [
        "label_name",
        "parental_warning",
        "p_line",
        "c_line",
        "release_date",
    ] {
        assert_eq!(release[key], Value::Null, "{key}");
    }
    for key in [
        "genres",
        "parental_warnings",
        "release_dates",
        "original_release_dates",
    ] {
        assert_eq!(release[key], json!([]), "{key}");
    }
}
