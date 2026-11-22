// core/src/transform/flatten.rs
//! Graph to flat model transformation

use crate::error::ParseError;

type Result<T> = std::result::Result<T, ParseError>;
use ddex_core::models::common::{Copyright, Identifier, LocalizedString, TerritoryScope};
use ddex_core::models::flat::{
    ArtistInfo, DateInfo, DealValidity, DistributionComplexity, FlattenedMessage, GenreInfo,
    MessageStats, Organization, ParsedDeal, ParsedRelease, ParsedResource, ParsedTrack, PriceTier,
    PriceType, ProprietaryId, ReleaseIdentifiers, TechnicalInfo, TerritoryComplexity,
    TerritoryInfo, TrackGroup,
};
use ddex_core::models::graph::{
    Artist, Deal, DealTerms, DisplayText, ERNMessage, Genre, MessageControlType, Party, Release,
    ReleaseEvent, ReleaseResourceReference, Resource, ResourceGroup, ResourceType, TrackRelease,
};
use ddex_core::models::versions::ERNVersion;
use indexmap::IndexMap;
use std::collections::HashMap;

pub struct Flattener;

impl Flattener {
    pub fn flatten(graph: ERNMessage) -> Result<FlattenedMessage> {
        let releases = if matches!(graph.version, ERNVersion::V4_2 | ERNVersion::V4_3) {
            Self::flatten_releases_ern4(&graph)?
        } else {
            Self::flatten_releases(&graph.releases, &graph.resources)?
        };
        let resources = Self::flatten_resources(&graph.resources)?;
        let deals = Self::flatten_deals(&graph.deals)?;
        let parties = Self::flatten_parties(&graph.parties)?;

        let stats = MessageStats {
            release_count: releases.len(),
            track_count: releases.iter().map(|r| r.tracks.len()).sum(),
            deal_count: graph.deals.len(),
            total_duration: 0, // Set to 0 if no duration
        };

        Ok(FlattenedMessage {
            message_id: graph.message_header.message_id.clone(),
            message_type: format!("{:?}", graph.message_header.message_type),
            message_date: graph.message_header.message_created_date_time,
            sender: Organization {
                // PartyName is optional per the XSD (MessagingParty: PartyName [0..1]).
                name: Self::get_primary_name_optional(
                    &graph.message_header.message_sender.party_name,
                ),
                id: Self::get_primary_id(
                    &graph.message_header.message_sender.party_id,
                    "MessageSender/PartyId",
                )?,
                extensions: None,
            },
            recipient: Organization {
                name: Self::get_primary_name_optional(
                    &graph.message_header.message_recipient.party_name,
                ),
                id: Self::get_primary_id(
                    &graph.message_header.message_recipient.party_id,
                    "MessageRecipient/PartyId",
                )?,
                extensions: None,
            },
            releases,
            resources,
            deals,
            parties,
            version: format!("{:?}", graph.version),
            profile: graph.profile.map(|p| format!("{:?}", p)),
            stats,
            extensions: None,
            message_control_type: graph.message_header.message_control_type.as_ref().map(
                |t| match t {
                    MessageControlType::LiveMessage => "LiveMessage".to_string(),
                    MessageControlType::TestMessage => "TestMessage".to_string(),
                    MessageControlType::Other(v) => v.clone(),
                },
            ),
        })
    }

