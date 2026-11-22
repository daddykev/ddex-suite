// core/src/models/graph/release.rs
//! Release types

use super::{Artist, DisplayText, ResourceGroup};
use crate::models::{
    common::{Copyright, Identifier, LocalizedString, TerritoryScope},
    AttributeMap, Comment, Extensions,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub release_reference: String,
    pub release_id: Vec<Identifier>,
    pub release_title: Vec<LocalizedString>,
    pub release_subtitle: Option<Vec<LocalizedString>>,
    pub release_type: Option<ReleaseType>,
    pub genre: Vec<Genre>,
    pub release_resource_reference_list: Vec<ReleaseResourceReference>,
    pub display_artist: Vec<Artist>,
    pub party_list: Vec<ReleaseParty>,
    pub release_date: Vec<ReleaseEvent>,
    pub territory_code: Vec<String>,
    pub excluded_territory_code: Vec<String>,
    /// ERN 4.x `DisplayTitleText` entries (empty for 3.8.x).
    #[serde(default)]
    pub display_title_text: Vec<DisplayText>,
    /// ERN 4.x `DisplayTitle/TitleText` entries (empty for 3.8.x).
    #[serde(default)]
    pub display_title: Vec<DisplayText>,
    /// ERN 4.x `DisplayArtistName` entries (empty for 3.8.x).
    #[serde(default)]
    pub display_artist_name: Vec<DisplayText>,
    /// ERN 4.x `ResourceGroup` tree (`None` for 3.8.x).
    #[serde(default)]
    pub resource_group: Option<ResourceGroup>,
    /// ERN 4.x `ReleaseType/@UserDefinedValue` (with `ReleaseType` `UserDefined`).
    #[serde(default)]
    pub release_type_user_defined_value: Option<String>,
    /// ERN 4.x `ParentalWarningType` values, in document order (empty for 3.8.x).
    #[serde(default)]
    pub parental_warning_type: Vec<String>,
    /// ERN 4.x `ReleaseLabelReference` values (party references), in document order.
    #[serde(default)]
    pub release_label_reference: Vec<String>,
    /// `ParentalWarningType` entries with their attributes / territory scope, in document
    /// order (`parental_warning_type` keeps the plain values).
    #[serde(default)]
    pub parental_warnings: Vec<ParentalWarning>,
    /// `PLine` entries: 4.x `Release/PLine`; 3.8.x `Release/PLine`, then each
    /// `ReleaseDetailsByTerritory/PLine`.
    #[serde(default)]
    pub p_line: Vec<Copyright>,
    /// `CLine` entries, same sources as `p_line`.
    #[serde(default)]
    pub c_line: Vec<Copyright>,
    /// Original release dates: 4.x `OriginalReleaseDate`; 3.8.x `GlobalOriginalReleaseDate`,
    /// then each `ReleaseDetailsByTerritory/OriginalReleaseDate`. (`release_date` holds the
    /// release dates the same way.)
    #[serde(default)]
    pub original_release_date: Vec<ReleaseEvent>,
    /// 3.8.x `ReleaseDetailsByTerritory/LabelName` entries (4.x names labels through
    /// `release_label_reference`).
    #[serde(default)]
    pub label_name: Vec<LabelName>,
    /// All XML attributes (standard and custom)
    pub attributes: Option<AttributeMap>,
    /// Extensions for release
    pub extensions: Option<Extensions>,
    /// Comments associated with release
    pub comments: Option<Vec<Comment>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseType {
    Album,
    Single,
    EP,
    Compilation,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Genre {
    pub genre_text: String,
    pub sub_genre: Option<String>,
    /// Where this genre applies (see [`TerritoryScope`]).
    #[serde(default)]
    pub scope: TerritoryScope,
    /// All XML attributes (standard and custom)
    pub attributes: Option<AttributeMap>,
    /// Extensions for genre
    pub extensions: Option<Extensions>,
    /// Comments associated with genre
    pub comments: Option<Vec<Comment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseResourceReference {
    pub resource_reference: String,
    pub sequence_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub track_number: Option<i32>,
    pub side: Option<String>,
    pub is_hidden: bool,
    pub is_bonus: bool,
    /// Extensions for resource reference
    pub extensions: Option<Extensions>,
    /// Comments associated with resource reference
    pub comments: Option<Vec<Comment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseParty {
    pub party_reference: String,
    pub role: Vec<String>,
    /// Extensions for release party
    pub extensions: Option<Extensions>,
    /// Comments associated with release party
    pub comments: Option<Vec<Comment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseEvent {
    /// The element name: `ReleaseDate`, `OriginalReleaseDate`, `GlobalReleaseDate`, ...
    pub release_event_type: String,
    /// The date at midnight UTC, only when the XML gives a full `YYYY-MM-DD` date (a partial
    /// `YYYY` / `YYYY-MM` date is never completed; see `date`).
    pub event_date: Option<DateTime<Utc>>,
    /// The single territory the date applies to, when exactly one is given.
    pub territory: Option<String>,
    /// The date as written (`ddex_IsoDate`: `YYYY`, `YYYY-MM` or `YYYY-MM-DD`).
    #[serde(default)]
    pub date: Option<String>,
    /// `@IsApproximate`.
    #[serde(default)]
    pub is_approximate: Option<bool>,
    /// Where this date applies (see [`TerritoryScope`]).
    #[serde(default)]
    pub scope: TerritoryScope,
    /// Extensions for release event
    pub extensions: Option<Extensions>,
    /// Comments associated with release event
    pub comments: Option<Vec<Comment>>,
}

/// A `ParentalWarningType` value with its attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParentalWarning {
    pub value: String,
    /// `@Namespace` (4.x, or 3.8.x for a `UserDefined` value).
    #[serde(default)]
    pub namespace: Option<String>,
    /// `@UserDefinedValue`.
    #[serde(default)]
    pub user_defined_value: Option<String>,
    /// Where this warning applies (see [`TerritoryScope`]).
    #[serde(default)]
    pub scope: TerritoryScope,
}

/// An ERN 3.8.x `LabelName`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelName {
    pub name: String,
    /// `@LabelNameType` (e.g. `DisplayLabelName`).
    #[serde(default)]
    pub label_name_type: Option<String>,
    /// Where this name applies (see [`TerritoryScope`]).
    #[serde(default)]
    pub scope: TerritoryScope,
}
