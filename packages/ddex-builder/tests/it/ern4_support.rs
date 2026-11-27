//! Shared helpers for the ERN 4.x (and 3.8.2) builder tests: complete build requests and
//! XSD validation against the official DDEX schemas with `xmllint`.
//!
//! XSD validation needs `xmllint` (libxml2) and the schemas fetched by
//! `scripts/fetch_ddex_schemas.sh` (run automatically when they are missing). If either is
//! unavailable the tests FAIL with instructions; set `DDEX_SKIP_XSD=1` to skip XSD
//! validation explicitly.

#![allow(dead_code)]

use ddex_builder::builder::{
    AvsValueRequest, DealRequest, DealTerms, LocalizedStringRequest, MessageHeaderRequest,
    PartyRequest, ProprietaryIdRequest, ReleaseIdRequest, ReleaseRequest, ResourceKind,
    TrackReleaseRequest, TrackRequest,
};
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

pub const VERSIONS: [&str; 2] = ["4.2", "4.3"];

pub fn text(value: &str) -> Vec<LocalizedStringRequest> {
    vec![LocalizedStringRequest {
        text: value.to_string(),
        language_code: None,
    }]
}

pub fn party(id: &str, name: &str) -> PartyRequest {
    PartyRequest {
        party_name: text(name),
        party_id: Some(id.to_string()),
        party_reference: None,
    }
}

pub fn header(message_id: &str) -> MessageHeaderRequest {
    MessageHeaderRequest {
        message_id: Some(message_id.to_string()),
        message_sender: party("PADPIDA0000000001", "Example Label"),
        message_recipient: party("PADPIDA0000000002", "Example DSP"),
        message_control_type: Some("LiveMessage".to_string()),
        message_created_date_time: Some("2024-09-13T12:00:00Z".to_string()),
    }
}

/// A complete sound recording.
pub fn track(n: u32, title: &str) -> TrackRequest {
    TrackRequest {
        track_id: format!("TRK{n:03}"),
        resource_reference: Some(format!("A{n}")),
        isrc: Some(format!("USRT1240{n:04}")),
        title: title.to_string(),
        duration: "PT3M42S".to_string(),
        artist: Some("Example Artist".to_string()),
        resource_type: Some("MusicalWorkSoundRecording".into()),
        parental_warning_type: Some("NotExplicit".to_string()),
        ..Default::default()
    }
}

/// A complete video.
pub fn video(n: u32, title: &str) -> TrackRequest {
    TrackRequest {
        resource_kind: ResourceKind::Video,
        resource_type: Some("ShortFormMusicalWorkVideo".into()),
        duration: "PT4M01S".to_string(),
        ..track(n, title)
    }
}

/// A complete release around `tracks`.
pub fn release(title: &str, release_type: &str, tracks: Vec<TrackRequest>) -> ReleaseRequest {
    ReleaseRequest {
        release_id: Some("A1-ABCDE-0000000001-M".to_string()),
        release_reference: Some("R0".to_string()),
        title: text(title),
        artist: Some("Example Artist".to_string()),
        label: Some("Example Records".to_string()),
        upc: Some("123456789012".to_string()),
        tracks,
        release_type: Some(release_type.into()),
        genre: Some("Pop".to_string()),
        parental_warning_type: Some("NotExplicit".to_string()),
        ..Default::default()
    }
}

pub fn deal(release_refs: &[&str], model: &str, use_type: &str) -> DealRequest {
    DealRequest {
        deal_reference: None,
        deal_terms: DealTerms {
            commercial_model_type: Some(model.to_string()),
            territory_code: vec!["Worldwide".to_string()],
            start_date: Some("2024-01-01".to_string()),
            use_types: vec![use_type.to_string()],
            ..Default::default()
        },
        release_references: release_refs.iter().map(|r| r.to_string()).collect(),
    }
}

pub fn request(version: &str, release: ReleaseRequest, deals: Vec<DealRequest>) -> BuildRequest {
    BuildRequest {
        header: header("MSG-ERN4-001"),
        version: version.to_string(),
        profile: None,
        releases: vec![release],
        deals,
        extensions: None,
        language_and_script_code: Some("en".to_string()),
        avs_version_id: (version == "4.3").then(|| "12".to_string()),
    }
}

/// One track, one deal.
pub fn minimal_single(version: &str) -> BuildRequest {
    request(
        version,
        release("Example Single", "Single", vec![track(1, "Example Track")]),
        vec![deal(&["R0"], "SubscriptionModel", "OnDemandStream")],
    )
}

