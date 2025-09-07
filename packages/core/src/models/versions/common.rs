// core/src/models/versions/common.rs
//! Common types for version-specific models

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Common ValidityPeriod for 4.2+
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidityPeriod42 {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Common ValidityPeriod for 4.3
pub type ValidityPeriod43 = ValidityPeriod42;

/// Common MessageControlType for 4.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageControlType43 {
    LiveMessage,
    TestMessage,
    DevelopmentMessage,
}

/// Common PartyDescriptor for 4.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyDescriptor43 {
    pub party_name: Vec<crate::models::common::LocalizedString>,
    pub party_id: Vec<crate::models::common::Identifier>,
    pub trading_name: Option<String>,
}

/// Common MessageAuditTrail for 4.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAuditTrail43 {
    pub message_audit_trail_event: Vec<MessageAuditTrailEvent43>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAuditTrailEvent43 {
    pub message_audit_trail_event_type: String,
    pub date_time: DateTime<Utc>,
    pub responsible_party_reference: Option<String>,
}

/// Territory code for 4.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryCode43 {
    pub territory_code: String,
    pub excluded: bool,
}

/// Distribution channel for 4.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionChannel43 {
    pub distribution_channel_type: String,
    pub distribution_channel_name: Option<String>,
}

/// Price information for 4.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInformation43 {
    pub price_type: String,
    pub price_range_type: Option<String>,
    pub price: Price43,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price43 {
    pub amount: f64,
    pub currency_code: String,
    pub price_tier: Option<String>,
}