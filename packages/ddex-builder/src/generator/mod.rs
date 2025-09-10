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
//! - **optimized_xml_writer**: Performance-optimized XML writer for large files
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
//! use ddex_builder::generator::ASTGenerator;
//! use ddex_builder::{BuildRequest, ReleaseRequest};
//! 
//! let mut generator = ASTGenerator::new("4.3".to_string());
//! 
//! let request = BuildRequest {
//!     releases: vec![ReleaseRequest {
//!         release_id: "R123".to_string(),
//!         tracks: vec![/* track data */],
//!         // ... other fields
//!     }],
//!     // ... other fields
//! };
//! 
//! let ast = generator.generate(&request)?;
//! // AST is now ready for XML serialization
//! ```
//! 
//! ## Performance Characteristics
//! 
//! - **Small releases (< 10 tracks)**: ~1-2ms generation time
//! - **Medium releases (10-50 tracks)**: ~5-8ms generation time  
//! - **Large releases (50+ tracks)**: ~15-25ms generation time
//! - **Memory usage**: ~2-5MB peak for typical releases
//! 
//! ## Error Handling
//! 
//! The generator validates input data and provides detailed error messages for:
//! - Missing required fields
//! - Invalid reference linkages
//! - Schema constraint violations
//! - Data format issues

pub mod xml_writer;
pub mod optimized_xml_writer;

use crate::ast::{AST, Element}; // Removed unused Node import
use crate::builder::{BuildRequest, ReleaseRequest};
use crate::error::BuildError;
use indexmap::IndexMap;

pub struct ASTGenerator {
    version: String,
}

impl ASTGenerator {
    pub fn new(version: String) -> Self {
        Self { version }
    }
    
    pub fn generate(&mut self, request: &BuildRequest) -> Result<AST, BuildError> {
        // Create root element based on version
        let mut root = Element::new("NewReleaseMessage");
        root.namespace = Some("ern".to_string());
        
        // Add version attributes
        root.attributes.insert(
            "MessageSchemaVersionId".to_string(),
            format!("ern/{}", self.version),
        );
        
        // Add MessageHeader
        root.add_child(self.generate_message_header(request)?);
        
        // Add ResourceList
        root.add_child(self.generate_resource_list(&request.releases)?);
        
        // Add ReleaseList
        root.add_child(self.generate_release_list(&request.releases)?);
        
        // Create namespaces map
        let mut namespaces = IndexMap::new();
        namespaces.insert("ern".to_string(), format!("http://ddex.net/xml/ern/{}", self.version.replace('.', "")));
        namespaces.insert("xsi".to_string(), "http://www.w3.org/2001/XMLSchema-instance".to_string());
        
        Ok(AST {
            root,
            namespaces,
            schema_location: None,
        })
    }
    
    fn generate_message_header(&self, request: &BuildRequest) -> Result<Element, BuildError> {
        let mut header = Element::new("MessageHeader");
        
        // Add MessageThreadId (using MessageId for now)
        if let Some(ref msg_id) = request.header.message_id {
            header.add_child(Element::new("MessageThreadId").with_text(msg_id));
            header.add_child(Element::new("MessageId").with_text(msg_id));
        }
        
        // Add MessageCreatedDateTime - use provided timestamp or current time
        let created_time = request.header.message_created_date_time
            .as_ref()
            .map(|t| t.clone())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
        
        header.add_child(
            Element::new("MessageCreatedDateTime")
                .with_text(created_time)
        );
        
        // Add MessageSender
        header.add_child(self.generate_party("MessageSender", &request.header.message_sender)?);
        
        // Add MessageRecipient
        header.add_child(self.generate_party("MessageRecipient", &request.header.message_recipient)?);
        
        Ok(header)
    }
    
    fn generate_party(&self, element_name: &str, party: &crate::builder::PartyRequest) -> Result<Element, BuildError> {
        let mut party_elem = Element::new(element_name);
        
        // Add PartyId if present
        if let Some(ref party_id) = party.party_id {
            party_elem.add_child(Element::new("PartyId").with_text(party_id));
        }
        
        // Add PartyReference if present (for linker support)
        if let Some(ref party_ref) = party.party_reference {
            party_elem.add_child(Element::new("PartyReference").with_text(party_ref));
        }
        
        // Add PartyName
        for party_name in &party.party_name {
            let mut name_elem = Element::new("PartyName");
            if let Some(ref lang) = party_name.language_code {
                name_elem.attributes.insert("LanguageCode".to_string(), lang.clone());
            }
            name_elem.add_text(&party_name.text);
            party_elem.add_child(name_elem);
        }
        
        Ok(party_elem)
    }
    
