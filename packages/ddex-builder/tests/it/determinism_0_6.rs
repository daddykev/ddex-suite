//! 0.6.0 determinism: the builder reads no clock or random source and uses no unordered
//! iteration on the build path, so the same request and options give the same bytes.
//!
//! Each test names its row in `dev-docs/DETERMINISM_AUDIT_0.6.0.md` (D* = finding, resolved
//! in 0.6.0 PR 6). `MessageId` and `MessageCreatedDateTime` are required inputs; references
//! the request leaves out are numbered in request order.
//!
//! The requests come from `ern4_support::multi_track_album` (the schema-validated album the
//! XSD tests use); larger ones clone its tracks, releases and deals with distinct references.

use crate::ern4_support::multi_track_album;
use ddex_builder::builder::{BuildOptions, BuildRequest, DDEXBuilder, MessageHeaderRequest};
use ddex_builder::streaming::StreamingBuilder;

const VERSIONS: [&str; 3] = ["4.3", "4.2", "3.8.2"];

fn build_default(request: BuildRequest) -> String {
    DDEXBuilder::new()
        .build(request, BuildOptions::default())
        .unwrap_or_else(|e| panic!("build failed: {e}"))
        .xml
}

/// A build on a new thread (its own `RandomState` seeds and thread-local state).
fn build_in_fresh_thread(request: BuildRequest) -> String {
    std::thread::spawn(move || build_default(request))
        .join()
        .expect("build thread panicked")
}

/// The fixture with every track's `ResourceReference` and every deal's `DealReference` left
/// to the builder. (The release references stay: the deals refer to them.)
fn without_references(mut request: BuildRequest) -> BuildRequest {
    for release in &mut request.releases {
        for track in &mut release.tracks {
            track.resource_reference = None;
        }
    }
    for deal in &mut request.deals {
        deal.deal_reference = None;
    }
    request
}

/// The texts of every `<{name}>` element, in document order.
fn texts(xml: &str, name: &str) -> Vec<String> {
    let open = format!("<{name}>");
    xml.match_indices(&open)
        .map(|(i, _)| {
            let start = i + open.len();
            let end = start + xml[start..].find('<').expect("element is closed");
            xml[start..end].to_string()
        })
        .collect()
}

fn build_error(request: BuildRequest) -> String {
    DDEXBuilder::new()
        .build(request, BuildOptions::default())
        .expect_err("the build must fail")
        .to_string()
}

// --- a. Default options -----------------------------------------------------------------

/// Audit rows D24, D25 and H1–H8: with every value supplied, default options give the same
/// bytes on repeated builds and on another thread (no hash-container order or timing in the
/// XML).
#[test]
fn default_options_full_request_is_byte_identical() {
    for version in VERSIONS {
        let request = multi_track_album(version);
        let first = build_default(request.clone());
        assert_eq!(
            first,
            build_default(request.clone()),
            "{version}: second build"
        );
        assert_eq!(
            first,
            build_in_fresh_thread(request),
            "{version}: other thread"
        );
    }
}

// --- b. References left out are sequential --------------------------------------------------

/// Audit rows D1–D7: references the request leaves out are numbered in request order
/// (`A1`, `A2`, ...; `D1`, `D2`, ...), and the builds are byte-identical.
#[test]
fn references_left_out_are_sequential_and_byte_identical() {
    for version in VERSIONS {
        let request = without_references(multi_track_album(version));
        let first = build_default(request.clone());
        assert_eq!(
            first,
            build_default(request.clone()),
            "{version}: second build"
        );
        assert_eq!(
            first,
            build_in_fresh_thread(request),
            "{version}: other thread"
        );
        let resources = texts(&first, "ResourceReference");
        for n in 1..=4 {
            assert!(
                resources.contains(&format!("A{n}")),
                "{version}: no A{n} in {resources:?}"
            );
        }
        let deals = texts(&first, "DealReference");
        for n in 1..=3 {
            assert!(
                deals.contains(&format!("D{n}")),
                "{version}: no D{n} in {deals:?}"
            );
        }
    }
}

