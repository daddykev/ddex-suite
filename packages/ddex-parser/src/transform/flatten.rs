// core/src/transform/flatten.rs
//! Graph to flat model transformation

use crate::error::ParseError;

type Result<T> = std::result::Result<T, ParseError>;
use ddex_core::models::common::{Identifier, LocalizedString};
use ddex_core::models::flat::{
    ArtistInfo, DealValidity, DistributionComplexity, FlattenedMessage, MessageStats, Organization,
    ParsedDeal, ParsedRelease, ParsedResource, ParsedTrack, PriceTier, PriceType, ProprietaryId,
    ReleaseIdentifiers, TechnicalInfo, TerritoryComplexity, TerritoryInfo,
};
use ddex_core::models::graph::{
    Artist, Deal, DealTerms, ERNMessage, Party, Release, ReleaseResourceReference, Resource,
    ResourceType,
};
use indexmap::IndexMap;
use std::collections::HashMap;

pub struct Flattener;

impl Flattener {
    pub fn flatten(graph: ERNMessage) -> Result<FlattenedMessage> {
        let releases = Self::flatten_releases(&graph.releases, &graph.resources)?;
        let resources = Self::flatten_resources(&graph.resources)?;
        let deals = Self::flatten_deals(&graph.deals)?;
        let parties = Self::flatten_parties(&graph.parties)?;

        let stats = MessageStats {
            release_count: graph.releases.len(),
            track_count: 0, // Set to 0 if no tracks
            deal_count: graph.deals.len(),
            total_duration: 0, // Set to 0 if no duration
        };

        Ok(FlattenedMessage {
            message_id: graph.message_header.message_id.clone(),
            message_type: format!("{:?}", graph.message_header.message_type),
            message_date: graph.message_header.message_created_date_time,
            sender: Organization {
                name: Self::get_primary_name(&graph.message_header.message_sender.party_name, "MessageSender/PartyName")?,
                id: Self::get_primary_id(&graph.message_header.message_sender.party_id, "MessageSender/PartyId")?,
                extensions: None,
            },
            recipient: Organization {
                name: Self::get_primary_name(&graph.message_header.message_recipient.party_name, "MessageRecipient/PartyName")?,
                id: Self::get_primary_id(&graph.message_header.message_recipient.party_id, "MessageRecipient/PartyId")?,
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
        })
    }

    fn flatten_releases(releases: &[Release], resources: &[Resource]) -> Result<Vec<ParsedRelease>> {
        releases
            .iter()
            .map(|release| Ok(ParsedRelease {
                release_id: release.release_reference.clone(),
                identifiers: Self::extract_identifiers(&release.release_id),
                title: release.release_title.clone(),
                default_title: Self::get_primary_title(&release.release_title, "Release/Title/TitleText")?,
                subtitle: release.release_subtitle.clone(),
                default_subtitle: release
                    .release_subtitle
                    .as_ref()
                    .map(|s| Self::get_primary_title_optional(s))
                    .flatten(),
                display_artist: Self::format_display_artist(&release.display_artist)?,
                artists: Self::extract_artists(&release.display_artist)?,
                release_type: release
                    .release_type
                    .as_ref()
                    .map(|t| format!("{:?}", t))
                    .ok_or_else(|| ParseError::MissingField("Release/ReleaseType".to_string()))?,
                genre: release.genre.first().map(|g| g.genre_text.clone()),
                sub_genre: release.genre.first().and_then(|g| g.sub_genre.clone()),
                tracks: Self::build_tracks(&release.release_resource_reference_list, resources)?,
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
            }))
            .collect()
    }

    fn flatten_resources(resources: &[Resource]) -> Result<IndexMap<String, ParsedResource>> {
        resources
            .iter()
            .map(|resource| {
                let parsed = ParsedResource {
                    resource_id: resource.resource_reference.clone(),
                    resource_type: format!("{:?}", resource.resource_type),
                    // ReferenceTitle is not present on all resource types (e.g., Image in ERN 3.8.2).
                    // Use optional getter and fallback to resource reference if not present.
                    title: Self::get_primary_title_optional(&resource.reference_title)
                        .filter(|t| !t.is_empty())
                        .unwrap_or_else(|| resource.resource_reference.clone()),
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
                };
                Ok((resource.resource_reference.clone(), parsed))
            })
            .collect()
    }

    fn flatten_deals(deals: &[Deal]) -> Result<Vec<ParsedDeal>> {
        deals
            .iter()
            .map(|deal| Ok(ParsedDeal {
                deal_id: deal
                    .deal_reference
                    .clone()
                    .ok_or_else(|| ParseError::MissingField("Deal/DealReference".to_string()))?,
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
            }))
            .collect()
    }

    fn flatten_parties(parties: &[Party]) -> Result<IndexMap<String, Party>> {
        parties
            .iter()
            .map(|party| {
                let id = Self::get_primary_id(&party.party_id, "Party/PartyId")?;
                Ok((id, party.clone()))
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

    fn get_primary_title(titles: &[LocalizedString], field_path: &str) -> Result<String> {
        titles
            .first()
            .map(|t| t.text.clone())
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
            .map(|artist| Ok(ArtistInfo {
                name: Self::get_primary_name(&artist.display_artist_name, "Artist/DisplayArtistName")?,
                role: artist
                    .artist_role
                    .first()
                    .cloned()
                    .ok_or_else(|| ParseError::MissingField("Artist/ArtistRole".to_string()))?,
                party_id: artist.party_reference.clone(),
            }))
            .collect()
    }

    fn build_tracks(refs: &[ReleaseResourceReference], resources: &[Resource]) -> Result<Vec<ParsedTrack>> {
        refs.iter()
            .enumerate()
            .filter_map(|(idx, rref)| {
                let resource = resources
                    .iter()
                    .find(|r| r.resource_reference == rref.resource_reference);

                // Skip non-audio resources (images, text, etc.) - they're not tracks
                if let Some(r) = resource {
                    if !matches!(r.resource_type, ResourceType::SoundRecording | ResourceType::Video) {
                        return None;
                    }
                }

                // For tracks, get title (fall back to resource reference if empty/missing)
                let title = resource
                    .and_then(|r| Self::get_primary_title_optional(&r.reference_title))
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| rref.resource_reference.clone());

                // Duration is optional for tracks (some valid DDEX files don't include it)
                let duration = resource.and_then(|r| r.duration);
                let duration_formatted = duration
                    .map(ParsedTrack::format_duration)
                    .unwrap_or_else(|| "0:00".to_string());

                Some(Ok(ParsedTrack {
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
                    track_number: rref.track_number,
                    disc_number: rref.disc_number,
                    side: rref.side.clone(),
                    title,
                    subtitle: None,
                    display_artist: String::new(),
                    artists: Vec::new(),
                    duration: duration.unwrap_or_default(),
                    duration_formatted,
                    file_format: None,
                    bitrate: None,
                    sample_rate: None,
                    is_hidden: rref.is_hidden,
                    is_bonus: rref.is_bonus,
                    is_explicit: false,
                    is_instrumental: false,
                }))
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