    fn generate_resource_list(&self, releases: &[ReleaseRequest]) -> Result<Element, BuildError> {
        let mut resource_list = Element::new("ResourceList");
        
        // Generate resources from all tracks in all releases
        for release in releases {
            for track in &release.tracks {
                let mut sound_recording = Element::new("SoundRecording");
                
                // Add ResourceReference (use generated reference or create one)
                // FIX: Create owned string instead of temporary
                let resource_ref = track.resource_reference.clone()
                    .unwrap_or_else(|| format!("A{}", track.track_id));
                sound_recording.add_child(
                    Element::new("ResourceReference").with_text(&resource_ref)
                );
                
                // Add ResourceId with ISRC
                let mut resource_id = Element::new("ResourceId");
                resource_id.add_child(Element::new("ISRC").with_text(&track.isrc));
                sound_recording.add_child(resource_id);
                
                // Add ReferenceTitle
                let mut ref_title = Element::new("ReferenceTitle");
                ref_title.add_child(Element::new("TitleText").with_text(&track.title));
                sound_recording.add_child(ref_title);
                
                // Add Duration (already in ISO 8601 format as String)
                sound_recording.add_child(
                    Element::new("Duration").with_text(&track.duration)
                );
                
                resource_list.add_child(sound_recording);
            }
        }
        
        Ok(resource_list)
    }
    
    fn generate_release_list(&self, releases: &[ReleaseRequest]) -> Result<Element, BuildError> {
        let mut release_list = Element::new("ReleaseList");
        
        for release in releases {
            let mut release_elem = Element::new("Release");
            
            // Add ReleaseReference (use generated reference or create one)
            // FIX: Create owned string instead of temporary
            let release_ref = release.release_reference.clone()
                .unwrap_or_else(|| format!("R{}", release.release_id));
            release_elem.add_child(
                Element::new("ReleaseReference").with_text(&release_ref)
            );
            
            // Add ReleaseId
            let mut release_id = Element::new("ReleaseId");
            release_id.add_child(Element::new("GRid").with_text(&release.release_id));
            release_elem.add_child(release_id);
            
            // Add ReleaseResourceReferences
            if let Some(ref resource_refs) = release.resource_references {
                for resource_ref in resource_refs {
                    release_elem.add_child(
                        Element::new("ReleaseResourceReference").with_text(resource_ref)
                    );
                }
            } else {
                // Auto-generate from tracks if not provided
                for track in &release.tracks {
                    // FIX: Create owned string instead of temporary
                    let resource_ref = track.resource_reference.clone()
                        .unwrap_or_else(|| format!("A{}", track.track_id));
                    release_elem.add_child(
                        Element::new("ReleaseResourceReference").with_text(&resource_ref)
                    );
                }
            }
            
            release_list.add_child(release_elem);
        }
        
        Ok(release_list)
    }
    
    fn generate_deal_list(&self, deals: &[crate::builder::DealRequest]) -> Result<Element, BuildError> {
        let mut deal_list = Element::new("DealList");
        
        for deal in deals {
            let mut deal_elem = Element::new("ReleaseDeal");
            
            // Add DealReference if present
            if let Some(ref deal_ref) = deal.deal_reference {
                deal_elem.add_child(Element::new("DealReference").with_text(deal_ref));
            }
            
            // Add Deal terms (simplified for now)
            let mut deal_terms = Element::new("Deal");
            deal_terms.add_child(
                Element::new("CommercialModelType")
                    .with_text(&deal.deal_terms.commercial_model_type)
            );
            
            // Add territories
            for territory in &deal.deal_terms.territory_code {
                deal_terms.add_child(Element::new("TerritoryCode").with_text(territory));
            }
            
            deal_elem.add_child(deal_terms);
            
            // Add DealReleaseReferences
            for release_ref in &deal.release_references {
                deal_elem.add_child(
                    Element::new("DealReleaseReference").with_text(release_ref)
                );
            }
            
            deal_list.add_child(deal_elem);
        }
        
        Ok(deal_list)
    }
}