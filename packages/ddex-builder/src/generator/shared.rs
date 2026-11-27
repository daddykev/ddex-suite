//! Pieces shared by the ERN 3.8.x ([`super::ern38`]) and 4.x ([`super::ern4`]) generators:
//! the error collector, the value checks whose XSD patterns are the same in both schema
//! families, and the elements whose content model is the same in both (`ReleaseId`,
//! `Genre`, AVS-typed values, the `ReleaseDeal` grouping).

use crate::ast::Element;
use crate::builder::{
    AvsValueRequest, DealRequest, ReleaseIdRequest, ReleaseRequest, ResourceGroupRequest,
    TrackRequest,
};
use crate::error::BuildError;

// ---------------------------------------------------------------------------
// Error collection
// ---------------------------------------------------------------------------

/// Collects every unmet schema requirement so the caller sees them all at once. Messages
/// name the ERN version, the schema element and the request field.
pub(super) struct Problems {
    label: &'static str,
    errors: Vec<String>,
}

impl Problems {
    pub(super) fn new(label: &'static str) -> Self {
        Self {
            label,
            errors: Vec::new(),
        }
    }

    pub(super) fn label(&self) -> &'static str {
        self.label
    }

    pub(super) fn finish(self) -> Result<(), BuildError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(BuildError::ValidationFailed {
                errors: self.errors,
            })
        }
    }

    /// A message used verbatim (it must name the version itself).
    pub(super) fn push(&mut self, message: String) {
        self.errors.push(message);
    }

    pub(super) fn missing(&mut self, element: &str, field: &str) {
        self.errors.push(format!(
            "ERN {} requires {} ({})",
            self.label, element, field
        ));
    }

    pub(super) fn invalid(&mut self, field: &str, message: impl std::fmt::Display) {
        self.errors
            .push(format!("ERN {}: {} {}", self.label, field, message));
    }

    pub(super) fn require_text(&mut self, value: &str, element: &str, field: &str) {
        if value.is_empty() {
            self.missing(element, field);
        }
    }

    /// A required text that the request type leaves optional (it is optional in another
    /// ERN version): `None` or `""` is missing.
    pub(super) fn require_some_text(&mut self, value: &Option<String>, element: &str, field: &str) {
        if value.as_deref().unwrap_or_default().is_empty() {
            self.missing(element, field);
        }
    }

    /// An optional text: `None` leaves the element out; `Some("")` is an error (before
    /// 0.6.0 an empty string meant "absent").
    pub(super) fn optional_text(&mut self, value: &Option<String>, field: &str) {
        if value.as_deref() == Some("") {
            self.invalid(
                field,
                "is empty; leave it out (None / null) when there is no value",
            );
        }
    }

    /// Every title needs its text (an empty title used to be skipped).
    pub(super) fn title_texts(
        &mut self,
        titles: &[crate::builder::LocalizedStringRequest],
        at: &str,
    ) {
        for (i, title) in titles.iter().enumerate() {
            if title.text.is_empty() {
                self.invalid(&format!("{at}.title[{i}].text"), "is empty");
            }
        }
    }

    pub(super) fn require_some<T>(&mut self, value: &Option<T>, element: &str, field: &str) {
        if value.is_none() {
            self.missing(element, field);
        }
    }

    pub(super) fn reference(&mut self, value: &str, prefix: char, field: &str) {
        if !is_reference(value, prefix) {
            self.invalid(
                field,
                format!(
                    "is {:?}; the schema requires a reference matching {}[\\d\\-_a-zA-Z]+",
                    value, prefix
                ),
            );
        }
    }

    pub(super) fn date(&mut self, value: &str, field: &str) {
        if !is_iso_date(value) {
            self.invalid(
                field,
                format!(
                    "is {:?}; the schema requires an ISO date YYYY[-MM[-DD]]",
                    value
                ),
            );
        }
    }

    pub(super) fn date_time(&mut self, value: &str, field: &str) {
        if !is_xs_date_time(value) {
            self.invalid(
                field,
                format!("is {:?}; the schema requires an xs:dateTime", value),
            );
        }
    }

    /// `Duration`: required, xs:duration.
    pub(super) fn duration(&mut self, value: &str, element: &str, field: &str) {
        if value.is_empty() {
            self.missing(element, field);
        } else if !is_xs_duration(value) {
            self.invalid(
                field,
                format!(
                    "is {:?}; the schema requires an xs:duration such as PT3M45S",
                    value
                ),
            );
        }
    }

    pub(super) fn avs_value(&mut self, value: &AvsValueRequest, at: &str) {
        if value.value.is_empty() {
            self.invalid(&format!("{at}.value"), "is empty");
        }
    }

    pub(super) fn proprietary_ids(&mut self, ids: &ReleaseIdRequest, at: &str) {
        for (i, p) in ids.proprietary_ids.iter().enumerate() {
            if p.namespace.is_empty() {
                self.missing(
                    "ProprietaryId/@Namespace",
                    &format!("{at}.proprietary_ids[{i}].namespace"),
                );
            }
        }
    }

    /// A release's `ReleaseId` must hold at least one identifier.
    pub(super) fn release_identifier(&mut self, release: &ReleaseRequest, at: &str) {
        match &release.release_ids {
            Some(ids) if ids.is_empty() => self.missing(
                "an identifier in Release/ReleaseId",
                &format!("{at}.release_ids"),
            ),
            Some(ids) => self.proprietary_ids(ids, &format!("{at}.release_ids")),
            None => {
                self.optional_text(&release.release_id, &format!("{at}.release_id"));
                if release.release_id.is_none() && release.upc.is_none() {
                    self.missing(
                        "an identifier in Release/ReleaseId",
                        &format!("{at}.release_id, {at}.upc or {at}.release_ids"),
                    );
                }
            }
        }
    }

    /// `Release/ResourceGroup` consistency: either the request's group tree or per-track
    /// disc/sequence numbers; every track is placed, and only tracks of this release.
    pub(super) fn grouping(
        &mut self,
        release: &ReleaseRequest,
        at: &str,
        track_refs: &[&str],
        types: GroupTypes,
    ) {
        let per_track = release
            .tracks
            .iter()
            .any(|t| t.disc_number.is_some() || t.sequence_number.is_some());
        match &release.resource_group {
            Some(group) => {
                if per_track {
                    self.invalid(
                        &format!("{at}.resource_group"),
                        "is set together with per-track disc_number/sequence_number; use one or the other",
                    );
                }
                if types == GroupTypes::Ern4 && group.resource_group_type.is_some() {
                    self.invalid(
                        &format!("{at}.resource_group.resource_group_type"),
                        "must be unset: the top-level Release/ResourceGroup has no ResourceGroupType",
                    );
                }
                let mut referenced = Vec::new();
                self.group(
                    group,
                    &format!("{at}.resource_group"),
                    track_refs,
                    &mut referenced,
                    true,
                    types,
                );
                for r in track_refs {
                    if !referenced.contains(r) {
                        self.invalid(
                            &format!("{at}.resource_group"),
                            format!("does not reference track resource {:?}", r),
                        );
                    }
                }
            }
            None => {
                let with_disc = release
                    .tracks
                    .iter()
                    .filter(|t| t.disc_number.is_some())
                    .count();
                if with_disc > 0 && with_disc < release.tracks.len() {
                    self.invalid(
                        &format!("{at}.tracks"),
                        "set disc_number on some tracks but not all; set it on every track or none",
                    );
                }
            }
        }
    }

    fn group<'a>(
        &mut self,
        group: &'a ResourceGroupRequest,
        at: &str,
        track_refs: &[&str],
        referenced: &mut Vec<&'a str>,
        top: bool,
        types: GroupTypes,
    ) {
        match types {
            GroupTypes::Ern4 if !top && group.resource_group_type.is_none() => self.missing(
                "ResourceGroup/@ResourceGroupType on nested groups",
                &format!("{at}.resource_group_type"),
            ),
            _ => {}
        }
        for (i, sub) in group.resource_groups.iter().enumerate() {
            self.group(
                sub,
                &format!("{at}.resource_groups[{i}]"),
                track_refs,
                referenced,
                false,
                types,
            );
        }
        for (i, item) in group.content_items.iter().enumerate() {
            let r = item.resource_reference.as_str();
            if !track_refs.contains(&r) {
                self.invalid(
                    &format!("{at}.content_items[{i}].resource_reference"),
                    format!("is {:?}, which is not a track of this release", r),
                );
            }
            referenced.push(r);
        }
    }
}

