//! Parsed message → build request: [`BuildRequest::from_parsed`].
//!
//! The parser's [`ParsedERNMessage`] lives in `ddex-core`, which the builder already
//! depends on, so the bridge needs no dependency on `ddex-parser`.

use crate::builder::{
    AvsValueRequest, BuildRequest, DealRequest, DealTerms, LocalizedStringRequest,
    MessageHeaderRequest, PartyRequest, ProprietaryIdRequest, ReleaseIdRequest, ReleaseRequest,
    ResourceGroupContentItemRequest, ResourceGroupRequest, ResourceKind, TrackReleaseRequest,
    TrackRequest,
};
use ddex_core::models::common::{Identifier, IdentifierType, LocalizedString};
use ddex_core::models::extensions::Extensions;
use ddex_core::models::flat::{ParsedERNMessage, ParsedRelease, ParsedTrack};
use ddex_core::models::graph::{
    Artist, DisplayText, ERNMessage, MessageType, Release, ReleaseType, Resource, ResourceGroup,
    ResourceType,
};
use ddex_core::models::versions::ERNVersion;

use crate::round_trip::parsed::{
    commercial_model_name, iso_duration, release_type_name, use_type_name,
};

/// The result of [`BuildRequest::from_parsed`]: the build request, and the dotted path of
/// every non-empty parsed value the request cannot hold.
#[derive(Debug, Clone)]
pub struct FromParsed {
    /// The build request: every value the request model can hold, and nothing invented.
    pub request: BuildRequest,
    /// The dotted path (`releases[0].p_line`, `resources[2]`, ...) of every non-empty
    /// parsed value the request cannot hold, in document order. Empty values are not listed.
    pub unmapped: Vec<String>,
}

impl BuildRequest {
    /// Map a parsed message (from `ddex_parser::DDEXParser::parse`) to a build request for
    /// the same ERN version.
    ///
    /// Nothing is invented: a value absent from the parsed message stays `None` (or an empty
    /// `Vec`) in the request, and an empty `MessageId` stays empty, so the build then fails
    /// naming the field.
    ///
    /// **Mapped** (where the parsed message has a value): the ERN version; the header
    /// (`MessageId`, `MessageCreatedDateTime`, sender and recipient (first party ID and
    /// name), `MessageControlType`); the root `@LanguageAndScriptCode` and (4.3)
    /// `@AvsVersionId`; every main release (reference, identifiers (GRid, ICPN,
    /// proprietary IDs), titles, display artist and its role, label, release type, first
    /// genre / sub-genre, first parental warning, release date (an `xs:date`), resource
    /// group tree); its tracks (reference, ISRC, proprietary IDs, title, duration, display
    /// artist and its role, sound recording or video, the resource's `Type`, first parental
    /// warning); track releases (ERN 4.x `TrackRelease`, or an ERN 3.8.x `Release` of type
    /// `TrackRelease` with one resource: reference, identifiers, title, first genre,
    /// label); and deals (reference, release references, territories, first commercial
    /// model, use types, start and end date).
    ///
    /// **Never carried** (listed in [`FromParsed::unmapped`] when non-empty), because the
    /// request model has no field for them:
    /// - message: `MessageThreadId`, a message type other than `NewReleaseMessage`, sender /
    ///   recipient trading names and further party IDs or names, root attributes other than
    ///   `LanguageAndScriptCode` / `AvsVersionId` (e.g. `ReleaseProfileVersionId`,
    ///   `xsi:schemaLocation`; `MessageSchemaVersionId` is written from the version), the
    ///   message audit trail, comments and extension fragments;
    /// - parties: their IDs (ISNI, IPI, proprietary), roles and contact details, and
    ///   parties no mapped artist or label refers to (contributors, ...);
    /// - releases: subtitles, further genres, display artists, parental warnings and labels,
    ///   party lists, territories, P and C lines, original release dates, further release
    ///   dates (and one that is not an `xs:date`), identifiers other than GRid / ICPN /
    ///   proprietary IDs, a resource-group tree that does not place every track, the side /
    ///   hidden / bonus flags of a release resource reference, and their attributes,
    ///   comments and extension fragments;
    /// - resources: resources that are not a track of a mapped release (images, texts, ...),
    ///   technical details, rights controllers, P and C lines, genres, label names, further
    ///   display artists and parental warnings, identifiers other than ISRC / proprietary
    ///   IDs;
    /// - track releases: linked resources, further genres and labels, `IsMainRelease`;
    /// - deals: excluded territories, distribution channels, further commercial models,
    ///   prices, pre-order / instant-gratification / takedown dates, and release references
    ///   to releases the request does not carry.
    ///
    /// ```
    /// use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};
    /// use ddex_parser::DDEXParser;
    /// use std::io::Cursor;
    ///
    /// let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/Samples43/1 Audio.xml");
    /// let xml = std::fs::read_to_string(path)?;
    ///
    /// // Parse, map to a build request, modify, build
    /// let parsed = DDEXParser::new().parse(Cursor::new(xml.as_bytes()))?;
    /// let mut bridge = BuildRequest::from_parsed(&parsed);
    /// bridge.request.releases[0].title[0].text = "Yume no Lullaby (Remastered)".to_string();
    /// let built = DDEXBuilder::new().build(bridge.request, BuildOptions::default())?;
    ///
    /// // Read it back
    /// let rebuilt = DDEXParser::new().parse(Cursor::new(built.xml.as_bytes()))?;
    /// assert_eq!(
    ///     rebuilt.flat.releases[0].default_title.as_deref(),
    ///     Some("Yume no Lullaby (Remastered)")
    /// );
    /// // What the request could not carry, e.g. the sample's MessageThreadId
    /// assert!(bridge.unmapped.contains(&"header.message_thread_id".to_string()));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_parsed(parsed: &ParsedERNMessage) -> FromParsed {
        let mut bridge = Bridge {
            graph: &parsed.graph,
            ern4: matches!(parsed.graph.version, ERNVersion::V4_2 | ERNVersion::V4_3),
            unmapped: Vec::new(),
            used_parties: Vec::new(),
        };
        let request = bridge.request(parsed);
        FromParsed {
            request,
            unmapped: bridge.unmapped,
        }
    }
}

