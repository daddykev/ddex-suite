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

use super::shared::{
    avs_element, genre, is_dpid, release_id, release_identifiers, resource_group, text_element,
    GroupTypes, Problems,
};
use crate::ast::{Element, AST};
use crate::builder::{
    AvsValueRequest, BuildRequest, DealRequest, ReleaseRequest, ResourceKind, TrackRequest,
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

/// The ERN 4.x requirements; [`Problems`] collects the unmet ones.
struct Checker {
    version: Ern4Version,
    p: Problems,
}

impl Checker {
    fn new(version: Ern4Version) -> Self {
        Self {
            version,
            p: Problems::new(version.label()),
        }
    }

    fn finish(self) -> Result<(), BuildError> {
        self.p.finish()
    }

    fn request(&mut self, request: &BuildRequest) {
        if request.language_and_script_code.is_none() {
            self.p.missing(
                "NewReleaseMessage/@LanguageAndScriptCode",
                "language_and_script_code",
            );
        }
        if self.version == Ern4Version::V43 && request.avs_version_id.is_none() {
            self.p
                .missing("NewReleaseMessage/@AvsVersionId", "avs_version_id");
        }

        let header = &request.header;
        for (party, element, field) in [
            (&header.message_sender, "MessageSender", "message_sender"),
            (
                &header.message_recipient,
                "MessageRecipient",
                "message_recipient",
            ),
        ] {
            match party.party_id.as_deref() {
                None => self.p.missing(
                    &format!("MessageHeader/{element}/PartyId"),
                    &format!("header.{field}.party_id"),
                ),
                Some(id) if !is_dpid(id) => self.p.invalid(
                    &format!("header.{field}.party_id"),
                    format!(
                        "is {:?}; the schema requires a DDEX Party ID (DPID) matching PADPIDA[a-zA-Z0-9]+",
                        id
                    ),
                ),
                Some(_) => {}
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
        self.p.message_header(header);

        match request.releases.len() {
            0 => self.p.push(format!(
                "ERN {} requires a Release (releases is empty)",
                self.version.label()
            )),
            1 => self.release(&request.releases[0]),
            n => self.p.push(format!(
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
            Some(r) => self.p.reference(r, 'R', &format!("{at}.release_reference")),
            None => self.p.missing(
                "Release/ReleaseReference",
                &format!("{at}.release_reference"),
            ),
        }
        self.p.require_some(
            &release.release_type,
            "Release/ReleaseType",
            &format!("{at}.release_type"),
        );
        if let Some(t) = &release.release_type {
            self.p.avs_value(t, &format!("{at}.release_type"));
        }
        self.p.release_identifier(release, at);
        if release.title.is_empty() {
            self.p
                .missing("Release/DisplayTitleText", &format!("{at}.title"));
        }
        self.p.title_texts(&release.title, at);
        self.p.require_some_text(
            &release.artist,
            "Release/DisplayArtistName",
            &format!("{at}.artist"),
        );
        if let Some(role) = &release.artist_role {
            self.p.avs_value(role, &format!("{at}.artist_role"));
        }
        self.p.require_some(
            &release.label,
            "Release/ReleaseLabelReference",
            &format!("{at}.label"),
        );
        self.p.require_some(
            &release.genre,
            "Release/Genre/GenreText",
            &format!("{at}.genre"),
        );
        self.p.require_some(
            &release.parental_warning_type,
            "Release/ParentalWarningType",
            &format!("{at}.parental_warning_type"),
        );
        if let Some(date) = &release.release_date {
            self.p.date(date, &format!("{at}.release_date"));
        }

        let mut seen: Vec<&str> = Vec::new();
        for (i, track) in release.tracks.iter().enumerate() {
            self.track(track, &format!("{at}.tracks[{i}]"));
            if let Some(r) = track.resource_reference.as_deref() {
                if seen.contains(&r) {
                    self.p.invalid(
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
                    self.p.invalid(
                        &format!("{at}.resource_references"),
                        format!("lists {:?}, which is not a track of this release", r),
                    );
                }
            }
        }
        self.p.grouping(release, at, &seen, GroupTypes::Ern4);
    }

    fn track(&mut self, track: &TrackRequest, at: &str) {
        let kind = match track.resource_kind {
            ResourceKind::SoundRecording => "SoundRecording",
            ResourceKind::Video => "Video",
        };
        match track.resource_reference.as_deref() {
            Some(r) => self
                .p
                .reference(r, 'A', &format!("{at}.resource_reference")),
            None => self.p.missing(
                &format!("{kind}/ResourceReference"),
                &format!("{at}.resource_reference"),
            ),
        }
        self.p.require_some(
            &track.resource_type,
            &format!("{kind}/Type"),
            &format!("{at}.resource_type"),
        );
        if let Some(t) = &track.resource_type {
            self.p.avs_value(t, &format!("{at}.resource_type"));
        }
        self.p.optional_text(&track.isrc, &format!("{at}.isrc"));
        if track.isrc.is_none() && track.proprietary_ids.is_empty() {
            self.p.missing(
                &format!("an identifier in {kind} ResourceId"),
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
            &format!("{kind}/DisplayTitleText"),
            &format!("{at}.title"),
        );
        self.p.require_some_text(
            &track.artist,
            &format!("{kind}/DisplayArtistName"),
            &format!("{at}.artist"),
        );
        if let Some(role) = &track.artist_role {
            self.p.avs_value(role, &format!("{at}.artist_role"));
        }
        self.p.duration(
            &track.duration,
            &format!("{kind}/Duration"),
            &format!("{at}.duration"),
        );
        self.p.require_some(
            &track.parental_warning_type,
            &format!("{kind}/ParentalWarningType"),
            &format!("{at}.parental_warning_type"),
        );
        if let Some(tr) = &track.track_release {
            let at = format!("{at}.track_release");
            self.p.reference(
                &tr.release_reference,
                'R',
                &format!("{at}.release_reference"),
            );
            if tr.release_ids.is_empty() {
                self.p.missing(
                    "an identifier in TrackRelease/ReleaseId",
                    &format!("{at}.release_ids"),
                );
            }
            self.p
                .proprietary_ids(&tr.release_ids, &format!("{at}.release_ids"));
            self.p.require_some(
                &tr.label,
                "TrackRelease/ReleaseLabelReference",
                &format!("{at}.label"),
            );
            self.p.require_some(
                &tr.genre,
                "TrackRelease/Genre/GenreText",
                &format!("{at}.genre"),
            );
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
                    format!("lists {:?}, which is not a Release or TrackRelease reference in this message", r),
                );
            }
        }
        match &deal.deal_terms.start_date {
            None => self.p.missing(
                "DealTerms/ValidityPeriod/StartDate",
                &format!("{at}.deal_terms.start_date"),
            ),
            Some(d) => self.p.date(d, &format!("{at}.deal_terms.start_date")),
        }
        if let Some(d) = &deal.deal_terms.end_date {
            self.p.date(d, &format!("{at}.deal_terms.end_date"));
        }
        self.p.optional_text(
            &deal.deal_terms.commercial_model_type,
            &format!("{at}.deal_terms.commercial_model_type"),
        );
    }
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
        table.add(
            PartyKind::Artist,
            release.artist.as_deref().unwrap_or_default(),
        );
        if let Some(label) = &release.label {
            table.add(PartyKind::Label, label);
        }
        for track in &release.tracks {
            table.add(
                PartyKind::Artist,
                track.artist.as_deref().unwrap_or_default(),
            );
            if let Some(label) = track
                .track_release
                .as_ref()
                .and_then(|tr| tr.label.as_ref())
            {
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

fn message_header(request: &BuildRequest) -> Element {
    let header = &request.header;
    let mut element = Element::new("MessageHeader");
    element.add_child(text_element("MessageId", &header.message_id));
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
    element.add_child(text_element(
        "MessageCreatedDateTime",
        &header.message_created_date_time,
    ));
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
    element.add_child(avs_element(
        "DisplayArtistRole",
        role.unwrap_or(&main_artist),
    ));
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
    if let Some(isrc) = &track.isrc {
        resource_id.add_child(text_element("ISRC", isrc));
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
    let track_artist = track.artist.as_deref().unwrap_or_default();
    element.add_child(text_element("DisplayArtistName", track_artist));
    element.add_child(display_artist(
        parties,
        track_artist,
        track.artist_role.as_ref(),
    ));
    element.add_child(text_element("Duration", &track.duration));
    if let Some(pw) = &track.parental_warning_type {
        element.add_child(text_element("ParentalWarningType", pw));
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
    element.add_child(release_id(&release_identifiers(release)));

    let titles: Vec<_> = release.title.iter().collect();
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
    let release_artist = release.artist.as_deref().unwrap_or_default();
    element.add_child(text_element("DisplayArtistName", release_artist));
    element.add_child(display_artist(
        parties,
        release_artist,
        release.artist_role.as_ref(),
    ));
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
    element.add_child(resource_group(release, GroupTypes::Ern4, |t| {
        t.resource_reference.clone().unwrap_or_default()
    }));
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
    if let Some(model) = &terms.commercial_model_type {
        terms_element.add_child(text_element("CommercialModelType", model));
    }
    for u in &terms.use_types {
        terms_element.add_child(text_element("UseType", u));
    }
    element.add_child(terms_element);
    element
}
