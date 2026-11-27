//! Main builder implementation

pub use super::preflight::PreflightLevel;
use crate::generator::{xml_writer::XmlWriter, ASTGenerator};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Build request for generating DDEX messages
///
/// Contains all the information needed to generate a complete DDEX XML message,
/// including message header, releases, deals, and any extensions.
/// The structure is designed for deterministic output with stable ordering.
///
/// All request structs implement [`Default`], so callers can set the fields they need
/// and fill in the rest with `..Default::default()`. Fields marked "ERN 4.x" are only
/// written for 4.2/4.3 targets; when the 4.x schema requires one of them and it is
/// missing, the build fails with an error naming the field (nothing is filled in).
///
/// # Example
/// ```
/// use ddex_builder::builder::{BuildRequest, MessageHeaderRequest, PartyRequest, LocalizedStringRequest};
///
/// let request = BuildRequest {
///     header: MessageHeaderRequest {
///         message_id: "MSG123".to_string(),
///         message_sender: PartyRequest {
///             party_name: vec![LocalizedStringRequest {
///                 text: "My Label".to_string(),
///                 language_code: Some("en".to_string()),
///             }],
///             party_id: Some("PADPIDA2014120301K".to_string()),
///             party_reference: None,
///         },
///         message_recipient: PartyRequest {
///         party_name: vec![LocalizedStringRequest {
///             text: "Example DSP".to_string(),
///             language_code: None,
///         }],
///         party_id: Some("PADPIDA2014120302K".to_string()),
///         party_reference: None,
///     },
///         message_control_type: Some("LiveMessage".to_string()),
///         message_created_date_time: "2024-12-15T10:00:00Z".to_string(),
///     },
///     version: "4.3".to_string(),
///     profile: None,
///     releases: vec![/* ReleaseRequest items */],
///     deals: vec![/* DealRequest items */],
///     extensions: None,
///     language_and_script_code: Some("en".to_string()),
///     avs_version_id: Some("12".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildRequest {
    /// Message header containing sender, recipient, and message metadata
    pub header: MessageHeaderRequest,

    /// ERN version (e.g., "3.8.2", "4.2", "4.3")
    pub version: String,

    /// DDEX profile identifier (optional)
    pub profile: Option<String>,

    /// List of releases in this message
    /// Uses Vec to maintain order while allowing duplicates if needed
    pub releases: Vec<ReleaseRequest>,

    /// List of commercial deals
    pub deals: Vec<DealRequest>,

    /// Custom extensions (uses IndexMap for deterministic ordering)
    pub extensions: Option<IndexMap<String, String>>,

    /// ERN 4.x: `NewReleaseMessage/@LanguageAndScriptCode` (required by the 4.x schema),
    /// the default language of the message's text, e.g. `"en"`.
    #[serde(default)]
    pub language_and_script_code: Option<String>,

    /// ERN 4.3: `NewReleaseMessage/@AvsVersionId` (required by the 4.3 schema), the
    /// version of the DDEX allowed-value sets the message's coded values come from.
    #[serde(default)]
    pub avs_version_id: Option<String>,
}

/// Message header information for DDEX messages
///
/// Contains metadata about the message including sender, recipient,
/// message type, and creation timestamp.
///
/// # Example
/// ```
/// use ddex_builder::builder::{MessageHeaderRequest, PartyRequest, LocalizedStringRequest};
///
/// let header = MessageHeaderRequest {
///     message_id: "MSG_20241215_001".to_string(),
///     message_sender: PartyRequest {
///         party_name: vec![LocalizedStringRequest {
///             text: "Warner Music Group".to_string(),
///             language_code: Some("en".to_string()),
///         }],
///         party_id: Some("PADPIDA2014120301K".to_string()),
///         party_reference: None,
///     },
///     message_recipient: PartyRequest {
///         party_name: vec![LocalizedStringRequest {
///             text: "Example DSP".to_string(),
///             language_code: None,
///         }],
///         party_id: Some("PADPIDA2014120302K".to_string()),
///         party_reference: None,
///     },
///     message_control_type: Some("LiveMessage".to_string()),
///     message_created_date_time: "2024-12-15T10:00:00Z".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageHeaderRequest {
    /// `MessageId` (required; the builder never generates it). An empty or missing value
    /// fails the build.
    #[serde(default, deserialize_with = "string_or_null")]
    pub message_id: String,
    /// Party sending the message
    pub message_sender: PartyRequest,
    /// Party receiving the message
    pub message_recipient: PartyRequest,
    /// `MessageControlType` (AVS: `LiveMessage` or `TestMessage`)
    pub message_control_type: Option<String>,
    /// `MessageCreatedDateTime`, an xs:dateTime such as `2024-12-15T10:00:00Z` (required; the
    /// builder reads no clock). An empty or missing value fails the build.
    #[serde(default, deserialize_with = "string_or_null")]
    pub message_created_date_time: String,
}