struct Bridge<'a> {
    graph: &'a ERNMessage,
    ern4: bool,
    unmapped: Vec<String>,
    /// Party references a mapped artist or label refers to
    used_parties: Vec<String>,
}

impl<'a> Bridge<'a> {
    fn note(&mut self, path: impl Into<String>) {
        self.unmapped.push(path.into());
    }

    fn note_if(&mut self, non_empty: bool, path: impl Into<String>) {
        if non_empty {
            self.note(path);
        }
    }

    fn note_rest<T>(&mut self, items: &[T], path: &str) {
        for i in 1..items.len() {
            self.note(format!("{path}[{i}]"));
        }
    }

    fn note_extras(
        &mut self,
        attributes: bool,
        extensions: &Option<Extensions>,
        comments: bool,
        at: &str,
    ) {
        self.note_if(attributes, format!("{at}.attributes"));
        self.note_if(has_fragments(extensions), format!("{at}.extensions"));
        self.note_if(comments, format!("{at}.comments"));
    }

    fn request(&mut self, parsed: &ParsedERNMessage) -> BuildRequest {
        let graph = self.graph;
        let flat = &parsed.flat;
        let version = graph.version.as_str().to_string();
        let header = self.header(parsed);

        // Releases: ERN 4.x lists track releases separately; in ERN 3.8.x a track release
        // is a `Release` of type `TrackRelease` with one resource.
        let ern4 = self.ern4;
        let is_track_release_38 = |r: &Release| {
            !ern4
                && matches!(&r.release_type, Some(ReleaseType::Other(t)) if t == "TrackRelease")
                && r.release_resource_reference_list.len() == 1
        };
        let track_releases_38: Vec<(usize, &Release)> = graph
            .releases
            .iter()
            .enumerate()
            .filter(|(_, r)| is_track_release_38(r))
            .collect();
        let mut releases = Vec::new();
        for (i, release) in graph.releases.iter().enumerate() {
            if is_track_release_38(release) {
                continue;
            }
            let flat_release = flat
                .releases
                .iter()
                .find(|f| f.release_id == release.release_reference);
            releases.push(self.release(release, flat_release, &track_releases_38, i));
        }
        // Track releases that no mapped track has
        let attached: Vec<String> = releases
            .iter()
            .flat_map(|r| r.tracks.iter())
            .filter_map(|t| t.track_release.as_ref())
            .map(|tr| tr.release_reference.clone())
            .collect();
        for (i, tr) in &track_releases_38 {
            self.note_if(
                !attached.contains(&tr.release_reference),
                format!("releases[{i}]"),
            );
        }
        for (i, tr) in graph.track_releases.iter().enumerate() {
            self.note_if(
                !attached.contains(&tr.release_reference),
                format!("track_releases[{i}]"),
            );
        }
        self.track_releases_unmapped();

        let release_refs: Vec<String> = releases
            .iter()
            .filter_map(|r: &ReleaseRequest| r.release_reference.clone())
            .chain(
                releases
                    .iter()
                    .flat_map(|r| r.tracks.iter())
                    .filter_map(|t| t.track_release.as_ref())
                    .map(|tr| tr.release_reference.clone()),
            )
            .collect();
        let deals = self.deals(&release_refs);
        self.resources_unmapped(&releases);
        self.parties_unmapped();

        let root_attr = |name: &str| {
            graph
                .attributes
                .as_ref()?
                .get_str(name)
                .map(|v| v.to_xml_value())
        };
        if let Some(attributes) = &graph.attributes {
            for (name, _) in attributes.iter() {
                let name = name.to_xml_name();
                let carried = matches!(
                    name.as_str(),
                    "LanguageAndScriptCode" | "AvsVersionId" | "MessageSchemaVersionId"
                ) || name == "xmlns"
                    || name.starts_with("xmlns:");
                if !carried {
                    self.note(format!("attributes.{name}"));
                }
            }
        }
        self.note_if(graph.message_audit_trail.is_some(), "message_audit_trail");
        self.note_if(
            graph.comments.as_ref().is_some_and(|c| !c.is_empty()),
            "comments",
        );
        self.note_if(has_fragments(&graph.extensions), "extensions");

        BuildRequest {
            header,
            avs_version_id: if graph.version == ERNVersion::V4_3 {
                root_attr("AvsVersionId")
            } else {
                None
            },
            language_and_script_code: root_attr("LanguageAndScriptCode"),
            version,
            profile: None,
            releases,
            deals,
            extensions: None,
        }
    }

