// core/src/models/mod.rs
//! DDEX data models

pub mod common;
pub mod graph;
pub mod flat;
pub mod versions;  // Add this line to export the versions module

pub use common::{Identifier, IdentifierType, LocalizedString};

pub mod extensions;
pub use extensions::Extensions;