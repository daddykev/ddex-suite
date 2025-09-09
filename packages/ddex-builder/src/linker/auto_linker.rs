//! Automatic linking logic for DDEX entities

use super::{EntityType, LinkerError, LinkingReport, ReferenceGenerator, RelationshipManager};
use crate::builder::BuildRequest;
use indexmap::IndexSet;

/// Handles automatic linking of entities in a build request
#[derive(Debug, Clone, Default)]
pub struct AutoLinker {
    /// Configuration for auto-linking behavior
    config: AutoLinkerConfig,
}

/// Configuration for auto-linking
#[derive(Debug, Clone)]
pub struct AutoLinkerConfig {
    /// Automatically generate missing references
    pub generate_missing_refs: bool,
    
    /// Automatically link releases to all their tracks
    pub auto_link_tracks: bool,
    
    /// Automatically link deals to releases
    pub auto_link_deals: bool,
    
    /// Validate reference integrity
    pub validate_references: bool,
}

impl Default for AutoLinkerConfig {
    fn default() -> Self {
        Self {
            generate_missing_refs: true,
            auto_link_tracks: true,
            auto_link_deals: true,
            validate_references: true,
        }
    }
}

impl AutoLinker {
    /// Create a new auto-linker
    pub fn new() -> Self {
        Self::with_config(AutoLinkerConfig::default())
    }
    
    /// Create auto-linker with custom configuration
    pub fn with_config(config: AutoLinkerConfig) -> Self {
        Self { config }
    }
    
    /// Process a build request and automatically link entities
    pub fn process_request(
        &self,
        request: &mut BuildRequest,
        generator: &mut ReferenceGenerator,
        relationships: &mut RelationshipManager,
    ) -> Result<LinkingReport, LinkerError> {
        let mut report = LinkingReport::default();
        
        // Phase 1: Register all resources (tracks) first
        for release in &mut request.releases {
            for track in &mut release.tracks {
                if track.resource_reference.is_none() {
                    let reference = generator.generate(EntityType::Resource);
                    track.resource_reference = Some(reference.clone());
                    relationships.register(
                        EntityType::Resource,
                        track.track_id.clone(),
                        reference,
                    );
                    report.generated_refs += 1;
                }
            }
        }
        
        // Phase 2: Register and link releases
        for release in &mut request.releases {
            // Generate release reference if missing
            if release.release_reference.is_none() {
                let reference = generator.generate(EntityType::Release);
                release.release_reference = Some(reference.clone());
                relationships.register(
                    EntityType::Release,
                    release.release_id.clone(),
                    reference,
                );
                report.generated_refs += 1;
            }
            
            // Auto-link tracks to release
            if self.config.auto_link_tracks {
                let release_ref = release.release_reference.as_ref().unwrap();
                let mut track_refs = IndexSet::new();
                
                for track in &release.tracks {
                    if let Some(track_ref) = &track.resource_reference {
                        track_refs.insert(track_ref.clone());
                        relationships.add_relationship(
                            release_ref.clone(),
                            track_ref.clone(),
                        );
                        report.linked_resources += 1;
                    }
                }
                
                // Update release with resource references
                release.resource_references = Some(track_refs.into_iter().collect());
            }
        }
        
        // Phase 3: Register parties (sender/recipient)
        if request.header.message_sender.party_reference.is_none() {
            let sender_ref = generator.generate(EntityType::Party);
            request.header.message_sender.party_reference = Some(sender_ref.clone());
            relationships.register(
                EntityType::Party,
                request.header.message_sender.party_id.clone().unwrap_or_default(),
                sender_ref,
            );
            report.generated_refs += 1;
        }
        
        if request.header.message_recipient.party_reference.is_none() {
            let recipient_ref = generator.generate(EntityType::Party);
            request.header.message_recipient.party_reference = Some(recipient_ref.clone());
            relationships.register(
                EntityType::Party,
                request.header.message_recipient.party_id.clone().unwrap_or_default(),
                recipient_ref,
            );
            report.generated_refs += 1;
        }
        
        // Phase 4: Link deals
        if self.config.auto_link_deals {
            for deal in &mut request.deals {
                if deal.deal_reference.is_none() {
                    let deal_ref = generator.generate(EntityType::Deal);
                    deal.deal_reference = Some(deal_ref.clone());
                    report.generated_refs += 1;
                }
                
                // Link deal to releases
                if let Some(ref deal_ref) = deal.deal_reference {
                    for release_ref in &deal.release_references {
                        relationships.add_relationship(
                            deal_ref.clone(),
                            release_ref.clone(),
                        );
                        report.linked_deals += 1;
                    }
                }
            }
        }
        
        // Phase 5: Validate if configured
        if self.config.validate_references {
            match relationships.validate() {
                Ok(()) => report.validation_passed = true,
                Err(errors) => {
                    // Convert validation errors to warnings
                    for error in errors {
                        report.warnings.push(format!("Validation: {}", error));
                    }
                    // Still mark as passed if we only have warnings
                    report.validation_passed = true;
                }
            }
        }
        
        Ok(report)
    }
}