// src/streaming/state.rs
//! State machine for streaming DDEX parser

use ddex_core::models::common::Copyright;
use ddex_core::models::{graph::*, versions::ERNVersion};
use ddex_core::models::{Identifier, LocalizedString};
use std::collections::HashMap;

/// Parser state for streaming processing
#[derive(Debug, Clone, Default)]
pub enum ParserState {
    /// Initial state - waiting for root element
    #[default]
    Initial,
    /// Parsing message header
    InHeader {
        header: PartialMessageHeader,
        depth: usize,
    },
    /// Parsing a release
    InRelease {
        release: PartialRelease,
        depth: usize,
    },
    /// Parsing a resource
    InResource {
        resource: PartialResource,
        depth: usize,
    },
    /// Parsing a party
    InParty { party: PartialParty, depth: usize },
    /// Parsing a deal
    InDeal { deal: PartialDeal, depth: usize },
    /// Skipping unknown element
    Skipping {
        start_depth: usize,
        current_depth: usize,
    },
    /// Parsing complete
    Complete,
    /// Error state
    Error(String),
}

/// Parsing context that tracks current state
#[derive(Debug)]
pub struct ParsingContext {
    pub state: ParserState,
    pub version: ERNVersion,
    pub current_path: Vec<String>,
    pub current_depth: usize,
    pub namespace_stack: Vec<HashMap<String, String>>,
    pub text_buffer: String,
    pub attributes: HashMap<String, String>,
}

impl ParsingContext {
    pub fn new(version: ERNVersion) -> Self {
        Self {
            state: ParserState::Initial,
            version,
            current_path: Vec::new(),
            current_depth: 0,
            namespace_stack: vec![HashMap::new()],
            text_buffer: String::new(),
            attributes: HashMap::new(),
        }
    }

    pub fn push_element(&mut self, name: &str) {
        self.current_path.push(name.to_string());
        self.current_depth += 1;
        // Push new namespace scope
        let parent_scope = self.namespace_stack.last().unwrap().clone();
        self.namespace_stack.push(parent_scope);
    }

    pub fn pop_element(&mut self) -> Option<String> {
        self.current_depth = self.current_depth.saturating_sub(1);
        self.namespace_stack.pop();
        self.current_path.pop()
    }

    pub fn current_element_path(&self) -> String {
        self.current_path.join("/")
    }

    pub fn is_at_path(&self, path: &[&str]) -> bool {
        self.current_path.len() >= path.len()
            && self.current_path[self.current_path.len() - path.len()..]
                .iter()
                .zip(path.iter())
                .all(|(a, b)| a == b)
    }

    pub fn clear_text_buffer(&mut self) {
        self.text_buffer.clear();
    }

    pub fn add_text(&mut self, text: &str) {
        if !text.trim().is_empty() {
            self.text_buffer.push_str(text);
        }
    }

    pub fn take_text(&mut self) -> String {
        std::mem::take(&mut self.text_buffer)
    }
}

/// Partial release being built during streaming
#[derive(Debug, Clone, Default)]
pub struct PartialRelease {
    pub release_reference: Option<String>,
    pub release_id: Vec<Identifier>,
    pub release_title: Vec<LocalizedString>,
    pub display_artist: Vec<Artist>,
    pub genre: Vec<Genre>,
    pub release_date: Vec<ReleaseEvent>,
    pub release_resource_reference_list: Vec<ReleaseResourceReference>,
    pub deal_reference_list: Vec<String>,
    pub completed_fields: usize,
    pub memory_estimate: usize,
}

/// Partial resource being built during streaming
#[derive(Debug, Clone, Default)]
pub struct PartialResource {
    pub resource_reference: Option<String>,
    pub resource_type: Option<ResourceType>,
    pub resource_id: Vec<Identifier>,
    pub reference_title: Vec<LocalizedString>,
    pub duration: Option<std::time::Duration>,
    pub technical_details: Vec<TechnicalDetails>,
    pub rights_controller: Vec<String>,
    pub p_line: Vec<Copyright>,
    pub c_line: Vec<Copyright>,
    pub completed_fields: usize,
    pub memory_estimate: usize,
}

