//! ERN 4.2 / 4.3 `NewReleaseMessage` generation.
//!
//! Follows the official DDEX schemas (`http://service.ddex.net/xml/ern/42/` and
//! `.../43/release-notification.xsd`): every element is written in XSD sequence order,
//! which is also the canonical order (the canonicalizer keeps 4.x document order).
//! 4.2 and 4.3 share this code; they differ only in `@AvsVersionId` (4.3 only) and in
//! where a resource's `ResourceId` sits (4.2: directly under `SoundRecording`/`Video`;
//! 4.3: under `SoundRecordingEdition`/`VideoEdition`).
//!
//! Nothing is invented: when the schema requires a value the request does not carry, the
//! build fails with one error per missing field, naming the 4.x element and the request
//! field (e.g. `ERN 4.3 requires Release/ReleaseType (releases[0].release_type)`). The only
//! derived value is `DisplayArtistRole` `MainArtist` for the release/track `artist`, which
//! the request API documents as the main artist (overridable with `artist_role`).
//!
//! `PartyList` entries are generated from the release and track artists and labels, in the
//! order they first appear (release artist, release label, then per track: artist and
//! TrackRelease label), with references `P1`, `P2`, ... An artist and a label with the
//! same name are separate parties.

use crate::ast::{Element, AST};
use crate::builder::{
    AvsValueRequest, BuildRequest, DealRequest, ReleaseIdRequest, ReleaseRequest,
    ResourceGroupRequest, ResourceKind, TrackRequest,
};
use crate::error::BuildError;
use indexmap::IndexMap;

/// The ERN 4.x schema version being written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ern4Version {
    V42,
    V43,
}

impl Ern4Version {
    fn label(self) -> &'static str {
        match self {
            Ern4Version::V42 => "4.2",
            Ern4Version::V43 => "4.3",
        }
    }

    fn namespace(self) -> &'static str {
        match self {
            Ern4Version::V42 => "http://ddex.net/xml/ern/42",
            Ern4Version::V43 => "http://ddex.net/xml/ern/43",
        }
    }
}