/// A required string that may arrive as `null` (from JSON, Node.js or Python): `null` becomes
/// `""`, which the build reports as missing, naming the field.
fn string_or_null<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

/// Party information request
///
/// Represents a party (sender, recipient, or other entity) in the DDEX message.
/// Supports multiple localized names and various identifier types.
///
/// # Example
/// ```
/// use ddex_builder::builder::{PartyRequest, LocalizedStringRequest};
///
/// let party = PartyRequest {
///     party_name: vec![
///         LocalizedStringRequest {
///             text: "Universal Music Group".to_string(),
///             language_code: Some("en".to_string()),
///         },
///         LocalizedStringRequest {
///             text: "Universal Music Group".to_string(),
///             language_code: Some("es".to_string()),
///         },
///     ],
///     party_id: Some("PADPIDA2014120301K".to_string()), // DPID
///     party_reference: Some("PARTY_REF_001".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartyRequest {
    /// Party names in multiple languages
    pub party_name: Vec<LocalizedStringRequest>,
    /// Party identifier (e.g., DPID, ISNI, Proprietary ID)
    pub party_id: Option<String>,
    /// Reference identifier for this party within the message
    pub party_reference: Option<String>,
}

/// Localized string with language code
///
/// Represents text content with an optional language identifier.
/// Used for titles, names, and other textual content that may need
/// to be provided in multiple languages.
///
/// # Example
/// ```
/// use ddex_builder::builder::LocalizedStringRequest;
///
/// let english_title = LocalizedStringRequest {
///     text: "My Song Title".to_string(),
///     language_code: Some("en".to_string()),
/// };
///
/// let spanish_title = LocalizedStringRequest {
///     text: "Mi Título de Canción".to_string(),
///     language_code: Some("es".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalizedStringRequest {
    /// Text content
    pub text: String,
    /// ISO 639-1 language code (e.g., "en", "es", "fr")
    pub language_code: Option<String>,
}

/// A coded value from a DDEX allowed-value set (AVS), e.g. a `ReleaseType`.
///
/// `value` is the AVS term (`"Album"`, `"MusicalWorkSoundRecording"`, ...). For
/// `"UserDefined"`, put the actual value in `user_defined_value` (and optionally the
/// party that defines it in `namespace`); they become the `UserDefinedValue` and
/// `Namespace` attributes. In JSON (and from Python/Node) either a plain string or an
/// object `{"value": ..., "namespace": ..., "user_defined_value": ...}` is accepted.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AvsValueRequest {
    /// The AVS term
    pub value: String,
    /// `@Namespace`: the party that defined a user-defined value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// `@UserDefinedValue`, used with `value: "UserDefined"`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_defined_value: Option<String>,
}

impl<'de> Deserialize<'de> for AvsValueRequest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Plain(String),
            Full {
                value: String,
                #[serde(default)]
                namespace: Option<String>,
                #[serde(default)]
                user_defined_value: Option<String>,
            },
        }
        Ok(match Repr::deserialize(deserializer)? {
            Repr::Plain(value) => AvsValueRequest {
                value,
                ..Default::default()
            },
            Repr::Full {
                value,
                namespace,
                user_defined_value,
            } => AvsValueRequest {
                value,
                namespace,
                user_defined_value,
            },
        })
    }
}

impl From<&str> for AvsValueRequest {
    fn from(value: &str) -> Self {
        AvsValueRequest {
            value: value.to_string(),
            ..Default::default()
        }
    }
}

