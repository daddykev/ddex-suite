//! Shared helpers for the ERN 4.x builder tests: complete build requests and XSD
//! validation against the official DDEX schemas with `xmllint`.
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
        isrc: format!("USRT1240{n:04}"),
        title: title.to_string(),
        duration: "PT3M42S".to_string(),
        artist: "Example Artist".to_string(),
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
        release_id: "A1-ABCDE-0000000001-M".to_string(),
        release_reference: Some("R0".to_string()),
        title: text(title),
        artist: "Example Artist".to_string(),
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
            commercial_model_type: model.to_string(),
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
            deal(&["R1", "R2", "R3", "R4"], "PayAsYouGoModel", "PermanentDownload"),
        ],
    )
}

/// Two discs (disc_number + sequence_number per track).
pub fn multi_disc_album(version: &str) -> BuildRequest {
    let mut tracks = Vec::new();
    for (n, (disc, seq)) in [(1, 1), (1, 2), (1, 3), (2, 1), (2, 2)].into_iter().enumerate() {
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
        let ok = Command::new("xmllint").arg("--version").output().is_ok_and(|o| o.status.success());
        assert!(
            ok,
            "xmllint not found. Install libxml2 (macOS: it ships with the system or `brew install \
             libxml2`; Debian/Ubuntu: `apt install libxml2-utils`), or set DDEX_SKIP_XSD=1 to \
             skip XSD validation explicitly."
        );
        let root = repo_root();
        let schemas = root.join("schemas");
        let have = |v: &str| schemas.join(format!("ern/{v}/release-notification.xsd")).is_file();
        if !(have("42") && have("43")) {
            let script = root.join("scripts/fetch_ddex_schemas.sh");
            let out = Command::new("sh")
                .arg(&script)
                .output()
                .unwrap_or_else(|e| panic!("could not run {}: {e}", script.display()));
            assert!(
                out.status.success() && have("42") && have("43"),
                "fetching the DDEX XSDs failed ({}). Run scripts/fetch_ddex_schemas.sh (needs \
                 network access), or set DDEX_SKIP_XSD=1 to skip XSD validation explicitly.\n{}",
                script.display(),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        schemas
    })
}

/// Validate `xml` against the official ERN `version` XSD. Returns xmllint's errors.
/// Panics if the tooling is missing (unless `DDEX_SKIP_XSD=1`, then always `Ok`).
pub fn validate_xsd(xml: &str, version: &str) -> Result<(), String> {
    if xsd_skipped() {
        return Ok(());
    }
    let schemas = ensure_xsd_tooling();
    let xsd = schemas.join(format!("ern/{}/release-notification.xsd", version.replace('.', "")));
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
    r.language_and_script_code.get_or_insert_with(|| "en".to_string());
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
        release.label.get_or_insert_with(|| "Test Label".to_string());
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

use ddex_builder::builder::{ResourceGroupContentItemRequest, ResourceGroupRequest};
use ddex_core::models::common::{Identifier, IdentifierType};
use ddex_core::models::flat::ParsedERNMessage;
use ddex_core::models::graph::{
    Artist, CommercialModelType, DisplayText, ERNMessage, ReleaseType, ResourceGroup,
    ResourceType, UseType,
};

fn iso_duration(seconds: u64) -> String {
    format!("PT{}M{}S", seconds / 60, seconds % 60)
}

fn root_attr(graph: &ERNMessage, name: &str) -> Option<String> {
    graph
        .attributes
        .as_ref()?
        .get_str(name)
        .map(|v| v.to_xml_value())
}

fn party_name(graph: &ERNMessage, reference: &str) -> Option<String> {
    graph
        .parties
        .iter()
        .find(|p| p.party_reference.as_deref() == Some(reference))
        .and_then(|p| p.party_name.first())
        .map(|n| n.text.clone())
}

fn release_ids(ids: &[Identifier]) -> ReleaseIdRequest {
    let mut out = ReleaseIdRequest::default();
    for id in ids {
        match id.id_type {
            IdentifierType::GRID | IdentifierType::GRid => out.grid = Some(id.value.clone()),
            IdentifierType::UPC => out.icpn = Some(id.value.clone()),
            IdentifierType::Proprietary => {
                if let Some(ns) = &id.namespace {
                    out.proprietary_ids.push(ProprietaryIdRequest {
                        namespace: ns.clone(),
                        value: id.value.clone(),
                    });
                }
            }
            _ => {}
        }
    }
    out
}

/// A role other than the builder's default `MainArtist`.
fn artist_role(artists: &[Artist]) -> Option<AvsValueRequest> {
    artists
        .first()
        .and_then(|a| a.artist_role.first())
        .filter(|r| r.as_str() != "MainArtist")
        .map(|r| r.as_str().into())
}

/// Default text first (the builder writes no `IsDefault`, so the default must lead).
fn titles(texts: &[DisplayText]) -> Vec<LocalizedStringRequest> {
    let Some(default) = DisplayText::pick_default(texts) else {
        return Vec::new();
    };
    std::iter::once(default)
        .chain(texts.iter().filter(|t| !std::ptr::eq(*t, default)))
        .map(|t| LocalizedStringRequest {
            text: t.text.clone(),
            language_code: t.language_and_script_code.clone(),
        })
        .collect()
}

fn release_type_name(t: &ReleaseType) -> String {
    match t {
        ReleaseType::Album => "Album".to_string(),
        ReleaseType::Single => "Single".to_string(),
        ReleaseType::EP => "EP".to_string(),
        ReleaseType::Compilation => "Compilation".to_string(),
        ReleaseType::Other(s) => s.clone(),
    }
}

fn commercial_model_name(c: &CommercialModelType) -> String {
    match c {
        CommercialModelType::PayAsYouGoModel => "PayAsYouGoModel".to_string(),
        CommercialModelType::SubscriptionModel => "SubscriptionModel".to_string(),
        CommercialModelType::AdSupportedModel => "AdSupportedModel".to_string(),
        CommercialModelType::Other(s) => s.clone(),
    }
}

fn use_type_name(u: &UseType) -> String {
    match u {
        UseType::Stream => "Stream".to_string(),
        UseType::Download => "Download".to_string(),
        UseType::OnDemandStream => "OnDemandStream".to_string(),
        UseType::NonInteractiveStream => "NonInteractiveStream".to_string(),
        UseType::Other(s) => s.clone(),
    }
}

/// The group tree with content items limited to `track_refs` (images, text etc. are not
/// carried by the builder).
fn group_request(group: &ResourceGroup, track_refs: &[String]) -> ResourceGroupRequest {
    ResourceGroupRequest {
        resource_group_type: group.resource_group_type.clone(),
        sequence_number: group.sequence_number.map(|n| n as u32),
        resource_groups: group
            .resource_groups
            .iter()
            .map(|g| group_request(g, track_refs))
            .collect(),
        content_items: group
            .content_items
            .iter()
            .filter(|i| track_refs.contains(&i.release_resource_reference))
            .map(|i| ResourceGroupContentItemRequest {
                sequence_number: i.sequence_number.map(|n| n as u32),
                resource_reference: i.release_resource_reference.clone(),
            })
            .collect(),
    }
}

/// Map a parsed ERN 4.x message (one main Release) to a build request for `version`,
/// carrying everything the build-request model can hold. See the samples test for what
/// it cannot hold yet.
pub fn request_from_parsed(parsed: &ParsedERNMessage, version: &str) -> BuildRequest {
    let graph = &parsed.graph;
    let flat = &parsed.flat;
    let header = &graph.message_header;
    let party = |ids: &[Identifier], names: &[ddex_core::models::common::LocalizedString]| {
        PartyRequest {
            party_name: names
                .first()
                .map(|n| text(&n.text))
                .unwrap_or_default(),
            party_id: ids.first().map(|i| i.value.clone()),
            party_reference: None,
        }
    };

    let release = &graph.releases[0];
    let flat_release = &flat.releases[0];
    let track_refs: Vec<String> = flat_release.tracks.iter().map(|t| t.track_id.clone()).collect();

    let mut seen = Vec::new();
    let tracks = flat_release
        .tracks
        .iter()
        .filter(|t| {
            let new = !seen.contains(&t.track_id);
            seen.push(t.track_id.clone());
            new
        })
        .map(|t| {
            let resource = graph
                .resources
                .iter()
                .find(|r| r.resource_reference == t.track_id)
                .expect("track resource");
            let track_release = graph
                .track_releases
                .iter()
                .find(|tr| tr.release_resource_reference == t.track_id)
                .map(|tr| TrackReleaseRequest {
                    release_reference: tr.release_reference.clone(),
                    release_ids: release_ids(&tr.release_id),
                    title: DisplayText::pick_default(&tr.display_title_text)
                        .or_else(|| DisplayText::pick_default(&tr.display_title))
                        .map(|d| d.text.clone()),
                    genre: tr.genre.first().map(|g| g.genre_text.clone()),
                    sub_genre: tr.genre.first().and_then(|g| g.sub_genre.clone()),
                    label: tr
                        .release_label_reference
                        .first()
                        .and_then(|r| party_name(graph, r)),
                });
            TrackRequest {
                track_id: t.track_id.clone(),
                resource_reference: Some(t.track_id.clone()),
                isrc: t.isrc.clone().unwrap_or_default(),
                title: t.title.clone().unwrap_or_default(),
                duration: t.duration.map(|d| iso_duration(d.as_secs())).unwrap_or_default(),
                artist: t.display_artist.clone().unwrap_or_default(),
                resource_kind: match resource.resource_type {
                    ResourceType::Video => ResourceKind::Video,
                    _ => ResourceKind::SoundRecording,
                },
                resource_type: resource.type_value.as_ref().map(|v| AvsValueRequest {
                    value: v.clone(),
                    namespace: None,
                    user_defined_value: resource.type_user_defined_value.clone(),
                }),
                parental_warning_type: resource.parental_warning_type.first().cloned(),
                artist_role: artist_role(&resource.display_artist),
                proprietary_ids: resource
                    .resource_id
                    .iter()
                    .filter(|id| matches!(id.id_type, IdentifierType::Proprietary))
                    .filter_map(|id| {
                        Some(ProprietaryIdRequest {
                            namespace: id.namespace.clone()?,
                            value: id.value.clone(),
                        })
                    })
                    .collect(),
                track_release,
                ..Default::default()
            }
        })
        .collect::<Vec<_>>();

    let release_refs: Vec<String> = std::iter::once(release.release_reference.clone())
        .chain(
            tracks
                .iter()
                .filter_map(|t| t.track_release.as_ref())
                .map(|tr| tr.release_reference.clone()),
        )
        .collect();
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
                    .unwrap_or_default(),
                territory_code: d.deal_terms.territory_code.clone(),
                start_date: d.deal_terms.start_date.map(|s| s.format("%Y-%m-%d").to_string()),
                end_date: None,
                use_types: d.deal_terms.use_type.iter().map(use_type_name).collect(),
            },
            release_references: d
                .deal_release_reference
                .iter()
                .filter(|r| release_refs.contains(r))
                .cloned()
                .collect(),
        })
        .collect();

    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some(header.message_id.clone()),
            message_sender: party(&header.message_sender.party_id, &header.message_sender.party_name),
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
        profile: None,
        releases: vec![ReleaseRequest {
            release_id: String::new(),
            release_reference: Some(release.release_reference.clone()),
            title: titles(&release.display_title_text),
            artist: flat_release.display_artist.clone().unwrap_or_default(),
            label: release
                .release_label_reference
                .first()
                .and_then(|r| party_name(graph, r)),
            release_date: None,
            upc: None,
            tracks,
            resource_references: None,
            release_type: release.release_type.as_ref().map(|t| AvsValueRequest {
                value: release_type_name(t),
                namespace: None,
                user_defined_value: release.release_type_user_defined_value.clone(),
            }),
            genre: release.genre.first().map(|g| g.genre_text.clone()),
            sub_genre: release.genre.first().and_then(|g| g.sub_genre.clone()),
            parental_warning_type: release.parental_warning_type.first().cloned(),
            artist_role: artist_role(&release.display_artist),
            release_ids: Some(release_ids(&release.release_id)),
            resource_group: release
                .resource_group
                .as_ref()
                .map(|g| group_request(g, &track_refs)),
        }],
        deals,
        extensions: None,
        language_and_script_code: root_attr(graph, "LanguageAndScriptCode"),
        avs_version_id: if version == "4.3" {
            root_attr(graph, "AvsVersionId")
        } else {
            None
        },
    }
}

pub fn parse(xml: &str) -> ParsedERNMessage {
    ddex_parser::DDEXParser::new()
        .parse(std::io::Cursor::new(xml.as_bytes()))
        .unwrap_or_else(|e| panic!("parser rejected the XML: {e}\n{xml}"))
}
