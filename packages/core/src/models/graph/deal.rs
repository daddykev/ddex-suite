// core/src/models/graph/deal.rs
//! Deal types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::common::{Price, ValidityPeriod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub deal_reference: Option<String>,
    pub deal_release_reference: Vec<String>,
    pub deal_terms: DealTerms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealTerms {
    pub validity_period: Option<ValidityPeriod>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub territory_code: Vec<String>,
    pub excluded_territory_code: Vec<String>,
    pub distribution_channel: Vec<DistributionChannel>,
    pub excluded_distribution_channel: Vec<DistributionChannel>,
    pub commercial_model_type: Vec<CommercialModelType>,
    pub use_type: Vec<UseType>,
    pub price_information: Vec<PriceInformation>,
    pub wholesale_price: Vec<Price>,
    pub suggested_retail_price: Vec<Price>,
    pub pre_order_date: Option<DateTime<Utc>>,
    pub pre_order_preview_date: Option<DateTime<Utc>>,
    pub instant_gratification_date: Option<DateTime<Utc>>,
    pub takedown_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributionChannel {
    Download,
    Stream,
    Physical,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommercialModelType {
    PayAsYouGoModel,
    SubscriptionModel,
    AdSupportedModel,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseType {
    Stream,
    Download,
    OnDemandStream,
    NonInteractiveStream,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInformation {
    pub price_type: String,
    pub price: Price,
    pub price_tier: Option<String>,
}