impl From<String> for AvsValueRequest {
    fn from(value: String) -> Self {
        AvsValueRequest {
            value,
            ..Default::default()
        }
    }
}

/// A proprietary identifier (`ProprietaryId` with its required `@Namespace`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProprietaryIdRequest {
    /// `@Namespace`: the party that issued the identifier (e.g. a DPID)
    pub namespace: String,
    /// The identifier
    pub value: String,
}

/// ERN 4.x: the identifiers of a release, written as one `ReleaseId` element.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseIdRequest {
    /// `GRid`
    #[serde(default)]
    pub grid: Option<String>,
    /// `ICPN` (UPC/EAN)
    #[serde(default)]
    pub icpn: Option<String>,
    /// `ProprietaryId`s
    #[serde(default)]
    pub proprietary_ids: Vec<ProprietaryIdRequest>,
}

impl ReleaseIdRequest {
    /// True when no identifier is set.
    pub fn is_empty(&self) -> bool {
        self.grid.is_none() && self.icpn.is_none() && self.proprietary_ids.is_empty()
    }
}

/// ERN 4.x: one node of a release's `ResourceGroup` tree.
///
/// The top-level node is `Release/ResourceGroup` (it has no type). Nested groups are
/// written as `ResourceGroup` children and need `resource_group_type` (the 4.x schema
/// requires `@ResourceGroupType` on them, e.g. `"Component"` for a disc).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceGroupRequest {
    /// `@ResourceGroupType` (required on nested groups, not allowed on the top level)
    #[serde(default)]
    pub resource_group_type: Option<String>,
    /// `SequenceNumber`
    #[serde(default)]
    pub sequence_number: Option<u32>,
    /// Nested groups (written before the content items, as the schema requires)
    #[serde(default)]
    pub resource_groups: Vec<ResourceGroupRequest>,
    /// `ResourceGroupContentItem`s
    #[serde(default)]
    pub content_items: Vec<ResourceGroupContentItemRequest>,
}

/// ERN 4.x: a `ResourceGroupContentItem`, pointing at one of the release's tracks.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceGroupContentItemRequest {
    /// `SequenceNumber`
    #[serde(default)]
    pub sequence_number: Option<u32>,
    /// `ReleaseResourceReference`: the `resource_reference` of a track in this release
    pub resource_reference: String,
}

/// ERN 4.x: a `TrackRelease` for one track (a release of exactly that resource).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackReleaseRequest {
    /// `ReleaseReference` (pattern `R[\d\-_a-zA-Z]+`)
    pub release_reference: String,
    /// `ReleaseId` (required, at least one identifier)
    #[serde(default)]
    pub release_ids: ReleaseIdRequest,
    /// `DisplayTitleText` / `DisplayTitle` (optional)
    #[serde(default)]
    pub title: Option<String>,
    /// `Genre/GenreText` (required)
    #[serde(default)]
    pub genre: Option<String>,
    /// `Genre/SubGenre`
    #[serde(default)]
    pub sub_genre: Option<String>,
    /// Label name, written as a `PartyList` party referenced by `ReleaseLabelReference`
    /// (required)
    #[serde(default)]
    pub label: Option<String>,
}

/// The kind of resource a [`TrackRequest`] describes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    /// `SoundRecording` (the default)
    #[default]
    SoundRecording,
    /// `Video` (ERN 4.x only)
    Video,
}

