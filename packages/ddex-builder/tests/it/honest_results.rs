//! APIs that used to return made-up results (success, scores, counts) now compute them
//! (the ones that could not were removed in 0.6.0), and the CLI stubs fail with "not
//! implemented in 0.5.x". Real results are checked on the official sample
//! `examples/Samples43/1 Audio.xml`.

use crate::ern4_support::{minimal_single, parse};
use ddex_builder::round_trip::{verify_round_trip, RoundTripTester};
use ddex_builder::versions::{ConverterResult, VersionConverter};
use ddex_builder::{Builder, CanonicalizationAlgorithm, DdexVersion, FidelityOptions, DB_C14N};
use ddex_core::models::flat::ParsedERNMessage;

fn sample() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Samples43/1 Audio.xml"
    );
    std::fs::read_to_string(path).expect("official sample examples/Samples43/1 Audio.xml")
}

fn parser(xml: &str) -> Result<ParsedERNMessage, String> {
    ddex_parser::DDEXParser::new()
        .parse(std::io::Cursor::new(xml.as_bytes()))
        .map_err(|e| e.to_string())
}

// ----------------------------------------------------------------- round trip

#[test]
fn round_trip_of_an_official_sample_passes() {
    let result = verify_round_trip(&sample(), parser).unwrap();
    assert!(result.success, "{:?}", result.differences);
    assert!(result.differences.is_empty());
    assert!(result
        .rebuilt_xml
        .contains("<TitleText>Yume no Lullaby</TitleText>"));
    // Two builds of one request are compared byte for byte, and the second-generation build
    // with SHA-256; print them (they are computed, not asserted constants).
    println!(
        "deterministic {}, canonical {}, checksums_match {}",
        result.deterministic, result.canonical, result.checksums_match
    );
}

#[test]
fn round_trip_reports_a_field_changed_before_the_build() {
    // The first parse (of the original) changes the release title the request is built
    // from; the comparison is with the original's parsed title, so the change is reported.
    let calls = std::cell::Cell::new(0);
    let changed = |xml: &str| {
        let mut parsed = parser(xml)?;
        calls.set(calls.get() + 1);
        if calls.get() == 1 {
            for title in &mut parsed.graph.releases[0].display_title_text {
                title.text = "Changed Title".to_string();
            }
        }
        Ok(parsed)
    };
    let result = verify_round_trip(&sample(), changed).unwrap();
    assert!(!result.success);
    assert!(
        result
            .differences
            .iter()
            .any(|d| d.starts_with("release title:") && d.contains("Changed Title")),
        "{:?}",
        result.differences
    );
}

#[test]
fn round_trip_testers_run_a_real_round_trip() {
    let tester = RoundTripTester::new();
    let with = tester.test_round_trip_with(&sample(), parser).unwrap();
    assert!(
        with.success && with.differences.is_empty(),
        "{:?}",
        with.differences
    );
    let with = Builder::new()
        .test_round_trip_fidelity_with(&sample(), parser)
        .unwrap();
    assert!(with.success, "{:?}", with.differences);
}

// ------------------------------------------------- canonicalize / fidelity build

#[test]
fn canonicalize_runs_db_c14n_or_nothing() {
    let xml = sample();
    let expected = DB_C14N::new().canonicalize(&xml).unwrap();
    assert_ne!(expected, xml, "DB-C14N changes the official sample");

    let with = |canonicalization| {
        Builder::with_fidelity_options(FidelityOptions {
            canonicalization,
            ..FidelityOptions::default()
        })
    };
    assert_eq!(
        with(CanonicalizationAlgorithm::DbC14N)
            .canonicalize(&xml)
            .unwrap(),
        expected
    );
    assert_eq!(
        with(CanonicalizationAlgorithm::None)
            .canonicalize(&xml)
            .unwrap(),
        xml
    );
}

#[test]
fn build_with_fidelity_counts_the_output() {
    let options = FidelityOptions {
        collect_statistics: true,
        canonicalization: CanonicalizationAlgorithm::DbC14N,
    };
    let builder = Builder::with_fidelity_options(options);
    let result = builder.build_with_fidelity(&minimal_single("4.3")).unwrap();

    let stats = result.statistics.unwrap();
    let elements = result.xml.matches('<').count()
        - result.xml.matches("</").count()
        - result.xml.matches("<?").count()
        - result.xml.matches("<!--").count();
    assert_eq!(stats.element_count, elements);
    assert!(stats.attribute_count > 0 && stats.namespace_count > 0);
    assert!(result.canonicalization_applied);
}

