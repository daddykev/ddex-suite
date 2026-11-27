// core/src/parser/security.rs
//! Security configuration for the parser.
//!
//! Every reader rejects a DOCTYPE and references to undefined entities (there is no setting
//! for either; see `ddex_core::xml_refs`). Before 0.6.0 this type also had `relaxed()`, which
//! made every `parse` fail, and eleven fields that no reader read or that only routed to
//! that failure (`disable_dtd`,
//! `disable_external_entities`, `max_entity_depth`, `max_attribute_size`, `max_text_size`,
//! `max_file_size`, `parse_timeout`, `stream_timeout`, `allow_network`, `allowed_schemas`,
//! `enable_fast_streaming`).

/// Limits applied while parsing
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Maximum number of entity and character references (`&amp;`, `&#233;`, ...) in
    /// character data, checked by the default parse
    pub max_entity_expansions: usize,
    /// Maximum element nesting depth
    pub max_element_depth: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self::strict()
    }
}

impl SecurityConfig {
    /// Strict security configuration (default)
    pub fn strict() -> Self {
        Self {
            max_entity_expansions: 100, // Reduced from 1000 to protect against XML bombs
            max_element_depth: 100,     // Keep at 100, this will block deep nesting attacks
        }
    }
}