/// Release information request
///
/// Represents a music release (album, single, EP, etc.) with all its metadata,
/// tracks, and identifiers. This is the core content of most DDEX messages.
///
/// # Example
/// ```
/// use ddex_builder::builder::{ReleaseRequest, LocalizedStringRequest, TrackRequest};
///
/// let release = ReleaseRequest {
///     release_id: Some("GRid:A1-12345678901234567890123456789012".to_string()),
///     release_reference: Some("REL_001".to_string()),
///     title: vec![LocalizedStringRequest {
///         text: "Greatest Hits".to_string(),
///         language_code: Some("en".to_string()),
///     }],
///     artist: Some("The Beatles".to_string()),
///     label: Some("Apple Records".to_string()),
///     release_date: Some("2024-01-15".to_string()),
///     upc: Some("123456789012".to_string()),
///     tracks: vec![
///         TrackRequest {
///             track_id: "T001".to_string(),
///             resource_reference: Some("RES_001".to_string()),
///             isrc: Some("GBUM71505078".to_string()),
///             title: "Here Comes The Sun".to_string(),
///             duration: "PT3M5S".to_string(),
///             artist: Some("The Beatles".to_string()),
///             ..Default::default()
///         }
///     ],
///     resource_references: Some(vec!["RES_001".to_string()]),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReleaseRequest {
    /// Release identifier, written as `ReleaseId/GRid` (unless `release_ids` is set for
    /// an ERN 4.x target); `None` writes no GRid
    pub release_id: Option<String>,
    /// Internal reference for this release within the message
    pub release_reference: Option<String>,
    /// Release titles in multiple languages
    pub title: Vec<LocalizedStringRequest>,
    /// Main artist name for the release (ERN 4.x: required, `DisplayArtistName`, and a
    /// `DisplayArtist` party with role `MainArtist` unless `artist_role` says otherwise;
    /// ERN 3.8.x: optional, `None` writes no display artist)
    pub artist: Option<String>,
    /// Record label name (ERN 4.x: required, written as a `PartyList` party referenced
    /// by `ReleaseLabelReference`)
    pub label: Option<String>,
    /// Release date in YYYY-MM-DD format
    pub release_date: Option<String>,
    /// Universal Product Code for the release (12-digit barcode)
    pub upc: Option<String>,
    /// List of tracks/resources in this release
    pub tracks: Vec<TrackRequest>,
    /// References to resources for linking purposes
    pub resource_references: Option<Vec<String>>,

    /// ERN 4.x: `ReleaseType` (required), e.g. `"Album"`
    #[serde(default)]
    pub release_type: Option<AvsValueRequest>,
    /// ERN 4.x: `Genre/GenreText` (required)
    #[serde(default)]
    pub genre: Option<String>,
    /// ERN 4.x: `Genre/SubGenre`
    #[serde(default)]
    pub sub_genre: Option<String>,
    /// ERN 4.x: `ParentalWarningType` (required), e.g. `"NotExplicit"`; pass DDEX's own
    /// `"NoAdviceAvailable"` / `"Unknown"` explicitly when that is what you mean
    #[serde(default)]
    pub parental_warning_type: Option<String>,
    /// ERN 4.x: `DisplayArtistRole` of `artist` (default `MainArtist`)
    #[serde(default)]
    pub artist_role: Option<AvsValueRequest>,
    /// ERN 4.x: all identifiers of the release, written as the one `ReleaseId`. When
    /// unset, `ReleaseId` holds `release_id` as `GRid` and `upc` as `ICPN`.
    #[serde(default)]
    pub release_ids: Option<ReleaseIdRequest>,
    /// ERN 4.x: the `ResourceGroup` tree. When unset, tracks with a `disc_number` are
    /// grouped into one `Component` group per disc, otherwise all tracks go into a
    /// single group. Cannot be combined with per-track `disc_number`/`sequence_number`.
    #[serde(default)]
    pub resource_group: Option<ResourceGroupRequest>,
}

