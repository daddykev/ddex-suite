//! ERN 3.8.2 (Legacy) version specification and handling
//!
//! ERN 3.8.2 is a legacy DDEX version that predates many modern features.
//! This module handles the specific requirements and constraints of ERN 3.8.2.

use super::*;
use crate::presets::DdexVersion;

/// Get ERN 3.8.2 version specification
pub fn get_version_spec() -> VersionSpec {
    let mut element_mappings = IndexMap::new();
    let mut namespace_prefixes = IndexMap::new();
    
    // Legacy namespace mappings
    namespace_prefixes.insert("ern".to_string(), "http://ddex.net/xml/ern/382".to_string());
    namespace_prefixes.insert("avs".to_string(), "http://ddex.net/xml/avs/avs".to_string());
    
    // Common element mappings from 3.8.2 to modern versions
    element_mappings.insert("NewReleaseMessage".to_string(), "NewReleaseMessage".to_string());
    element_mappings.insert("MessageHeader".to_string(), "MessageHeader".to_string());
    element_mappings.insert("MessageId".to_string(), "MessageId".to_string());
    element_mappings.insert("MessageSender".to_string(), "MessageSender".to_string());
    element_mappings.insert("MessageRecipient".to_string(), "MessageRecipient".to_string());
    element_mappings.insert("MessageCreatedDateTime".to_string(), "MessageCreatedDateTime".to_string());
    
    // Resource-related mappings
    element_mappings.insert("ResourceList".to_string(), "ResourceList".to_string());
    element_mappings.insert("SoundRecording".to_string(), "SoundRecording".to_string());
    element_mappings.insert("ResourceReference".to_string(), "ResourceReference".to_string());
    element_mappings.insert("ResourceId".to_string(), "ResourceId".to_string());
    element_mappings.insert("ReferenceTitle".to_string(), "ReferenceTitle".to_string());
    element_mappings.insert("DisplayArtist".to_string(), "DisplayArtist".to_string());
    element_mappings.insert("ISRC".to_string(), "ISRC".to_string());
    element_mappings.insert("Duration".to_string(), "Duration".to_string());
    
    // Release-related mappings
    element_mappings.insert("ReleaseList".to_string(), "ReleaseList".to_string());
    element_mappings.insert("Release".to_string(), "Release".to_string());
    element_mappings.insert("ReleaseId".to_string(), "ReleaseId".to_string());
    element_mappings.insert("Title".to_string(), "Title".to_string());
    element_mappings.insert("DisplayArtist".to_string(), "DisplayArtist".to_string());
    element_mappings.insert("LabelName".to_string(), "LabelName".to_string());
    element_mappings.insert("UPC".to_string(), "UPC".to_string());
    element_mappings.insert("ReleaseDate".to_string(), "ReleaseDate".to_string());
    element_mappings.insert("Genre".to_string(), "Genre".to_string());
    
    // Deal-related mappings (limited in 3.8.2)
    element_mappings.insert("DealList".to_string(), "DealList".to_string());
    element_mappings.insert("ReleaseDeal".to_string(), "ReleaseDeal".to_string());
    element_mappings.insert("DealTerms".to_string(), "DealTerms".to_string());
    element_mappings.insert("CommercialModelType".to_string(), "CommercialModelType".to_string());
    element_mappings.insert("TerritoryCode".to_string(), "TerritoryCode".to_string());
    
    // Legacy-specific elements that don't exist in newer versions
    element_mappings.insert("LegacyTechnicalDetails".to_string(), "TechnicalResourceDetails".to_string());
    element_mappings.insert("BasicPrice".to_string(), "Price".to_string());
    element_mappings.insert("SimpleTerritory".to_string(), "Territory".to_string());
    
    VersionSpec {
        version: DdexVersion::Ern382,
        namespace: "http://ddex.net/xml/ern/382".to_string(),
        schema_location: Some("http://ddex.net/xml/ern/382 http://ddex.net/xml/ern/382/release-notification.xsd".to_string()),
        message_schema_version_id: "ern/382".to_string(),
        supported_message_types: vec![
            "NewReleaseMessage".to_string(),
            "CatalogListMessage".to_string(),
        ],
        element_mappings,
        required_elements: vec![
            "MessageId".to_string(),
            "MessageSender".to_string(),
            "MessageRecipient".to_string(),
            "MessageCreatedDateTime".to_string(),
            "ResourceList".to_string(),
            "ReleaseList".to_string(),
        ],
        deprecated_elements: vec![
            // These elements exist in 3.8.2 but are deprecated in newer versions
            "LegacyTechnicalDetails".to_string(),
            "BasicPrice".to_string(),
            "SimpleTerritory".to_string(),
            "OldStyleGenre".to_string(),
        ],
        new_elements: vec![
            // Elements that are new in 3.8.2 (none, as this is the baseline)
        ],
        namespace_prefixes,
    }
}

