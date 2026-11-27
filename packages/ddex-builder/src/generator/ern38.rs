//! ERN 3.8.2 / 3.8.1 `NewReleaseMessage` generation.
//!
//! Follows the official DDEX 3.8.2 schema (`http://service.ddex.net/xml/ern/382/
//! release-notification.xsd`, fetched by `scripts/fetch_ddex_schemas.sh`): every element is
//! written in XSD sequence order, which is also the canonical order (the canonicalizer keeps
//! document order). 3.8.1 is written by the same code with the `ern/381` namespace; the
//! 3.8.1 content models of every element written here are the same as 3.8.2's (checked
//! against an unofficial copy of the 3.8.1 schema; no official copy is available).
//!
//! Where the 3.8.x structure differs from 4.x:
//! - Resource identifiers are in `SoundRecordingId`; titles are `ReferenceTitle`.
//! - Display artist, label, titles, genre, parental warning, grouping and release date sit
//!   in `SoundRecordingDetailsByTerritory` / `ReleaseDetailsByTerritory`. Those require a
//!   `TerritoryCode`; the request's values are not territorial (as in 4.x, where they apply
//!   everywhere), so they are written for `Worldwide`.
//! - Artists are written by name (`PartyName/FullName`, `ArtistRole`); there is no
//!   `PartyList`.
//! - A 4.x `TrackRelease` (`tracks[].track_release`) is a `Release` with `ReleaseType`
//!   `TrackRelease`.
//! - `ResourceGroup` has no `@ResourceGroupType`: a request's group types (which 4.x
//!   requires on nested groups) are not written; the group tree, sequence numbers and
//!   content items are.
//! - `DealTerms` need `Usage/UseType` and `TerritoryCode`. `DealTerms` is optional: a deal
//!   with no terms set is written as its `DealReference` alone.
//!
//! Nothing is invented: when the schema requires a value the request does not carry, the
//! build fails with one error per missing field, naming the 3.8.x element and the request
//! field (e.g. `ERN 3.8.2 requires SoundRecording/Duration (releases[0].tracks[0].duration)`).
//! Derived values: the `Worldwide` territory above, `ArtistRole` `MainArtist` for the
//! release/track `artist` (as in 4.x; overridable with `artist_role`), `TitleType`
//! `DisplayTitle` for the release titles (4.x writes them as `DisplayTitle`),
//! `ReleaseType` `TrackRelease` for track releases, and a track release's `ReferenceTitle`:
//! its `title`, else its track's title (the schema requires one; 4.x does not). References that the request leaves
//! unset are derived as before (`A{track_id}`, `R{release_id}`) when the ID strategy has
//! not filled them in.

use super::shared::{
    avs_element, genre, release_id, release_identifiers, resource_group, text_element, GroupTypes,
    Problems,
};
use crate::ast::{Element, AST};
use crate::builder::{
    AvsValueRequest, BuildRequest, DealRequest, PartyRequest, ReleaseRequest, ResourceKind,
    TrackRequest,
};
use crate::error::BuildError;
use indexmap::IndexMap;

/// The territory the (non-territorial) request details are written for.
const WORLDWIDE: &str = "Worldwide";

/// The ERN 3.8.x schema version being written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ern38Version {
    V381,
    V382,
}

impl Ern38Version {
    fn label(self) -> &'static str {
        match self {
            Ern38Version::V381 => "3.8.1",
            Ern38Version::V382 => "3.8.2",
        }
    }

    fn namespace(self) -> &'static str {
        match self {
            Ern38Version::V381 => "http://ddex.net/xml/ern/381",
            Ern38Version::V382 => "http://ddex.net/xml/ern/382",
        }
    }

    fn schema_version_id(self) -> &'static str {
        match self {
            Ern38Version::V381 => "ern/381",
            Ern38Version::V382 => "ern/382",
        }
    }
}

/// A track's `ResourceReference`: the request's, else `A{track_id}`.
fn resource_reference(track: &TrackRequest) -> String {
    track
        .resource_reference
        .clone()
        .unwrap_or_else(|| format!("A{}", track.track_id))
}