/// Track information request
///
/// Represents a single track/sound recording within a release.
/// Contains all the metadata needed for proper DDEX representation.
///
/// # Example
/// ```
/// use ddex_builder::builder::TrackRequest;
///
/// let track = TrackRequest {
///     track_id: "T001".to_string(),
///     resource_reference: Some("A12345".to_string()),
///     isrc: Some("USUM71504847".to_string()),
///     title: "Bohemian Rhapsody".to_string(),
///     duration: "PT5M55S".to_string(), // 5 minutes 55 seconds
///     artist: Some("Queen".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackRequest {
    /// Unique identifier for this track within the message
    pub track_id: String,
    /// Reference to the sound recording resource
    pub resource_reference: Option<String>,
    /// International Standard Recording Code (12-character alphanumeric); `None` writes no
    /// ISRC (the resource then needs `proprietary_ids`)
    pub isrc: Option<String>,
    /// Track title
    pub title: String,
    /// Duration in ISO 8601 format (e.g., "PT3M45S" for 3 minutes 45 seconds)
    pub duration: String,
    /// Track artist name (may differ from release artist for compilations). ERN 4.x:
    /// required; ERN 3.8.x: optional, `None` writes no display artist
    pub artist: Option<String>,

    /// ERN 4.x: `SoundRecording` (default) or `Video`
    #[serde(default)]
    pub resource_kind: ResourceKind,
    /// ERN 4.x: the resource's `Type` (required), e.g. `"MusicalWorkSoundRecording"`
    /// or `"MusicalWorkVideoScreen"`
    #[serde(default)]
    pub resource_type: Option<AvsValueRequest>,
    /// ERN 4.x: `ParentalWarningType` (required)
    #[serde(default)]
    pub parental_warning_type: Option<String>,
    /// ERN 4.x: `DisplayArtistRole` of `artist` (default `MainArtist`)
    #[serde(default)]
    pub artist_role: Option<AvsValueRequest>,
    /// ERN 4.x: disc number; tracks with one are grouped into `Component` groups
    #[serde(default)]
    pub disc_number: Option<u32>,
    /// ERN 4.x: `ResourceGroupContentItem/SequenceNumber`
    #[serde(default)]
    pub sequence_number: Option<u32>,
    /// ERN 4.x: `ProprietaryId`s written next to the ISRC in `ResourceId`
    #[serde(default)]
    pub proprietary_ids: Vec<ProprietaryIdRequest>,
    /// ERN 4.x: a `TrackRelease` for this track
    #[serde(default)]
    pub track_release: Option<TrackReleaseRequest>,
}

/// Commercial deal request
///
/// Represents the commercial terms and licensing information for releases.
/// Defines how and where the music can be distributed and monetized.
///
/// # Example
/// ```
/// use ddex_builder::builder::{DealRequest, DealTerms};
///
/// let deal = DealRequest {
///     deal_reference: Some("DEAL_001".to_string()),
///     deal_terms: DealTerms {
///         commercial_model_type: Some("PayAsYouGoModel".to_string()),
///         territory_code: vec!["Worldwide".to_string()],
///         start_date: Some("2024-01-01".to_string()),
///         ..Default::default()
///     },
///     release_references: vec!["REL_001".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DealRequest {
    /// Reference identifier for this deal within the message
    pub deal_reference: Option<String>,
    /// Commercial terms and licensing conditions
    pub deal_terms: DealTerms,
    /// References to releases covered by this deal
    pub release_references: Vec<String>,
}

/// Commercial deal terms
///
/// Defines the specific commercial and territorial terms of a licensing deal.
/// These terms control how the music can be distributed and monetized.
///
/// # Example
/// ```
/// use ddex_builder::builder::DealTerms;
///
/// let terms = DealTerms {
///     commercial_model_type: Some("SubscriptionModel".to_string()),
///     territory_code: vec!["US".to_string(), "CA".to_string(), "MX".to_string()],
///     start_date: Some("2024-01-01".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DealTerms {
    /// Type of commercial model (e.g., "PayAsYouGoModel", "SubscriptionModel",
    /// "FreeOfChargeModel"); `None` writes no `CommercialModelType`
    pub commercial_model_type: Option<String>,
    /// Territory codes where deal applies (ISO 3166-1 alpha-2 codes or "Worldwide")
    pub territory_code: Vec<String>,
    /// Deal start date in YYYY-MM-DD format (ERN 4.x: required, `ValidityPeriod/StartDate`)
    pub start_date: Option<String>,
    /// ERN 4.x: deal end date in YYYY-MM-DD format (`ValidityPeriod/EndDate`)
    #[serde(default)]
    pub end_date: Option<String>,
    /// ERN 4.x: `UseType`s, e.g. `"Stream"`, `"PermanentDownload"`
    #[serde(default)]
    pub use_types: Vec<String>,
}

impl DealTerms {
    /// True when no term is set (ERN 3.8.x then writes the `Deal` without `DealTerms`).
    pub(crate) fn is_empty(&self) -> bool {
        self.commercial_model_type.is_none()
            && self.territory_code.is_empty()
            && self.use_types.is_empty()
            && self.start_date.is_none()
            && self.end_date.is_none()
    }
}