    fn header(&mut self, parsed: &ParsedERNMessage) -> MessageHeaderRequest {
        let header = &self.graph.message_header;
        self.note_if(
            !matches!(header.message_type, MessageType::NewReleaseMessage),
            "header.message_type",
        );
        self.note_if(
            header
                .message_thread_id
                .as_deref()
                .is_some_and(|s| !s.is_empty()),
            "header.message_thread_id",
        );
        for (party, at) in [
            (
                (
                    &header.message_sender.party_id,
                    &header.message_sender.party_name,
                    &header.message_sender.trading_name,
                ),
                "header.message_sender",
            ),
            (
                (
                    &header.message_recipient.party_id,
                    &header.message_recipient.party_name,
                    &header.message_recipient.trading_name,
                ),
                "header.message_recipient",
            ),
        ] {
            let (ids, names, trading) = party;
            self.note_rest(ids, &format!("{at}.party_id"));
            self.note_rest(names, &format!("{at}.party_name"));
            self.note_if(
                trading.as_deref().is_some_and(|s| !s.is_empty()),
                format!("{at}.trading_name"),
            );
        }
        let party = |ids: &[Identifier], names: &[LocalizedString]| PartyRequest {
            party_name: names
                .first()
                .map(|n| {
                    vec![LocalizedStringRequest {
                        text: n.text.clone(),
                        language_code: n.language_code.clone(),
                    }]
                })
                .unwrap_or_default(),
            party_id: ids.first().map(|i| i.value.clone()),
            party_reference: None,
        };
        MessageHeaderRequest {
            message_id: header.message_id.clone(),
            message_sender: party(
                &header.message_sender.party_id,
                &header.message_sender.party_name,
            ),
            message_recipient: party(
                &header.message_recipient.party_id,
                &header.message_recipient.party_name,
            ),
            message_control_type: parsed.flat.message_control_type.clone(),
            message_created_date_time: parsed
                .flat
                .message_date
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        }
    }

