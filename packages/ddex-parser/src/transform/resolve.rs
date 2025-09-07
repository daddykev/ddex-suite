// core/src/transform/resolve.rs
//! Reference resolution

use std::collections::HashMap;
use ddex_core::models::graph::{ERNMessage, Party, Resource, Release, Deal};

pub struct ReferenceResolver {
    party_map: HashMap<String, Party>,
    resource_map: HashMap<String, Resource>,
    release_map: HashMap<String, Release>,
    deal_map: HashMap<String, Deal>,
}

impl ReferenceResolver {
    pub fn new() -> Self {
        Self {
            party_map: HashMap::new(),
            resource_map: HashMap::new(),
            release_map: HashMap::new(),
            deal_map: HashMap::new(),
        }
    }
    
    pub fn build_maps(&mut self, message: &ERNMessage) {
        // Build party map
        for party in &message.parties {
            if let Some(id) = party.party_id.first() {
                self.party_map.insert(id.value.clone(), party.clone());
            }
        }
        
        // Build resource map
        for resource in &message.resources {
            self.resource_map.insert(resource.resource_reference.clone(), resource.clone());
        }
        
        // Build release map
        for release in &message.releases {
            self.release_map.insert(release.release_reference.clone(), release.clone());
        }
        
        // Build deal map
        for (idx, deal) in message.deals.iter().enumerate() {
            let key = deal.deal_reference.clone()
                .unwrap_or_else(|| format!("deal_{}", idx));
            self.deal_map.insert(key, deal.clone());
        }
    }
    
    pub fn resolve_party_reference(&self, reference: &str) -> Option<&Party> {
        self.party_map.get(reference)
    }
    
    pub fn resolve_resource_reference(&self, reference: &str) -> Option<&Resource> {
        self.resource_map.get(reference)
    }
    
    pub fn resolve_release_reference(&self, reference: &str) -> Option<&Release> {
        self.release_map.get(reference)
    }
    
    pub fn validate_references(&self, message: &ERNMessage) -> Vec<UnresolvedReference> {
        let mut unresolved = Vec::new();
        
        // Check release resource references
        for release in &message.releases {
            for rref in &release.release_resource_reference_list {
                if !self.resource_map.contains_key(&rref.resource_reference) {
                    unresolved.push(UnresolvedReference {
                        reference_type: "Resource".to_string(),
                        reference_value: rref.resource_reference.clone(),
                        location: format!("Release/{}/ResourceReference", release.release_reference),
                    });
                }
            }
        }
        
        // Check deal release references
        for (idx, deal) in message.deals.iter().enumerate() {
            for release_ref in &deal.deal_release_reference {
                if !self.release_map.contains_key(release_ref as &str) {
                    unresolved.push(UnresolvedReference {
                        reference_type: "Release".to_string(),
                        reference_value: release_ref.clone(),
                        location: format!("Deal[{}]/ReleaseReference", idx),
                    });
                }
            }
        }
        
        unresolved
    }
}

#[derive(Debug, Clone)]
pub struct UnresolvedReference {
    pub reference_type: String,
    pub reference_value: String,
    pub location: String,
}