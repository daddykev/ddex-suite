// core/src/models/versions/ern_43.rs
//! ERN 4.3 specific model variations (latest and most complete)

use serde::{Deserialize, Serialize};
use crate::models::common::LocalizedString;
use chrono::{DateTime, Utc};
use super::common::{
    ValidityPeriod43, 
    MessageControlType43,
    PartyDescriptor43,
    MessageAuditTrail43,
    TerritoryCode43,
    DistributionChannel43,
    PriceInformation43,
};  // Import from common module

/// MessageHeader for ERN 4.3 (most complete)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader43 {
    pub message_id: String,
    pub message_thread_id: Option<String>,
    pub message_type: MessageType43,
    pub message_sender: PartyDescriptor43,
    pub message_recipient: PartyDescriptor43,
    pub message_created_date_time: DateTime<Utc>,
    pub message_audit_trail: Option<MessageAuditTrail43>,
    pub message_control_type: Option<MessageControlType43>,
    pub profile: Option<ReleaseProfile43>,  // New in 4.3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType43 {
    NewReleaseMessage,
    CatalogListMessage,
    UpdateReleaseMessage,
    TakedownMessage,  // New in 4.3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReleaseProfile43 {
    AudioAlbumMusicOnly,
    AudioSingle,
    VideoAlbum,
    VideoSingle,
    LongFormVideo,
    Mixed,
}

/// DealTerms for ERN 4.3 (extended structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealTerms43 {
    pub deal_reference: Option<String>,
    pub commercial_model_type: Vec<CommercialModelType43>,
    pub use_type: Vec<UseType43>,
    pub territory_code: Vec<TerritoryCode43>,
    pub distribution_channel: Vec<DistributionChannel43>,
    pub price_information: Vec<PriceInformation43>,
    pub validity_period: Option<ValidityPeriod43>,
    pub pre_order_date: Option<DateTime<Utc>>,  // New in 4.3
    pub pre_order_preview_date: Option<DateTime<Utc>>,  // New in 4.3
    pub instant_gratification_date: Option<DateTime<Utc>>,  // New in 4.3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommercialModelType43 {
    PayAsYouGoModel,
    SubscriptionModel,
    AdSupportedModel,
    FreeOfChargeModel,  // New in 4.3
    BundledModel,  // New in 4.3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UseType43 {
    Stream,
    Download,
    OnDemandStream,
    NonInteractiveStream,
    ConditionalDownload,  // New in 4.3
    TetheredDownload,  // New in 4.3
}

/// ResourceGroup introduced in ERN 4.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGroup43 {
    pub resource_group_reference: String,
    pub resource_group_type: ResourceGroupType43,
    pub resource_reference: Vec<String>,
    pub sequence_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceGroupType43 {
    MainRelease,
    BonusResources,
    Chapter,
    Session,
}

/// ChapterInformation introduced in ERN 4.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterInformation43 {
    pub chapter_reference: String,
    pub chapter_title: Vec<LocalizedString>,
    pub start_time: String,  // ISO 8601 duration
    pub end_time: String,
    pub chapter_type: Option<String>,
}