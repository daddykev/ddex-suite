//! `BuildRequest::from_parsed` semantic round trips: parse -> from_parsed -> build (same
//! version) -> parse -> compare the two parsed messages on every field the bridge maps.
//!
//! The comparison maps both parsed messages with the bridge and compares the two build
//! requests field by field (as JSON values), so every mapped field is compared and nothing
//! else; XML bytes are not compared. Two values the builder fills in when the request leaves
//! them out are not differences: a deal reference (numbered `D1`, ...) and the resource
//! group tree (one group per release) when the original had none.
//!
//! Sources: the official DDEX ERN 4.2 / 4.3 samples (`examples/Samples42`, `Samples43`) and
//! the ERN 3.8.x sources the 3.8.2 round-trip test uses (YouTube's published 3.8.2 samples,
//! fetched when missing, and the parser's 3.8.x fixtures). Run with `--nocapture` for the
//! results table.

use crate::ern38_round_trip::{repo_fixtures, repo_root, sorted_xml, youtube_samples};
use crate::ern4_support::{minimal_single, multi_disc_album, multi_track_album, parse};
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};
use ddex_core::models::flat::ParsedERNMessage;
use serde_json::Value;

/// Builds that must fail: (source name or `youtube/` prefix, fragment of the error, reason).
const EXPECTED_FAILURES: &[(&str, &str, &str)] = &[
    (
        "Samples42/8 DjMix.xml",
        "requires MessageHeader/MessageId (header.message_id)",
        "the sample's <MessageId/> is empty; the bridge keeps it empty and the builder requires a MessageId",
    ),
    (
        "Samples43/8 DjMix.xml",
        "requires MessageHeader/MessageId (header.message_id)",
        "the sample's <MessageId/> is empty; the bridge keeps it empty and the builder requires a MessageId",
    ),
];

/// The 3.8.x expected failures of the 3.8.2 round-trip test (video resources, ISRC-only
/// release identifiers, fixtures that lack values the XSD requires).
fn expected_failure(name: &str) -> Option<(&'static str, &'static str)> {
    EXPECTED_FAILURES
        .iter()
        .chain(crate::ern38_round_trip::EXPECTED_FAILURES)
        .find(|(n, _, _)| *n == name || (n.starts_with("youtube/") && name.starts_with(n)))
        .map(|(_, e, r)| (*e, *r))
}

fn sources() -> Vec<(String, String)> {
    let mut out = Vec::new();
    let examples = repo_root().join("examples");
    for dir in ["Samples42", "Samples43"] {
        for p in sorted_xml(&examples.join(dir)) {
            let name = format!("{dir}/{}", p.file_name().unwrap().to_string_lossy());
            out.push((name, std::fs::read_to_string(&p).unwrap()));
        }
    }
    if std::env::var("DDEX_SKIP_SAMPLES").as_deref() != Ok("1") {
        for p in youtube_samples() {
            let name = format!("youtube/{}", p.file_name().unwrap().to_string_lossy());
            out.push((name, std::fs::read_to_string(&p).unwrap()));
        }
    }
    for p in repo_fixtures() {
        let name = format!("fixtures/{}", p.file_name().unwrap().to_string_lossy());
        out.push((name, std::fs::read_to_string(&p).unwrap()));
    }
    out
}

/// The request the bridge makes, with the values the builder fills in (deal references,
/// a default resource group) cleared where `original` left them out.
fn mapped(parsed: &ParsedERNMessage, original: Option<&BuildRequest>) -> Value {
    let mut request = BuildRequest::from_parsed(parsed).request;
    if let Some(original) = original {
        for (deal, o) in request.deals.iter_mut().zip(&original.deals) {
            if o.deal_reference.is_none() {
                deal.deal_reference = None;
            }
        }
        for (release, o) in request.releases.iter_mut().zip(&original.releases) {
            if o.resource_group.is_none() {
                release.resource_group = None;
            }
        }
    }
    serde_json::to_value(&request).unwrap()
}

/// `path: original -> rebuilt` for every leaf that differs.
fn diff(path: &str, a: &Value, b: &Value, out: &mut Vec<String>) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for (k, v) in x {
                diff(
                    &format!("{path}.{k}"),
                    v,
                    y.get(k).unwrap_or(&Value::Null),
                    out,
                );
            }
            for (k, v) in y {
                if !x.contains_key(k) {
                    diff(&format!("{path}.{k}"), &Value::Null, v, out);
                }
            }
        }
        (Value::Array(x), Value::Array(y)) if x.len() == y.len() => {
            for (i, (v, w)) in x.iter().zip(y).enumerate() {
                diff(&format!("{path}[{i}]"), v, w, out);
            }
        }
        _ if a != b => out.push(format!("{path}: {a} -> {b}")),
        _ => {}
    }
}