/// ERN 3.8.2 specific constraints and validation rules
pub struct Ern382Constraints {
    /// Maximum allowed resources per release
    pub max_resources_per_release: usize,
    /// Supported image formats
    pub supported_image_formats: Vec<String>,
    /// Supported audio formats
    pub supported_audio_formats: Vec<String>,
    /// Maximum deal complexity
    pub max_deal_terms: usize,
}

impl Default for Ern382Constraints {
    fn default() -> Self {
        Self {
            max_resources_per_release: 100,
            supported_image_formats: vec!["JPEG".to_string(), "PNG".to_string()],
            supported_audio_formats: vec!["MP3".to_string(), "WAV".to_string(), "FLAC".to_string()],
            max_deal_terms: 10,
        }
    }
}

/// Get ERN 3.8.2 specific element validation rules
pub fn get_validation_rules() -> IndexMap<String, ValidationRule> {
    let mut rules = IndexMap::new();
    
    // Message ID format (simpler in 3.8.2)
    rules.insert("MessageId".to_string(), ValidationRule {
        rule_type: ValidationRuleType::Pattern,
        pattern: Some("^[A-Za-z0-9_-]{1,50}$".to_string()),
        enum_values: None,
        required: true,
        description: "Message identifier format for ERN 3.8.2".to_string(),
    });
    
    // ISRC format (standard)
    rules.insert("ISRC".to_string(), ValidationRule {
        rule_type: ValidationRuleType::Pattern,
        pattern: Some("^[A-Z]{2}[A-Z0-9]{3}\\d{7}$".to_string()),
        enum_values: None,
        required: true,
        description: "ISRC format validation".to_string(),
    });
    
    // UPC format (12 digits)
    rules.insert("UPC".to_string(), ValidationRule {
        rule_type: ValidationRuleType::Pattern,
        pattern: Some("^\\d{12}$".to_string()),
        enum_values: None,
        required: false,
        description: "UPC format validation".to_string(),
    });
    
    // Duration format (simpler in 3.8.2)
    rules.insert("Duration".to_string(), ValidationRule {
        rule_type: ValidationRuleType::Pattern,
        pattern: Some("^PT\\d+M\\d+S$".to_string()),
        enum_values: None,
        required: false,
        description: "Duration in PTnMnS format".to_string(),
    });
    
    // Release date format
    rules.insert("ReleaseDate".to_string(), ValidationRule {
        rule_type: ValidationRuleType::Pattern,
        pattern: Some("^\\d{4}-\\d{2}-\\d{2}$".to_string()),
        enum_values: None,
        required: false,
        description: "Release date in YYYY-MM-DD format".to_string(),
    });
    
    // Territory code (simpler set in 3.8.2)
    rules.insert("TerritoryCode".to_string(), ValidationRule {
        rule_type: ValidationRuleType::Enum,
        pattern: None,
        enum_values: Some(vec![
            "US".to_string(),
            "GB".to_string(),
            "DE".to_string(),
            "FR".to_string(),
            "JP".to_string(),
            "Worldwide".to_string(),
        ]),
        required: true,
        description: "Supported territory codes in ERN 3.8.2".to_string(),
    });
    
    rules
}

