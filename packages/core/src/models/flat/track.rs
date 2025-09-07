// core/src/models/flat/track.rs
//! Parsed track types

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
    pub title: String,
    pub subtitle: Option<String>,
    pub display_artist: String,
    pub artists: Vec<ArtistInfo>,
    pub duration: Duration,
    pub duration_formatted: String,
    pub file_format: Option<String>,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub is_hidden: bool,
    pub is_bonus: bool,
    pub is_explicit: bool,
    pub is_instrumental: bool,
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
    pub title: String,
    pub duration: Option<Duration>,
    pub technical_details: TechnicalInfo,
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