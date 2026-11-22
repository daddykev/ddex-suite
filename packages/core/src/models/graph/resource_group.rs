// core/src/models/graph/resource_group.rs
//! ERN 4.x release structure: display texts, the `ResourceGroup` tree and `TrackRelease`.
//!
//! These types mirror the ERN 4.2/4.3 XSD faithfully so the full structure can be
//! round-tripped. Absent elements are `None` / empty; nothing is synthesized.

use crate::models::common::Identifier;
use serde::{Deserialize, Serialize};

/// A localizable display string such as `DisplayTitleText`, `DisplayTitle/TitleText`
/// or `DisplayArtistName`, with the attributes needed to pick the default one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayText {
    pub text: String,
    /// `@LanguageAndScriptCode`, verbatim (e.g. `ja-Jpan`).
    #[serde(default)]
    pub language_and_script_code: Option<String>,
    /// `@ApplicableTerritoryCode`, verbatim.
    #[serde(default)]
    pub applicable_territory_code: Option<String>,
    /// `@IsDefault`; `None` when the attribute is absent.
    #[serde(default)]
    pub is_default: Option<bool>,
}

impl DisplayText {
    /// The `IsDefault="true"` entry if there is one, otherwise the first entry.
    pub fn pick_default(texts: &[DisplayText]) -> Option<&DisplayText> {
        texts
            .iter()
            .find(|t| t.is_default == Some(true))
            .or_else(|| texts.first())
    }
}

/// `LinkedReleaseResourceReference`: a resource linked to a group or item (e.g. cover art,
/// a booklet or video screen captures).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkedReleaseResourceReference {
    pub resource_reference: String,
    /// `@LinkDescription`, verbatim.
    #[serde(default)]
    pub link_description: Option<String>,
}

/// `ResourceGroup` (the release's top-level group) or a nested `ResourceSubGroup`.
///
/// Per the XSD, nested groups precede content items, so `resource_groups` followed by
/// `content_items` is document order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceGroup {
    /// `@ResourceGroupType` (`Component`, `ReleaseComponent`, `MultiWorkPart`, `Side`, ...).
    /// Required on nested groups; absent on the release's top-level group.
    #[serde(default)]
    pub resource_group_type: Option<String>,
    #[serde(default)]
    pub sequence_number: Option<i32>,
    /// The group's own `ReleaseId` (e.g. a component release inside a boxed set).
    #[serde(default)]
    pub release_id: Vec<Identifier>,
    #[serde(default)]
    pub resource_group_release_reference: Option<String>,
    #[serde(default)]
    pub resource_groups: Vec<ResourceGroup>,
    #[serde(default)]
    pub content_items: Vec<ResourceGroupContentItem>,
    #[serde(default)]
    pub linked_release_resource_references: Vec<LinkedReleaseResourceReference>,
}

/// `ResourceGroupContentItem`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceGroupContentItem {
    #[serde(default)]
    pub sequence_number: Option<i32>,
    pub release_resource_reference: String,
    #[serde(default)]
    pub linked_release_resource_references: Vec<LinkedReleaseResourceReference>,
}

/// ERN 4.x `TrackRelease`: a release of exactly one resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackRelease {
    pub release_reference: String,
    pub release_id: Vec<Identifier>,
    #[serde(default)]
    pub display_title_text: Vec<DisplayText>,
    #[serde(default)]
    pub display_title: Vec<DisplayText>,
    pub release_resource_reference: String,
    #[serde(default)]
    pub linked_release_resource_references: Vec<LinkedReleaseResourceReference>,
    #[serde(default)]
    pub release_label_reference: Vec<String>,
    /// `Genre` entries, in document order.
    #[serde(default)]
    pub genre: Vec<super::Genre>,
    /// ERN 4.3 `@IsMainRelease`; `None` when absent (always absent in 4.2).
    #[serde(default)]
    pub is_main_release: Option<bool>,
}