/// Validation rule definition
#[derive(Debug, Clone)]
pub struct ValidationRule {
    /// Type of validation rule
    pub rule_type: ValidationRuleType,
    /// Regex pattern for pattern validation
    pub pattern: Option<String>,
    /// Enum values for enum validation
    pub enum_values: Option<Vec<String>>,
    /// Whether field is required
    pub required: bool,
    /// Description of the rule
    pub description: String,
}

/// Type of validation rule
#[derive(Debug, Clone)]
pub enum ValidationRuleType {
    /// Pattern-based validation
    Pattern,
    /// Enum value validation
    Enum,
    /// Length validation
    Length { min: Option<usize>, max: Option<usize> },
    /// Custom validation function
    Custom(String),
}

/// Get ERN 3.8.2 namespace mappings
pub fn get_namespace_mappings() -> IndexMap<String, String> {
    let mut mappings = IndexMap::new();
    
    mappings.insert("ern".to_string(), "http://ddex.net/xml/ern/382".to_string());
    mappings.insert("avs".to_string(), "http://ddex.net/xml/avs/avs".to_string());
    mappings.insert("drm".to_string(), "http://ddex.net/xml/drm/drm".to_string());
    
    mappings
}

/// Get ERN 3.8.2 specific XML template
pub fn get_xml_template() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/382" 
                  xmlns:avs="http://ddex.net/xml/avs/avs"
                  MessageSchemaVersionId="ern/382">
    <MessageHeader>
        <MessageId>{message_id}</MessageId>
        <MessageSender>
            <PartyName>{sender_name}</PartyName>
        </MessageSender>
        <MessageRecipient>
            <PartyName>{recipient_name}</PartyName>
        </MessageRecipient>
        <MessageCreatedDateTime>{created_datetime}</MessageCreatedDateTime>
    </MessageHeader>
    
    <ResourceList>
        <!-- Resources will be populated here -->
    </ResourceList>
    
    <ReleaseList>
        <!-- Releases will be populated here -->
    </ReleaseList>
    
    <DealList>
        <!-- Deals will be populated here -->
    </DealList>
</NewReleaseMessage>"#
}

/// ERN 3.8.2 specific element builders
pub mod builders {
    use super::*;
    use crate::ast::{Element, Node};
    
    /// Build ERN 3.8.2 message header
    pub fn build_message_header(
        message_id: &str,
        sender_name: &str,
        recipient_name: &str,
        created_datetime: &str,
    ) -> Element {
        let mut header = Element::new("MessageHeader");
        
        // Message ID
        let mut msg_id = Element::new("MessageId");
        msg_id.add_text(message_id);
        header.add_child(msg_id);
        
        // Message Sender
        let mut sender = Element::new("MessageSender");
        let mut sender_party = Element::new("PartyName");
        sender_party.add_text(sender_name);
        sender.add_child(sender_party);
        header.add_child(sender);
        
        // Message Recipient
        let mut recipient = Element::new("MessageRecipient");
        let mut recipient_party = Element::new("PartyName");
        recipient_party.add_text(recipient_name);
        recipient.add_child(recipient_party);
        header.add_child(recipient);
        
        // Created DateTime
        let mut created = Element::new("MessageCreatedDateTime");
        created.add_text(created_datetime);
        header.add_child(created);
        
        header
    }
    