    fn flatten_releases(
        releases: &[Release],
        resources: &[Resource],
    ) -> Result<Vec<ParsedRelease>> {
        releases
            .iter()
            .map(|release| {
                let mut parsed = ParsedRelease {
                    release_id: release.release_reference.clone(),
                    identifiers: Self::extract_identifiers(&release.release_id),
                    title: release.release_title.clone(),
                    default_title: Self::get_primary_title_optional(&release.release_title),
                    subtitle: release.release_subtitle.clone(),
                    default_subtitle: release
                        .release_subtitle
                        .as_ref()
                        .map(|s| Self::get_primary_title_optional(s))
                        .flatten(),
                    display_artist: Some(Self::format_display_artist(&release.display_artist)?)
                        .filter(|s| !s.is_empty()),
                    artists: Self::extract_artists(&release.display_artist)?,
                    release_type: release
                        .release_type
                        .as_ref()
                        .map(|t| format!("{:?}", t))
                        .ok_or_else(|| {
                            ParseError::MissingField("Release/ReleaseType".to_string())
                        })?,
                    genre: release.genre.first().map(|g| g.genre_text.clone()),
                    sub_genre: release.genre.first().and_then(|g| g.sub_genre.clone()),
                    tracks: Self::build_tracks(
                        &release.release_resource_reference_list,
                        resources,
                        release.resource_group.as_ref(),
                    )?,
                    track_count: release.release_resource_reference_list.len(),
                    disc_count: Self::count_discs(&release.release_resource_reference_list),
                    videos: Vec::new(),
                    images: Vec::new(),
                    cover_art: None,
                    release_date: release.release_date.first().and_then(|e| e.event_date),
                    original_release_date: None,
                    territories: Self::build_territories(
                        &release.territory_code,
                        &release.excluded_territory_code,
                    ),
                    p_line: None,
                    c_line: None,
                    parent_release: None,
                    child_releases: Vec::new(),
                    extensions: None,
                    genres: Vec::new(),
                    label_name: None,
                    original_release_dates: Vec::new(),
                    parental_warning: None,
                    parental_warnings: Vec::new(),
                    release_dates: Vec::new(),
                };
                fill_release_details(&mut parsed, release, None);
                Ok(parsed)
            })
            .collect()
    }

    fn flatten_resources(resources: &[Resource]) -> Result<IndexMap<String, ParsedResource>> {
        resources
            .iter()
            .map(|resource| {
                let mut parsed = ParsedResource {
                    resource_id: resource.resource_reference.clone(),
                    resource_type: format!("{:?}", resource.resource_type),
                    // ReferenceTitle is not present on all resource types (e.g. Image), so it stays None
                    // when absent rather than being filled in.
                    title: Self::get_primary_title_optional(&resource.reference_title)
                        .filter(|t| !t.is_empty()),
                    duration: resource.duration,
                    technical_details: TechnicalInfo {
                        file_format: resource
                            .technical_details
                            .first()
                            .and_then(|t| t.file_format.clone()),
                        bitrate: resource.technical_details.first().and_then(|t| t.bitrate),
                        sample_rate: resource
                            .technical_details
                            .first()
                            .and_then(|t| t.sample_rate),
                        file_size: resource.technical_details.first().and_then(|t| t.file_size),
                    },
                    genre: None,
                    sub_genre: None,
                    parental_warning: None,
                    p_line: None,
                    c_line: None,
                    label_name: None,
                };
                let details = ResourceDetails::of(resource);
                parsed.genre = details.genre;
                parsed.sub_genre = details.sub_genre;
                parsed.parental_warning = details.parental_warning;
                parsed.p_line = details.p_line;
                parsed.c_line = details.c_line;
                parsed.label_name = details.label_name;
                Ok((resource.resource_reference.clone(), parsed))
            })
            .collect()
    }

    fn flatten_deals(deals: &[Deal]) -> Result<Vec<ParsedDeal>> {
        deals
            .iter()
            .map(|deal| {
                Ok(ParsedDeal {
                    // DealReference is optional per DDEX ERN spec (minOccurs="0").
                    deal_id: deal.deal_reference.clone(),
                    releases: deal.deal_release_reference.clone(),
                    validity: DealValidity {
                        start: deal.deal_terms.start_date,
                        end: deal.deal_terms.end_date,
                    },
                    territories: TerritoryComplexity {
                        included: deal.deal_terms.territory_code.clone(),
                        excluded: deal.deal_terms.excluded_territory_code.clone(),
                    },
                    distribution_channels: DistributionComplexity {
                        included: deal
                            .deal_terms
                            .distribution_channel
                            .iter()
                            .map(|c| format!("{:?}", c))
                            .collect(),
                        excluded: deal
                            .deal_terms
                            .excluded_distribution_channel
                            .iter()
                            .map(|c| format!("{:?}", c))
                            .collect(),
                    },
                    pricing: Self::build_price_tiers(&deal.deal_terms),
                    usage_rights: deal
                        .deal_terms
                        .use_type
                        .iter()
                        .map(|u| format!("{:?}", u))
                        .collect(),
                    restrictions: Vec::new(),
                })
            })
            .collect()
    }

    /// Key parties by `PartyReference`, falling back to the first `PartyId`. A party with
    /// neither violates the schema and is reported as an error (no key is invented).
    fn flatten_parties(parties: &[Party]) -> Result<IndexMap<String, Party>> {
        parties
            .iter()
            .map(|party| {
                let key = party
                    .party_reference
                    .clone()
                    .or_else(|| Self::get_primary_id_optional(&party.party_id))
                    .ok_or_else(|| {
                        ParseError::MissingField(
                            "Party/PartyReference (or Party/PartyId)".to_string(),
                        )
                    })?;
                Ok((key, party.clone()))
            })
            .collect()
    }