/// Several tracks, each with a TrackRelease; deals on the album and on the TrackReleases;
/// GRid + ICPN, sub-genre, release date and titles in two languages.
pub fn multi_track_album(version: &str) -> BuildRequest {
    let tracks = (1..=4)
        .map(|n| TrackRequest {
            sequence_number: Some(n),
            track_release: Some(TrackReleaseRequest {
                release_reference: format!("R{n}"),
                release_ids: ReleaseIdRequest {
                    proprietary_ids: vec![ProprietaryIdRequest {
                        namespace: "PADPIDA0000000001".to_string(),
                        value: format!("TR-{n}"),
                    }],
                    ..Default::default()
                },
                title: Some(format!("Track {n}")),
                genre: Some("Pop".to_string()),
                label: Some("Example Records".to_string()),
                ..Default::default()
            }),
            ..track(n, &format!("Track {n}"))
        })
        .collect();
    let mut album = release("Example Album", "Album", tracks);
    album.title.push(LocalizedStringRequest {
        text: "Album de ejemplo".to_string(),
        language_code: Some("es".to_string()),
    });
    album.title[0].language_code = Some("en".to_string());
    album.sub_genre = Some("Synth-pop".to_string());
    album.release_date = Some("2024-03-01".to_string());
    album.release_ids = Some(ReleaseIdRequest {
        grid: Some("A1-ABCDE-0000000001-M".to_string()),
        icpn: Some("123456789012".to_string()),
        ..Default::default()
    });
    request(
        version,
        album,
        vec![
            deal(&["R0"], "PayAsYouGoModel", "PermanentDownload"),
            deal(&["R0"], "SubscriptionModel", "OnDemandStream"),
            deal(
                &["R1", "R2", "R3", "R4"],
                "PayAsYouGoModel",
                "PermanentDownload",
            ),
        ],
    )
}

/// Two discs (disc_number + sequence_number per track).
pub fn multi_disc_album(version: &str) -> BuildRequest {
    let mut tracks = Vec::new();
    for (n, (disc, seq)) in [(1, 1), (1, 2), (1, 3), (2, 1), (2, 2)]
        .into_iter()
        .enumerate()
    {
        tracks.push(TrackRequest {
            disc_number: Some(disc),
            sequence_number: Some(seq),
            ..track(n as u32 + 1, &format!("Disc {disc} Track {seq}"))
        });
    }
    request(
        version,
        release("Example Double Album", "Album", tracks),
        vec![deal(&["R0"], "SubscriptionModel", "OnDemandStream")],
    )
}

/// Audio and video resources in one release; a user-defined release type.
pub fn release_with_video(version: &str) -> BuildRequest {
    let mut r = release(
        "Example Bundle",
        "UserDefined",
        vec![track(1, "Audio Track"), video(2, "Music Video")],
    );
    r.release_type = Some(AvsValueRequest {
        value: "UserDefined".to_string(),
        namespace: Some("PADPIDA0000000001".to_string()),
        user_defined_value: Some("Bundle".to_string()),
    });
    request(
        version,
        r,
        vec![deal(&["R0"], "AdvertisementSupportedModel", "Stream")],
    )
}