    /// Build ERN 3.8.2 sound recording resource
    pub fn build_sound_recording(
        resource_ref: &str,
        resource_id: &str,
        title: &str,
        artist: &str,
        isrc: &str,
        duration: &str,
    ) -> Element {
        let mut sound_recording = Element::new("SoundRecording");
        
        // Resource Reference
        let mut res_ref = Element::new("ResourceReference");
        res_ref.add_text(resource_ref);
        sound_recording.add_child(res_ref);
        
        // Type (always SoundRecording in this context)
        let mut res_type = Element::new("Type");
        res_type.add_text("SoundRecording");
        sound_recording.add_child(res_type);
        
        // Resource ID
        let mut res_id = Element::new("ResourceId");
        res_id.add_text(resource_id);
        sound_recording.add_child(res_id);
        
        // Reference Title
        let mut ref_title = Element::new("ReferenceTitle");
        ref_title.add_text(title);
        sound_recording.add_child(ref_title);
        
        // Display Artist
        let mut display_artist = Element::new("DisplayArtist");
        display_artist.add_text(artist);
        sound_recording.add_child(display_artist);
        
        // ISRC
        let mut isrc_elem = Element::new("ISRC");
        isrc_elem.add_text(isrc);
        sound_recording.add_child(isrc_elem);
        
        // Duration
        let mut duration_elem = Element::new("Duration");
        duration_elem.add_text(duration);
        sound_recording.add_child(duration_elem);
        
        sound_recording
    }
    
    /// Build ERN 3.8.2 release
    pub fn build_release(
        release_ref: &str,
        release_id: &str,
        title: &str,
        artist: &str,
        label: &str,
        upc: Option<&str>,
        release_date: Option<&str>,
        genre: Option<&str>,
        resource_refs: &[String],
    ) -> Element {
        let mut release = Element::new("Release");
        
        // Release Reference
        let mut rel_ref = Element::new("ReleaseReference");
        rel_ref.add_text(release_ref);
        release.add_child(rel_ref);
        
        // Release ID
        let mut rel_id = Element::new("ReleaseId");
        rel_id.add_text(release_id);
        release.add_child(rel_id);
        
        // Release Type (default to Album for ERN 3.8.2)
        let mut rel_type = Element::new("ReleaseType");
        rel_type.add_text("Album");
        release.add_child(rel_type);
        
        // Title
        let mut title_elem = Element::new("Title");
        title_elem.add_text(title);
        release.add_child(title_elem);
        
        // Display Artist
        let mut artist_elem = Element::new("DisplayArtist");
        artist_elem.add_text(artist);
        release.add_child(artist_elem);
        
        // Label Name
        let mut label_elem = Element::new("LabelName");
        label_elem.add_text(label);
        release.add_child(label_elem);
        
        // UPC (optional)
        if let Some(upc_val) = upc {
            let mut upc_elem = Element::new("UPC");
            upc_elem.add_text(upc_val);
            release.add_child(upc_elem);
        }
        
        // Release Date (optional)
        if let Some(date) = release_date {
            let mut date_elem = Element::new("ReleaseDate");
            date_elem.add_text(date);
            release.add_child(date_elem);
        }
        
        // Genre (optional)
        if let Some(genre_val) = genre {
            let mut genre_elem = Element::new("Genre");
            genre_elem.add_text(genre_val);
            release.add_child(genre_elem);
        }
        
        // Resource Group (simplified in 3.8.2)
        if !resource_refs.is_empty() {
            let mut resource_group = Element::new("ResourceGroup");
            for res_ref in resource_refs {
                let mut ref_elem = Element::new("ResourceReference");
                ref_elem.add_text(res_ref);
                resource_group.add_child(ref_elem);
            }
            release.add_child(resource_group);
        }
        
        release
    }
    