/// Generate the AST of an ERN 4.x message, or fail listing every schema requirement the
/// request does not meet.
pub(crate) fn generate(request: &BuildRequest, version: Ern4Version) -> Result<AST, BuildError> {
    let mut check = Checker::new(version);
    check.request(request);
    check.finish()?;

    // Validated above: exactly one release.
    let release = &request.releases[0];
    let parties = PartyTable::new(release);

    let mut root = Element::new("NewReleaseMessage");
    root.namespace = Some("ern".to_string());
    if version == Ern4Version::V43 {
        root.attributes.insert(
            "AvsVersionId".to_string(),
            request.avs_version_id.clone().unwrap_or_default(),
        );
    }
    root.attributes.insert(
        "LanguageAndScriptCode".to_string(),
        request.language_and_script_code.clone().unwrap_or_default(),
    );

    root.add_child(message_header(request));
    root.add_child(parties.party_list());
    root.add_child(resource_list(release, &parties, version));
    root.add_child(release_list(release, &parties));
    if !request.deals.is_empty() {
        root.add_child(deal_list(&request.deals));
    }

    let mut namespaces = IndexMap::new();
    namespaces.insert("ern".to_string(), version.namespace().to_string());
    namespaces.insert(
        "xsi".to_string(),
        "http://www.w3.org/2001/XMLSchema-instance".to_string(),
    );
    Ok(AST {
        root,
        namespaces,
        schema_location: None,
    })
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Collects every unmet schema requirement so the caller sees them all at once.
struct Checker {
    version: Ern4Version,
    errors: Vec<String>,
}

impl Checker {
    fn new(version: Ern4Version) -> Self {
        Self {
            version,
            errors: Vec::new(),
        }
    }

    fn finish(self) -> Result<(), BuildError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(BuildError::ValidationFailed {
                errors: self.errors,
            })
        }
    }

    fn missing(&mut self, element: &str, field: &str) {
        self.errors.push(format!(
            "ERN {} requires {} ({})",
            self.version.label(),
            element,
            field
        ));
    }

    fn invalid(&mut self, field: &str, message: impl std::fmt::Display) {
        self.errors.push(format!(
            "ERN {}: {} {}",
            self.version.label(),
            field,
            message
        ));
    }

    fn require_text(&mut self, value: &str, element: &str, field: &str) {
        if value.is_empty() {
            self.missing(element, field);
        }
    }

    fn require_some<T>(&mut self, value: &Option<T>, element: &str, field: &str) {
        if value.is_none() {
            self.missing(element, field);
        }
    }

    fn reference(&mut self, value: &str, prefix: char, field: &str) {
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

    fn date(&mut self, value: &str, field: &str) {
        if !is_iso_date(value) {
            self.invalid(
                field,
                format!("is {:?}; the schema requires an ISO date YYYY[-MM[-DD]]", value),
            );
        }
    }

    fn request(&mut self, request: &BuildRequest) {
        if request.language_and_script_code.is_none() {
            self.missing(
                "NewReleaseMessage/@LanguageAndScriptCode",
                "language_and_script_code",
            );
        }
        if self.version == Ern4Version::V43 && request.avs_version_id.is_none() {
            self.missing("NewReleaseMessage/@AvsVersionId", "avs_version_id");
        }

        let header = &request.header;
        for (party, element, field) in [
            (&header.message_sender, "MessageSender", "message_sender"),
            (&header.message_recipient, "MessageRecipient", "message_recipient"),
        ] {
            match party.party_id.as_deref() {
                None => self.missing(
                    &format!("MessageHeader/{element}/PartyId"),
                    &format!("header.{field}.party_id"),
                ),
                Some(id) if !is_dpid(id) => self.invalid(
                    &format!("header.{field}.party_id"),
                    format!(
                        "is {:?}; the schema requires a DDEX Party ID (DPID) matching PADPIDA[a-zA-Z0-9]+",
                        id
                    ),
                ),
                Some(_) => {}
            }
            if party.party_name.len() > 1 {
                self.invalid(
                    &format!("header.{field}.party_name"),
                    format!(
                        "has {} names; the schema allows one {element}/PartyName",
                        party.party_name.len()
                    ),
                );
            }
        }
        if let Some(created) = &header.message_created_date_time {
            if !is_xs_date_time(created) {
                self.invalid(
                    "header.message_created_date_time",
                    format!("is {:?}; the schema requires an xs:dateTime", created),
                );
            }
        }

        match request.releases.len() {
            0 => self.errors.push(format!(
                "ERN {} requires a Release (releases is empty)",
                self.version.label()
            )),
            1 => self.release(&request.releases[0]),
            n => self.errors.push(format!(
                "ERN {} allows one Release per NewReleaseMessage, the request has {n} \
                 (releases); build one message per release",
                self.version.label()
            )),
        }

        let mut release_refs: Vec<&str> = Vec::new();
        if let Some(release) = request.releases.first() {
            if let Some(r) = release.release_reference.as_deref() {
                release_refs.push(r);
            }
            release_refs.extend(
                release
                    .tracks
                    .iter()
                    .filter_map(|t| t.track_release.as_ref())
                    .map(|tr| tr.release_reference.as_str()),
            );
        }
        for (i, deal) in request.deals.iter().enumerate() {
            self.deal(deal, i, &release_refs);
        }
    }

    fn release(&mut self, release: &ReleaseRequest) {
        let at = "releases[0]";
        match release.release_reference.as_deref() {
            Some(r) => self.reference(r, 'R', &format!("{at}.release_reference")),
            None => self.missing("Release/ReleaseReference", &format!("{at}.release_reference")),
        }
        self.require_some(&release.release_type, "Release/ReleaseType", &format!("{at}.release_type"));
        if let Some(t) = &release.release_type {
            self.avs_value(t, &format!("{at}.release_type"));
        }
        match &release.release_ids {
            Some(ids) if ids.is_empty() => self.missing(
                "an identifier in Release/ReleaseId",
                &format!("{at}.release_ids"),
            ),
            Some(ids) => self.proprietary_ids(ids, &format!("{at}.release_ids")),
            None => {
                if release.release_id.is_empty() && release.upc.is_none() {
                    self.missing(
                        "an identifier in Release/ReleaseId",
                        &format!("{at}.release_id, {at}.upc or {at}.release_ids"),
                    );
                }
            }
        }
        if release.title.iter().all(|t| t.text.is_empty()) {
            self.missing("Release/DisplayTitleText", &format!("{at}.title"));
        }
        self.require_text(&release.artist, "Release/DisplayArtistName", &format!("{at}.artist"));
        if let Some(role) = &release.artist_role {
            self.avs_value(role, &format!("{at}.artist_role"));
        }
        self.require_some(&release.label, "Release/ReleaseLabelReference", &format!("{at}.label"));
        self.require_some(&release.genre, "Release/Genre/GenreText", &format!("{at}.genre"));
        self.require_some(
            &release.parental_warning_type,
            "Release/ParentalWarningType",
            &format!("{at}.parental_warning_type"),
        );
        if let Some(date) = &release.release_date {
            self.date(date, &format!("{at}.release_date"));
        }

        let mut seen: Vec<&str> = Vec::new();
        for (i, track) in release.tracks.iter().enumerate() {
            self.track(track, &format!("{at}.tracks[{i}]"));
            if let Some(r) = track.resource_reference.as_deref() {
                if seen.contains(&r) {
                    self.invalid(
                        &format!("{at}.tracks[{i}].resource_reference"),
                        format!("{:?} is used by more than one track", r),
                    );
                }
                seen.push(r);
            }
        }
        if let Some(refs) = &release.resource_references {
            for r in refs {
                if !seen.contains(&r.as_str()) {
                    self.invalid(
                        &format!("{at}.resource_references"),
                        format!("lists {:?}, which is not a track of this release", r),
                    );
                }
            }
        }
        self.grouping(release, &seen);
    }

    fn grouping(&mut self, release: &ReleaseRequest, track_refs: &[&str]) {
        let at = "releases[0]";
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
                if group.resource_group_type.is_some() {
                    self.invalid(
                        &format!("{at}.resource_group.resource_group_type"),
                        "must be unset: the top-level Release/ResourceGroup has no ResourceGroupType",
                    );
                }
                let mut referenced = Vec::new();
                self.group(group, &format!("{at}.resource_group"), track_refs, &mut referenced, true);
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
                let with_disc = release.tracks.iter().filter(|t| t.disc_number.is_some()).count();
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
    ) {
        if !top && group.resource_group_type.is_none() {
            self.missing("ResourceGroup/@ResourceGroupType on nested groups", &format!("{at}.resource_group_type"));
        }
        for (i, sub) in group.resource_groups.iter().enumerate() {
            self.group(sub, &format!("{at}.resource_groups[{i}]"), track_refs, referenced, false);
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

    fn track(&mut self, track: &TrackRequest, at: &str) {
        let kind = match track.resource_kind {
            ResourceKind::SoundRecording => "SoundRecording",
            ResourceKind::Video => "Video",
        };
        match track.resource_reference.as_deref() {
            Some(r) => self.reference(r, 'A', &format!("{at}.resource_reference")),
            None => self.missing(&format!("{kind}/ResourceReference"), &format!("{at}.resource_reference")),
        }
        self.require_some(&track.resource_type, &format!("{kind}/Type"), &format!("{at}.resource_type"));
        if let Some(t) = &track.resource_type {
            self.avs_value(t, &format!("{at}.resource_type"));
        }
        if track.isrc.is_empty() && track.proprietary_ids.is_empty() {
            self.missing(
                &format!("an identifier in {kind} ResourceId"),
                &format!("{at}.isrc or {at}.proprietary_ids"),
            );
        }
        for (i, p) in track.proprietary_ids.iter().enumerate() {
            if p.namespace.is_empty() {
                self.missing("ProprietaryId/@Namespace", &format!("{at}.proprietary_ids[{i}].namespace"));
            }
        }
        self.require_text(&track.title, &format!("{kind}/DisplayTitleText"), &format!("{at}.title"));
        self.require_text(&track.artist, &format!("{kind}/DisplayArtistName"), &format!("{at}.artist"));
        if let Some(role) = &track.artist_role {
            self.avs_value(role, &format!("{at}.artist_role"));
        }
        if track.duration.is_empty() {
            self.missing(&format!("{kind}/Duration"), &format!("{at}.duration"));
        } else if !is_xs_duration(&track.duration) {
            self.invalid(
                &format!("{at}.duration"),
                format!("is {:?}; the schema requires an xs:duration such as PT3M45S", track.duration),
            );
        }
        self.require_some(
            &track.parental_warning_type,
            &format!("{kind}/ParentalWarningType"),
            &format!("{at}.parental_warning_type"),
        );
        if let Some(tr) = &track.track_release {
            let at = format!("{at}.track_release");
            self.reference(&tr.release_reference, 'R', &format!("{at}.release_reference"));
            if tr.release_ids.is_empty() {
                self.missing("an identifier in TrackRelease/ReleaseId", &format!("{at}.release_ids"));
            }
            self.proprietary_ids(&tr.release_ids, &format!("{at}.release_ids"));
            self.require_some(&tr.label, "TrackRelease/ReleaseLabelReference", &format!("{at}.label"));
            self.require_some(&tr.genre, "TrackRelease/Genre/GenreText", &format!("{at}.genre"));
        }
    }

    fn proprietary_ids(&mut self, ids: &ReleaseIdRequest, at: &str) {
        for (i, p) in ids.proprietary_ids.iter().enumerate() {
            if p.namespace.is_empty() {
                self.missing("ProprietaryId/@Namespace", &format!("{at}.proprietary_ids[{i}].namespace"));
            }
        }
    }

    fn avs_value(&mut self, value: &AvsValueRequest, at: &str) {
        if value.value.is_empty() {
            self.invalid(&format!("{at}.value"), "is empty");
        }
    }

    fn deal(&mut self, deal: &DealRequest, i: usize, release_refs: &[&str]) {
        let at = format!("deals[{i}]");
        if deal.release_references.is_empty() {
            self.missing("ReleaseDeal/DealReleaseReference", &format!("{at}.release_references"));
        }
        for r in &deal.release_references {
            if !release_refs.contains(&r.as_str()) {
                self.invalid(
                    &format!("{at}.release_references"),
                    format!("lists {:?}, which is not a Release or TrackRelease reference in this message", r),
                );
            }
        }
        match &deal.deal_terms.start_date {
            None => self.missing(
                "DealTerms/ValidityPeriod/StartDate",
                &format!("{at}.deal_terms.start_date"),
            ),
            Some(d) => self.date(d, &format!("{at}.deal_terms.start_date")),
        }
        if let Some(d) = &deal.deal_terms.end_date {
            self.date(d, &format!("{at}.deal_terms.end_date"));
        }
    }
}

/// `X[\d\-_a-zA-Z]+` (the 4.x reference patterns).
fn is_reference(value: &str, prefix: char) -> bool {
    let mut chars = value.chars();
    chars.next() == Some(prefix)
        && value.len() > 1
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// `PADPIDA[a-zA-Z0-9]+`
fn is_dpid(value: &str) -> bool {
    value
        .strip_prefix("PADPIDA")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_alphanumeric()))
}

/// `[0-9]{4}(-[0-9]{2}){0,1}(-[0-9]{2}){0,1}` (ddex_IsoDate)
fn is_iso_date(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    let digits = |s: &str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    !parts.is_empty()
        && parts.len() <= 3
        && digits(parts[0], 4)
        && parts[1..].iter().all(|p| digits(p, 2))
}

/// xs:dateTime: `YYYY-MM-DDThh:mm:ss[.s+][Z|(+|-)hh:mm]`
fn is_xs_date_time(value: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(value).is_ok()
        || chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f").is_ok()
}

/// xs:duration: `-?P(nY)?(nM)?(nD)?(T(nH)?(nM)?(n(.n)?S)?)?` with at least one component.
fn is_xs_duration(value: &str) -> bool {
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
// Parties
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum PartyKind {
    Artist,
    Label,
}

/// `PartyList` entries and their references, in order of first appearance.
struct PartyTable {
    parties: Vec<(PartyKind, String)>,
}

impl PartyTable {
    fn new(release: &ReleaseRequest) -> Self {
        let mut table = Self {
            parties: Vec::new(),
        };
        table.add(PartyKind::Artist, &release.artist);
        if let Some(label) = &release.label {
            table.add(PartyKind::Label, label);
        }
        for track in &release.tracks {
            table.add(PartyKind::Artist, &track.artist);
            if let Some(label) = track.track_release.as_ref().and_then(|tr| tr.label.as_ref()) {
                table.add(PartyKind::Label, label);
            }
        }
        table
    }

    fn add(&mut self, kind: PartyKind, name: &str) {
        if !self.parties.iter().any(|(k, n)| *k == kind && n == name) {
            self.parties.push((kind, name.to_string()));
        }
    }

    fn reference(&self, kind: PartyKind, name: &str) -> String {
        let index = self
            .parties
            .iter()
            .position(|(k, n)| *k == kind && n == name)
            .expect("every artist and label was added to the party table");
        format!("P{}", index + 1)
    }

    fn party_list(&self) -> Element {
        let mut list = Element::new("PartyList");
        for (i, (_, name)) in self.parties.iter().enumerate() {
            let mut party = Element::new("Party");
            party.add_child(Element::new("PartyReference").with_text(format!("P{}", i + 1)));
            let mut party_name = Element::new("PartyName");
            party_name.add_child(Element::new("FullName").with_text(name));
            party.add_child(party_name);
            list.add_child(party);
        }
        list
    }
}

// ---------------------------------------------------------------------------
// Elements
// ---------------------------------------------------------------------------

fn text_element(name: &str, text: &str) -> Element {
    Element::new(name).with_text(text)
}

/// An AVS-typed element: value as text, plus `@Namespace` / `@UserDefinedValue`.
fn avs_element(name: &str, value: &AvsValueRequest) -> Element {
    let mut element = text_element(name, &value.value);
    if let Some(ns) = &value.namespace {
        element.attributes.insert("Namespace".to_string(), ns.clone());
    }
    if let Some(udv) = &value.user_defined_value {
        element
            .attributes
            .insert("UserDefinedValue".to_string(), udv.clone());
    }
    element
}

fn message_header(request: &BuildRequest) -> Element {
    let header = &request.header;
    let mut element = Element::new("MessageHeader");
    element.add_child(text_element(
        "MessageId",
        header.message_id.as_deref().unwrap_or_default(),
    ));
    for (name, party) in [
        ("MessageSender", &header.message_sender),
        ("MessageRecipient", &header.message_recipient),
    ] {
        let mut party_element = Element::new(name);
        party_element.add_child(text_element(
            "PartyId",
            party.party_id.as_deref().unwrap_or_default(),
        ));
        if let Some(party_name) = party.party_name.first() {
            let mut name_element = Element::new("PartyName");
            name_element.add_child(text_element("FullName", &party_name.text));
            party_element.add_child(name_element);
        }
        element.add_child(party_element);
    }
    let created = header
        .message_created_date_time
        .clone()
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    element.add_child(text_element("MessageCreatedDateTime", &created));
    if let Some(control) = &header.message_control_type {
        element.add_child(text_element("MessageControlType", control));
    }
    element
}

fn display_artist(parties: &PartyTable, artist: &str, role: Option<&AvsValueRequest>) -> Element {
    let mut element = Element::new("DisplayArtist");
    element.add_child(text_element(
        "ArtistPartyReference",
        &parties.reference(PartyKind::Artist, artist),
    ));
    let main_artist = AvsValueRequest::from("MainArtist");
    element.add_child(avs_element("DisplayArtistRole", role.unwrap_or(&main_artist)));
    element
}

fn resource_list(release: &ReleaseRequest, parties: &PartyTable, version: Ern4Version) -> Element {
    let mut list = Element::new("ResourceList");
    // The schema lists all SoundRecordings before all Videos.
    for kind in [ResourceKind::SoundRecording, ResourceKind::Video] {
        for track in release.tracks.iter().filter(|t| t.resource_kind == kind) {
            list.add_child(resource(track, parties, version));
        }
    }
    list
}

fn resource(track: &TrackRequest, parties: &PartyTable, version: Ern4Version) -> Element {
    let (name, edition) = match track.resource_kind {
        ResourceKind::SoundRecording => ("SoundRecording", "SoundRecordingEdition"),
        ResourceKind::Video => ("Video", "VideoEdition"),
    };
    let mut element = Element::new(name);
    element.add_child(text_element(
        "ResourceReference",
        track.resource_reference.as_deref().unwrap_or_default(),
    ));
    if let Some(t) = &track.resource_type {
        element.add_child(avs_element("Type", t));
    }

    let mut resource_id = Element::new("ResourceId");
    if !track.isrc.is_empty() {
        resource_id.add_child(text_element("ISRC", &track.isrc));
    }
    for p in &track.proprietary_ids {
        resource_id.add_child(
            text_element("ProprietaryId", &p.value).with_attr("Namespace", p.namespace.clone()),
        );
    }
    match version {
        Ern4Version::V42 => element.add_child(resource_id),
        Ern4Version::V43 => {
            let mut edition_element = Element::new(edition);
            edition_element.add_child(resource_id);
            element.add_child(edition_element);
        }
    }

    element.add_child(text_element("DisplayTitleText", &track.title));
    let mut display_title = Element::new("DisplayTitle");
    display_title.add_child(text_element("TitleText", &track.title));
    element.add_child(display_title);
    element.add_child(text_element("DisplayArtistName", &track.artist));
    element.add_child(display_artist(parties, &track.artist, track.artist_role.as_ref()));
    element.add_child(text_element("Duration", &track.duration));
    if let Some(pw) = &track.parental_warning_type {
        element.add_child(text_element("ParentalWarningType", pw));
    }
    element
}

fn release_id(ids: &ReleaseIdRequest) -> Element {
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

fn genre(genre_text: &str, sub_genre: Option<&String>) -> Element {
    let mut element = Element::new("Genre");
    element.add_child(text_element("GenreText", genre_text));
    if let Some(sub) = sub_genre {
        element.add_child(text_element("SubGenre", sub));
    }
    element
}

fn release_list(release: &ReleaseRequest, parties: &PartyTable) -> Element {
    let mut list = Element::new("ReleaseList");
    let mut element = Element::new("Release");
    element.add_child(text_element(
        "ReleaseReference",
        release.release_reference.as_deref().unwrap_or_default(),
    ));
    if let Some(t) = &release.release_type {
        element.add_child(avs_element("ReleaseType", t));
    }
    // Exactly one ReleaseId: release_ids, else release_id as GRid and upc as ICPN.
    let ids = release.release_ids.clone().unwrap_or_else(|| ReleaseIdRequest {
        grid: Some(release.release_id.clone()).filter(|id| !id.is_empty()),
        icpn: release.upc.clone(),
        proprietary_ids: Vec::new(),
    });
    element.add_child(release_id(&ids));

    let titles: Vec<_> = release.title.iter().filter(|t| !t.text.is_empty()).collect();
    for title in &titles {
        let mut e = text_element("DisplayTitleText", &title.text);
        if let Some(lang) = &title.language_code {
            e.attributes
                .insert("LanguageAndScriptCode".to_string(), lang.clone());
        }
        element.add_child(e);
    }
    for title in &titles {
        let mut e = Element::new("DisplayTitle");
        if let Some(lang) = &title.language_code {
            e.attributes
                .insert("LanguageAndScriptCode".to_string(), lang.clone());
        }
        e.add_child(text_element("TitleText", &title.text));
        element.add_child(e);
    }
    element.add_child(text_element("DisplayArtistName", &release.artist));
    element.add_child(display_artist(parties, &release.artist, release.artist_role.as_ref()));
    if let Some(label) = &release.label {
        element.add_child(text_element(
            "ReleaseLabelReference",
            &parties.reference(PartyKind::Label, label),
        ));
    }
    if let Some(g) = &release.genre {
        element.add_child(genre(g, release.sub_genre.as_ref()));
    }
    if let Some(date) = &release.release_date {
        element.add_child(text_element("ReleaseDate", date));
    }
    if let Some(pw) = &release.parental_warning_type {
        element.add_child(text_element("ParentalWarningType", pw));
    }
    element.add_child(resource_group(release));
    list.add_child(element);

    for track in &release.tracks {
        if let Some(tr) = &track.track_release {
            let mut e = Element::new("TrackRelease");
            e.add_child(text_element("ReleaseReference", &tr.release_reference));
            e.add_child(release_id(&tr.release_ids));
            if let Some(title) = &tr.title {
                e.add_child(text_element("DisplayTitleText", title));
                let mut display_title = Element::new("DisplayTitle");
                display_title.add_child(text_element("TitleText", title));
                e.add_child(display_title);
            }
            e.add_child(text_element(
                "ReleaseResourceReference",
                track.resource_reference.as_deref().unwrap_or_default(),
            ));
            if let Some(label) = &tr.label {
                e.add_child(text_element(
                    "ReleaseLabelReference",
                    &parties.reference(PartyKind::Label, label),
                ));
            }
            if let Some(g) = &tr.genre {
                e.add_child(genre(g, tr.sub_genre.as_ref()));
            }
            list.add_child(e);
        }
    }
    list
}

fn content_item(sequence_number: Option<u32>, resource_reference: &str) -> Element {
    let mut item = Element::new("ResourceGroupContentItem");
    if let Some(n) = sequence_number {
        item.add_child(text_element("SequenceNumber", &n.to_string()));
    }
    item.add_child(text_element("ReleaseResourceReference", resource_reference));
    item
}

/// `Release/ResourceGroup`: the request's tree; else one `Component` group per disc
/// (when tracks carry `disc_number`); else one flat group of all tracks.
fn resource_group(release: &ReleaseRequest) -> Element {
    fn from_request(group: &ResourceGroupRequest) -> Element {
        let mut element = Element::new("ResourceGroup");
        if let Some(t) = &group.resource_group_type {
            element
                .attributes
                .insert("ResourceGroupType".to_string(), t.clone());
        }
        if let Some(n) = group.sequence_number {
            element.add_child(text_element("SequenceNumber", &n.to_string()));
        }
        for sub in &group.resource_groups {
            element.add_child(from_request(sub));
        }
        for item in &group.content_items {
            element.add_child(content_item(item.sequence_number, &item.resource_reference));
        }
        element
    }

    if let Some(group) = &release.resource_group {
        return from_request(group);
    }

    let reference = |t: &TrackRequest| t.resource_reference.clone().unwrap_or_default();
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
            group
                .attributes
                .insert("ResourceGroupType".to_string(), "Component".to_string());
            group.add_child(text_element("SequenceNumber", &disc.to_string()));
            for track in release.tracks.iter().filter(|t| t.disc_number == Some(disc)) {
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

/// `DealList`: consecutive deals covering the same releases share one `ReleaseDeal`.
fn deal_list(deals: &[DealRequest]) -> Element {
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

fn deal(deal: &DealRequest) -> Element {
    let terms = &deal.deal_terms;
    let mut element = Element::new("Deal");
    if let Some(r) = &deal.deal_reference {
        element.add_child(text_element("DealReference", r));
    }
    let mut terms_element = Element::new("DealTerms");
    for t in &terms.territory_code {
        terms_element.add_child(text_element("TerritoryCode", t));
    }
    let mut validity = Element::new("ValidityPeriod");
    if let Some(start) = &terms.start_date {
        validity.add_child(text_element("StartDate", start));
    }
    if let Some(end) = &terms.end_date {
        validity.add_child(text_element("EndDate", end));
    }
    terms_element.add_child(validity);
    if !terms.commercial_model_type.is_empty() {
        terms_element.add_child(text_element("CommercialModelType", &terms.commercial_model_type));
    }
    for u in &terms.use_types {
        terms_element.add_child(text_element("UseType", u));
    }
    element.add_child(terms_element);
    element
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
        for bad in ["", "24-01-01", "2024-1-1", "2024-01-15T00:00:00Z", "2024-01-01-01"] {
            assert!(!is_iso_date(bad), "{bad}");
        }
    }

    #[test]
    fn durations() {
        for ok in ["PT3M45S", "PT0H3M24S", "PT1H", "P1D", "PT30.5S", "P1Y2M3DT4H5M6S", "-PT1S"] {
            assert!(is_xs_duration(ok), "{ok}");
        }
        for bad in ["", "P", "PT", "3M45S", "PT3S45M", "PT1.5M", "PTxS", "P1DT"] {
            assert!(!is_xs_duration(bad), "{bad}");
        }
    }
}