    fn label(&mut self, references: &[String], names: &[String]) -> Option<String> {
        if let Some(reference) = references.first() {
            if let Some(name) = party_name(self.graph, reference) {
                self.used_parties.push(reference.clone());
                return Some(name);
            }
        }
        names.first().cloned()
    }

    fn artist_role(&mut self, artists: &[Artist]) -> Option<AvsValueRequest> {
        for artist in artists.iter().take(1) {
            if let Some(reference) = &artist.party_reference {
                self.used_parties.push(reference.clone());
            }
        }
        artists
            .first()
            .and_then(|a| a.artist_role.first())
            .filter(|r| r.as_str() != "MainArtist")
            .map(|r| r.as_str().into())
    }

    fn release_ids(&mut self, ids: &[Identifier], at: &str) -> ReleaseIdRequest {
        let mut out = ReleaseIdRequest::default();
        for (i, id) in ids.iter().enumerate() {
            match (&id.id_type, &id.namespace) {
                (IdentifierType::GRID | IdentifierType::GRid, _) if out.grid.is_none() => {
                    out.grid = Some(id.value.clone())
                }
                (IdentifierType::UPC, _) if out.icpn.is_none() => out.icpn = Some(id.value.clone()),
                (IdentifierType::Proprietary, Some(ns)) => {
                    out.proprietary_ids.push(ProprietaryIdRequest {
                        namespace: ns.clone(),
                        value: id.value.clone(),
                    })
                }
                _ => self.note(format!("{at}[{i}]")),
            }
        }
        out
    }

    fn release(
        &mut self,
        release: &Release,
        flat_release: Option<&ParsedRelease>,
        track_releases_38: &[(usize, &Release)],
        index: usize,
    ) -> ReleaseRequest {
        let at = format!("releases[{index}]");

        // Tracks, in the flat order, each once
        let mut tracks = Vec::new();
        let mut seen: Vec<&str> = Vec::new();
        for t in flat_release
            .map(|f| f.tracks.as_slice())
            .unwrap_or_default()
        {
            if seen.contains(&t.track_id.as_str()) {
                continue;
            }
            seen.push(&t.track_id);
            tracks.push(self.track(t, track_releases_38));
        }
        let track_refs: Vec<String> = tracks.iter().map(|t| t.track_id.clone()).collect();

        let title = if self.ern4 {
            titles(&release.display_title_text)
        } else {
            release
                .release_title
                .iter()
                .map(|t| LocalizedStringRequest {
                    text: t.text.clone(),
                    language_code: t.language_code.clone(),
                })
                .collect()
        };
        let label_names: Vec<String> = release.label_name.iter().map(|l| l.name.clone()).collect();
        let label = self.label(&release.release_label_reference, &label_names);
        let artist_role = self.artist_role(&release.display_artist);
        let release_ids = self.release_ids(&release.release_id, &format!("{at}.release_id"));

        // ReleaseDate: the first one, when it is an xs:date the builder can write
        let dates = flat_release
            .map(|f| f.release_dates.as_slice())
            .unwrap_or_default();
        let release_date = dates
            .first()
            .map(|d| d.date.clone())
            .filter(|d| is_xs_date(d));
        if dates.first().is_some() && release_date.is_none() {
            self.note(format!("{at}.release_date[0]"));
        }
        self.note_rest(dates, &format!("{at}.release_date"));

        // The group tree, when it places every track (the builder requires that)
        let resource_group = release.resource_group.as_ref().and_then(|g| {
            let group = group_request(g, &track_refs);
            let mut placed = Vec::new();
            placed_refs(&group, &mut placed);
            if track_refs.iter().all(|r| placed.contains(&r.as_str())) {
                Some(group)
            } else {
                None
            }
        });
        if release.resource_group.is_some() && resource_group.is_none() {
            self.note(format!("{at}.resource_group"));
        }

        // What the release carries that the request cannot hold
        self.note_if(
            release
                .release_subtitle
                .as_ref()
                .is_some_and(|s| !s.is_empty()),
            format!("{at}.release_subtitle"),
        );
        self.note_rest(&release.genre, &format!("{at}.genre"));
        self.note_rest(&release.display_artist, &format!("{at}.display_artist"));
        self.note_rest(
            &release.parental_warning_type,
            &format!("{at}.parental_warning_type"),
        );
        self.note_rest(
            &release.release_label_reference,
            &format!("{at}.release_label_reference"),
        );
        self.note_rest(&release.label_name, &format!("{at}.label_name"));
        self.note_if(!release.party_list.is_empty(), format!("{at}.party_list"));
        self.note_if(
            !release.territory_code.is_empty(),
            format!("{at}.territory_code"),
        );
        self.note_if(
            !release.excluded_territory_code.is_empty(),
            format!("{at}.excluded_territory_code"),
        );
        self.note_if(!release.p_line.is_empty(), format!("{at}.p_line"));
        self.note_if(!release.c_line.is_empty(), format!("{at}.c_line"));
        self.note_if(
            !release.original_release_date.is_empty(),
            format!("{at}.original_release_date"),
        );
        for (i, r) in release.release_resource_reference_list.iter().enumerate() {
            let rat = format!("{at}.release_resource_reference_list[{i}]");
            self.note_if(r.side.is_some(), format!("{rat}.side"));
            self.note_if(r.is_hidden, format!("{rat}.is_hidden"));
            self.note_if(r.is_bonus, format!("{rat}.is_bonus"));
        }
        self.note_extras(
            release.attributes.as_ref().is_some_and(|a| !a.is_empty()),
            &release.extensions,
            release.comments.as_ref().is_some_and(|c| !c.is_empty()),
            &at,
        );

        ReleaseRequest {
            release_id: None,
            release_reference: Some(release.release_reference.clone()),
            title,
            artist: flat_release
                .and_then(|f| f.display_artist.clone())
                .filter(|s| !s.is_empty()),
            label,
            release_date,
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
            artist_role,
            release_ids: Some(release_ids),
            resource_group,
        }
    }