/// Partial party being built during streaming
#[derive(Debug, Clone, Default)]
pub struct PartialParty {
    pub party_reference: Option<String>,
    pub party_name: Vec<LocalizedString>,
    pub party_id: Vec<Identifier>,
    pub role: Vec<String>,
    pub completed_fields: usize,
    pub memory_estimate: usize,
}

/// Partial deal being built during streaming
#[derive(Debug, Clone, Default)]
pub struct PartialDeal {
    pub deal_reference: Option<String>,
    pub deal_terms: Option<DealTerms>,
    pub commercial_model_type: Vec<String>,
    pub use_type: Vec<String>,
    pub territory_code: Vec<String>,
    pub completed_fields: usize,
    pub memory_estimate: usize,
}

/// Partial message header being built during streaming
#[derive(Debug, Clone, Default)]
pub struct PartialMessageHeader {
    pub sender: Option<MessageSender>,
    pub recipient: Vec<MessageRecipient>,
    pub message_created_date_time: Option<String>,
    pub message_id: Option<Identifier>,
    pub message_file_name: Option<String>,
    pub completed_fields: usize,
    pub memory_estimate: usize,
}

impl PartialRelease {
    pub fn estimate_memory(&self) -> usize {
        // Rough memory estimation
        let mut size = std::mem::size_of::<PartialRelease>();
        size += self.release_reference.as_ref().map_or(0, |s| s.len());
        size += self.release_id.len() * std::mem::size_of::<Identifier>();
        size += self.release_title.len() * std::mem::size_of::<LocalizedString>();
        // Add estimates for other fields
        size
    }

    pub fn is_complete(&self) -> bool {
        self.release_reference.is_some() && !self.release_title.is_empty()
    }

    pub fn into_release(self) -> Release {
        Release {
            release_reference: self.release_reference.unwrap_or_default(),
            release_id: self.release_id,
            release_title: self.release_title,
            release_subtitle: None,
            release_type: None,
            genre: self.genre,
            release_resource_reference_list: self.release_resource_reference_list,
            display_artist: self.display_artist,
            party_list: vec![],
            release_date: self.release_date,
            territory_code: vec![],
            excluded_territory_code: vec![],
            attributes: None,
            extensions: None,
            comments: None,
            display_title_text: Vec::new(),
            display_title: Vec::new(),
            display_artist_name: Vec::new(),
            resource_group: None,
        }
    }
}

impl PartialResource {
    pub fn estimate_memory(&self) -> usize {
        let mut size = std::mem::size_of::<PartialResource>();
        size += self.resource_reference.as_ref().map_or(0, |s| s.len());
        size += self
            .resource_type
            .as_ref()
            .map_or(0, |_| std::mem::size_of::<ResourceType>());
        size += self.resource_id.len() * std::mem::size_of::<Identifier>();
        size += self.reference_title.len() * std::mem::size_of::<LocalizedString>();
        size
    }

    pub fn is_complete(&self) -> bool {
        self.resource_reference.is_some() && !self.reference_title.is_empty()
    }

    pub fn into_resource(self) -> Resource {
        Resource {
            resource_reference: self.resource_reference.unwrap_or_default(),
            resource_type: self.resource_type.unwrap_or(ResourceType::SoundRecording),
            resource_id: self.resource_id,
            reference_title: self.reference_title,
            duration: self.duration,
            technical_details: self.technical_details,
            rights_controller: self.rights_controller,
            p_line: self.p_line,
            c_line: self.c_line,
            extensions: None,
            display_title_text: Vec::new(),
            display_title: Vec::new(),
            display_artist_name: Vec::new(),
            display_artist: Vec::new(),
        }
    }
}