    // Helper methods
    fn get_primary_name(names: &[LocalizedString], field_path: &str) -> Result<String> {
        names
            .first()
            .map(|n| n.text.clone())
            .ok_or_else(|| ParseError::MissingField(field_path.to_string()))
    }

    fn get_primary_id(ids: &[Identifier], field_path: &str) -> Result<String> {
        ids.first()
            .map(|id| id.value.clone())
            .ok_or_else(|| ParseError::MissingField(field_path.to_string()))
    }

    // Optional variants that return Option<T> instead of failing
    fn get_primary_name_optional(names: &[LocalizedString]) -> Option<String> {
        names.first().map(|n| n.text.clone())
    }

    fn get_primary_title_optional(titles: &[LocalizedString]) -> Option<String> {
        titles.first().map(|t| t.text.clone())
    }

    fn get_primary_id_optional(ids: &[Identifier]) -> Option<String> {
        ids.first().map(|id| id.value.clone())
    }

    #[allow(dead_code)]
    fn count_tracks(releases: &[ParsedRelease]) -> usize {
        releases.iter().map(|r| r.track_count).sum()
    }

    #[allow(dead_code)]
    fn calculate_total_duration(resources: &HashMap<String, ParsedResource>) -> u64 {
        resources
            .values()
            .filter_map(|r| r.duration)
            .map(|d| d.as_secs())
            .sum()
    }

    fn extract_identifiers(ids: &[Identifier]) -> ReleaseIdentifiers {
        let mut identifiers = ReleaseIdentifiers {
            upc: None,
            ean: None,
            catalog_number: None,
            grid: None,
            proprietary: Vec::new(),
            isrc: None,
        };

        for id in ids {
            match &id.id_type {
                ddex_core::models::common::IdentifierType::UPC => {
                    identifiers.upc = Some(id.value.clone())
                }
                ddex_core::models::common::IdentifierType::EAN => {
                    identifiers.ean = Some(id.value.clone())
                }
                ddex_core::models::common::IdentifierType::GRID => {
                    identifiers.grid = Some(id.value.clone())
                }
                ddex_core::models::common::IdentifierType::ISRC => {
                    identifiers.isrc = Some(id.value.clone())
                }
                ddex_core::models::common::IdentifierType::Proprietary => {
                    if let Some(ns) = &id.namespace {
                        identifiers.proprietary.push(ProprietaryId {
                            namespace: ns.clone(),
                            value: id.value.clone(),
                        });
                    }
                }
                _ => {}
            }
        }

        identifiers
    }

    fn format_display_artist(artists: &[Artist]) -> Result<String> {
        let names: Result<Vec<String>> = artists
            .iter()
            .map(|a| Self::get_primary_name(&a.display_artist_name, "Artist/DisplayArtistName"))
            .collect();
        Ok(names?.join(", "))
    }

    fn extract_artists(artists: &[Artist]) -> Result<Vec<ArtistInfo>> {
        artists
            .iter()
            .map(|artist| {
                Ok(ArtistInfo {
                    name: Self::get_primary_name(
                        &artist.display_artist_name,
                        "Artist/DisplayArtistName",
                    )?,
                    role: artist
                        .artist_role
                        .first()
                        .cloned()
                        .ok_or_else(|| ParseError::MissingField("Artist/ArtistRole".to_string()))?,
                    party_id: artist.party_reference.clone(),
                })
            })
            .collect()
    }

