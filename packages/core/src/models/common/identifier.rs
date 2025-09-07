// core/src/models/common/identifier.rs
//! Identifier types for DDEX

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct Identifier {
    pub id_type: IdentifierType,
    pub namespace: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub enum IdentifierType {
    Proprietary,
    ISRC,
    ISWC,
    UPC,
    EAN,
    GRID,
    GRid,
    ISNI,
    IPI,
}