/// parse -> from_parsed -> build -> parse -> compare. `Err` holds the build error.
fn round_trip(xml: &str) -> Result<Vec<String>, String> {
    let original = parse(xml);
    let request = BuildRequest::from_parsed(&original).request;
    let built = DDEXBuilder::new()
        .build(request.clone(), BuildOptions::default())
        .map_err(|e| e.to_string())?
        .xml;
    let rebuilt = parse(&built);
    let mut diffs = Vec::new();
    diff(
        "request",
        &mapped(&original, None),
        &mapped(&rebuilt, Some(&request)),
        &mut diffs,
    );
    Ok(diffs)
}

#[test]
fn bridge_round_trips_every_sample() {
    let mut table = vec!["| source | result |".to_string(), "|---|---|".to_string()];
    let mut failures = Vec::new();
    for (name, xml) in sources() {
        match (round_trip(&xml), expected_failure(&name)) {
            (Ok(diffs), None) if diffs.is_empty() => table.push(format!("| {name} | pass |")),
            (Ok(diffs), None) => {
                table.push(format!("| {name} | **differs** |"));
                failures.push(format!("{name}: {}", diffs.join("; ")));
            }
            (Ok(_), Some((e, _))) => {
                table.push(format!("| {name} | **built (expected failure)** |"));
                failures.push(format!(
                    "{name}: expected the build to fail with {e:?}, it succeeded"
                ));
            }
            (Err(err), Some((e, reason))) if err.contains(e) => {
                table.push(format!("| {name} | fails as expected: {reason} |"))
            }
            (Err(err), _) => {
                table.push(format!("| {name} | **build failed** |"));
                failures.push(format!("{name}: build failed: {err}"));
            }
        }
    }
    println!("{}", table.join("\n"));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The builder's own output holds only values the request carries: nothing is unmapped.
#[test]
fn unmapped_is_empty_for_the_builders_own_output() {
    for version in ["4.3", "4.2", "3.8.2"] {
        for (name, make) in [
            ("minimal single", minimal_single as fn(&str) -> BuildRequest),
            ("multi-track album", multi_track_album),
            ("multi-disc album", multi_disc_album),
        ] {
            let xml = DDEXBuilder::new()
                .build(make(version), BuildOptions::default())
                .unwrap()
                .xml;
            let bridge = BuildRequest::from_parsed(&parse(&xml));
            assert!(
                bridge.unmapped.is_empty(),
                "{version} {name}: {:?}",
                bridge.unmapped
            );
        }
    }
}

/// An official sample with values the request cannot hold lists them.
#[test]
fn unmapped_lists_what_the_request_cannot_hold() {
    let path = repo_root().join("examples/Samples43/1 Audio.xml");
    let bridge = BuildRequest::from_parsed(&parse(&std::fs::read_to_string(path).unwrap()));
    for expected in [
        "header.message_thread_id",
        "attributes.ReleaseProfileVersionId",
        "resources[0].p_line",
    ] {
        assert!(
            bridge.unmapped.iter().any(|p| p == expected),
            "{expected} not in {:?}",
            bridge.unmapped
        );
    }
}

/// parse -> from_parsed -> change the release title -> build -> parse: only that title
/// differs.
#[test]
fn modifying_the_request_changes_only_that_field() {
    let path = repo_root().join("examples/Samples43/1 Audio.xml");
    let original = parse(&std::fs::read_to_string(path).unwrap());
    let mut request = BuildRequest::from_parsed(&original).request;
    request.releases[0].title[0].text = "Changed Title".to_string();
    let built = DDEXBuilder::new()
        .build(request.clone(), BuildOptions::default())
        .unwrap()
        .xml;
    let mut diffs = Vec::new();
    diff(
        "request",
        &mapped(&original, None),
        &mapped(&parse(&built), Some(&request)),
        &mut diffs,
    );
    assert_eq!(diffs.len(), 1, "{diffs:?}");
    assert!(
        diffs[0].starts_with("request.releases[0].title[0].text:")
            && diffs[0].ends_with("-> \"Changed Title\""),
        "{diffs:?}"
    );
}