/// Generated references skip the numbers the request already uses, and supplied references
/// are not changed.
#[test]
fn generated_references_skip_supplied_numbers() {
    for version in VERSIONS {
        let mut request = without_references(multi_track_album(version));
        request.releases[0].tracks[0].resource_reference = Some("A2".to_string());
        let xml = build_default(request);
        let mut resources = texts(&xml, "ResourceReference");
        resources.sort();
        resources.dedup();
        assert_eq!(resources, ["A1", "A2", "A3", "A4"], "{version}");
        // The supplied A2 is the first track's (written first).
        assert_eq!(
            texts(&xml, "ResourceReference").first().map(String::as_str),
            Some("A2"),
            "{version}"
        );
    }
}

// --- c. MessageId and MessageCreatedDateTime are required -----------------------------------

/// Audit rows D1, D3, D4, D5: the builder never generates a MessageId.
#[test]
fn omitted_message_id_is_an_error() {
    for version in VERSIONS {
        let mut request = multi_track_album(version);
        request.header.message_id = String::new();
        let error = build_error(request);
        assert!(
            error.contains("requires MessageHeader/MessageId (header.message_id)"),
            "{version}: {error}"
        );
    }
}

/// Audit row D8: the builder reads no clock.
#[test]
fn omitted_message_created_date_time_is_an_error() {
    for version in VERSIONS {
        let mut request = multi_track_album(version);
        request.header.message_created_date_time = String::new();
        let error = build_error(request);
        assert!(
            error.contains(
                "requires MessageHeader/MessageCreatedDateTime (header.message_created_date_time)"
            ),
            "{version}: {error}"
        );
    }
}

/// From JSON (and Node.js / Python), a missing or `null` header value gives the same error.
#[test]
fn missing_or_null_header_values_from_json_are_named() {
    let request = multi_track_album("4.3");
    for value in [serde_json::Value::Null, serde_json::json!("")] {
        let mut json = serde_json::to_value(&request).unwrap();
        json["header"]["message_id"] = value.clone();
        json["header"]
            .as_object_mut()
            .unwrap()
            .remove("message_created_date_time");
        let parsed: BuildRequest = serde_json::from_value(json).expect("deserializes");
        let error = build_error(parsed);
        assert!(error.contains("(header.message_id)"), "{error}");
        assert!(
            error.contains("(header.message_created_date_time)"),
            "{error}"
        );
    }
}

// --- f. Streaming writer --------------------------------------------------------------------

/// The fixture's album through the streaming writer (3.8-style; it rejects 4.x targets).
fn stream(header: &MessageHeaderRequest) -> Vec<u8> {
    let request = multi_track_album("3.8.2");
    let release = &request.releases[0];
    let mut writer = StreamingBuilder::new(Vec::new()).expect("streaming builder");
    writer
        .start_message(header, "3.8.2")
        .expect("start_message");
    let mut references = Vec::new();
    for track in &release.tracks {
        references.push(
            writer
                .write_resource(
                    &track.track_id,
                    &track.title,
                    track.artist.as_deref().unwrap_or_default(),
                    track.isrc.as_deref(),
                    Some(&track.duration),
                    None,
                )
                .expect("write_resource"),
        );
    }
    writer
        .finish_resources_start_releases()
        .expect("finish_resources_start_releases");
    writer
        .write_release(
            release.release_id.as_deref().unwrap_or_default(),
            &release.title[0].text,
            release.artist.as_deref().unwrap_or_default(),
            release.label.as_deref(),
            release.upc.as_deref(),
            release.release_date.as_deref(),
            release.genre.as_deref(),
            &references,
        )
        .expect("write_release");
    writer.finish_message().expect("finish_message");
    writer.into_inner().expect("into_inner")
}

/// Audit row D14: the streamed bytes repeat; references are 128 bits of the SHA-256 of the
/// caller's IDs.
#[test]
fn streaming_writer_full_header_is_byte_identical() {
    let header = multi_track_album("3.8.2").header;
    assert_eq!(stream(&header), stream(&header));
}

