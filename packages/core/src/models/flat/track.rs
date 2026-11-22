// core/src/models/flat/track.rs
//! Parsed track types

use crate::models::common::Copyright;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTrack {
    pub track_id: String,
    pub isrc: Option<String>,
    pub iswc: Option<String>,
    pub position: usize,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub side: Option<String>,
    /// `ReferenceTitle/TitleText` of the underlying resource; `None` when the source has none.
    pub title: Option<String>,
    pub subtitle: Option<String>,
    /// Display artist; `None` when the source has none.
    pub display_artist: Option<String>,
    pub artists: Vec<ArtistInfo>,
    /// `Duration` of the underlying resource; `None` when the source has none.
    pub duration: Option<Duration>,
    /// `duration` formatted as `m:ss`; `None` when `duration` is `None`.
    pub duration_formatted: Option<String>,
    pub file_format: Option<String>,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub is_hidden: bool,
    pub is_bonus: bool,
    pub is_explicit: bool,
    pub is_instrumental: bool,
    /// ERN 4.x: `ReleaseReference` of the `TrackRelease` that releases this track's resource.
    /// Deals reference track releases by this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_release_reference: Option<String>,
    /// ERN 4.x: first identifier in that `TrackRelease`'s `ReleaseId`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_release_id: Option<String>,
    /// ERN 4.x: the nested `ResourceGroup`s enclosing this track, outermost first
    /// (the release's top-level group is not included). Empty for 3.8.x.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_path: Vec<TrackGroup>,
    /// The resource's default genre (3.8.x; 4.x resources carry none).
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub sub_genre: Option<String>,
    /// The resource's default `ParentalWarningType`.
    #[serde(default)]
    pub parental_warning: Option<String>,
    /// The resource's default P line / C line.
    #[serde(default)]
    pub p_line: Option<Copyright>,
    #[serde(default)]
    pub c_line: Option<Copyright>,
    /// The resource's default label name (3.8.x).
    #[serde(default)]
    pub label_name: Option<String>,
    /// ERN 4.x: the default genre of this track's TrackRelease.
    #[serde(default)]
    pub track_release_genre: Option<String>,
    #[serde(default)]
    pub track_release_sub_genre: Option<String>,
}

/// One nested `ResourceGroup` on a track's `group_path`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackGroup {
    /// `@ResourceGroupType` (required on nested groups by the XSD).
    pub group_type: Option<String>,
    pub sequence_number: Option<i32>,
}

use crate::models::flat::release::ArtistInfo;

impl ParsedTrack {
    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{}:{:02}", minutes, seconds)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedResource {
    pub resource_id: String,
    pub resource_type: String,
    /// `ReferenceTitle/TitleText`; `None` for resources without one (e.g. images).
    pub title: Option<String>,
    pub duration: Option<Duration>,
    pub technical_details: TechnicalInfo,
    /// Default genre (3.8.x; 4.x resources carry none).
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub sub_genre: Option<String>,
    /// Default `ParentalWarningType`.
    #[serde(default)]
    pub parental_warning: Option<String>,
    /// Default P line / C line.
    #[serde(default)]
    pub p_line: Option<Copyright>,
    #[serde(default)]
    pub c_line: Option<Copyright>,
    /// Default label name (3.8.x).
    #[serde(default)]
    pub label_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalInfo {
    pub file_format: Option<String>,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub file_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedImage {
    pub image_id: String,
    pub image_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedVideo {
    pub video_id: String,
    pub video_type: String,
    pub duration: Option<Duration>,
    pub resolution: Option<String>,
}
