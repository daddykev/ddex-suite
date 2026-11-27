//! ERN 3.8.x round trips: parse -> build 3.8.2 -> XSD validation -> re-parse, comparing the
//! re-parsed flat output with the original on: release count, per release title / ICPN /
//! type / display artist / track count, per track ISRC / title / duration, deal count and
//! deal release references.
//!
//! Sources:
//! - YouTube's published ERN 3.8.2 sample messages (`scripts/fetch_ern382_samples.sh`,
//!   fetched when missing; not vendored). No official DDEX 3.8.2 samples are available.
//! - The 3.8.x fixtures in this repository (`ddex-parser/tests/fixtures`).
//! - The builder's own 3.8.2 output for the XSD-test requests (single, album, multi-disc).
//!
//! Some sources cannot be rebuilt: the request model has no video resources for 3.8.x and
//! no `TakeDown` deals, and some fixtures lack values the 3.8.2 XSD requires. Those builds
//! must fail naming the field; `EXPECTED_FAILURES` lists them. Run with `--nocapture` to
//! print the results table.

use crate::ern4_support::{
    build, commercial_model_name, iso_duration, minimal_single, multi_disc_album,
    multi_track_album, parse, release_ids, release_type_name, text, use_type_name, validate_xsd,
    xsd_skipped,
};
use ddex_builder::builder::{
    AvsValueRequest, DealRequest, DealTerms, MessageHeaderRequest, PartyRequest,
    ProprietaryIdRequest, ReleaseRequest, ResourceKind, TrackReleaseRequest, TrackRequest,
};
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};
use ddex_core::models::common::{Identifier, IdentifierType, LocalizedString};
use ddex_core::models::flat::{FlattenedMessage, ParsedERNMessage};
use ddex_core::models::graph::{ReleaseType, ResourceType};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Sources whose build must fail: (source name, fragment of the expected error, reason).
/// `youtube/` entries are file-name prefixes; the others are exact source names.
const EXPECTED_FAILURES: &[(&str, &str, &str)] = &[
    (
        "youtube/721620118165_",
        "requires an identifier in Release/ReleaseId (track release)",
        "builder: ReleaseIdRequest has no ISRC; the TrackReleases' only identifier is ReleaseId/ISRC (parsed since 0.5.0)",
    ),
    (
        "youtube/USGCV1172318_",
        "requires an identifier in Release/ReleaseId (releases[0].release_ids)",
        "builder: ReleaseIdRequest has no ISRC; the release's only identifier is ReleaseId/ISRC (parsed since 0.5.0)",
    ),
    (
        "youtube/QZ6RS1700001_music_video",
        "resource_kind is Video",
        "builder: no Video resources for 3.8.x",
    ),
    ("youtube/music_video", "resource_kind is Video", "builder: no Video resources for 3.8.x"),
    ("youtube/web_video", "resource_kind is Video", "builder: no Video resources for 3.8.x"),
    (
        "fixtures/ern382_basic.xml",
        "requires a ReleaseResourceReference in Release/ReleaseResourceReferenceList (releases[0].tracks)",
        "fixture: no resources",
    ),
    (
        "fixtures/ern_381.xml",
        "requires an identifier in Release/ReleaseId (releases[0].release_ids)",
        "fixture: no ReleaseId",
    ),
    (
        "fixtures/image_without_reference_title.xml",
        "requires an identifier in Release/ReleaseId (releases[0].release_ids)",
        "fixture: no ReleaseId",
    ),
    (
        "fixtures/sound_recording_without_duration.xml",
        "requires SoundRecording/Duration (releases[0].tracks[0].duration)",
        "fixture: no ReleaseId, no Duration",
    ),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn youtube_samples() -> Vec<PathBuf> {
    let dir = repo_root().join("samples/youtube-ern382");
    if !dir.is_dir() || std::fs::read_dir(&dir).map_or(0, |d| d.count()) < 20 {
        let script = repo_root().join("scripts/fetch_ern382_samples.sh");
        let out = Command::new("sh")
            .arg(&script)
            .output()
            .unwrap_or_else(|e| panic!("could not run {}: {e}", script.display()));
        assert!(
            out.status.success(),
            "fetching the ERN 3.8.2 samples failed. Run scripts/fetch_ern382_samples.sh \
             (needs network access), or set DDEX_SKIP_SAMPLES=1 to skip them explicitly.\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    sorted_xml(&dir)
}

fn sorted_xml(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("samples directory")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "xml"))
        .collect();
    files.sort();
    files
}

fn repo_fixtures() -> Vec<PathBuf> {
    let dir = repo_root().join("packages/ddex-parser/tests/fixtures");
    sorted_xml(&dir)
        .into_iter()
        .filter(|p| {
            std::fs::read_to_string(p).is_ok_and(|x| {
                x.contains("ddex.net/xml/ern/381") || x.contains("ddex.net/xml/ern/382")
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Parsed ERN 3.8.x message -> BuildRequest
// ---------------------------------------------------------------------------

fn texts(strings: &[LocalizedString]) -> Vec<ddex_builder::builder::LocalizedStringRequest> {
    strings
        .iter()
        .map(|s| ddex_builder::builder::LocalizedStringRequest {
            text: s.text.clone(),
            language_code: s.language_code.clone(),
        })
        .collect()
}

fn proprietary(ids: &[Identifier]) -> Vec<ProprietaryIdRequest> {
    ids.iter()
        .filter(|id| matches!(id.id_type, IdentifierType::Proprietary))
        .filter_map(|id| {
            Some(ProprietaryIdRequest {
                namespace: id.namespace.clone()?,
                value: id.value.clone(),
            })
        })
        .collect()
}

fn is_track_release(t: &Option<ReleaseType>) -> bool {
    matches!(t, Some(ReleaseType::Other(s)) if s == "TrackRelease")
}

/// Map a parsed ERN 3.8.x message to a build request for `version`. `TrackRelease`
/// releases of a single resource become that track's `track_release`.
fn request_from_parsed_38(parsed: &ParsedERNMessage, version: &str) -> BuildRequest {
    let graph = &parsed.graph;
    let flat = &parsed.flat;
    let header = &graph.message_header;
    let party = |ids: &[Identifier], names: &[LocalizedString]| PartyRequest {
        party_name: names.first().map(|n| text(&n.text)).unwrap_or_default(),
        party_id: ids.first().map(|i| i.value.clone()),
        party_reference: None,
    };
    assert_eq!(
        graph.releases.len(),
        flat.releases.len(),
        "graph/flat release counts"
    );

    let track_releases: Vec<_> = graph
        .releases
        .iter()
        .filter(|r| {
            is_track_release(&r.release_type) && r.release_resource_reference_list.len() == 1
        })
        .collect();

    let mut releases = Vec::new();
    for (release, flat_release) in graph.releases.iter().zip(&flat.releases) {
        if track_releases.iter().any(|tr| std::ptr::eq(*tr, release)) {
            continue;
        }
        let tracks = flat_release
            .tracks
            .iter()
            .map(|t| {
                let resource = graph
                    .resources
                    .iter()
                    .find(|r| r.resource_reference == t.track_id)
                    .expect("track resource");
                let track_release = track_releases
                    .iter()
                    .find(|tr| {
                        tr.release_resource_reference_list[0].resource_reference == t.track_id
                    })
                    .map(|tr| TrackReleaseRequest {
                        release_reference: tr.release_reference.clone(),
                        release_ids: release_ids(&tr.release_id),
                        title: tr.release_title.first().map(|t| t.text.clone()),
                        ..Default::default()
                    });
                TrackRequest {
                    track_id: t.track_id.clone(),
                    resource_reference: Some(t.track_id.clone()),
                    isrc: t.isrc.clone().filter(|s| !s.is_empty()),
                    title: t.title.clone().unwrap_or_default(),
                    duration: t
                        .duration
                        .map(|d| iso_duration(d.as_secs()))
                        .unwrap_or_default(),
                    artist: resource
                        .display_artist
                        .first()
                        .and_then(|a| a.display_artist_name.first())
                        .map(|n| n.text.clone())
                        .filter(|s| !s.is_empty()),
                    resource_kind: match resource.resource_type {
                        ResourceType::Video => ResourceKind::Video,
                        _ => ResourceKind::SoundRecording,
                    },
                    proprietary_ids: proprietary(&resource.resource_id),
                    track_release,
                    ..Default::default()
                }
            })
            .collect();
        releases.push(ReleaseRequest {
            release_id: None,
            release_reference: Some(release.release_reference.clone()),
            title: texts(&release.release_title),
            artist: flat_release
                .display_artist
                .clone()
                .filter(|s| !s.is_empty()),
            tracks,
            release_type: release
                .release_type
                .as_ref()
                .map(|t| AvsValueRequest::from(release_type_name(t).as_str())),
            release_ids: Some(release_ids(&release.release_id)),
            ..Default::default()
        });
    }

    let deals = graph
        .deals
        .iter()
        .map(|d| DealRequest {
            deal_reference: d.deal_reference.clone(),
            deal_terms: DealTerms {
                commercial_model_type: d
                    .deal_terms
                    .commercial_model_type
                    .first()
                    .map(commercial_model_name)
                    .filter(|s| !s.is_empty()),
                territory_code: d.deal_terms.territory_code.clone(),
                start_date: d
                    .deal_terms
                    .start_date
                    .map(|s| s.format("%Y-%m-%d").to_string()),
                end_date: d
                    .deal_terms
                    .end_date
                    .map(|s| s.format("%Y-%m-%d").to_string()),
                use_types: d.deal_terms.use_type.iter().map(use_type_name).collect(),
            },
            release_references: d.deal_release_reference.clone(),
        })
        .collect();

    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some(header.message_id.clone()),
            message_sender: party(
                &header.message_sender.party_id,
                &header.message_sender.party_name,
            ),
            message_recipient: party(
                &header.message_recipient.party_id,
                &header.message_recipient.party_name,
            ),
            message_control_type: None,
            message_created_date_time: Some(
                flat.message_date
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            ),
        },
        version: version.to_string(),
        releases,
        deals,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Comparison
// ---------------------------------------------------------------------------

/// Differences between the original and the re-parsed message on the compared fields.
fn compare(original: &FlattenedMessage, rebuilt: &FlattenedMessage) -> Vec<String> {
    let mut diffs = Vec::new();
    let mut check = |what: String, a: String, b: String| {
        if a != b {
            diffs.push(format!("{what}: {a} -> {b}"));
        }
    };
    check(
        "release count".into(),
        original.releases.len().to_string(),
        rebuilt.releases.len().to_string(),
    );
    for (n, (o, r)) in original.releases.iter().zip(&rebuilt.releases).enumerate() {
        let at = |f: &str| format!("release {} {f}", n + 1);
        check(
            at("title"),
            format!("{:?}", o.default_title),
            format!("{:?}", r.default_title),
        );
        check(
            at("ICPN"),
            format!("{:?}", o.identifiers.upc),
            format!("{:?}", r.identifiers.upc),
        );
        check(at("type"), o.release_type.clone(), r.release_type.clone());
        check(
            at("display artist"),
            format!("{:?}", o.display_artist),
            format!("{:?}", r.display_artist),
        );
        check(
            at("track count"),
            o.tracks.len().to_string(),
            r.tracks.len().to_string(),
        );
        for (i, (ot, rt)) in o.tracks.iter().zip(&r.tracks).enumerate() {
            let t = |f: &str| format!("release {} track {} {f}", n + 1, i + 1);
            check(
                t("ISRC"),
                format!("{:?}", ot.isrc),
                format!("{:?}", rt.isrc),
            );
            check(
                t("title"),
                format!("{:?}", ot.title),
                format!("{:?}", rt.title),
            );
            check(
                t("duration"),
                format!("{:?}", ot.duration.map(|d| d.as_secs())),
                format!("{:?}", rt.duration.map(|d| d.as_secs())),
            );
        }
    }
    check(
        "deal count".into(),
        original.deals.len().to_string(),
        rebuilt.deals.len().to_string(),
    );
    for (i, (od, rd)) in original.deals.iter().zip(&rebuilt.deals).enumerate() {
        check(
            format!("deal {} releases", i + 1),
            format!("{:?}", od.releases),
            format!("{:?}", rd.releases),
        );
    }
    diffs
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

/// `multi_track_album` with GRids instead of proprietary IDs on the track releases (both
/// identifier kinds must survive the 3.8.2 round trip).
fn album_with_grid_track_releases(version: &str) -> BuildRequest {
    let mut r = multi_track_album(version);
    for (n, track) in r.releases[0].tracks.iter_mut().enumerate() {
        let tr = track.track_release.as_mut().unwrap();
        tr.release_ids = ddex_builder::builder::ReleaseIdRequest {
            grid: Some(format!("A1-ABCDE-000000001{n}-M")),
            ..Default::default()
        };
    }
    r
}

struct Outcome {
    xsd: String,
    fields: String,
    notes: String,
}

/// parse -> build 3.8.2 -> XSD -> re-parse -> compare. `Err` holds the build error.
fn round_trip(name: &str, xml: &str, failures: &mut Vec<String>) -> Result<Outcome, String> {
    let original = parse(xml);
    let request = request_from_parsed_38(&original, "3.8.2");
    let rebuilt = DDEXBuilder::new()
        .build(request, BuildOptions::default())
        .map_err(|e| e.to_string())?
        .xml;
    if let Ok(dir) = std::env::var("DDEX_SAMPLES_OUT") {
        let out = PathBuf::from(dir).join(format!("ern382_{}", name.replace('/', "_")));
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        std::fs::write(out, &rebuilt).unwrap();
    }
    let xsd = if xsd_skipped() {
        "skipped".to_string()
    } else {
        match validate_xsd(&rebuilt, "3.8.2") {
            Ok(()) => "yes".to_string(),
            Err(errors) => {
                failures.push(format!("{name}: not schema-valid:\n{errors}"));
                "**no**".to_string()
            }
        }
    };
    let diffs = compare(&original.flat, &parse(&rebuilt).flat);
    if !diffs.is_empty() {
        failures.push(format!("{name}: {}", diffs.join("; ")));
    }
    let tracks = original
        .flat
        .releases
        .iter()
        .map(|r| r.tracks.len())
        .sum::<usize>();
    Ok(Outcome {
        xsd,
        fields: if diffs.is_empty() {
            "yes".into()
        } else {
            "**no**".into()
        },
        notes: if diffs.is_empty() {
            format!(
                "{} releases, {tracks} tracks, {} deals",
                original.flat.releases.len(),
                original.flat.deals.len()
            )
        } else {
            diffs.join("; ")
        },
    })
}

#[test]
fn ern38_round_trip() {
    let mut sources: Vec<(String, String)> = Vec::new();
    if std::env::var("DDEX_SKIP_SAMPLES").as_deref() != Ok("1") {
        for p in youtube_samples() {
            let name = format!("youtube/{}", p.file_name().unwrap().to_string_lossy());
            sources.push((name, std::fs::read_to_string(&p).unwrap()));
        }
    }
    for p in repo_fixtures() {
        let name = format!("fixtures/{}", p.file_name().unwrap().to_string_lossy());
        sources.push((name, std::fs::read_to_string(&p).unwrap()));
    }
    for (name, make) in [
        ("minimal single", minimal_single as fn(&str) -> BuildRequest),
        ("multi-track album", multi_track_album),
        (
            "multi-track album, GRid track releases",
            album_with_grid_track_releases,
        ),
        ("multi-disc album", multi_disc_album),
    ] {
        sources.push((format!("builder/{name}"), build(make("3.8.2"))));
    }

    let mut table = vec![
        "| source | build | XSD valid | fields equal | notes |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ];
    let mut failures = Vec::new();
    for (name, xml) in &sources {
        let expected = EXPECTED_FAILURES
            .iter()
            .find(|(n, _, _)| n == name || (n.starts_with("youtube/") && name.starts_with(n)));
        match (round_trip(name, xml, &mut failures), expected) {
            (Ok(o), None) => table.push(format!(
                "| {name} | ok | {} | {} | {} |",
                o.xsd, o.fields, o.notes
            )),
            (Ok(_), Some((_, e, _))) => {
                failures.push(format!("{name}: expected the build to fail with {e:?}, it succeeded; update EXPECTED_FAILURES"));
                table.push(format!(
                    "| {name} | **ok (expected failure)** | - | - | - |"
                ));
            }
            (Err(err), Some((_, e, reason))) if err.contains(e) => {
                table.push(format!(
                    "| {name} | fails as expected | - | - | {reason}: \"{e}\" |"
                ));
            }
            (Err(err), _) => {
                failures.push(format!("{name}: build failed: {err}"));
                table.push(format!("| {name} | **failed** | - | - | {err} |"));
            }
        }
    }
    println!("{}", table.join("\n"));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The official DDEX 4.2 / 4.3 samples (examples/Samples42, Samples43) rebuilt as ERN
/// 3.8.2: realistic data through the 3.8.2 generator, checked against the 3.8.2 XSD. The
/// main release and the deals are compared with the original (3.8.x has no `TrackRelease`
/// element, so the rebuilt message lists track releases as further `Release`s). Samples
/// with video resources must fail with only the "no Video for 3.8.x" errors; samples with an
/// empty `<MessageId/>` may also fail on the 3.8.2 builder's MessageId requirement.
#[test]
fn official_4x_samples_rebuilt_as_ern382() {
    use crate::ern4_support::request_from_parsed;
    let root = repo_root().join("examples");
    let mut table = vec![
        "| sample | build | XSD valid | main release equal | notes |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ];
    let mut failures = Vec::new();
    for dir in ["Samples42", "Samples43"] {
        for path in sorted_xml(&root.join(dir)) {
            let name = format!("{dir}/{}", path.file_name().unwrap().to_string_lossy());
            let original = parse(&std::fs::read_to_string(&path).unwrap());
            let request = request_from_parsed(&original, "3.8.2");
            let xml = match DDEXBuilder::new().build(request, BuildOptions::default()) {
                Ok(r) => r.xml,
                // Expected: samples with video resources, and nothing else wrong with them
                Err(ddex_builder::BuildError::ValidationFailed { errors })
                    if errors.iter().all(|e| e.contains("resource_kind is Video")) =>
                {
                    table.push(format!(
                        "| {name} | fails as expected | - | - | builder: no Video resources for 3.8.x ({} video resources) |",
                        errors.len()
                    ));
                    continue;
                }
                // Expected: the sample's `<MessageId/>` is empty (schema-valid xs:string, read
                // as ""), and the 3.8.2 builder treats an empty MessageId as missing.
                Err(ddex_builder::BuildError::ValidationFailed { errors })
                    if original.graph.message_header.message_id.is_empty()
                        && errors.iter().all(|e| {
                            e.contains("resource_kind is Video")
                                || e.contains("requires MessageHeader/MessageId")
                        }) =>
                {
                    table.push(format!(
                        "| {name} | fails as expected | - | - | builder: empty MessageId (`<MessageId/>` in the sample) is treated as missing for 3.8.2 |"
                    ));
                    continue;
                }
                Err(e) => {
                    table.push(format!("| {name} | **failed** | - | - | {e} |"));
                    failures.push(format!("{name}: build failed: {e}"));
                    continue;
                }
            };
            let xsd = if xsd_skipped() {
                "skipped".to_string()
            } else {
                match validate_xsd(&xml, "3.8.2") {
                    Ok(()) => "yes".to_string(),
                    Err(errors) => {
                        failures.push(format!("{name}: not schema-valid:\n{errors}"));
                        "**no**".to_string()
                    }
                }
            };
            let rebuilt = parse(&xml);
            let mut o = original.flat.clone();
            let mut r = rebuilt.flat.clone();
            o.releases.truncate(1);
            r.releases.truncate(1);
            let diffs = compare(&o, &r);
            if !diffs.is_empty() {
                failures.push(format!("{name}: {}", diffs.join("; ")));
            }
            table.push(format!(
                "| {name} | ok | {xsd} | {} | {} |",
                if diffs.is_empty() { "yes" } else { "**no**" },
                if diffs.is_empty() {
                    format!(
                        "{} tracks, {} deals",
                        o.releases[0].tracks.len(),
                        original.flat.deals.len()
                    )
                } else {
                    diffs.join("; ")
                }
            ));
        }
    }
    println!("{}", table.join("\n"));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
