//! APIs that used to return made-up results (success, scores, counts) now compute them
//! (the ones that could not were removed in 0.6.0), and the CLI stubs fail with "not
//! implemented in 0.5.x". Real results are checked on the official sample
//! `examples/Samples43/1 Audio.xml`.

use crate::ern4_support::{minimal_single, parse};
use ddex_builder::round_trip::{verify_round_trip, RoundTripTester};
use ddex_builder::{BuildOptions, CanonicalizationAlgorithm, DDEXBuilder, DB_C14N};
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
    let with = DDEXBuilder::new()
        .check_round_trip(&sample(), parser)
        .unwrap();
    assert!(with.success, "{:?}", with.differences);
}

// ---------------------------------------------- canonicalize / statistics build

#[test]
fn canonicalize_runs_db_c14n_or_nothing() {
    let xml = sample();
    let expected = DB_C14N::new().canonicalize(&xml).unwrap();
    assert_ne!(expected, xml, "DB-C14N changes the official sample");

    let with = |canonicalization| {
        let options = BuildOptions {
            canonicalization,
            ..BuildOptions::default()
        };
        DDEXBuilder::new().canonicalize(&xml, &options).unwrap()
    };
    assert_eq!(with(CanonicalizationAlgorithm::DbC14N), expected);
    assert_eq!(with(CanonicalizationAlgorithm::None), xml);
}

#[test]
fn build_with_statistics_counts_the_output() {
    let options = BuildOptions {
        collect_statistics: true,
        ..BuildOptions::default()
    };
    let result = DDEXBuilder::new()
        .build_with_statistics(minimal_single("4.3"), options)
        .unwrap();

    let stats = result.statistics.unwrap();
    let xml = &result.build.xml;
    let elements = xml.matches('<').count()
        - xml.matches("</").count()
        - xml.matches("<?").count()
        - xml.matches("<!--").count();
    assert_eq!(stats.element_count, elements);
    assert!(stats.attribute_count > 0 && stats.namespace_count > 0);
    let without = DDEXBuilder::new()
        .build_with_statistics(minimal_single("4.3"), BuildOptions::default())
        .unwrap();
    assert!(without.statistics.is_none());
}

/// Warn (the default) returns each preflight error in `BuildResult::warnings` with its code;
/// Strict fails on it. (Before 0.6.0 Warn discarded the errors.)
#[test]
fn warn_mode_reports_preflight_errors_as_warnings() {
    let mut request = minimal_single("4.3");
    request.releases[0].upc = Some("123456789013".to_string()); // wrong check digit
    let built = DDEXBuilder::new()
        .build(request.clone(), BuildOptions::default())
        .expect("Warn builds");
    assert!(
        built
            .warnings
            .iter()
            .any(|w| w.code == "INVALID_UPC" && w.message.contains("123456789013")),
        "{:?}",
        built.warnings
    );
    let preflight = DDEXBuilder::new()
        .preflight(&request, ddex_builder::preflight::PreflightLevel::Warn)
        .unwrap();
    assert!(preflight.errors.iter().any(|e| e.code == "INVALID_UPC"));
    let strict = BuildOptions {
        preflight_level: ddex_builder::preflight::PreflightLevel::Strict,
        ..BuildOptions::default()
    };
    let error = DDEXBuilder::new().build(request, strict).unwrap_err();
    assert!(error.to_string().contains("INVALID_UPC"), "{error}");
}

#[test]
fn info_strings_claim_nothing_that_is_not_there() {
    assert!(!ddex_builder::version_info().contains("Perfect"));
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
    incomplete.releases[0].tracks[0].isrc = None;
    std::fs::write(&request, serde_json::to_string(&incomplete).unwrap()).unwrap();
    let run = cli(&[
        "build",
        "-i",
        request.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ]);
    assert!(!run.status.success());

    // MessageId and MessageCreatedDateTime are required: a request without them fails,
    // naming both fields, unless --message-id / --message-created-date-time give them.
    let mut no_header = serde_json::to_value(minimal_single("4.3")).unwrap();
    no_header["header"]["message_id"] = serde_json::Value::Null;
    no_header["header"]
        .as_object_mut()
        .unwrap()
        .remove("message_created_date_time");
    std::fs::write(&request, no_header.to_string()).unwrap();
    let path = request.to_str().unwrap();
    let run = cli(&["build", "-i", path, "-o", output.to_str().unwrap()]);
    assert!(!run.status.success());
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("(header.message_id)")
            && stderr.contains("(header.message_created_date_time)"),
        "{stderr}"
    );
    let run = cli(&[
        "build",
        "-i",
        path,
        "-o",
        output.to_str().unwrap(),
        "--message-id",
        "MSG-ERN4-001",
        "--message-created-date-time",
        "2024-09-13T12:00:00Z",
        "--check-repeatable",
        "--repeat-count",
        "2",
    ]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(std::fs::read_to_string(&output).unwrap(), expected);
    assert!(String::from_utf8_lossy(&run.stdout).contains("2 builds were byte-identical"));

    // The validate, watch and server subcommands were removed in 0.6.0.
    for subcommand in ["validate", "watch", "server"] {
        let run = cli(&[subcommand]);
        assert!(!run.status.success(), "{subcommand}");
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("unrecognized subcommand"),
            "{subcommand}: {stderr}"
        );
    }
}
