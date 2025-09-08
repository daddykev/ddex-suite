//! DDEX Core - Shared models and types for DDEX Suite

pub mod models;
pub mod error;
pub mod ffi;

// Re-export commonly used types
pub use error::{DDEXError, ErrorLocation};
pub use models::versions::ERNVersion;