#[test]
fn info_strings_claim_nothing_that_is_not_there() {
    assert!(!ddex_builder::version_info().contains("Perfect"));
}

// ------------------------------------------------------------ versions

#[test]
fn version_conversion_writes_one_namespace_declaration_and_counts_elements() {
    let xml = sample();
    match VersionConverter::new().convert(&xml, DdexVersion::Ern43, DdexVersion::Ern42, None) {
        ConverterResult::Success { xml: out, report } => {
            let root = &out[out.find("<ern:NewReleaseMessage").unwrap()..];
            let root = &root[..root.find('>').unwrap()];
            assert_eq!(root.matches("xmlns:ern=").count(), 1, "{root}");
            assert!(
                root.contains(r#"xmlns:ern="http://ddex.net/xml/ern/42""#),
                "{root}"
            );
            // Well-formed: every attribute once.
            let mut reader = quick_xml::Reader::from_str(&out);
            loop {
                match reader.read_event().unwrap() {
                    quick_xml::events::Event::Start(e) | quick_xml::events::Event::Empty(e) => {
                        for attr in e.attributes() {
                            attr.unwrap();
                        }
                    }
                    quick_xml::events::Event::Eof => break,
                    _ => {}
                }
            }
            assert!(report.elements_converted > 0);
            assert!(report.elements_converted <= xml.matches('<').count());
            assert_eq!(report.elements_added, 0);
        }
        ConverterResult::Failure { error, .. } => panic!("{error}"),
    }
}

#[test]
fn version_detection_needs_evidence() {
    let builder = Builder::new();
    assert_eq!(
        builder.detect_version(&sample()).unwrap(),
        DdexVersion::Ern43
    );
    assert!(builder.detect_version("<html><body/></html>").is_err());
}

// ------------------------------------------------------------------ schema

#[test]
fn schema_if_then_else_follows_the_condition() {
    use ddex_builder::schema::{JsonSchema, SchemaValidator};
    let schema: JsonSchema = serde_json::from_value(serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "if": {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "properties": {"kind": {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "enum": ["a"]
            }},
            "required": ["kind"]
        },
        "then": {"$schema": "https://json-schema.org/draft/2020-12/schema", "required": ["x"]},
        "else": {"$schema": "https://json-schema.org/draft/2020-12/schema", "required": ["y"]}
    }))
    .unwrap();
    let validator = SchemaValidator::new(schema);
    // `else` applies when the condition fails (before 0.5.1 `then` always applied).
    let other = validator.validate(&serde_json::json!({"kind": "b", "y": 1}));
    assert!(other.valid, "{:?}", other.errors);
    assert!(
        !validator
            .validate(&serde_json::json!({"kind": "b", "x": 1}))
            .valid
    );
    assert!(
        !validator
            .validate(&serde_json::json!({"kind": "a", "y": 1}))
            .valid
    );
    assert!(
        validator
            .validate(&serde_json::json!({"kind": "a", "x": 1}))
            .valid
    );
}

// ------------------------------------------------------------------ CLI

fn cli(args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_ddex-builder"))
        .args(args)
        .output()
        .expect("run ddex-builder")
}

#[test]
fn cli_build_writes_real_xml_and_the_stubs_fail() {
    let dir = tempfile::tempdir().unwrap();
    let request = dir.path().join("request.json");
    let output = dir.path().join("out.xml");
    std::fs::write(
        &request,
        serde_json::to_string(&minimal_single("4.3")).unwrap(),
    )
    .unwrap();
    let run = cli(&[
        "build",
        "-i",
        request.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let xml = std::fs::read_to_string(&output).unwrap();
    let expected = ddex_builder::DDEXBuilder::new()
        .build(minimal_single("4.3"), Default::default())
        .unwrap()
        .xml;
    assert_eq!(xml, expected);
    parse(&xml);

    // An incomplete request fails preflight validation instead of printing success.
    let mut incomplete = minimal_single("4.3");
    incomplete.releases[0].tracks[0].isrc.clear();
    std::fs::write(&request, serde_json::to_string(&incomplete).unwrap()).unwrap();
    let run = cli(&[
        "build",
        "-i",
        request.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ]);
    assert!(!run.status.success());

    let sample = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Samples43/1 Audio.xml"
    );
    for args in [vec!["validate", sample]] {
        let run = cli(&args);
        assert!(!run.status.success(), "{args:?}");
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("not implemented in 0.5.x"),
            "{args:?}: {stderr}"
        );
    }
}