    fn track(&mut self, t: &ParsedTrack, track_releases_38: &[(usize, &Release)]) -> TrackRequest {
        let graph = self.graph;
        let resource = graph
            .resources
            .iter()
            .find(|r| r.resource_reference == t.track_id);
        let artist = if self.ern4 {
            t.display_artist.clone()
        } else {
            resource
                .and_then(|r| r.display_artist.first())
                .and_then(|a| a.display_artist_name.first())
                .map(|n| n.text.clone())
        }
        .filter(|s| !s.is_empty());
        let artist_role = resource
            .map(|r| self.artist_role(&r.display_artist))
            .unwrap_or_default();

        let track_release = if self.ern4 {
            graph
                .track_releases
                .iter()
                .enumerate()
                .find(|(_, tr)| tr.release_resource_reference == t.track_id)
                .map(|(i, tr)| {
                    let at = format!("track_releases[{i}]");
                    let label = self.label(&tr.release_label_reference, &[]);
                    TrackReleaseRequest {
                        release_reference: tr.release_reference.clone(),
                        release_ids: self.release_ids(&tr.release_id, &format!("{at}.release_id")),
                        title: DisplayText::pick_default(&tr.display_title_text)
                            .or_else(|| DisplayText::pick_default(&tr.display_title))
                            .map(|d| d.text.clone()),
                        genre: tr.genre.first().map(|g| g.genre_text.clone()),
                        sub_genre: tr.genre.first().and_then(|g| g.sub_genre.clone()),
                        label,
                    }
                })
        } else {
            track_releases_38
                .iter()
                .find(|(_, tr)| {
                    tr.release_resource_reference_list[0].resource_reference == t.track_id
                })
                .map(|(i, tr)| {
                    let at = format!("releases[{i}]");
                    let names: Vec<String> = tr.label_name.iter().map(|l| l.name.clone()).collect();
                    let label = self.label(&tr.release_label_reference, &names);
                    self.note_rest(&tr.genre, &format!("{at}.genre"));
                    self.note_if(!tr.p_line.is_empty(), format!("{at}.p_line"));
                    self.note_if(!tr.c_line.is_empty(), format!("{at}.c_line"));
                    TrackReleaseRequest {
                        release_reference: tr.release_reference.clone(),
                        release_ids: self.release_ids(&tr.release_id, &format!("{at}.release_id")),
                        title: tr.release_title.first().map(|t| t.text.clone()),
                        genre: tr.genre.first().map(|g| g.genre_text.clone()),
                        sub_genre: tr.genre.first().and_then(|g| g.sub_genre.clone()),
                        label,
                    }
                })
        };

        TrackRequest {
            track_id: t.track_id.clone(),
            resource_reference: Some(t.track_id.clone()),
            isrc: t.isrc.clone().filter(|s| !s.is_empty()),
            title: t.title.clone().unwrap_or_default(),
            duration: t
                .duration
                .map(|d| iso_duration(d.as_secs()))
                .unwrap_or_default(),
            artist,
            resource_kind: match resource.map(|r| &r.resource_type) {
                Some(ResourceType::Video) => ResourceKind::Video,
                _ => ResourceKind::SoundRecording,
            },
            resource_type: resource.and_then(|r| {
                r.type_value.as_ref().map(|v| AvsValueRequest {
                    value: v.clone(),
                    namespace: None,
                    user_defined_value: r.type_user_defined_value.clone(),
                })
            }),
            parental_warning_type: resource.and_then(|r| r.parental_warning_type.first().cloned()),
            artist_role,
            proprietary_ids: resource
                .map(|r| {
                    r.resource_id
                        .iter()
                        .filter(|id| matches!(id.id_type, IdentifierType::Proprietary))
                        .filter_map(|id| {
                            Some(ProprietaryIdRequest {
                                namespace: id.namespace.clone()?,
                                value: id.value.clone(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
            track_release,
            ..Default::default()
        }
    }

    /// ERN 4.x track releases: what the request cannot hold.
    fn track_releases_unmapped(&mut self) {
        for (i, tr) in self.graph.track_releases.iter().enumerate() {
            let at = format!("track_releases[{i}]");
            self.note_if(
                !tr.linked_release_resource_references.is_empty(),
                format!("{at}.linked_release_resource_references"),
            );
            self.note_rest(&tr.genre, &format!("{at}.genre"));
            self.note_rest(
                &tr.release_label_reference,
                &format!("{at}.release_label_reference"),
            );
            self.note_if(
                tr.is_main_release.is_some(),
                format!("{at}.is_main_release"),
            );
        }
    }

    fn deals(&mut self, release_refs: &[String]) -> Vec<DealRequest> {
        let mut deals = Vec::new();
        for (i, d) in self.graph.deals.iter().enumerate() {
            let at = format!("deals[{i}]");
            let terms = &d.deal_terms;
            for (j, r) in d.deal_release_reference.iter().enumerate() {
                self.note_if(
                    !release_refs.contains(r),
                    format!("{at}.deal_release_reference[{j}]"),
                );
            }
            self.note_if(
                !terms.excluded_territory_code.is_empty(),
                format!("{at}.excluded_territory_code"),
            );
            self.note_if(
                !terms.distribution_channel.is_empty(),
                format!("{at}.distribution_channel"),
            );
            self.note_if(
                !terms.excluded_distribution_channel.is_empty(),
                format!("{at}.excluded_distribution_channel"),
            );
            self.note_rest(
                &terms.commercial_model_type,
                &format!("{at}.commercial_model_type"),
            );
            self.note_if(
                !terms.price_information.is_empty(),
                format!("{at}.price_information"),
            );
            self.note_if(
                !terms.wholesale_price.is_empty(),
                format!("{at}.wholesale_price"),
            );
            self.note_if(
                !terms.suggested_retail_price.is_empty(),
                format!("{at}.suggested_retail_price"),
            );
            for (date, name) in [
                (&terms.pre_order_date, "pre_order_date"),
                (&terms.pre_order_preview_date, "pre_order_preview_date"),
                (
                    &terms.instant_gratification_date,
                    "instant_gratification_date",
                ),
                (&terms.takedown_date, "takedown_date"),
            ] {
                self.note_if(date.is_some(), format!("{at}.{name}"));
            }
            deals.push(DealRequest {
                deal_reference: d.deal_reference.clone(),
                deal_terms: DealTerms {
                    commercial_model_type: terms
                        .commercial_model_type
                        .first()
                        .map(commercial_model_name)
                        .filter(|s| !s.is_empty()),
                    territory_code: terms.territory_code.clone(),
                    start_date: terms.start_date.map(|s| s.format("%Y-%m-%d").to_string()),
                    end_date: terms.end_date.map(|s| s.format("%Y-%m-%d").to_string()),
                    use_types: terms.use_type.iter().map(use_type_name).collect(),
                },
                release_references: d
                    .deal_release_reference
                    .iter()
                    .filter(|r| release_refs.contains(r))
                    .cloned()
                    .collect(),
            });
        }
        deals
    }

    /// Resources: the ones that are not a mapped track, and what a track's resource
    /// carries that the request cannot hold.
    fn resources_unmapped(&mut self, releases: &[ReleaseRequest]) {
        let tracks: Vec<&str> = releases
            .iter()
            .flat_map(|r| r.tracks.iter())
            .map(|t| t.track_id.as_str())
            .collect();
        for (i, resource) in self.graph.resources.iter().enumerate() {
            let at = format!("resources[{i}]");
            if !tracks.contains(&resource.resource_reference.as_str()) {
                self.note(at);
                continue;
            }
            self.resource_unmapped(resource, &at);
        }
    }

    fn resource_unmapped(&mut self, r: &Resource, at: &str) {
        for (j, id) in r.resource_id.iter().enumerate() {
            let carried = matches!(id.id_type, IdentifierType::ISRC)
                || (matches!(id.id_type, IdentifierType::Proprietary) && id.namespace.is_some());
            self.note_if(!carried, format!("{at}.resource_id[{j}]"));
        }
        self.note_if(
            !r.technical_details.is_empty(),
            format!("{at}.technical_details"),
        );
        self.note_if(
            !r.rights_controller.is_empty(),
            format!("{at}.rights_controller"),
        );
        self.note_if(!r.p_line.is_empty(), format!("{at}.p_line"));
        self.note_if(!r.c_line.is_empty(), format!("{at}.c_line"));
        self.note_if(!r.genre.is_empty(), format!("{at}.genre"));
        self.note_if(!r.label_name.is_empty(), format!("{at}.label_name"));
        self.note_rest(&r.display_artist, &format!("{at}.display_artist"));
        self.note_rest(
            &r.parental_warning_type,
            &format!("{at}.parental_warning_type"),
        );
        self.note_if(has_fragments(&r.extensions), format!("{at}.extensions"));
    }

    /// Parties: their IDs, roles and contact details, and the ones no mapped artist or
    /// label refers to.
    fn parties_unmapped(&mut self) {
        for (i, p) in self.graph.parties.iter().enumerate() {
            let at = format!("parties[{i}]");
            let used = p
                .party_reference
                .as_ref()
                .is_some_and(|r| self.used_parties.contains(r));
            if !used {
                self.note(at);
                continue;
            }
            self.note_if(!p.party_id.is_empty(), format!("{at}.party_id"));
            self.note_if(p.isni.is_some(), format!("{at}.isni"));
            self.note_if(p.ipi.is_some(), format!("{at}.ipi"));
            self.note_if(!p.party_role.is_empty(), format!("{at}.party_role"));
            self.note_if(p.contact_details.is_some(), format!("{at}.contact_details"));
        }
    }
}

/// Extension fragments, comments or processing instructions (namespace declarations are
/// written by the builder and are not counted).
fn has_fragments(extensions: &Option<Extensions>) -> bool {
    extensions.as_ref().is_some_and(|e| {
        !e.fragments.is_empty()
            || !e.document_comments.is_empty()
            || !e.document_processing_instructions.is_empty()
    })
}

fn party_name(graph: &ERNMessage, reference: &str) -> Option<String> {
    graph
        .parties
        .iter()
        .find(|p| p.party_reference.as_deref() == Some(reference))
        .and_then(|p| p.party_name.first())
        .map(|n| n.text.clone())
}

/// `YYYY-MM-DD`
fn is_xs_date(value: &str) -> bool {
    let b = value.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
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

fn placed_refs<'g>(group: &'g ResourceGroupRequest, out: &mut Vec<&'g str>) {
    for item in &group.content_items {
        out.push(&item.resource_reference);
    }
    for sub in &group.resource_groups {
        placed_refs(sub, out);
    }
}