/// Build options: the one options type of [`DDEXBuilder`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOptions {
    /// How [`DDEXBuilder::build`] writes its output (DB-C14N/1.0 by default)
    pub output: super::output::OutputConfig,

    /// Algorithm used by [`DDEXBuilder::canonicalize`]. It does not change what
    /// [`DDEXBuilder::build`] writes (that is [`BuildOptions::output`]).
    pub canonicalization: super::CanonicalizationAlgorithm,

    /// Return [`super::BuildStatistics`] from [`DDEXBuilder::build_with_statistics`]
    pub collect_statistics: bool,

    /// Validation level
    pub preflight_level: super::preflight::PreflightLevel,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            output: super::output::OutputConfig::default(),
            canonicalization: super::CanonicalizationAlgorithm::default(),
            collect_statistics: false,
            preflight_level: super::preflight::PreflightLevel::Warn,
        }
    }
}

/// Build result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    /// Generated XML
    pub xml: String,

    /// Warnings
    pub warnings: Vec<BuildWarning>,

    /// Errors (if any)
    pub errors: Vec<super::error::BuildError>,

    /// Statistics
    pub statistics: BuildStatistics,

    /// Canonical hash (if deterministic)
    pub canonical_hash: Option<String>,

    /// Generator banner, naming the builder and DB-C14N versions (with
    /// `OutputConfig::emit_generator_banner`; it is not written into the XML)
    pub generator_banner: Option<String>,
}

/// Result of [`DDEXBuilder::build_with_statistics`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsBuildResult {
    /// The build, as [`DDEXBuilder::build`] returns it
    pub build: BuildResult,
    /// Counts from the output (with `BuildOptions::collect_statistics`)
    pub statistics: Option<super::BuildStatistics>,
}

/// Build warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildWarning {
    /// Warning code for programmatic handling
    pub code: String,
    /// Human-readable warning message
    pub message: String,
    /// Location in the structure where warning occurred
    pub location: Option<String>,
}

/// Build statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStatistics {
    /// Number of releases built
    pub releases: usize,
    /// Total number of tracks processed
    pub tracks: usize,
    /// Number of deals processed
    pub deals: usize,
    /// Time taken to generate in milliseconds
    pub generation_time_ms: u64,
    /// Size of generated XML in bytes
    pub xml_size_bytes: usize,
}

impl Default for BuildStatistics {
    fn default() -> Self {
        Self {
            releases: 0,
            tracks: 0,
            deals: 0,
            generation_time_ms: 0,
            xml_size_bytes: 0,
        }
    }
}

/// Resolve a `BuildRequest::version` string to the ERN version the builder will target.
///
/// Accepts `"3.8.1"`, `"381"` and `"ern/381"` style spellings for 3.8.1, 3.8.2, 4.2 and 4.3.
/// Anything else is rejected so an unsupported target never produces a made-up namespace.
pub fn parse_target_version(
    version: &str,
) -> Result<ddex_core::models::versions::ERNVersion, super::error::BuildError> {
    use ddex_core::models::versions::ERNVersion;
    let compact: String = version
        .trim()
        .trim_start_matches("ern/")
        .chars()
        .filter(|c| *c != '.')
        .collect();
    match compact.as_str() {
        "381" => Ok(ERNVersion::V3_8_1),
        "382" => Ok(ERNVersion::V3_8_2),
        "42" => Ok(ERNVersion::V4_2),
        "43" => Ok(ERNVersion::V4_3),
        _ => Err(super::error::BuildError::InvalidFormat {
            field: "version".to_string(),
            message: format!(
                "unsupported ERN version '{}'; the builder can target 3.8.1, 3.8.2, 4.2 or 4.3",
                version
            ),
        }),
    }
}

/// The DDEX builder: builds ERN XML from a [`BuildRequest`], and canonicalizes, counts and
/// round-trips it.
///
/// ```rust
/// use ddex_builder::DDEXBuilder;
///
/// let builder = DDEXBuilder::new();
/// println!("Supported versions: {:?}", builder.supported_versions());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct DDEXBuilder;

