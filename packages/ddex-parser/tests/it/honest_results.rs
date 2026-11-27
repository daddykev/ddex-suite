//! APIs that used to return made-up values (an invented header, placeholder IDs, zero
//! statistics, the current time for a missing date) now report what the document holds
//! (the ones that could not were removed in 0.6.0). Checked on the official samples, with the expected
//! values read from the XML text independently of the parser.

use ddex_parser::parser::multi_release_parser::MultiReleaseParser;
use ddex_parser::streaming::{ParallelStreamingParser, WorkingStreamingElement};
use ddex_parser::{DDEXParser, ERNVersion};
use std::io::Cursor;

fn sample(dir: &str) -> String {
    let path = format!(
        "{}/../../examples/{dir}/1 Audio.xml",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The text of each `<tag>...</tag>` in `xml`, in order.
fn texts<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let (open, close) = (format!("<{tag}>"), format!("</{tag}>"));
    xml.split(open.as_str())
        .skip(1)
        .filter_map(|rest| rest.split(close.as_str()).next())
        .collect()
}

/// Seconds of a `PT{m}M{s}S` / `PT{h}H{m}M{s}S` duration.
fn seconds(duration: &str) -> u64 {
    let mut total = 0;
    let mut number = String::new();
    for c in duration.trim_start_matches("PT").chars() {
        match c {
            'H' => total += number.parse::<u64>().unwrap() * 3600,
            'M' => total += number.parse::<u64>().unwrap() * 60,
            'S' => total += number.parse::<f64>().unwrap() as u64,
            _ => {
                number.push(c);
                continue;
            }
        }
        number.clear();
    }
    total
}

#[test]
fn total_duration_is_the_sum_of_the_resource_durations() {
    let xml = sample("Samples43");
    let expected: u64 = texts(&xml, "SoundRecording")
        .iter()
        .flat_map(|recording| texts(recording, "Duration"))
        .map(seconds)
        .sum();
    assert!(expected > 0);
    let parsed = DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .unwrap();
    assert_eq!(parsed.flat.stats.total_duration, expected);
}

#[test]
fn multi_release_parser_invents_no_header() {
    let xml = sample("Samples43");
    let result = MultiReleaseParser::new()
        .parse_releases(Cursor::new(xml.as_bytes()))
        .unwrap();
    for release in &result.releases {
        assert!(
            !release.release_reference.starts_with("REL_"),
            "{}",
            release.release_reference
        );
        assert!(release
            .release_title
            .iter()
            .all(|t| t.text != "Untitled Release"));
    }
}

#[test]
fn streaming_iterators_report_the_document_not_defaults() {
    let xml = sample("Samples43");
    let created = texts(&xml, "MessageCreatedDateTime")[0];
    let message_id = texts(&xml, "MessageId")[0];
    let release_refs: Vec<&str> = texts(&xml, "ReleaseResourceReference");
    assert!(!release_refs.is_empty());

    let parser = DDEXParser::new();
    let elements: Vec<_> = parser
        .stream(Cursor::new(xml.as_bytes()))
        .collect::<Result<_, _>>()
        .unwrap();
    let header = elements
        .iter()
        .find_map(|e| match e {
            WorkingStreamingElement::MessageHeader {
                message_id,
                created_date_time,
                ..
            } => Some((message_id.clone(), created_date_time.clone())),
            _ => None,
        })
        .expect("a MessageHeader element");
    assert_eq!(header, (message_id.to_string(), created.to_string()));
    let refs: Vec<String> = elements
        .iter()
        .filter_map(|e| match e {
            WorkingStreamingElement::Release {
                resource_references,
                ..
            } => Some(resource_references.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(
        refs.iter().any(|r| r == release_refs[0]),
        "{refs:?} lacks {}",
        release_refs[0]
    );

    // The parallel iterator (FastZeroCopyParser): the header's date and the namespace's
    // version, here on the ERN 4.2 sample.
    let xml42 = sample("Samples42");
    let created42 = texts(&xml42, "MessageCreatedDateTime")[0];
    let header = parser
        .stream_parallel(Cursor::new(xml42.as_bytes()))
        .filter_map(Result::ok)
        .find_map(|e| match e {
            WorkingStreamingElement::MessageHeader {
                created_date_time,
                version,
                ..
            } => Some((created_date_time, version)),
            _ => None,
        })
        .expect("a MessageHeader element");
    assert_eq!(header, (created42.to_string(), ERNVersion::V4_2));
}

#[test]
fn parser_statistics_are_measured_or_zero() {
    let xml = sample("Samples43");
    let parallel = ParallelStreamingParser::new();
    parallel.parse_parallel(xml.as_bytes()).unwrap();
    let stats = parallel.get_stats();
    assert_eq!(stats.bytes_processed, xml.len() as u64);
    assert_eq!(stats.max_depth_reached, 0);
}

fn cli(args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_ddex-parser"))
        .args(args)
        .output()
        .expect("run ddex-parser")
}

#[test]
fn cli_reports_the_detected_version() {
    let path = format!(
        "{}/../../examples/Samples42/1 Audio.xml",
        env!("CARGO_MANIFEST_DIR")
    );
    let stats = cli(&["stats", &path]);
    assert!(
        stats.status.success(),
        "{}",
        String::from_utf8_lossy(&stats.stderr)
    );
    let out = String::from_utf8_lossy(&stats.stdout);
    assert!(out.contains("V4_2") && !out.contains("V4_3"), "{out}");

    let ok = cli(&["validate", &path, "--ddex-version", "4.2"]);
    assert!(
        ok.status.success(),
        "{}",
        String::from_utf8_lossy(&ok.stdout)
    );
    let wrong = cli(&["validate", &path, "--ddex-version", "4.3"]);
    assert!(!wrong.status.success());
    assert!(String::from_utf8_lossy(&wrong.stdout).contains("Expected ERN 4.3, found ERN 4.2"));
}
