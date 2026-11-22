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
//! Compiled but not run (`no_run`): the two XML strings stand in for complete
//! messages.
//!
//! ```no_run
//! use ddex_builder::messages::UpdateGenerator;
//!
//! let original_xml = std::fs::read_to_string("original.xml")?;
//! let updated_xml = std::fs::read_to_string("updated.xml")?;
//!
//! // Describe the changes between two NewReleaseMessages as an UpdateReleaseMessage
//! let mut generator = UpdateGenerator::new();
//! let update = generator.create_update(&original_xml, &updated_xml, "ORIG-MSG-001")?;
//! println!("{} update operations", update.update_list.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
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