    fn build_tracks(
        refs: &[ReleaseResourceReference],
        resources: &[Resource],
        resource_group: Option<&ResourceGroup>,
    ) -> Result<Vec<ParsedTrack>> {
        // ERN 3.8.x: item SequenceNumber and enclosing groups from the ResourceGroup tree
        // (first occurrence of each reference). 3.8.2 groups carry no ResourceGroupType, so
        // no disc number is derived from them.
        let mut layout: IndexMap<&str, (Option<i32>, Vec<TrackGroup>)> = IndexMap::new();
        if let Some(group) = resource_group {
            group_layout(group, &mut Vec::new(), &mut layout);
        }
        refs.iter()
            .map(|rref| {
                let resource = resources
                    .iter()
                    .find(|r| r.resource_reference == rref.resource_reference);
                (rref, resource)
            })
            // Only references that resolve to a SoundRecording or Video are tracks. Images, text,
            // etc. are skipped, and so are references to resources that weren't parsed, since
            // their type is unknown. Filter before numbering so positions aren't shifted.
            .filter(|(_, resource)| {
                resource.map_or(false, |r| {
                    matches!(
                        r.resource_type,
                        ResourceType::SoundRecording | ResourceType::Video
                    )
                })
            })
            .enumerate()
            .map(|(idx, (rref, resource))| {
                // Title and duration are optional in the source; absent values stay None.
                let title = resource
                    .and_then(|r| Self::get_primary_title_optional(&r.reference_title))
                    .filter(|t| !t.is_empty());
                let duration = resource.and_then(|r| r.duration);
                let duration_formatted = duration.map(ParsedTrack::format_duration);
                let details = resource.map(ResourceDetails::of).unwrap_or_default();

                Ok(ParsedTrack {
                    track_id: rref.resource_reference.clone(),
                    isrc: resource.and_then(|r| {
                        r.resource_id
                            .iter()
                            .find(|id| {
                                matches!(
                                    id.id_type,
                                    ddex_core::models::common::IdentifierType::ISRC
                                )
                            })
                            .map(|id| id.value.clone())
                    }),
                    iswc: None,
                    position: idx + 1,
                    track_number: rref.track_number.or_else(|| {
                        layout
                            .get(rref.resource_reference.as_str())
                            .and_then(|(n, _)| *n)
                    }),
                    disc_number: rref.disc_number,
                    side: rref.side.clone(),
                    title,
                    subtitle: None,
                    display_artist: resource.and_then(|r| {
                        DisplayText::pick_default(&r.display_artist_name)
                            .map(|t| t.text.clone())
                            .or_else(|| {
                                let names: Vec<&str> = r
                                    .display_artist
                                    .iter()
                                    .filter_map(|a| a.display_artist_name.first())
                                    .map(|n| n.text.as_str())
                                    .collect();
                                (!names.is_empty()).then(|| names.join(", "))
                            })
                    }),
                    // Artists with both a name and a role (nothing is filled in).
                    artists: resource
                        .map(|r| {
                            r.display_artist
                                .iter()
                                .filter_map(|a| {
                                    Some(ArtistInfo {
                                        name: a.display_artist_name.first()?.text.clone(),
                                        role: a.artist_role.first()?.clone(),
                                        party_id: a.party_reference.clone(),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                    duration,
                    duration_formatted,
                    file_format: None,
                    bitrate: None,
                    sample_rate: None,
                    is_hidden: rref.is_hidden,
                    is_bonus: rref.is_bonus,
                    is_explicit: false,
                    is_instrumental: false,
                    track_release_reference: None,
                    track_release_id: None,
                    group_path: layout
                        .get(rref.resource_reference.as_str())
                        .map(|(_, path)| path.clone())
                        .unwrap_or_default(),
                    c_line: details.c_line,
                    genre: details.genre,
                    label_name: details.label_name,
                    p_line: details.p_line,
                    parental_warning: details.parental_warning,
                    sub_genre: details.sub_genre,
                    track_release_genre: None,
                    track_release_sub_genre: None,
                })
            })
            .collect()
    }

    fn count_discs(refs: &[ReleaseResourceReference]) -> Option<usize> {
        refs.iter()
            .filter_map(|r| r.disc_number)
            .max()
            .map(|n| n as usize)
    }

    fn build_territories(included: &[String], excluded: &[String]) -> Vec<TerritoryInfo> {
        let mut territories = Vec::new();

        for code in included {
            territories.push(TerritoryInfo {
                code: code.clone(),
                included: true,
                start_date: None,
                end_date: None,
                distribution_channels: Vec::new(),
            });
        }

        for code in excluded {
            territories.push(TerritoryInfo {
                code: code.clone(),
                included: false,
                start_date: None,
                end_date: None,
                distribution_channels: Vec::new(),
            });
        }

        territories
    }

    fn build_price_tiers(terms: &DealTerms) -> Vec<PriceTier> {
        let mut tiers = Vec::new();

        for price in &terms.wholesale_price {
            tiers.push(PriceTier {
                tier_name: None,
                price_type: PriceType::Wholesale,
                price: price.clone(),
                territory: price.territory.clone(),
                start_date: terms.start_date,
                end_date: terms.end_date,
            });
        }

        for price in &terms.suggested_retail_price {
            tiers.push(PriceTier {
                tier_name: None,
                price_type: PriceType::SuggestedRetail,
                price: price.clone(),
                territory: price.territory.clone(),
                start_date: terms.start_date,
                end_date: terms.end_date,
            });
        }

        tiers
    }
}

/// Lookups shared while flattening an ERN 4.x message.
struct Ern4Index<'a> {
    resources: IndexMap<&'a str, &'a Resource>,
    /// Resource reference -> first `TrackRelease` (document order) releasing it.
    track_release_by_resource: IndexMap<&'a str, &'a TrackRelease>,
    /// `PartyReference` -> first `PartyName/FullName`.
    party_names: IndexMap<&'a str, &'a str>,
}

impl<'a> Ern4Index<'a> {
    fn new(graph: &'a ERNMessage) -> Self {
        let resources = graph
            .resources
            .iter()
            .map(|r| (r.resource_reference.as_str(), r))
            .collect();
        let mut track_release_by_resource = IndexMap::new();
        for tr in &graph.track_releases {
            track_release_by_resource
                .entry(tr.release_resource_reference.as_str())
                .or_insert(tr);
        }
        let party_names = graph
            .parties
            .iter()
            .filter_map(|p| {
                Some((
                    p.party_reference.as_deref()?,
                    p.party_name.first()?.text.as_str(),
                ))
            })
            .collect();
        Self {
            resources,
            track_release_by_resource,
            party_names,
        }
    }

    /// `DisplayArtistName` (IsDefault, else first); otherwise the names of the
    /// `DisplayArtist` parties joined with ", "; otherwise `None`.
    fn display_artist(&self, names: &[DisplayText], artists: &[Artist]) -> Option<String> {
        DisplayText::pick_default(names)
            .map(|t| t.text.clone())
            .or_else(|| {
                let resolved: Vec<&str> = artists
                    .iter()
                    .filter_map(|a| self.party_name(a.party_reference.as_deref()?))
                    .collect();
                (!resolved.is_empty()).then(|| resolved.join(", "))
            })
    }

    /// `DisplayArtist` entries whose party resolves to a name and that carry a role.
    fn artist_infos(&self, artists: &[Artist]) -> Vec<ArtistInfo> {
        artists
            .iter()
            .filter_map(|a| {
                let reference = a.party_reference.as_deref()?;
                Some(ArtistInfo {
                    name: self.party_name(reference)?.to_string(),
                    role: a.artist_role.first()?.clone(),
                    party_id: Some(reference.to_string()),
                })
            })
            .collect()
    }

    fn party_name(&self, reference: &str) -> Option<&'a str> {
        self.party_names.get(reference).copied()
    }

    /// The resource behind a reference, if it is track-eligible (SoundRecording / Video).
    fn track_resource(&self, reference: &str) -> Option<&'a Resource> {
        self.resources.get(reference).copied().filter(|r| {
            matches!(
                r.resource_type,
                ResourceType::SoundRecording | ResourceType::Video
            )
        })
    }

    fn track(
        &self,
        resource: &Resource,
        position: usize,
        track_number: Option<i32>,
        disc_number: Option<i32>,
        group_path: Vec<TrackGroup>,
    ) -> ParsedTrack {
        let track_release = self
            .track_release_by_resource
            .get(resource.resource_reference.as_str());
        let duration = resource.duration;
        let details = ResourceDetails::of(resource);
        let track_release_genre =
            track_release.and_then(|tr| TerritoryScope::pick_default(&tr.genre, |g| &g.scope));
        ParsedTrack {
            track_id: resource.resource_reference.clone(),
            isrc: resource
                .resource_id
                .iter()
                .find(|id| matches!(id.id_type, ddex_core::models::common::IdentifierType::ISRC))
                .map(|id| id.value.clone()),
            iswc: None,
            position,
            track_number,
            disc_number,
            side: None,
            title: display_title(
                &resource.display_title_text,
                &resource.display_title,
                &resource.reference_title,
            ),
            subtitle: None,
            display_artist: self
                .display_artist(&resource.display_artist_name, &resource.display_artist),
            artists: self.artist_infos(&resource.display_artist),
            duration,
            duration_formatted: duration.map(ParsedTrack::format_duration),
            file_format: None,
            bitrate: None,
            sample_rate: None,
            is_hidden: false,
            is_bonus: false,
            is_explicit: false,
            is_instrumental: false,
            track_release_reference: track_release.map(|tr| tr.release_reference.clone()),
            track_release_id: track_release
                .and_then(|tr| tr.release_id.first())
                .map(|id| id.value.clone()),
            group_path,
            c_line: details.c_line,
            genre: details.genre,
            label_name: details.label_name,
            p_line: details.p_line,
            parental_warning: details.parental_warning,
            sub_genre: details.sub_genre,
            track_release_genre: track_release_genre.map(|g| g.genre_text.clone()),
            track_release_sub_genre: track_release_genre.and_then(|g| g.sub_genre.clone()),
        }
    }
}

/// `DisplayTitleText` (IsDefault, else first), falling back to `DisplayTitle/TitleText`
/// (IsDefault, else first), then to the first legacy title. Empty strings count as absent.
fn display_title(
    display_title_text: &[DisplayText],
    display_title: &[DisplayText],
    legacy: &[LocalizedString],
) -> Option<String> {
    DisplayText::pick_default(display_title_text)
        .or_else(|| DisplayText::pick_default(display_title))
        .map(|t| t.text.clone())
        .or_else(|| legacy.first().map(|t| t.text.clone()))
        .filter(|t| !t.is_empty())
}

/// Disc anchor for position numbering: the group that defines the disc.
#[derive(Clone, Copy)]
struct DiscAnchor {
    /// Unique id of the anchoring group within this release walk.
    id: usize,
    sequence_number: Option<i32>,
}

/// Walks a release's `ResourceGroup` tree in document order and builds its tracks.
///
/// * `disc_number`: `SequenceNumber` of the nearest enclosing `Component` group; if there
///   is none, of the nearest `ReleaseComponent`. `MultiWorkPart` / `Side` never set it.
/// * `position`: 1-based order among tracks within that disc (or the release if no disc).
/// * `track_number`: the item's own `SequenceNumber`, as written.
struct TrackWalker<'i, 'a> {
    index: &'i Ern4Index<'a>,
    tracks: Vec<ParsedTrack>,
    /// Anchor id -> tracks numbered so far (id 0 = the release itself).
    positions: IndexMap<usize, usize>,
    next_group_id: usize,
}

impl<'i, 'a> TrackWalker<'i, 'a> {
    fn walk(
        &mut self,
        group: &ResourceGroup,
        path: &mut Vec<TrackGroup>,
        component: Option<DiscAnchor>,
        release_component: Option<DiscAnchor>,
    ) {
        // Nested groups precede content items in the XSD, so this is document order.
        for sub in &group.resource_groups {
            self.next_group_id += 1;
            let anchor = DiscAnchor {
                id: self.next_group_id,
                sequence_number: sub.sequence_number,
            };
            let (component, release_component) = match sub.resource_group_type.as_deref() {
                Some("Component") => (Some(anchor), release_component),
                Some("ReleaseComponent") => (component, Some(anchor)),
                _ => (component, release_component),
            };
            path.push(TrackGroup {
                group_type: sub.resource_group_type.clone(),
                sequence_number: sub.sequence_number,
            });
            self.walk(sub, path, component, release_component);
            path.pop();
        }
        for item in &group.content_items {
            let Some(resource) = self.index.track_resource(&item.release_resource_reference) else {
                continue; // images, text, unknown references: not tracks
            };
            let disc = component.or(release_component);
            let counter = self.positions.entry(disc.map_or(0, |d| d.id)).or_insert(0);
            *counter += 1;
            let position = *counter;
            self.tracks.push(self.index.track(
                resource,
                position,
                item.sequence_number,
                disc.and_then(|d| d.sequence_number),
                path.clone(),
            ));
        }
    }
}

impl Flattener {
    /// ERN 4.x: one flat release per main `Release` (TrackReleases are linked to tracks,
    /// not emitted as releases). In 4.3, when there is no `Release`, each `TrackRelease`
    /// with `IsMainRelease="true"` becomes the flat release instead.
    fn flatten_releases_ern4(graph: &ERNMessage) -> Result<Vec<ParsedRelease>> {
        let index = Ern4Index::new(graph);
        let mut releases = graph
            .releases
            .iter()
            .map(|release| Self::flatten_release_ern4(release, &index))
            .collect::<Result<Vec<_>>>()?;
        if graph.releases.is_empty() {
            releases.extend(
                graph
                    .track_releases
                    .iter()
                    .filter(|tr| tr.is_main_release == Some(true))
                    .map(|tr| Self::flatten_main_track_release(tr, &index)),
            );
        }
        Ok(releases)
    }

    fn flatten_release_ern4(release: &Release, index: &Ern4Index) -> Result<ParsedRelease> {
        let mut walker = TrackWalker {
            index,
            tracks: Vec::new(),
            positions: IndexMap::new(),
            next_group_id: 0,
        };
        if let Some(group) = &release.resource_group {
            walker.walk(group, &mut Vec::new(), None, None);
        }
        let discs = walker.positions.keys().filter(|id| **id != 0).count();
        let tracks = walker.tracks;
        let mut parsed = ParsedRelease {
            release_id: release.release_reference.clone(),
            identifiers: Self::extract_identifiers(&release.release_id),
            title: release.release_title.clone(),
            default_title: display_title(
                &release.display_title_text,
                &release.display_title,
                &release.release_title,
            ),
            subtitle: release.release_subtitle.clone(),
            default_subtitle: release
                .release_subtitle
                .as_ref()
                .and_then(|s| Self::get_primary_title_optional(s)),
            display_artist: index
                .display_artist(&release.display_artist_name, &release.display_artist),
            artists: index.artist_infos(&release.display_artist),
            release_type: release
                .release_type
                .as_ref()
                .map(|t| format!("{:?}", t))
                .ok_or_else(|| ParseError::MissingField("Release/ReleaseType".to_string()))?,
            genre: release.genre.first().map(|g| g.genre_text.clone()),
            sub_genre: release.genre.first().and_then(|g| g.sub_genre.clone()),
            track_count: tracks.len(),
            disc_count: (discs > 0).then_some(discs),
            tracks,
            videos: Vec::new(),
            images: Vec::new(),
            cover_art: None,
            release_date: release.release_date.first().and_then(|e| e.event_date),
            original_release_date: None,
            territories: Self::build_territories(
                &release.territory_code,
                &release.excluded_territory_code,
            ),
            p_line: None,
            c_line: None,
            parent_release: None,
            child_releases: Vec::new(),
            extensions: None,
            genres: Vec::new(),
            label_name: None,
            original_release_dates: Vec::new(),
            parental_warning: None,
            parental_warnings: Vec::new(),
            release_dates: Vec::new(),
        };
        let label = release
            .release_label_reference
            .first()
            .and_then(|r| index.party_name(r))
            .map(str::to_string);
        fill_release_details(&mut parsed, release, label);
        Ok(parsed)
    }

    /// A 4.3 `TrackRelease IsMainRelease="true"` standing in for the absent `Release`.
    /// `release_type` is the AVS term `TrackRelease`; the release's one track is its
    /// resource. A TrackRelease has no display artist in the XSD, so that stays `None`.
    fn flatten_main_track_release(tr: &TrackRelease, index: &Ern4Index) -> ParsedRelease {
        let tracks: Vec<ParsedTrack> = index
            .track_resource(&tr.release_resource_reference)
            .map(|resource| index.track(resource, 1, None, None, Vec::new()))
            .into_iter()
            .collect();
        let title: Vec<LocalizedString> = tr
            .display_title_text
            .iter()
            .map(|t| LocalizedString::new(t.text.clone()))
            .collect();
        let genre = TerritoryScope::pick_default(&tr.genre, |g| &g.scope);
        ParsedRelease {
            release_id: tr.release_reference.clone(),
            identifiers: Self::extract_identifiers(&tr.release_id),
            default_title: display_title(&tr.display_title_text, &tr.display_title, &title),
            title,
            subtitle: None,
            default_subtitle: None,
            display_artist: None,
            artists: Vec::new(),
            release_type: "TrackRelease".to_string(),
            genre: genre.map(|g| g.genre_text.clone()),
            sub_genre: genre.and_then(|g| g.sub_genre.clone()),
            track_count: tracks.len(),
            disc_count: None,
            tracks,
            videos: Vec::new(),
            images: Vec::new(),
            cover_art: None,
            release_date: None,
            original_release_date: None,
            territories: Vec::new(),
            p_line: None,
            c_line: None,
            parent_release: None,
            child_releases: Vec::new(),
            extensions: None,
            genres: tr.genre.iter().map(genre_info).collect(),
            label_name: tr
                .release_label_reference
                .first()
                .and_then(|r| index.party_name(r))
                .map(str::to_string),
            original_release_dates: Vec::new(),
            parental_warning: None,
            parental_warnings: Vec::new(),
            release_dates: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Territorial values: defaults and full lists
// ---------------------------------------------------------------------------

/// Reference -> (item SequenceNumber, enclosing nested groups) for a 3.8.x ResourceGroup
/// tree; the top-level group is not part of the path (as for 4.x).
fn group_layout<'a>(
    group: &'a ResourceGroup,
    path: &mut Vec<TrackGroup>,
    out: &mut IndexMap<&'a str, (Option<i32>, Vec<TrackGroup>)>,
) {
    for sub in &group.resource_groups {
        path.push(TrackGroup {
            group_type: sub.resource_group_type.clone(),
            sequence_number: sub.sequence_number,
        });
        group_layout(sub, path, out);
        path.pop();
    }
    for item in &group.content_items {
        out.entry(item.release_resource_reference.as_str())
            .or_insert_with(|| (item.sequence_number, path.clone()));
    }
}

fn genre_info(g: &Genre) -> GenreInfo {
    GenreInfo {
        genre: g.genre_text.clone(),
        sub_genre: g.sub_genre.clone(),
        scope: g.scope.clone(),
    }
}

fn date_info(e: &ReleaseEvent) -> Option<DateInfo> {
    Some(DateInfo {
        date: e.date.clone()?,
        is_approximate: e.is_approximate,
        scope: e.scope.clone(),
    })
}

/// Default (see [`TerritoryScope::pick_default`]) and full-list values of a release.
/// `label` is the 4.x label resolved from `ReleaseLabelReference`; a 3.8.x `LabelName`
/// takes precedence when present.
fn fill_release_details(r: &mut ParsedRelease, release: &Release, label: Option<String>) {
    let genre = TerritoryScope::pick_default(&release.genre, |g| &g.scope);
    r.genre = genre.map(|g| g.genre_text.clone());
    r.sub_genre = genre.and_then(|g| g.sub_genre.clone());
    r.genres = release.genre.iter().map(genre_info).collect();
    r.parental_warning = TerritoryScope::pick_default(&release.parental_warnings, |w| &w.scope)
        .map(|w| w.value.clone());
    r.parental_warnings = release
        .parental_warnings
        .iter()
        .map(|w| w.value.clone())
        .collect();
    r.p_line = TerritoryScope::pick_default(&release.p_line, |c| &c.scope).cloned();
    r.c_line = TerritoryScope::pick_default(&release.c_line, |c| &c.scope).cloned();
    r.release_date = TerritoryScope::pick_default(&release.release_date, |e| &e.scope)
        .and_then(|e| e.event_date);
    r.original_release_date =
        TerritoryScope::pick_default(&release.original_release_date, |e| &e.scope)
            .and_then(|e| e.event_date);
    r.release_dates = release.release_date.iter().filter_map(date_info).collect();
    r.original_release_dates = release
        .original_release_date
        .iter()
        .filter_map(date_info)
        .collect();
    r.label_name = TerritoryScope::pick_default(&release.label_name, |l| &l.scope)
        .map(|l| l.name.clone())
        .or(label);
}

/// Default values of a resource, shared by `ParsedResource` and `ParsedTrack`.
#[derive(Default)]
struct ResourceDetails {
    genre: Option<String>,
    sub_genre: Option<String>,
    parental_warning: Option<String>,
    p_line: Option<Copyright>,
    c_line: Option<Copyright>,
    label_name: Option<String>,
}

impl ResourceDetails {
    fn of(resource: &Resource) -> Self {
        let genre = TerritoryScope::pick_default(&resource.genre, |g| &g.scope);
        Self {
            genre: genre.map(|g| g.genre_text.clone()),
            sub_genre: genre.and_then(|g| g.sub_genre.clone()),
            parental_warning: TerritoryScope::pick_default(&resource.parental_warnings, |w| {
                &w.scope
            })
            .map(|w| w.value.clone()),
            p_line: TerritoryScope::pick_default(&resource.p_line, |c| &c.scope).cloned(),
            c_line: TerritoryScope::pick_default(&resource.c_line, |c| &c.scope).cloned(),
            label_name: TerritoryScope::pick_default(&resource.label_name, |l| &l.scope)
                .map(|l| l.name.clone()),
        }
    }
}