pub fn build(request: BuildRequest) -> String {
    DDEXBuilder::new()
        .build(request, BuildOptions::default())
        .unwrap_or_else(|e| panic!("build failed: {e}"))
        .xml
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// True when XSD validation is explicitly disabled with `DDEX_SKIP_XSD=1`.
pub fn xsd_skipped() -> bool {
    std::env::var("DDEX_SKIP_XSD").as_deref() == Ok("1")
}

/// Make sure `xmllint` and the schemas are available; panic with instructions otherwise.
fn ensure_xsd_tooling() -> &'static PathBuf {
    static SCHEMAS: OnceLock<PathBuf> = OnceLock::new();
    SCHEMAS.get_or_init(|| {
        let ok = Command::new("xmllint")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success());
        assert!(
            ok,
            "xmllint not found. Install libxml2 (macOS: it ships with the system or `brew install \
             libxml2`; Debian/Ubuntu: `apt install libxml2-utils`), or set DDEX_SKIP_XSD=1 to \
             skip XSD validation explicitly."
        );
        let root = repo_root();
        let schemas = root.join("schemas");
        let have = |v: &str| {
            schemas
                .join(format!("ern/{v}/release-notification.xsd"))
                .is_file()
        };
        let have_all = || have("382") && have("42") && have("43");
        if !have_all() {
            let script = root.join("scripts/fetch_ddex_schemas.sh");
            let out = Command::new("sh")
                .arg(&script)
                .output()
                .unwrap_or_else(|e| panic!("could not run {}: {e}", script.display()));
            assert!(
                out.status.success() && have_all(),
                "fetching the DDEX XSDs failed ({}). Run scripts/fetch_ddex_schemas.sh (needs \
                 network access), or set DDEX_SKIP_XSD=1 to skip XSD validation explicitly.\n{}",
                script.display(),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        schemas
    })
}

/// Validate `xml` against the official ERN `version` XSD (3.8.2, 4.2 or 4.3). Returns
/// xmllint's errors.
/// Panics if the tooling is missing (unless `DDEX_SKIP_XSD=1`, then always `Ok`).
pub fn validate_xsd(xml: &str, version: &str) -> Result<(), String> {
    if xsd_skipped() {
        return Ok(());
    }
    let schemas = ensure_xsd_tooling();
    let xsd = schemas.join(format!(
        "ern/{}/release-notification.xsd",
        version.replace('.', "")
    ));
    let mut file = tempfile::NamedTempFile::new().expect("temp file");
    std::io::Write::write_all(&mut file, xml.as_bytes()).expect("write temp file");
    let out = Command::new("xmllint")
        .args(["--noout", "--nonet", "--schema"])
        .arg(&xsd)
        .arg(file.path())
        .output()
        .expect("run xmllint");
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}

/// Assert `xml` validates, showing the errors and the document otherwise.
pub fn assert_xsd_valid(xml: &str, version: &str, what: &str) {
    if let Err(errors) = validate_xsd(xml, version) {
        panic!("{what}: ERN {version} output is not schema-valid:\n{errors}\n{xml}");
    }
}

/// Test data for tests that are not about ERN 4.x structure (determinism, linking,
/// security, ...) but build a 4.x target: sets, where the request leaves them unset, the
/// values the 4.x schema requires, using explicit test values. (The library itself never
/// fills these in; a 4.x build without them fails.) Sender/recipient IDs that are not
/// DPIDs are replaced by test DPIDs, as the 4.x schema only allows DPIDs there.
pub fn ern4_test_data(mut r: BuildRequest) -> BuildRequest {
    let is_43 = ddex_builder::builder::parse_target_version(&r.version)
        .is_ok_and(|v| v == ddex_core::models::versions::ERNVersion::V4_3);
    r.language_and_script_code
        .get_or_insert_with(|| "en".to_string());
    if is_43 {
        r.avs_version_id.get_or_insert_with(|| "12".to_string());
    }
    for (party, dpid) in [
        (&mut r.header.message_sender, "PADPIDA0000000001"),
        (&mut r.header.message_recipient, "PADPIDA0000000002"),
    ] {
        let is_dpid = party
            .party_id
            .as_deref()
            .is_some_and(|id| id.starts_with("PADPIDA") && id.len() > 7);
        if !is_dpid {
            party.party_id = Some(dpid.to_string());
        }
    }
    for release in &mut r.releases {
        release.release_type.get_or_insert_with(|| "Album".into());
        release.genre.get_or_insert_with(|| "Pop".to_string());
        release
            .parental_warning_type
            .get_or_insert_with(|| "NotExplicit".to_string());
        release
            .label
            .get_or_insert_with(|| "Test Label".to_string());
        for track in &mut release.tracks {
            track
                .resource_type
                .get_or_insert_with(|| "MusicalWorkSoundRecording".into());
            track
                .parental_warning_type
                .get_or_insert_with(|| "NotExplicit".to_string());
        }
    }
    r
}

// ---------------------------------------------------------------------------
// Parsed ERN 4.x message -> BuildRequest (for parse -> build round trips)
// ---------------------------------------------------------------------------

// Parsed message -> build request: the library's (ddex_builder::round_trip::parsed), which
// these tests exercise.
pub use ddex_builder::round_trip::parsed::{
    commercial_model_name, iso_duration, release_ids, release_type_name, use_type_name,
};
use ddex_core::models::flat::ParsedERNMessage;

/// [`ddex_builder::round_trip::request_from_parsed`], panicking on an error.
pub fn request_from_parsed(parsed: &ParsedERNMessage, version: &str) -> ddex_builder::BuildRequest {
    ddex_builder::round_trip::request_from_parsed(parsed, version)
        .unwrap_or_else(|e| panic!("request_from_parsed: {e}"))
}

pub fn parse(xml: &str) -> ParsedERNMessage {
    ddex_parser::DDEXParser::new()
        .parse(std::io::Cursor::new(xml.as_bytes()))
        .unwrap_or_else(|e| panic!("parser rejected the XML: {e}\n{xml}"))
}