/// A release's `ReleaseReference`: the request's, else `R{release_id}`.
fn release_reference(release: &ReleaseRequest) -> String {
    release
        .release_reference
        .clone()
        .unwrap_or_else(|| format!("R{}", release.release_id.as_deref().unwrap_or_default()))
}

/// The release's `ReferenceTitle`: its first title.
fn reference_title(release: &ReleaseRequest) -> Option<&crate::builder::LocalizedStringRequest> {
    release.title.first()
}

/// Generate the AST of an ERN 3.8.x message, or fail listing every schema requirement the
/// request does not meet.
pub(crate) fn generate(request: &BuildRequest, version: Ern38Version) -> Result<AST, BuildError> {
    let mut check = Checker::new(version);
    check.request(request);
    check.finish()?;

    let mut root = Element::new("NewReleaseMessage");
    root.namespace = Some("ern".to_string());
    root.attributes.insert(
        "MessageSchemaVersionId".to_string(),
        version.schema_version_id().to_string(),
    );
    if let Some(lang) = &request.language_and_script_code {
        root.attributes
            .insert("LanguageAndScriptCode".to_string(), lang.clone());
    }

    root.add_child(message_header(request));
    root.add_child(resource_list(&request.releases));
    root.add_child(release_list(&request.releases));
    if !request.deals.is_empty() {
        root.add_child(super::shared::deal_list(&request.deals, deal));
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

/// The fields of a sound recording written once per `ResourceReference`; tracks sharing a
/// reference (the same recording on several releases) must agree on them.
fn recording_key(track: &TrackRequest) -> String {
    format!(
        "{:?}",
        (
            &track.isrc,
            &track.title,
            &track.duration,
            &track.artist,
            &track.resource_type,
            &track.parental_warning_type,
            &track.artist_role,
            &track.proprietary_ids,
        )
    )
}

/// The ERN 3.8.x requirements; [`Problems`] collects the unmet ones.
struct Checker {
    p: Problems,
}

impl Checker {
    fn new(version: Ern38Version) -> Self {
        Self {
            p: Problems::new(version.label()),
        }
    }

    fn finish(self) -> Result<(), BuildError> {
        self.p.finish()
    }

    fn request(&mut self, request: &BuildRequest) {
        let header = &request.header;
        if header.message_id.as_deref().unwrap_or_default().is_empty() {
            self.p
                .missing("MessageHeader/MessageId", "header.message_id");
        }
        for (party, element, field) in [
            (&header.message_sender, "MessageSender", "message_sender"),
            (
                &header.message_recipient,
                "MessageRecipient",
                "message_recipient",
            ),
        ] {
            self.party(party, element, field);
        }
        if let Some(created) = &header.message_created_date_time {
            self.p
                .date_time(created, "header.message_created_date_time");
        }

        if request.releases.is_empty() {
            let label = self.p.label();
            self.p.push(format!(
                "ERN {label} requires a Release (releases is empty)"
            ));
        }

        // Release references (main releases and track releases) and recordings by reference
        let mut release_refs: Vec<(String, String)> = Vec::new();
        let mut recordings: Vec<(String, String, String)> = Vec::new();
        for (r, release) in request.releases.iter().enumerate() {
            let at = format!("releases[{r}]");
            self.release(release, &at);
            release_refs.push((
                release_reference(release),
                format!("{at}.release_reference"),
            ));
            for (t, track) in release.tracks.iter().enumerate() {
                let track_at = format!("{at}.tracks[{t}]");
                if let Some(tr) = &track.track_release {
                    release_refs.push((
                        tr.release_reference.clone(),
                        format!("{track_at}.track_release.release_reference"),
                    ));
                }
                let reference = resource_reference(track);
                let key = recording_key(track);
                match recordings.iter().find(|(r, _, _)| *r == reference) {
                    Some((_, other_key, other_at)) if *other_key != key => self.p.invalid(
                        &format!("{track_at}.resource_reference"),
                        format!(
                            "is {:?}, also used by {other_at} with different recording details; \
                             a ResourceReference identifies one SoundRecording",
                            reference
                        ),
                    ),
                    Some(_) => {}
                    None => recordings.push((reference, key, track_at)),
                }
            }
        }
        for (i, (reference, at)) in release_refs.iter().enumerate() {
            if let Some((_, first)) = release_refs[..i].iter().find(|(r, _)| r == reference) {
                self.p.invalid(
                    at,
                    format!(
                        "is {:?}, already used by {first}; release references must be unique",
                        reference
                    ),
                );
            }
        }

        let refs: Vec<&str> = release_refs.iter().map(|(r, _)| r.as_str()).collect();
        for (i, deal) in request.deals.iter().enumerate() {
            self.deal(deal, i, &refs);
        }
    }

    fn party(&mut self, party: &PartyRequest, element: &str, field: &str) {
        if party.party_id.as_deref().unwrap_or_default().is_empty() {
            self.p.missing(
                &format!("MessageHeader/{element}/PartyId"),
                &format!("header.{field}.party_id"),
            );
        }
        if party.party_name.len() > 1 {
            self.p.invalid(
                &format!("header.{field}.party_name"),
                format!(
                    "has {} names; the schema allows one {element}/PartyName",
                    party.party_name.len()
                ),
            );
        }
    }

    fn release(&mut self, release: &ReleaseRequest, at: &str) {
        self.p.reference(
            &release_reference(release),
            'R',
            &format!("{at}.release_reference"),
        );
        self.p.release_identifier(release, at);
        if reference_title(release).is_none() {
            self.p
                .missing("Release/ReferenceTitle", &format!("{at}.title"));
        }
        self.p.title_texts(&release.title, at);
        self.p
            .optional_text(&release.artist, &format!("{at}.artist"));
        if let Some(t) = &release.release_type {
            self.p.avs_value(t, &format!("{at}.release_type"));
        }
        if let Some(role) = &release.artist_role {
            self.p.avs_value(role, &format!("{at}.artist_role"));
        }
        if let Some(date) = &release.release_date {
            self.p.date(date, &format!("{at}.release_date"));
        }

        let mut seen: Vec<String> = Vec::new();
        for (i, track) in release.tracks.iter().enumerate() {
            let track_at = format!("{at}.tracks[{i}]");
            self.track(track, &track_at);
            let r = resource_reference(track);
            if seen.contains(&r) {
                self.p.invalid(
                    &format!("{track_at}.resource_reference"),
                    format!("{:?} is used by more than one track", r),
                );
            }
            seen.push(r);
        }
        match &release.resource_references {
            Some(refs) => {
                if refs.is_empty() {
                    self.p.missing(
                        "a ReleaseResourceReference in Release/ReleaseResourceReferenceList",
                        &format!("{at}.resource_references"),
                    );
                }
                for r in refs {
                    if !seen.contains(r) {
                        self.p.invalid(
                            &format!("{at}.resource_references"),
                            format!("lists {:?}, which is not a track of this release", r),
                        );
                    }
                }
            }
            None => {
                if release.tracks.is_empty() {
                    self.p.missing(
                        "a ReleaseResourceReference in Release/ReleaseResourceReferenceList",
                        &format!("{at}.tracks"),
                    );
                }
            }
        }
        let seen: Vec<&str> = seen.iter().map(String::as_str).collect();
        self.p.grouping(release, at, &seen, GroupTypes::Ern38);
    }

    fn track(&mut self, track: &TrackRequest, at: &str) {
        if track.resource_kind == ResourceKind::Video {
            let label = self.p.label();
            self.p.invalid(
                &format!("{at}.resource_kind"),
                format!("is Video; the builder writes Video resources for ERN 4.2/4.3 only, not {label}"),
            );
            return;
        }
        self.p.reference(
            &resource_reference(track),
            'A',
            &format!("{at}.resource_reference"),
        );
        if let Some(t) = &track.resource_type {
            self.p.avs_value(t, &format!("{at}.resource_type"));
        }
        self.p.optional_text(&track.isrc, &format!("{at}.isrc"));
        self.p.optional_text(&track.artist, &format!("{at}.artist"));
        if track.isrc.is_none() && track.proprietary_ids.is_empty() {
            self.p.missing(
                "an identifier in SoundRecording/SoundRecordingId",
                &format!("{at}.isrc or {at}.proprietary_ids"),
            );
        }
        for (i, p) in track.proprietary_ids.iter().enumerate() {
            if p.namespace.is_empty() {
                self.p.missing(
                    "ProprietaryId/@Namespace",
                    &format!("{at}.proprietary_ids[{i}].namespace"),
                );
            }
        }
        self.p.require_text(
            &track.title,
            "SoundRecording/ReferenceTitle",
            &format!("{at}.title"),
        );
        self.p.duration(
            &track.duration,
            "SoundRecording/Duration",
            &format!("{at}.duration"),
        );
        if let Some(role) = &track.artist_role {
            self.p.avs_value(role, &format!("{at}.artist_role"));
        }
        if let Some(tr) = &track.track_release {
            let at = format!("{at}.track_release");
            self.p.reference(
                &tr.release_reference,
                'R',
                &format!("{at}.release_reference"),
            );
            if tr.release_ids.is_empty() {
                self.p.missing(
                    "an identifier in Release/ReleaseId (track release)",
                    &format!("{at}.release_ids"),
                );
            }
            self.p
                .proprietary_ids(&tr.release_ids, &format!("{at}.release_ids"));
        }
    }

    fn deal(&mut self, deal: &DealRequest, i: usize, release_refs: &[&str]) {
        let at = format!("deals[{i}]");
        if deal.release_references.is_empty() {
            self.p.missing(
                "ReleaseDeal/DealReleaseReference",
                &format!("{at}.release_references"),
            );
        }
        for r in &deal.release_references {
            if !release_refs.contains(&r.as_str()) {
                self.p.invalid(
                    &format!("{at}.release_references"),
                    format!(
                        "lists {:?}, which is not a release reference in this message",
                        r
                    ),
                );
            }
        }
        let terms = &deal.deal_terms;
        if terms.is_empty() {
            // A Deal without DealTerms (e.g. a reference to a deal agreed elsewhere) is
            // schema-valid, but must say something: its DealReference.
            if deal
                .deal_reference
                .as_deref()
                .unwrap_or_default()
                .is_empty()
            {
                self.p.missing(
                    "Deal/DealReference when a deal has no DealTerms",
                    &format!("{at}.deal_reference or {at}.deal_terms"),
                );
            }
            return;
        }
        let at = format!("{at}.deal_terms");
        if terms.use_types.is_empty() {
            self.p
                .missing("DealTerms/Usage/UseType", &format!("{at}.use_types"));
        }
        if terms.use_types.iter().any(String::is_empty) {
            self.p
                .invalid(&format!("{at}.use_types"), "contains an empty value");
        }
        if terms.territory_code.is_empty() {
            self.p
                .missing("DealTerms/TerritoryCode", &format!("{at}.territory_code"));
        }
        if terms.territory_code.iter().any(String::is_empty) {
            self.p
                .invalid(&format!("{at}.territory_code"), "contains an empty value");
        }
        if terms.start_date.is_none() && terms.end_date.is_none() {
            self.p.missing(
                "a StartDate or EndDate in DealTerms/ValidityPeriod",
                &format!("{at}.start_date or {at}.end_date"),
            );
        }
        if let Some(d) = &terms.start_date {
            self.p.date(d, &format!("{at}.start_date"));
        }
        if let Some(d) = &terms.end_date {
            self.p.date(d, &format!("{at}.end_date"));
        }
        self.p.optional_text(
            &terms.commercial_model_type,
            &format!("{at}.commercial_model_type"),
        );
    }
}

// ---------------------------------------------------------------------------
// Elements
// ---------------------------------------------------------------------------

fn messaging_party(name: &str, party: &PartyRequest) -> Element {
    let mut element = Element::new(name);
    element.add_child(text_element(
        "PartyId",
        party.party_id.as_deref().unwrap_or_default(),
    ));
    if let Some(party_name) = party.party_name.first() {
        let mut name_element = Element::new("PartyName");
        if let Some(lang) = &party_name.language_code {
            name_element
                .attributes
                .insert("LanguageAndScriptCode".to_string(), lang.clone());
        }
        name_element.add_child(text_element("FullName", &party_name.text));
        element.add_child(name_element);
    }
    element
}

#[allow(
    clippy::disallowed_methods,
    reason = "known bug D8: removed in 0.6.0 determinism PR"
)]
fn message_header(request: &BuildRequest) -> Element {
    let header = &request.header;
    let mut element = Element::new("MessageHeader");
    element.add_child(text_element(
        "MessageId",
        header.message_id.as_deref().unwrap_or_default(),
    ));
    element.add_child(messaging_party("MessageSender", &header.message_sender));
    element.add_child(messaging_party(
        "MessageRecipient",
        &header.message_recipient,
    ));
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

/// `DisplayArtist` (3.8.x `Artist`: by name, with a role).
fn display_artist(artist: &str, role: Option<&AvsValueRequest>) -> Element {
    let mut element = Element::new("DisplayArtist");
    let mut party_name = Element::new("PartyName");
    party_name.add_child(text_element("FullName", artist));
    element.add_child(party_name);
    let main_artist = AvsValueRequest::from("MainArtist");
    element.add_child(avs_element("ArtistRole", role.unwrap_or(&main_artist)));
    element
}

/// `SoundRecording`s of all releases, each `ResourceReference` once, in input order.
fn resource_list(releases: &[ReleaseRequest]) -> Element {
    let mut list = Element::new("ResourceList");
    let mut written: Vec<String> = Vec::new();
    for track in releases.iter().flat_map(|r| &r.tracks) {
        let reference = resource_reference(track);
        if !written.contains(&reference) {
            list.add_child(sound_recording(track, &reference));
            written.push(reference);
        }
    }
    list
}

fn sound_recording(track: &TrackRequest, reference: &str) -> Element {
    let mut element = Element::new("SoundRecording");
    if let Some(t) = &track.resource_type {
        element.add_child(avs_element("SoundRecordingType", t));
    }
    let mut id = Element::new("SoundRecordingId");
    if let Some(isrc) = &track.isrc {
        id.add_child(text_element("ISRC", isrc));
    }
    for p in &track.proprietary_ids {
        id.add_child(
            text_element("ProprietaryId", &p.value).with_attr("Namespace", p.namespace.clone()),
        );
    }
    element.add_child(id);
    element.add_child(text_element("ResourceReference", reference));
    let mut title = Element::new("ReferenceTitle");
    title.add_child(text_element("TitleText", &track.title));
    element.add_child(title);
    element.add_child(text_element("Duration", &track.duration));

    let mut details = Element::new("SoundRecordingDetailsByTerritory");
    details.add_child(text_element("TerritoryCode", WORLDWIDE));
    if let Some(artist) = &track.artist {
        details.add_child(display_artist(artist, track.artist_role.as_ref()));
        details.add_child(text_element("DisplayArtistName", artist));
    }
    if let Some(pw) = &track.parental_warning_type {
        details.add_child(text_element("ParentalWarningType", pw));
    }
    element.add_child(details);
    element
}

fn reference_title_element(text: &str, language: Option<&String>) -> Element {
    let mut title = Element::new("ReferenceTitle");
    let mut title_text = text_element("TitleText", text);
    if let Some(lang) = language {
        title_text
            .attributes
            .insert("LanguageAndScriptCode".to_string(), lang.clone());
    }
    title.add_child(title_text);
    title
}

fn release_resource_reference_list<'a>(refs: impl IntoIterator<Item = &'a str>) -> Element {
    let mut list = Element::new("ReleaseResourceReferenceList");
    for r in refs {
        list.add_child(text_element("ReleaseResourceReference", r));
    }
    list
}