    /// Build ERN 3.8.2 simple deal
    pub fn build_simple_deal(
        deal_ref: &str,
        commercial_model: &str,
        territory: &str,
        start_date: Option<&str>,
        price: Option<f64>,
        currency: Option<&str>,
        release_refs: &[String],
    ) -> Element {
        let mut deal = Element::new("ReleaseDeal");
        
        // Deal Reference
        let mut deal_ref_elem = Element::new("DealReference");
        deal_ref_elem.add_text(deal_ref);
        deal.add_child(deal_ref_elem);
        
        // Deal Terms (simplified in 3.8.2)
        let mut deal_terms = Element::new("DealTerms");
        
        // Commercial Model Type
        let mut model_elem = Element::new("CommercialModelType");
        model_elem.add_text(commercial_model);
        deal_terms.add_child(model_elem);
        
        // Territory Code
        let mut territory_elem = Element::new("TerritoryCode");
        territory_elem.add_text(territory);
        deal_terms.add_child(territory_elem);
        
        // Validity Period (if start date provided)
        if let Some(start) = start_date {
            let mut validity = Element::new("ValidityPeriod");
            let mut start_elem = Element::new("StartDate");
            start_elem.add_text(start);
            validity.add_child(start_elem);
            deal_terms.add_child(validity);
        }
        
        // Price (if provided)
        if let Some(price_val) = price {
            let mut price_elem = Element::new("Price");
            let mut amount_elem = Element::new("PriceAmount");
            amount_elem.add_text(&price_val.to_string());
            price_elem.add_child(amount_elem);
            
            if let Some(currency_code) = currency {
                let mut currency_elem = Element::new("PriceCurrencyCode");
                currency_elem.add_text(currency_code);
                price_elem.add_child(currency_elem);
            }
            
            deal_terms.add_child(price_elem);
        }
        
        deal.add_child(deal_terms);
        
        // Release References
        for rel_ref in release_refs {
            let mut ref_elem = Element::new("ReleaseReference");
            ref_elem.add_text(rel_ref);
            deal.add_child(ref_elem);
        }
        
        deal
    }
}

/// ERN 3.8.2 validation functions
pub mod validation {
    use super::*;
    use regex::Regex;
    use once_cell::sync::Lazy;
    
    // Precompiled regex patterns for ERN 3.8.2
    static ISRC_PATTERN_382: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[A-Z]{2}[A-Z0-9]{3}\d{7}$").unwrap()
    });
    
    static UPC_PATTERN_382: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\d{12}$").unwrap()
    });
    
    static DURATION_PATTERN_382: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^PT\d+M\d+S$").unwrap()
    });
    
    static DATE_PATTERN_382: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap()
    });
    
    /// Validate ISRC format for ERN 3.8.2
    pub fn validate_isrc(isrc: &str) -> bool {
        ISRC_PATTERN_382.is_match(isrc)
    }
    
    /// Validate UPC format for ERN 3.8.2
    pub fn validate_upc(upc: &str) -> bool {
        UPC_PATTERN_382.is_match(upc)
    }
    
    /// Validate duration format for ERN 3.8.2
    pub fn validate_duration(duration: &str) -> bool {
        DURATION_PATTERN_382.is_match(duration)
    }
    
    /// Validate date format for ERN 3.8.2
    pub fn validate_date(date: &str) -> bool {
        DATE_PATTERN_382.is_match(date)
    }
    
    /// Validate territory code for ERN 3.8.2
    pub fn validate_territory_code(territory: &str) -> bool {
        matches!(territory, "US" | "GB" | "DE" | "FR" | "JP" | "Worldwide")
    }
    
    /// Validate commercial model type for ERN 3.8.2
    pub fn validate_commercial_model(model: &str) -> bool {
        matches!(model, 
            "SubscriptionModel" | 
            "PurchaseModel" | 
            "AdSupportedModel" |
            "FreeOfChargeModel"
        )
    }
    
    /// Get all validation errors for an ERN 3.8.2 message
    pub fn validate_ern_382_message(xml_content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        // Check required namespace
        if !xml_content.contains("http://ddex.net/xml/ern/382") {
            errors.push("Missing ERN 3.8.2 namespace".to_string());
        }
        
        // Check message schema version ID
        if !xml_content.contains("ern/382") {
            errors.push("Missing or incorrect MessageSchemaVersionId".to_string());
        }
        
        // Check for required elements
        let required_elements = [
            "MessageId",
            "MessageSender", 
            "MessageRecipient",
            "MessageCreatedDateTime",
            "ResourceList",
            "ReleaseList",
        ];
        
        for element in &required_elements {
            if !xml_content.contains(&format!("<{}", element)) {
                errors.push(format!("Missing required element: {}", element));
            }
        }
        
        errors
    }
}