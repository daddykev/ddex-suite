//! # AST Generation Engine
//!
//! This module handles the transformation of user-friendly `BuildRequest` structures
//! into intermediate Abstract Syntax Trees (AST) that can be rendered as DDEX XML.
//!
//! ## Architecture Overview
//!
//! ```text
//! Generation Pipeline
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │  BuildRequest   │───▶│   ASTGenerator   │───▶│      AST        │
//! │ (user-friendly) │    │                  │    │ (tree structure)│
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//!           │                       │                       │
//!           ▼                       ▼                       ▼
//!    ┌─────────────┐      ┌─────────────────┐    ┌─────────────────┐
//!    │ • Releases  │      │ • Schema Rules  │    │ • Elements      │
//!    │ • Tracks    │      │ • Validation    │    │ • Attributes    │
//!    │ • Metadata  │      │ • Linking       │    │ • Namespaces    │
//!    │ • Deals     │      │ • References    │    │ • Structure     │
//!    └─────────────┘      └─────────────────┘    └─────────────────┘
//! ```
//!
//! ## Key Components
//!
//! - **ASTGenerator**: Main orchestrator that converts requests to AST
//! - **xml_writer**: High-level XML writer with formatting
//!
//! ## Generation Process
//!
//! 1. **Schema Selection**: Choose DDEX version and namespace configuration
//! 2. **Structure Generation**: Create hierarchical element structure
//! 3. **Reference Linking**: Establish cross-references between elements
//! 4. **Validation**: Ensure generated AST meets schema requirements
//! 5. **Optimization**: Apply performance optimizations for large documents
//!
//! ## Usage Example
//!
//! ```rust
//! use ddex_builder::builder::{
//!     BuildRequest, LocalizedStringRequest, MessageHeaderRequest, PartyRequest,
//! };
//! use ddex_builder::generator::ASTGenerator;
//! # let party = |name: &str| PartyRequest {
//! #     party_name: vec![LocalizedStringRequest { text: name.to_string(), language_code: None }],
//! #     party_id: Some("PADPIDA0000000001".to_string()),
//! #     party_reference: None,
//! # };
//! # let request = BuildRequest {
//! #     header: MessageHeaderRequest {
//! #         message_id: "MSG-1".to_string(),
//! #         message_sender: party("Label"),
//! #         message_recipient: party("DSP"),
//! #         message_control_type: None,
//! #         message_created_date_time: "2024-01-01T00:00:00Z".to_string(),
//! #     },
//! #     version: "4.3".to_string(),
//! #     profile: None,
//! #     releases: vec![ddex_builder::builder::ReleaseRequest {
//! #         release_id: Some("A1-ABCDE-0000000001-M".to_string()),
//! #         release_reference: Some("R1".to_string()),
//! #         title: vec![LocalizedStringRequest { text: "Album".to_string(), language_code: None }],
//! #         artist: Some("Artist".to_string()),
//! #         label: Some("Label".to_string()),
//! #         release_type: Some("Album".into()),
//! #         genre: Some("Pop".to_string()),
//! #         parental_warning_type: Some("NotExplicit".to_string()),
//! #         ..Default::default()
//! #     }],
//! #     deals: vec![],
//! #     extensions: None,
//! #     language_and_script_code: Some("en".to_string()),
//! #     avs_version_id: Some("12".to_string()),
//! #     ..Default::default()
//! # };
//!
//! let mut generator = ASTGenerator::new("4.3".to_string());
//! let ast = generator.generate(&request)?;
//! // `ast` is now ready for XML serialization (see `xml_writer`)
//! # let _ = ast;
//! # Ok::<(), ddex_builder::BuildError>(())
//! ```
//!
//! ## Performance Characteristics
//!
//! Criterion benches of `DDEXBuilder::build` for ERN 4.3, 0.6.0, on a GitHub-hosted runner
//! (Intel Xeon Platinum 8573C, 4 vCPUs), release profile (`dev-docs/PERFORMANCE.md`):
//!
//! - **1 track**: 0.73 ms
//! - **12 tracks**: 3.4 ms
//! - **50 tracks**: 12.5 ms
//! - **100 tracks**: 24 ms (roughly 0.24 ms per track plus about 0.5 ms per message)
//!
//! Memory use has not been measured.
//!
//! ## Error Handling
//!
//! The generator validates input data and provides detailed error messages for:
//! - Missing required fields
//! - Invalid reference linkages
//! - Schema constraint violations
//! - Data format issues

mod ern38;
mod ern4;
mod shared;
pub mod xml_writer;

use crate::ast::AST;
use crate::builder::{BuildRequest, MessageHeaderRequest};
use crate::error::BuildError;

/// `MessageId` and `MessageCreatedDateTime` checked as `DDEXBuilder::build` checks them (both
/// required, the time an xs:dateTime), for writers that do not go through
/// [`ASTGenerator::generate`] (the streaming writer). `label` is the ERN version, e.g. `"3.8.2"`.
pub(crate) fn check_message_header(
    label: &'static str,
    header: &MessageHeaderRequest,
) -> Result<(), BuildError> {
    let mut problems = shared::Problems::new(label);
    problems.message_header(header);
    problems.finish()
}

/// AST generator for converting build requests to abstract syntax trees
pub struct ASTGenerator {
    version: String,
}

impl ASTGenerator {
    /// Create a new AST generator for the specified version
    pub fn new(version: String) -> Self {
        Self { version }
    }

    /// Generate an AST from a build request
    ///
    /// Each version follows its official DDEX XSD: ERN 4.2 and 4.3 are written by `ern4`,
    /// ERN 3.8.2 and 3.8.1 by `ern38`. Both fail, naming every missing request field,
    /// when the schema requires a value the request does not carry.
    pub fn generate(&mut self, request: &BuildRequest) -> Result<AST, BuildError> {
        use ddex_core::models::versions::ERNVersion;
        match crate::builder::parse_target_version(&self.version)? {
            ERNVersion::V4_2 => ern4::generate(request, ern4::Ern4Version::V42),
            ERNVersion::V4_3 => ern4::generate(request, ern4::Ern4Version::V43),
            ERNVersion::V3_8_1 => ern38::generate(request, ern38::Ern38Version::V381),
            ERNVersion::V3_8_2 => ern38::generate(request, ern38::Ern38Version::V382),
        }
    }
}