/// Audit rows D12, D13: `start_message` requires both header values, with the same errors
/// as `DDEXBuilder::build`.
#[test]
fn streaming_writer_omitted_header_values_are_errors() {
    let mut header = multi_track_album("3.8.2").header;
    header.message_id = String::new();
    header.message_created_date_time = String::new();
    let mut writer = StreamingBuilder::new(Vec::new()).expect("streaming builder");
    let error = writer
        .start_message(&header, "3.8.2")
        .expect_err("start_message must fail")
        .to_string();
    assert!(
        error.contains("ERN 3.8.2 requires MessageHeader/MessageId (header.message_id)"),
        "{error}"
    );
    assert!(
        error.contains(
            "ERN 3.8.2 requires MessageHeader/MessageCreatedDateTime (header.message_created_date_time)"
        ),
        "{error}"
    );
}

// --- g. Order sensitivity with many items ---------------------------------------------------

const MANY: u32 = 60;

/// ERN 4.3 allows one main release, so: 60 sound recordings, each with a TrackRelease
/// (61 releases), and 62 deals.
fn large_ern43() -> BuildRequest {
    let mut request = multi_track_album("4.3");
    let template_track = request.releases[0].tracks[0].clone();
    request.releases[0].tracks = (1..=MANY)
        .map(|n| {
            let mut track = template_track.clone();
            track.track_id = format!("TRK{n:03}");
            track.resource_reference = Some(format!("A{n}"));
            track.isrc = Some(format!("USRT1240{n:04}"));
            track.title = format!("Track {n}");
            track.sequence_number = Some(n);
            if let Some(track_release) = &mut track.track_release {
                track_release.release_reference = format!("R{n}");
                track_release.title = Some(format!("Track {n}"));
                for id in &mut track_release.release_ids.proprietary_ids {
                    id.value = format!("TR-{n}");
                }
            }
            track
        })
        .collect();
    // Keep the two album deals (R0); one deal per TrackRelease.
    let track_deal = request.deals[2].clone();
    request.deals.truncate(2);
    request.deals.extend((1..=MANY).map(|n| {
        let mut deal = track_deal.clone();
        deal.release_references = vec![format!("R{n}")];
        deal
    }));
    request
}

/// ERN 3.8.2 allows several releases: 60 copies of the album (240 recordings, 300 releases
/// with the TrackReleases, 180 deals).
fn large_ern382() -> BuildRequest {
    let mut request = multi_track_album("3.8.2");
    let album = request.releases[0].clone();
    let deals = std::mem::take(&mut request.deals);
    request.releases.clear();
    for k in 0..MANY {
        let mut release = album.clone();
        release.release_reference = Some(format!("R{k}"));
        for track in &mut release.tracks {
            let n = track.sequence_number.expect("fixture tracks are numbered");
            track.track_id = format!("TRK{k:03}{n:03}");
            track.resource_reference = Some(format!("A{k}T{n}"));
            track.isrc = Some(format!("USRT1{k:03}{n:04}"));
            if let Some(track_release) = &mut track.track_release {
                track_release.release_reference = format!("R{k}T{n}");
            }
        }
        request.releases.push(release);
        for deal in &deals {
            let mut deal = deal.clone();
            deal.release_references = deal
                .release_references
                .iter()
                .map(|r| match r.as_str() {
                    "R0" => format!("R{k}"),
                    track_release => format!("R{k}T{}", &track_release[1..]),
                })
                .collect();
            request.deals.push(deal);
        }
    }
    request
}

/// Audit rows D25, H1–H8: a large request built on two fresh threads gives the same bytes.
#[test]
fn large_ern43_request_is_byte_identical_across_threads() {
    let request = large_ern43();
    assert_eq!(
        build_in_fresh_thread(request.clone()),
        build_in_fresh_thread(request)
    );
}

/// Audit rows D25, H1–H8 (3.8.2: many main releases).
#[test]
fn large_ern382_request_is_byte_identical_across_threads() {
    let request = large_ern382();
    assert_eq!(
        build_in_fresh_thread(request.clone()),
        build_in_fresh_thread(request)
    );
}
