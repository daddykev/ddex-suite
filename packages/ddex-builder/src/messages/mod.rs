//! # DDEX Message Types
//! 
//! This module contains implementations for various DDEX message types
//! used in music distribution workflows. The DDEX standard defines several
//! message types for different scenarios in the music supply chain.
//! 
//! ## Message Types
//! 
//! - **NewReleaseMessage (ERN)**: The primary message type for delivering
//!   new releases to music platforms and distributors
//! - **UpdateReleaseMessage**: Used to update existing releases with new
//!   metadata, resources, or deal information
//! - **PurgeReleaseMessage**: For removing releases from distribution
//! 
//! ## Architecture
//! 
//! ```text
//! Message Types
//! ├── NewReleaseMessage      # Primary release delivery
//! │   ├── MessageHeader      # Routing and control info
//! │   ├── ResourceList       # Audio/video resources
//! │   ├── ReleaseList        # Release metadata
//! │   └── DealList          # Distribution terms
//! └── UpdateReleaseMessage   # Release updates
//!     ├── MessageHeader      # Update control info
//!     ├── UpdateList        # What to update
//!     └── Instructions      # How to apply updates
//! ```
//! 
//! ## Usage Example
//! 
//! ```rust
//! use ddex_builder::messages::*;
//! use ddex_builder::{Builder, BuildRequest};
//! 
//! // Create a new release message
//! let mut builder = Builder::new();
//! let request = BuildRequest {
//!     message_type: MessageType::NewRelease,
//!     // ... other fields
//! };
//! 
//! let result = builder.build_internal(&request)?;
//! ```
//! 
//! ## Message Validation
//! 
//! All message types include:
//! - Schema validation against DDEX XSD
//! - Business rule validation
//! - Territory and rights validation
//! - Resource reference integrity checks

pub mod update_release;

pub use update_release::*;