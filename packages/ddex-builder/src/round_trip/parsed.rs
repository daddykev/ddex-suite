//! Parsed message → build request, and the comparison of two parsed messages: the
//! pieces of a Parse → Build → Parse round trip.
//!
//! [`request_from_parsed`] carries everything the build-request model can hold for an
//! ERN 4.x message with one main release (the official ERN 4.2 and 4.3 samples round-trip
//! with no difference on the compared fields). It cannot carry contributors, P and C lines,
//! images and other data the request model has no field for; [`compare_flat`] compares
//! only what it carries.

use crate::builder::{
    AvsValueRequest, BuildRequest, DealRequest, DealTerms, LocalizedStringRequest,
    MessageHeaderRequest, PartyRequest, ProprietaryIdRequest, ReleaseIdRequest, ReleaseRequest,
    ResourceGroupContentItemRequest, ResourceGroupRequest, ResourceKind, TrackReleaseRequest,
    TrackRequest,
};
use crate::error::BuildError;
use ddex_core::models::common::{Identifier, IdentifierType};
use ddex_core::models::flat::{FlattenedMessage, ParsedERNMessage};
use ddex_core::models::graph::{
    Artist, CommercialModelType, DisplayText, ERNMessage, ReleaseType, ResourceGroup, ResourceType,
    UseType,
};

fn text(value: &str) -> Vec<LocalizedStringRequest> {
    vec![LocalizedStringRequest {
        text: value.to_string(),
        language_code: None,
    }]
}

/// `PT{minutes}M{seconds}S`, the duration format the builder accepts.
pub fn iso_duration(seconds: u64) -> String {
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

/// The ICPN, GRid and proprietary IDs of a parsed release.
pub fn release_ids(ids: &[Identifier]) -> ReleaseIdRequest {
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

/// The AVS name of a parsed release type.
pub fn release_type_name(t: &ReleaseType) -> String {
    match t {
        ReleaseType::Album => "Album".to_string(),
        ReleaseType::Single => "Single".to_string(),
        ReleaseType::EP => "EP".to_string(),
        ReleaseType::Compilation => "Compilation".to_string(),
        ReleaseType::Other(s) => s.clone(),
    }
}

/// The AVS name of a parsed commercial model.
pub fn commercial_model_name(c: &CommercialModelType) -> String {
    match c {
        CommercialModelType::PayAsYouGoModel => "PayAsYouGoModel".to_string(),
        CommercialModelType::SubscriptionModel => "SubscriptionModel".to_string(),
        CommercialModelType::AdSupportedModel => "AdSupportedModel".to_string(),
        CommercialModelType::Other(s) => s.clone(),
    }
}

/// The AVS name of a parsed use type.
pub fn use_type_name(u: &UseType) -> String {
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

/// Map a parsed ERN 4.x message (its first, main release) to a build request for
/// `version` ("3.8.2", "4.2" or "4.3"), carrying everything the build-request model can
/// hold (see the module docs). Fails if the message has no release or a track's resource
/// is missing.
pub fn request_from_parsed(
    parsed: &ParsedERNMessage,
    version: &str,
) -> Result<BuildRequest, BuildError> {
    let graph = &parsed.graph;
    let flat = &parsed.flat;
    let header = &graph.message_header;
    let party =
        |ids: &[Identifier], names: &[ddex_core::models::common::LocalizedString]| PartyRequest {
            party_name: names.first().map(|n| text(&n.text)).unwrap_or_default(),
            party_id: ids.first().map(|i| i.value.clone()),
            party_reference: None,
        };

    let (Some(release), Some(flat_release)) = (graph.releases.first(), flat.releases.first())
    else {
        return Err(BuildError::MissingRequired {
            field: "the parsed message's releases (it has none)".to_string(),
        });
    };
    let track_refs: Vec<String> = flat_release
        .tracks
        .iter()
        .map(|t| t.track_id.clone())
        .collect();

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
                .ok_or_else(|| BuildError::InvalidReference {
                    reference: format!("track {} has no resource in the message", t.track_id),
                })?;
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
            Ok(TrackRequest {
                track_id: t.track_id.clone(),
                resource_reference: Some(t.track_id.clone()),
                isrc: t.isrc.clone().filter(|s| !s.is_empty()),
                title: t.title.clone().unwrap_or_default(),
                duration: t
                    .duration
                    .map(|d| iso_duration(d.as_secs()))
                    .unwrap_or_default(),
                artist: t.display_artist.clone().filter(|s| !s.is_empty()),
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
            })
        })
        .collect::<Result<Vec<_>, BuildError>>()?;

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
                    .filter(|s| !s.is_empty()),
                territory_code: d.deal_terms.territory_code.clone(),
                start_date: d
                    .deal_terms
                    .start_date
                    .map(|s| s.format("%Y-%m-%d").to_string()),
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

    Ok(BuildRequest {
        header: MessageHeaderRequest {
            message_id: header.message_id.clone(),
            message_sender: party(
                &header.message_sender.party_id,
                &header.message_sender.party_name,
            ),
            message_recipient: party(
                &header.message_recipient.party_id,
                &header.message_recipient.party_name,
            ),
            message_control_type: None,
            message_created_date_time: flat
                .message_date
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        },
        version: version.to_string(),
        profile: None,
        releases: vec![ReleaseRequest {
            release_id: None,
            release_reference: Some(release.release_reference.clone()),
            title: titles(&release.display_title_text),
            artist: flat_release
                .display_artist
                .clone()
                .filter(|s| !s.is_empty()),
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
    })
}

/// Differences between an original and a re-parsed message on the fields
/// [`request_from_parsed`] carries: release count, title, ICPN, type and display artist;
/// track count and each track's ISRC, title, duration, disc and position; deal count and
/// each deal's release references. Each difference reads `what: original -> rebuilt`.
pub fn compare_flat(original: &FlattenedMessage, rebuilt: &FlattenedMessage) -> Vec<String> {
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
    for (o, r) in original.releases.iter().zip(&rebuilt.releases) {
        check(
            "release title".into(),
            format!("{:?}", o.default_title),
            format!("{:?}", r.default_title),
        );
        check(
            "ICPN".into(),
            format!("{:?}", o.identifiers.upc),
            format!("{:?}", r.identifiers.upc),
        );
        check(
            "release type".into(),
            o.release_type.clone(),
            r.release_type.clone(),
        );
        check(
            "display artist".into(),
            format!("{:?}", o.display_artist),
            format!("{:?}", r.display_artist),
        );
        check(
            "track count".into(),
            o.tracks.len().to_string(),
            r.tracks.len().to_string(),
        );
        for (i, (ot, rt)) in o.tracks.iter().zip(&r.tracks).enumerate() {
            let t = |f: &str| format!("track {} {f}", i + 1);
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
            check(
                t("disc"),
                format!("{:?}", ot.disc_number),
                format!("{:?}", rt.disc_number),
            );
            check(
                t("position"),
                ot.position.to_string(),
                rt.position.to_string(),
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
