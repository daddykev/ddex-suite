//! 0.6.0 determinism targets: the behaviour the 0.6.0 builder must have, pinned now.
//!
//! Each test names its row in `dev-docs/DETERMINISM_AUDIT_0.6.0.md` (D* = finding). Tests
//! the 0.5.x code fails are `#[ignore = "known bug: …"]` and listed in `KNOWN_ISSUES.md`;
//! run them with `cargo test -p ddex-builder --test it determinism_0_6 -- --include-ignored`.
//!
//! The requests come from `ern4_support::multi_track_album` (the schema-validated album the
//! XSD tests use); larger ones clone its tracks, releases and deals with distinct references.

use crate::ern4_support::multi_track_album;
use ddex_builder::builder::{
    BuildOptions, BuildRequest, DDEXBuilder, IdStrategy, MessageHeaderRequest,
};
use ddex_builder::streaming::StreamingBuilder;
use ddex_builder::{StableHashConfig, StableHashGenerator};

const VERSIONS: [&str; 3] = ["4.3", "4.2", "3.8.2"];

fn build_with(request: BuildRequest, id_strategy: IdStrategy) -> String {
    let options = BuildOptions {
        id_strategy,
        ..BuildOptions::default()
    };
    DDEXBuilder::new()
        .build(request, options)
        .unwrap_or_else(|e| panic!("build failed ({id_strategy:?}): {e}"))
        .xml
}

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

/// The fixture with the values an ID strategy fills in left out: the `MessageId` and each
/// track's `ResourceReference`. (The release references stay: the deals refer to them.)
fn without_generated_ids(mut request: BuildRequest) -> BuildRequest {
    request.header.message_id = None;
    for release in &mut request.releases {
        for track in &mut release.tracks {
            track.resource_reference = None;
        }
    }
    request
}

/// The text of the first `<MessageId>` (prefixed or not).
fn message_id(xml: &str) -> String {
    let start = xml
        .match_indices("MessageId>")
        .find(|(i, _)| *i > 0 && xml.as_bytes()[i - 1] != b'/')
        .map(|(i, m)| i + m.len())
        .unwrap_or_else(|| panic!("no MessageId in\n{xml}"));
    let end = start + xml[start..].find('<').expect("MessageId is closed");
    xml[start..end].to_string()
}

/// The MessageId must not come from the clock: no UTC date of today, and no run of digits
/// within two days of the current Unix time (seconds or milliseconds).
#[allow(
    clippy::disallowed_methods,
    reason = "test only: reads the clock to check the MessageId is not time-derived"
)]
fn assert_not_time_derived(id: &str) {
    let now = chrono::Utc::now();
    for format in ["%Y-%m-%d", "%Y%m%d"] {
        let today = now.format(format).to_string();
        assert!(
            !id.contains(&today),
            "MessageId {id:?} contains today's date {today}"
        );
    }
    let unix = now.timestamp();
    for run in id
        .split(|c: char| !c.is_ascii_digit())
        .filter(|run| !run.is_empty())
    {
        if let Ok(n) = run.parse::<i64>() {
            for seconds in [n, n / 1000] {
                assert!(
                    (seconds - unix).abs() > 2 * 86_400,
                    "MessageId {id:?} contains {run}, within two days of the current Unix time"
                );
            }
        }
    }
}

