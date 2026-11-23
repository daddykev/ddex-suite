//! RUSTSEC-2026-0194 regression: a start tag with tens of thousands of attributes.
//!
//! quick-xml < 0.41 checked each attribute name against every earlier one, so a tag with N
//! attributes took O(N²) time in every `attributes()` pass (the parser makes several). The
//! attack shapes are an official sample whose root start tag gets 50,000 extra attributes,
//! all distinct (the quadratic case) or all with the same name (must fail cleanly).
//!
//! Gating, so slow CI runners cannot make it flaky: the timing check compares two sizes in
//! the same process instead of using an absolute limit. 5,000 and 50,000 attributes are
//! parsed (best of three each); linear work gives a ratio of about 10, the old quadratic
//! check about 100, and the test fails above 40. A slow runner slows both sizes alike. The
//! only absolute limit, 120 s, is there to catch a hang. Run with `--nocapture` to see the
//! timings (CI does).

use ddex_parser::DDEXParser;
use std::io::Cursor;
use std::time::{Duration, Instant};

fn sample() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Samples43/1 Audio.xml"
    );
    std::fs::read_to_string(path).expect("official sample examples/Samples43/1 Audio.xml")
}

/// The sample with `attrs` inserted at the end of the root start tag.
fn with_root_attributes(xml: &str, attrs: &str) -> String {
    let root = xml
        .find("<ern:NewReleaseMessage")
        .expect("ERN 4.3 root element");
    let end = root + xml[root..].find('>').expect("end of the root start tag");
    format!("{} {}{}", &xml[..end], attrs, &xml[end..])
}

fn distinct_attributes(n: usize) -> String {
    (0..n).map(|i| format!("x{i}=\"v\" ")).collect()
}

fn best_of_three(xml: &str) -> (Duration, Result<usize, String>) {
    let mut best = Duration::MAX;
    let mut outcome = Err(String::new());
    for _ in 0..3 {
        let start = Instant::now();
        let result = DDEXParser::new()
            .parse(Cursor::new(xml.as_bytes()))
            .map(|m| m.flat.releases.len())
            .map_err(|e| e.to_string());
        best = best.min(start.elapsed());
        outcome = result;
    }
    (best, outcome)
}

#[test]
fn many_distinct_attributes_parse_in_linear_time() {
    let xml = sample();
    let expected = DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .expect("the unmodified sample parses")
        .flat
        .releases
        .len();

    let (small, small_result) =
        best_of_three(&with_root_attributes(&xml, &distinct_attributes(5_000)));
    let (large, large_result) =
        best_of_three(&with_root_attributes(&xml, &distinct_attributes(50_000)));
    let ratio = large.as_secs_f64() / small.as_secs_f64().max(1e-9);
    println!(
        "attribute flood: 5,000 distinct attributes {small:?}, 50,000 {large:?}, ratio {ratio:.1}"
    );

    // Unknown attributes on the root are kept, not rejected: the releases are unchanged.
    assert_eq!(small_result, Ok(expected));
    assert_eq!(large_result, Ok(expected));
    assert!(
        large < Duration::from_secs(120),
        "hang: {large:?} for 50,000 attributes"
    );
    assert!(
        ratio < 40.0,
        "50,000 attributes took {ratio:.1}x as long as 5,000 ({large:?} vs {small:?}); \
         linear work gives about 10"
    );
}

#[test]
fn many_duplicate_attributes_fail_cleanly() {
    let attrs: String = (0..50_000).map(|_| "dup=\"v\" ").collect();
    let xml = with_root_attributes(&sample(), &attrs);
    let start = Instant::now();
    let result = DDEXParser::new().parse(Cursor::new(xml.as_bytes()));
    let elapsed = start.elapsed();
    println!("attribute flood: 50,000 duplicate attributes rejected in {elapsed:?}");

    let err = result.expect_err("duplicate attribute names are not well-formed XML");
    assert!(
        err.to_string().to_lowercase().contains("duplicat"),
        "error should name the duplicate attribute: {err}"
    );
    assert!(elapsed < Duration::from_secs(120), "hang: {elapsed:?}");
}