/// Each release, followed by its tracks' track releases.
fn release_list(releases: &[ReleaseRequest]) -> Element {
    let mut list = Element::new("ReleaseList");
    for release in releases {
        list.add_child(main_release(release));
        for track in &release.tracks {
            if let Some(tr) = &track.track_release {
                list.add_child(track_release(track, tr));
            }
        }
    }
    list
}

fn main_release(release: &ReleaseRequest) -> Element {
    let mut element = Element::new("Release");
    element.add_child(release_id(&release_identifiers(release)));
    element.add_child(text_element(
        "ReleaseReference",
        &release_reference(release),
    ));
    if let Some(title) = reference_title(release) {
        element.add_child(reference_title_element(
            &title.text,
            title.language_code.as_ref(),
        ));
    }
    let track_refs: Vec<String> = release.tracks.iter().map(resource_reference).collect();
    element.add_child(match &release.resource_references {
        Some(refs) => release_resource_reference_list(refs.iter().map(String::as_str)),
        None => release_resource_reference_list(track_refs.iter().map(String::as_str)),
    });
    if let Some(t) = &release.release_type {
        element.add_child(avs_element("ReleaseType", t));
    }

    let mut details = Element::new("ReleaseDetailsByTerritory");
    details.add_child(text_element("TerritoryCode", WORLDWIDE));
    if let Some(artist) = &release.artist {
        details.add_child(text_element("DisplayArtistName", artist));
    }
    if let Some(label) = &release.label {
        details.add_child(text_element("LabelName", label));
    }
    for title in &release.title {
        let mut e = Element::new("Title");
        if let Some(lang) = &title.language_code {
            e.attributes
                .insert("LanguageAndScriptCode".to_string(), lang.clone());
        }
        e.attributes
            .insert("TitleType".to_string(), "DisplayTitle".to_string());
        e.add_child(text_element("TitleText", &title.text));
        details.add_child(e);
    }
    if let Some(artist) = &release.artist {
        details.add_child(display_artist(artist, release.artist_role.as_ref()));
    }
    if let Some(pw) = &release.parental_warning_type {
        details.add_child(text_element("ParentalWarningType", pw));
    }
    if !release.tracks.is_empty() {
        details.add_child(resource_group(
            release,
            GroupTypes::Ern38,
            resource_reference,
        ));
    }
    if let Some(g) = &release.genre {
        details.add_child(genre(g, release.sub_genre.as_ref()));
    }
    if let Some(date) = &release.release_date {
        details.add_child(text_element("ReleaseDate", date));
    }
    element.add_child(details);
    element
}

