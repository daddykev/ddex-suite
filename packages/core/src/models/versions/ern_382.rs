// core/src/models/versions/ern_382.rs
//! ERN 3.8.2 specific model variations

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// MessageHeader for ERN 3.8.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader382 {
    pub message_thread_id: String,  // Required in 3.8.2
    pub message_id: String,
    pub message_file_name: Option<String>,
    pub message_sender: PartyDescriptor382,
    pub sent_on_behalf_of: Option<PartyDescriptor382>,
    pub message_recipient: PartyDescriptor382,
    pub message_created_date_time: DateTime<Utc>,
    pub message_control_type: Option<String>,
}

/// PartyDescriptor for ERN 3.8.2 (simpler structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyDescriptor382 {
    pub party_name: String,  // Single name, not array
    pub party_id: Option<String>,  // Single ID, not array
}

/// DealTerms for ERN 3.8.2 (different structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealTerms382 {
    pub commercial_model_type: String,  // Single value in 3.8.2
    pub usage: Option<Usage382>,
    pub territory_code: Vec<String>,
    pub excluded_territory_code: Vec<String>,
    pub distribution_channel: Vec<String>,
    pub price_information: Option<PriceInformation382>,
    pub validity_period: Option<ValidityPeriod382>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage382 {
    pub use_type: Vec<String>,
    pub user_interface_type: Vec<String>,
    pub distribution_format_type: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInformation382 {
    pub price_type: String,
    pub price: Price382,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price382 {
    pub amount: f64,
    pub currency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidityPeriod382 {
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}

/// SoundRecording for ERN 3.8.2 (no TechnicalInstantiation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundRecording382 {
    pub resource_reference: String,
    pub resource_id: Vec<ProprietaryId382>,
    pub title: Vec<String>,  // Not LocalizedString in 3.8.2
    pub duration: Option<String>,  // ISO 8601 duration string
    pub creation_date: Option<String>,
    pub mastered_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProprietaryId382 {
    pub proprietary_id: String,
    pub namespace: String,
}