/// Which `ResourceGroup` dialect is written: ERN 4.x has `@ResourceGroupType` (required on
/// nested groups, `Component` for discs); ERN 3.8.x has no such attribute, so request
/// group types are not written there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GroupTypes {
    Ern4,
    Ern38,
}

// ---------------------------------------------------------------------------
// Value checks (the same patterns in the 3.8.x and 4.x schemas)
// ---------------------------------------------------------------------------

/// `X[\d\-_a-zA-Z]+` (the ERN reference patterns).
pub(super) fn is_reference(value: &str, prefix: char) -> bool {
    let mut chars = value.chars();
    chars.next() == Some(prefix)
        && value.len() > 1
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// `PADPIDA[a-zA-Z0-9]+`
pub(super) fn is_dpid(value: &str) -> bool {
    value
        .strip_prefix("PADPIDA")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_alphanumeric()))
}

/// `[0-9]{4}(-[0-9]{2}){0,1}(-[0-9]{2}){0,1}` (ddex_IsoDate)
pub(super) fn is_iso_date(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    let digits = |s: &str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    !parts.is_empty()
        && parts.len() <= 3
        && digits(parts[0], 4)
        && parts[1..].iter().all(|p| digits(p, 2))
}

/// xs:dateTime: `YYYY-MM-DDThh:mm:ss[.s+][Z|(+|-)hh:mm]`
pub(super) fn is_xs_date_time(value: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(value).is_ok()
        || chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f").is_ok()
}