fn generated_message_id(id_strategy: IdStrategy) -> String {
    let mut request = multi_track_album("4.3");
    request.header.message_id = None;
    message_id(&build_with(request, id_strategy))
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

/// Audit rows D1, D2: with default options, a request that leaves the MessageId and the
/// resource references to the builder still builds byte-identically.
#[test]
#[ignore = "known bug: default IdStrategy::UUID fills an omitted MessageId and references with random UUIDs (0.6.0 determinism)"]
fn default_options_generated_ids_are_byte_identical() {
    for version in VERSIONS {
        let request = without_generated_ids(multi_track_album(version));
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

// --- b. Every IdStrategy ------------------------------------------------------------------

fn assert_strategy_repeatable(id_strategy: IdStrategy) {
    for version in VERSIONS {
        let request = without_generated_ids(multi_track_album(version));
        assert_eq!(
            build_with(request.clone(), id_strategy),
            build_with(request, id_strategy),
            "{version}, {id_strategy:?}"
        );
    }
}

/// Audit rows D1, D2.
#[test]
#[ignore = "known bug: IdStrategy::UUID generates random UUID v4 IDs (0.6.0 determinism)"]
fn id_strategy_uuid_is_byte_identical() {
    assert_strategy_repeatable(IdStrategy::UUID);
}

/// Audit row D3.
#[test]
#[ignore = "known bug: IdStrategy::UUIDv7 falls back to random UUID v4 IDs (0.6.0 determinism)"]
fn id_strategy_uuidv7_is_byte_identical() {
    assert_strategy_repeatable(IdStrategy::UUIDv7);
}

/// Audit row D4. (Passes when both builds fall in the same second.)
#[test]
#[ignore = "known bug: IdStrategy::Sequential MessageId is the current Unix time (0.6.0 determinism)"]
fn id_strategy_sequential_is_byte_identical() {
    assert_strategy_repeatable(IdStrategy::Sequential);
}

/// Audit rows D5, D6.
#[test]
#[ignore = "known bug: IdStrategy::StableHash hashes today's date into the MessageId, and its B3:/SHA256: references are not valid ERN references (0.6.0 determinism)"]
fn id_strategy_stable_hash_is_byte_identical() {
    assert_strategy_repeatable(IdStrategy::StableHash);
}

// --- c. MessageId is not derived from the clock ----------------------------------------------

/// Audit rows D1, D3: random, but not time-derived.
#[test]
fn uuid_message_ids_are_not_time_derived() {
    for strategy in [IdStrategy::UUID, IdStrategy::UUIDv7] {
        assert_not_time_derived(&generated_message_id(strategy));
    }
}

/// Audit row D4.
#[test]
#[ignore = "known bug: IdStrategy::Sequential MessageId is the current Unix time (0.6.0 determinism)"]
fn sequential_message_id_is_not_time_derived() {
    assert_not_time_derived(&generated_message_id(IdStrategy::Sequential));
}

/// Audit row D5: the hash is not today's date in clear, so this also checks it is not the
/// stable hash of the same inputs with today's date added.
#[test]
#[ignore = "known bug: IdStrategy::StableHash hashes today's date into the MessageId (0.6.0 determinism)"]
#[allow(
    clippy::disallowed_methods,
    reason = "test only: reads the clock to check the MessageId is not time-derived"
)]
fn stable_hash_message_id_is_not_time_derived() {
    let id = generated_message_id(IdStrategy::StableHash);
    assert_not_time_derived(&id);
    let header = multi_track_album("4.3").header;
    let today = chrono::Utc::now().format("%Y%m%d").to_string();
    let with_today = StableHashGenerator::new(StableHashConfig::default())
        .generate_party_id(
            &format!(
                "{}-{}",
                header.message_sender.party_name[0].text,
                header.message_recipient.party_name[0].text
            ),
            "MessageHeader",
            &[today],
        )
        .expect("stable hash");
    assert_ne!(id, with_today, "the MessageId is a hash of today's date");
}

// --- d. MessageCreatedDateTime is required --------------------------------------------------

/// Audit row D8. `message_created_date_time` is an `Option`, so "omitted" is `None`.
#[test]
#[ignore = "known bug: an omitted MessageCreatedDateTime is filled with the current time instead of an error (0.6.0 determinism)"]
fn omitted_message_created_date_time_is_an_error() {
    for version in VERSIONS {
        let mut request = multi_track_album(version);
        request.header.message_created_date_time = None;
        let error = DDEXBuilder::new()
            .build(request, BuildOptions::default())
            .expect_err("a build without MessageCreatedDateTime must fail")
            .to_string();
        assert!(
            error.contains("message_created_date_time") || error.contains("MessageCreatedDateTime"),
            "{version}: the error does not name the field: {error}"
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
                    &track.artist,
                    Some(&track.isrc),
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
            &release.release_id,
            &release.title[0].text,
            &release.artist,
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

/// Audit row D14: with the header complete, the streamed bytes repeat (references are
/// SHA-256 of the caller's IDs).
#[test]
fn streaming_writer_full_header_is_byte_identical() {
    let header = multi_track_album("3.8.2").header;
    assert_eq!(stream(&header), stream(&header));
}

/// Audit rows D12, D13.
#[test]
#[ignore = "known bug: the streaming writer fills an omitted MessageId with a random UUID and MessageCreatedDateTime with the current time (0.6.0 determinism)"]
fn streaming_writer_omitted_header_values_are_byte_identical() {
    let mut header = multi_track_album("3.8.2").header;
    header.message_id = None;
    header.message_created_date_time = None;
    assert_eq!(stream(&header), stream(&header));
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
            track.isrc = format!("USRT1240{n:04}");
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
            track.isrc = format!("USRT1{k:03}{n:04}");
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