fn track_release(track: &TrackRequest, tr: &crate::builder::TrackReleaseRequest) -> Element {
    let mut element = Element::new("Release");
    element.add_child(release_id(&tr.release_ids));
    element.add_child(text_element("ReleaseReference", &tr.release_reference));
    let title = tr
        .title
        .as_deref()
        .filter(|t| !t.is_empty())
        .unwrap_or(&track.title);
    element.add_child(reference_title_element(title, None));
    let reference = resource_reference(track);
    element.add_child(release_resource_reference_list([reference.as_str()]));
    element.add_child(text_element("ReleaseType", "TrackRelease"));
    let mut details = Element::new("ReleaseDetailsByTerritory");
    details.add_child(text_element("TerritoryCode", WORLDWIDE));
    if let Some(label) = &tr.label {
        details.add_child(text_element("LabelName", label));
    }
    if let Some(g) = &tr.genre {
        details.add_child(genre(g, tr.sub_genre.as_ref()));
    }
    element.add_child(details);
    element
}

fn deal(deal: &DealRequest) -> Element {
    let terms = &deal.deal_terms;
    let mut element = Element::new("Deal");
    if let Some(r) = &deal.deal_reference {
        element.add_child(text_element("DealReference", r));
    }
    if terms.is_empty() {
        return element;
    }
    let mut terms_element = Element::new("DealTerms");
    if let Some(model) = &terms.commercial_model_type {
        terms_element.add_child(text_element("CommercialModelType", model));
    }
    let mut usage = Element::new("Usage");
    for u in &terms.use_types {
        usage.add_child(text_element("UseType", u));
    }
    terms_element.add_child(usage);
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
    element.add_child(terms_element);
    element
}