impl DDEXBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self
    }

    /// The preflight checks [`DDEXBuilder::build`] runs at `level`: their errors and
    /// warnings, without building. ([`super::preflight::PreflightLevel::None`] checks
    /// nothing.)
    pub fn preflight(
        &self,
        request: &BuildRequest,
        level: super::preflight::PreflightLevel,
    ) -> Result<super::preflight::ValidationResult, super::error::BuildError> {
        super::preflight::PreflightValidator::new(super::preflight::ValidationConfig {
            level,
            profile: request.profile.clone(),
            validate_identifiers: true,
            check_required_fields: true,
            validate_references: true,
        })
        .validate(request)
    }

    /// The ERN versions the builder supports
    pub fn supported_versions(&self) -> Vec<super::DdexVersion> {
        super::versions::utils::supported_versions()
    }

    /// [`DDEXBuilder::build`], and with `options.collect_statistics` the counts of the
    /// output's elements, attributes, namespace declarations, comments and processing
    /// instructions.
    pub fn build_with_statistics(
        &self,
        request: BuildRequest,
        options: BuildOptions,
    ) -> Result<StatisticsBuildResult, super::error::BuildError> {
        let start = web_time::Instant::now();
        let collect = options.collect_statistics;
        let build = self.build(request, options)?;
        let statistics = collect.then(|| {
            let mut statistics = super::BuildStatistics {
                build_time: start.elapsed(),
                output_size_bytes: build.xml.len(),
                ..Default::default()
            };
            super::count_xml(&build.xml, &mut statistics);
            statistics
        });
        Ok(StatisticsBuildResult { build, statistics })
    }

    /// Round trip XML → Parse → Build → Parse → compare, with `parse` the parser's parse
    /// function ([`super::round_trip::RoundTripTester::test_round_trip_with`]).
    pub fn check_round_trip<F>(
        &self,
        original_xml: &str,
        parse: F,
    ) -> Result<super::round_trip::RoundTripResult, super::error::BuildError>
    where
        F: Fn(&str) -> Result<ddex_core::models::flat::ParsedERNMessage, String>,
    {
        super::round_trip::RoundTripTester::new().test_round_trip_with(original_xml, parse)
    }

    /// Canonicalize XML with `options.canonicalization`: DB-C14N/1.0 (`DbC14N`, with
    /// [`super::DB_C14N`]) or nothing (`None`, the input is returned unchanged).
    pub fn canonicalize(
        &self,
        xml: &str,
        options: &BuildOptions,
    ) -> Result<String, super::error::BuildError> {
        match options.canonicalization {
            super::CanonicalizationAlgorithm::None => Ok(xml.to_string()),
            super::CanonicalizationAlgorithm::DbC14N => super::DB_C14N::new().canonicalize(xml),
        }
    }

    /// Build DDEX XML from request
    pub fn build(
        &self,
        mut request: BuildRequest,
        options: BuildOptions,
    ) -> Result<BuildResult, super::error::BuildError> {
        // web_time: std::time on native targets; `performance.now()` on wasm32 (std panics there)
        let start = web_time::Instant::now();
        let mut warnings = Vec::new();

        // 0. Reject target versions the builder cannot emit, instead of writing a bogus namespace.
        parse_target_version(&request.version)?;

        // 0b. Reject characters that XML 1.0 does not allow (they cannot be escaped), so the
        // output is well-formed for any accepted input.
        super::xml_escape::check_request_chars(&request)?;

        // 1. Preflight checks
        let validation_result = self.preflight(&request, options.preflight_level)?;

        // Convert validation warnings to build warnings
        for warning in validation_result.warnings {
            warnings.push(BuildWarning {
                code: warning.code,
                message: warning.message,
                location: Some(warning.location),
            });
        }

        // Strict: preflight errors (and warnings) fail the build. Warn (the default): each
        // preflight error is returned in `BuildResult::warnings` with its code and message,
        // and the build goes on (before 0.6.0 they were discarded). None: nothing is checked.
        if options.preflight_level == super::preflight::PreflightLevel::Strict
            && !validation_result.passed
        {
            return Err(super::error::BuildError::ValidationFailed {
                errors: validation_result
                    .errors
                    .iter()
                    .map(|e| format!("{}: {}", e.code, e.message))
                    .collect(),
            });
        }
        for error in validation_result.errors {
            warnings.push(BuildWarning {
                code: error.code,
                message: error.message,
                location: Some(error.location),
            });
        }

        // 2. Number the references the request leaves out
        generate_references(&mut request);

        // 3. Generate AST
        let mut generator = ASTGenerator::new(request.version.clone());
        let ast = generator.generate(&request)?;

        // 4. Output configuration
        let config = options.output.clone();

        // 5. Generate XML
        let writer = XmlWriter::new(config.clone());
        let xml = writer.write(&ast)?;

        // 6. Apply canonicalization if requested
        let (final_xml, canonical_hash) = if config.canon_mode == super::output::CanonMode::DbC14n {
            let canonicalizer = super::canonical::DB_C14N::new();
            let canonical = canonicalizer.canonicalize(&xml)?;
            let hash = Some(canonicalizer.canonical_hash(&canonical)?);
            (canonical, hash)
        } else {
            (xml, None)
        };

        // 7. Generator banner, if requested
        let generator_banner = if config.emit_generator_banner {
            Some(format!(
                "Generated by DDEX Builder v{} with DB-C14N/{}",
                env!("CARGO_PKG_VERSION"),
                super::DB_C14N_VERSION
            ))
        } else {
            None
        };

        let elapsed = start.elapsed();

        Ok(BuildResult {
            xml: final_xml.clone(),
            warnings,
            errors: Vec::new(),
            statistics: BuildStatistics {
                releases: request.releases.len(),
                tracks: request.releases.iter().map(|r| r.tracks.len()).sum(),
                deals: request.deals.len(),
                generation_time_ms: elapsed.as_millis() as u64,
                xml_size_bytes: final_xml.len(),
            },
            canonical_hash,
            generator_banner,
        })
    }
}

