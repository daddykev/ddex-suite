//! Parsed message → build request, and the comparison of two parsed messages: the
//! pieces of a Parse → Build → Parse round trip.
//!
//! [`request_from_parsed`] is [`BuildRequest::from_parsed`] for another target version.
//! The request cannot carry contributors, P and C lines, images and other data it has no
//! field for (see [`BuildRequest::from_parsed`]); [`compare_flat`] compares only what it
//! carries.

use crate::builder::{BuildRequest, ProprietaryIdRequest, ReleaseIdRequest};
use crate::error::BuildError;
use ddex_core::models::common::{Identifier, IdentifierType};
use ddex_core::models::flat::{FlattenedMessage, ParsedERNMessage};
use ddex_core::models::graph::{CommercialModelType, ReleaseType, UseType};

/// `PT{minutes}M{seconds}S`, the duration format the builder accepts.
pub fn iso_duration(seconds: u64) -> String {
    format!("PT{}M{}S", seconds / 60, seconds % 60)
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

/// [`BuildRequest::from_parsed`] for a build of `version` ("3.8.2", "4.2" or "4.3")
/// instead of the parsed message's own version (`@AvsVersionId` is kept for 4.3 only).
/// Fails if the message has no release.
pub fn request_from_parsed(
    parsed: &ParsedERNMessage,
    version: &str,
) -> Result<BuildRequest, BuildError> {
    let mut request = BuildRequest::from_parsed(parsed).request;
    if request.releases.is_empty() {
        return Err(BuildError::MissingRequired {
            field: "the parsed message's releases (it has none)".to_string(),
        });
    }
    if version != "4.3" {
        request.avs_version_id = None;
    } else if request.avs_version_id.is_none() {
        request.avs_version_id = parsed
            .graph
            .attributes
            .as_ref()
            .and_then(|a| a.get_str("AvsVersionId"))
            .map(|v| v.to_xml_value());
    }
    request.version = version.to_string();
    Ok(request)
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
