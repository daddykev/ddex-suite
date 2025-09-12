//! Integration tests for DDEX Suite
//! 
//! This crate provides comprehensive end-to-end integration tests between
//! the DDEX parser and builder packages, verifying round-trip fidelity
//! and cross-package compatibility.

pub mod fixtures;
pub mod round_trip;
pub mod utils;
pub mod comment_retention_tests;
pub mod namespace_integration_tests;

/// Test fixture data and utilities
pub use fixtures::*;

/// Round-trip testing functionality
pub use round_trip::*;

/// Testing utilities
pub use utils::*;

/// Result type for integration tests
pub type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;