/// xs:duration: `-?P(nY)?(nM)?(nD)?(T(nH)?(nM)?(n(.n)?S)?)?` with at least one component.
pub(super) fn is_xs_duration(value: &str) -> bool {
    let v = value.strip_prefix('-').unwrap_or(value);
    let Some(rest) = v.strip_prefix('P') else {
        return false;
    };
    let (date, time) = match rest.split_once('T') {
        Some((d, t)) => (d, Some(t)),
        None => (rest, None),
    };
    // Parse "<number><unit>" groups with units in the given order
    fn components(s: &str, units: &[char], allow_fraction_last: bool) -> Option<usize> {
        let mut count = 0;
        let mut rest = s;
        let mut next_unit = 0;
        while !rest.is_empty() {
            let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.'))?;
            let (num, tail) = rest.split_at(end);
            let unit = tail.chars().next()?;
            let pos = units[next_unit..].iter().position(|u| *u == unit)? + next_unit;
            let fraction_ok = allow_fraction_last && pos == units.len() - 1;
            let valid_num = !num.is_empty()
                && (num.bytes().all(|b| b.is_ascii_digit())
                    || (fraction_ok
                        && num.split_once('.').is_some_and(|(a, b)| {
                            !a.is_empty()
                                && !b.is_empty()
                                && a.bytes().all(|x| x.is_ascii_digit())
                                && b.bytes().all(|x| x.is_ascii_digit())
                        })));
            if !valid_num {
                return None;
            }
            next_unit = pos + 1;
            count += 1;
            rest = &tail[unit.len_utf8()..];
        }
        Some(count)
    }
    let Some(date_count) = components(date, &['Y', 'M', 'D'], false) else {
        return false;
    };
    let time_count = match time {
        Some(t) => match components(t, &['H', 'M', 'S'], true) {
            Some(0) | None => return false,
            Some(n) => n,
        },
        None => 0,
    };
    date_count + time_count > 0
}

// ---------------------------------------------------------------------------
// Elements with the same content model in 3.8.x and 4.x
// ---------------------------------------------------------------------------

pub(super) fn text_element(name: &str, text: &str) -> Element {
    Element::new(name).with_text(text)
}

/// An AVS-typed element: value as text, plus `@Namespace` / `@UserDefinedValue`.
pub(super) fn avs_element(name: &str, value: &AvsValueRequest) -> Element {
    let mut element = text_element(name, &value.value);
    if let Some(ns) = &value.namespace {
        element
            .attributes
            .insert("Namespace".to_string(), ns.clone());
    }
    if let Some(udv) = &value.user_defined_value {
        element
            .attributes
            .insert("UserDefinedValue".to_string(), udv.clone());
    }
    element
}

/// The release's identifiers: `release_ids`, else `release_id` as GRid and `upc` as ICPN.
pub(super) fn release_identifiers(release: &ReleaseRequest) -> ReleaseIdRequest {
    release
        .release_ids
        .clone()
        .unwrap_or_else(|| ReleaseIdRequest {
            grid: release.release_id.clone(),
            icpn: release.upc.clone(),
            proprietary_ids: Vec::new(),
        })
}

/// `ReleaseId`: GRid, ICPN, ProprietaryId (the same order in 3.8.x and 4.x).
pub(super) fn release_id(ids: &ReleaseIdRequest) -> Element {
    let mut element = Element::new("ReleaseId");
    if let Some(grid) = &ids.grid {
        element.add_child(text_element("GRid", grid));
    }
    if let Some(icpn) = &ids.icpn {
        element.add_child(text_element("ICPN", icpn));
    }
    for p in &ids.proprietary_ids {
        element.add_child(
            text_element("ProprietaryId", &p.value).with_attr("Namespace", p.namespace.clone()),
        );
    }
    element
}

pub(super) fn genre(genre_text: &str, sub_genre: Option<&String>) -> Element {
    let mut element = Element::new("Genre");
    element.add_child(text_element("GenreText", genre_text));
    if let Some(sub) = sub_genre {
        element.add_child(text_element("SubGenre", sub));
    }
    element
}