/// Numbers the references the request leaves out, in request order, one counter per kind:
/// `R1`, `R2`, ... for releases, `A1`, `A2`, ... for resources (the `R[\d\-_a-zA-Z]+` and
/// `A[\d\-_a-zA-Z]+` patterns of every ERN version's XSD), and `D1`, `D2`, ... for deals
/// with terms (`DealReference` is an `xs:string`). A number the request already uses for a
/// reference of that kind (for example a supplied `R2`) is skipped, so generated and
/// supplied references never collide. Supplied references are never changed.
fn generate_references(request: &mut BuildRequest) {
    let mut releases = Counter::new(
        'R',
        request.releases.iter().flat_map(|r| {
            r.release_reference.as_deref().into_iter().chain(
                r.tracks
                    .iter()
                    .filter_map(|t| t.track_release.as_ref())
                    .map(|tr| tr.release_reference.as_str()),
            )
        }),
    );
    let mut resources = Counter::new(
        'A',
        request
            .releases
            .iter()
            .flat_map(|r| r.tracks.iter())
            .filter_map(|t| t.resource_reference.as_deref()),
    );
    let mut deals = Counter::new(
        'D',
        request
            .deals
            .iter()
            .filter_map(|d| d.deal_reference.as_deref()),
    );
    for release in &mut request.releases {
        if release.release_reference.is_none() {
            release.release_reference = Some(releases.next());
        }
        for track in &mut release.tracks {
            if track.resource_reference.is_none() {
                track.resource_reference = Some(resources.next());
            }
        }
    }
    for deal in &mut request.deals {
        // A deal without terms is identified only by the caller's reference; one is not
        // made up for it (the generator reports it as missing).
        if deal.deal_reference.is_none() && !deal.deal_terms.is_empty() {
            deal.deal_reference = Some(deals.next());
        }
    }
}

/// A sequential reference counter for one kind (`prefix` + 1, 2, ...) that skips the numbers
/// the request already uses.
struct Counter {
    prefix: char,
    last: u64,
    used: std::collections::BTreeSet<u64>,
}

impl Counter {
    fn new<'a>(prefix: char, supplied: impl Iterator<Item = &'a str>) -> Self {
        let used = supplied
            .filter_map(|r| r.strip_prefix(prefix))
            .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
            .filter_map(|n| n.parse().ok())
            .collect();
        Self {
            prefix,
            last: 0,
            used,
        }
    }

    fn next(&mut self) -> String {
        loop {
            self.last += 1;
            if !self.used.contains(&self.last) {
                return format!("{}{}", self.prefix, self.last);
            }
        }
    }
}
