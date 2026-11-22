// core/src/models/common/mod.rs
//! Common types shared between models

mod identifier;
mod localized;
mod territory;

pub use identifier::{Identifier, IdentifierType};
pub use localized::LocalizedString;
pub use territory::{Copyright, Price, TerritoryCode, TerritoryScope, ValidityPeriod};