fn content_item(sequence_number: Option<u32>, resource_reference: &str) -> Element {
    let mut item = Element::new("ResourceGroupContentItem");
    if let Some(n) = sequence_number {
        item.add_child(text_element("SequenceNumber", &n.to_string()));
    }
    item.add_child(text_element("ReleaseResourceReference", resource_reference));
    item
}

/// `Release/ResourceGroup` (`ReleaseDetailsByTerritory/ResourceGroup` in 3.8.x): the
/// request's tree (without group types in 3.8.x); else one group per disc (when tracks carry `disc_number`; typed
/// `Component` in 4.x); else one flat group of all tracks. `reference` gives a track's
/// resource reference.
pub(super) fn resource_group(
    release: &ReleaseRequest,
    types: GroupTypes,
    reference: impl Fn(&TrackRequest) -> String,
) -> Element {
    fn from_request(group: &ResourceGroupRequest, types: GroupTypes) -> Element {
        let mut element = Element::new("ResourceGroup");
        if let (GroupTypes::Ern4, Some(t)) = (types, &group.resource_group_type) {
            element
                .attributes
                .insert("ResourceGroupType".to_string(), t.clone());
        }
        if let Some(n) = group.sequence_number {
            element.add_child(text_element("SequenceNumber", &n.to_string()));
        }
        for sub in &group.resource_groups {
            element.add_child(from_request(sub, types));
        }
        for item in &group.content_items {
            element.add_child(content_item(item.sequence_number, &item.resource_reference));
        }
        element
    }

    if let Some(group) = &release.resource_group {
        return from_request(group, types);
    }

    let mut top = Element::new("ResourceGroup");
    if release.tracks.iter().any(|t| t.disc_number.is_some()) {
        // Discs in order of first appearance
        let mut discs: Vec<u32> = Vec::new();
        for disc in release.tracks.iter().filter_map(|t| t.disc_number) {
            if !discs.contains(&disc) {
                discs.push(disc);
            }
        }
        for disc in discs {
            let mut group = Element::new("ResourceGroup");
            if types == GroupTypes::Ern4 {
                group
                    .attributes
                    .insert("ResourceGroupType".to_string(), "Component".to_string());
            }
            group.add_child(text_element("SequenceNumber", &disc.to_string()));
            for track in release
                .tracks
                .iter()
                .filter(|t| t.disc_number == Some(disc))
            {
                group.add_child(content_item(track.sequence_number, &reference(track)));
            }
            top.add_child(group);
        }
    } else {
        for track in &release.tracks {
            top.add_child(content_item(track.sequence_number, &reference(track)));
        }
    }
    top
}

/// `DealList`: consecutive deals covering the same releases share one `ReleaseDeal`
/// (`DealReleaseReference`+ then `Deal`+, in both schema families). `deal` writes one
/// version-specific `Deal`.
pub(super) fn deal_list(deals: &[DealRequest], deal: impl Fn(&DealRequest) -> Element) -> Element {
    let mut list = Element::new("DealList");
    let mut i = 0;
    while i < deals.len() {
        let refs = &deals[i].release_references;
        let mut release_deal = Element::new("ReleaseDeal");
        for r in refs {
            release_deal.add_child(text_element("DealReleaseReference", r));
        }
        while i < deals.len() && &deals[i].release_references == refs {
            release_deal.add_child(deal(&deals[i]));
            i += 1;
        }
        list.add_child(release_deal);
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_patterns() {
        assert!(is_reference("A1", 'A'));
        assert!(is_reference("R_rel-1", 'R'));
        assert!(!is_reference("A", 'A'));
        assert!(!is_reference("B1", 'A'));
        assert!(!is_reference("R1:2", 'R'));
    }

    #[test]
    fn dpid_pattern() {
        assert!(is_dpid("PADPIDA2014120301K"));
        assert!(!is_dpid("PADPIDA"));
        assert!(!is_dpid("DPID123"));
    }

    #[test]
    fn iso_dates() {
        for ok in ["2024", "2024-01", "2024-01-15"] {
            assert!(is_iso_date(ok), "{ok}");
        }
        for bad in [
            "",
            "24-01-01",
            "2024-1-1",
            "2024-01-15T00:00:00Z",
            "2024-01-01-01",
        ] {
            assert!(!is_iso_date(bad), "{bad}");
        }
    }

    #[test]
    fn durations() {
        for ok in [
            "PT3M45S",
            "PT0H3M24S",
            "PT1H",
            "P1D",
            "PT30.5S",
            "P1Y2M3DT4H5M6S",
            "-PT1S",
        ] {
            assert!(is_xs_duration(ok), "{ok}");
        }
        for bad in ["", "P", "PT", "3M45S", "PT3S45M", "PT1.5M", "PTxS", "P1DT"] {
            assert!(!is_xs_duration(bad), "{bad}");
        }
    }
}
