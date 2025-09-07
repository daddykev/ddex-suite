// core/src/models/versions/ern_42.rs
//! ERN 4.2 specific model variations

use serde::{Deserialize, Serialize};
use crate::models::common::{LocalizedString, Identifier};
use chrono::{DateTime, Utc};
use super::common::ValidityPeriod42;

/// MessageHeader for ERN 4.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader42 {
    pub message_id: String,
    pub message_thread_id: Option<String>,  // Optional in 4.2
    pub message_type: MessageType42,
    pub message_sender: PartyDescriptor42,
    pub message_recipient: PartyDescriptor42,
    pub message_created_date_time: DateTime<Utc>,
    pub message_audit_trail: Option<MessageAuditTrail42>,  // New in 4.2
    pub message_control_type: Option<MessageControlType42>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType42 {
    NewReleaseMessage,
    CatalogListMessage,
    UpdateReleaseMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageControlType42 {
    LiveMessage,
    TestMessage,
}

/// PartyDescriptor for ERN 4.2 (enhanced)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyDescriptor42 {
    pub party_name: Vec<LocalizedString>,  // Array in 4.2
    pub party_id: Vec<Identifier>,  // Array of typed identifiers
    pub trading_name: Option<String>,  // New in 4.2
}

/// MessageAuditTrail introduced in ERN 4.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAuditTrail42 {
    pub message_audit_trail_event: Vec<MessageAuditTrailEvent42>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAuditTrailEvent42 {
    pub message_audit_trail_event_type: String,
    pub date_time: DateTime<Utc>,
    pub responsible_party_reference: Option<String>,
}

/// DealTerms for ERN 4.2 (standard structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealTerms42 {
    pub deal_reference: Option<String>,  // New in 4.2
    pub commercial_model_type: Vec<String>,  // Array in 4.2
    pub use_type: Vec<String>,
    pub territory_code: Vec<TerritoryCode42>,
    pub distribution_channel: Vec<DistributionChannel42>,
    pub price_information: Vec<PriceInformation42>,
    pub validity_period: Option<ValidityPeriod42>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryCode42 {
    pub territory_code: String,
    pub excluded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionChannel42 {
    pub distribution_channel_type: String,
    pub distribution_channel_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInformation42 {
    pub price_type: String,
    pub price_range_type: Option<String>,
    pub price: Price42,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price42 {
    pub amount: f64,
    pub currency_code: String,
    pub price_tier: Option<String>,  // New in 4.2
}

/// TechnicalInstantiation introduced in ERN 4.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalInstantiation42 {
    pub technical_resource_details_reference: String,
    pub coding_type: Option<String>,
    pub bit_rate: Option<i32>,
    pub file: Option<File42>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File42 {
    pub file_name: String,
    pub file_path: Option<String>,
    pub hash_sum: Option<HashSum42>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashSum42 {
    pub hash_sum: String,
    pub hash_sum_algorithm_type: String,
}