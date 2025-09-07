// core/src/models/flat/deal.rs
//! Parsed deal types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::common::Price;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDeal {
    pub deal_id: String,
    pub releases: Vec<String>,
    pub validity: DealValidity,
    pub territories: TerritoryComplexity,
    pub distribution_channels: DistributionComplexity,
    pub pricing: Vec<PriceTier>,
    pub usage_rights: Vec<String>,
    pub restrictions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealValidity {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryComplexity {
    pub included: Vec<String>,
    pub excluded: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionComplexity {
    pub included: Vec<String>,
    pub excluded: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceTier {
    pub tier_name: Option<String>,
    pub price_type: PriceType,
    pub price: Price,
    pub territory: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceType {
    Wholesale,
    SuggestedRetail,
    Minimum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryInfo {
    pub code: String,
    pub included: bool,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub distribution_channels: Vec<String>,
}