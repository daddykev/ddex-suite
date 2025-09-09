//! ERN 4.2 version specification and handling
//!
//! ERN 4.2 is an intermediate DDEX version that introduced many modern features
//! while maintaining compatibility with legacy systems.

use super::*;
use crate::presets::DdexVersion;

/// Get ERN 4.2 version specification
pub fn get_version_spec() -> VersionSpec {
    let mut element_mappings = IndexMap::new();
    let mut namespace_prefixes = IndexMap::new();
    
    // Modern namespace mappings
    namespace_prefixes.insert("ern".to_string(), "http://ddex.net/xml/ern/42".to_string());
    namespace_prefixes.insert("avs".to_string(), "http://ddex.net/xml/avs/avs".to_string());
    
    // Enhanced element mappings for ERN 4.2
    element_mappings.insert("NewReleaseMessage".to_string(), "NewReleaseMessage".to_string());
    element_mappings.insert("MessageHeader".to_string(), "MessageHeader".to_string());
    element_mappings.insert("MessageId".to_string(), "MessageId".to_string());
    element_mappings.insert("MessageSender".to_string(), "MessageSender".to_string());
    element_mappings.insert("MessageRecipient".to_string(), "MessageRecipient".to_string());
    element_mappings.insert("MessageCreatedDateTime".to_string(), "MessageCreatedDateTime".to_string());
    element_mappings.insert("MessageControlType".to_string(), "MessageControlType".to_string());
    
    // Enhanced resource mappings
    element_mappings.insert("ResourceList".to_string(), "ResourceList".to_string());
    element_mappings.insert("SoundRecording".to_string(), "SoundRecording".to_string());
    element_mappings.insert("ResourceReference".to_string(), "ResourceReference".to_string());
    element_mappings.insert("ResourceId".to_string(), "ResourceId".to_string());
    element_mappings.insert("ReferenceTitle".to_string(), "ReferenceTitle".to_string());
    element_mappings.insert("DisplayArtist".to_string(), "DisplayArtist".to_string());
    element_mappings.insert("ISRC".to_string(), "ISRC".to_string());
    element_mappings.insert("Duration".to_string(), "Duration".to_string());
    element_mappings.insert("TechnicalResourceDetails".to_string(), "TechnicalResourceDetails".to_string());
    element_mappings.insert("AudioCodecType".to_string(), "AudioCodecType".to_string());
    element_mappings.insert("BitRate".to_string(), "BitRate".to_string());
    element_mappings.insert("FileName".to_string(), "FileName".to_string());
    
    // Enhanced release mappings
    element_mappings.insert("ReleaseList".to_string(), "ReleaseList".to_string());
    element_mappings.insert("Release".to_string(), "Release".to_string());
    element_mappings.insert("ReleaseReference".to_string(), "ReleaseReference".to_string());
    element_mappings.insert("ReleaseId".to_string(), "ReleaseId".to_string());
    element_mappings.insert("ReleaseType".to_string(), "ReleaseType".to_string());
    element_mappings.insert("Title".to_string(), "Title".to_string());
    element_mappings.insert("DisplayArtist".to_string(), "DisplayArtist".to_string());
    element_mappings.insert("LabelName".to_string(), "LabelName".to_string());
    element_mappings.insert("UPC".to_string(), "UPC".to_string());
    element_mappings.insert("ReleaseDate".to_string(), "ReleaseDate".to_string());
    element_mappings.insert("Genre".to_string(), "Genre".to_string());
    element_mappings.insert("ResourceGroup".to_string(), "ResourceGroup".to_string());
    
    // Enhanced deal mappings
    element_mappings.insert("DealList".to_string(), "DealList".to_string());
    element_mappings.insert("ReleaseDeal".to_string(), "ReleaseDeal".to_string());
    element_mappings.insert("DealReference".to_string(), "DealReference".to_string());
    element_mappings.insert("DealTerms".to_string(), "DealTerms".to_string());
    element_mappings.insert("CommercialModelType".to_string(), "CommercialModelType".to_string());
    element_mappings.insert("Territory".to_string(), "Territory".to_string());
    element_mappings.insert("TerritoryCode".to_string(), "TerritoryCode".to_string());
    element_mappings.insert("ValidityPeriod".to_string(), "ValidityPeriod".to_string());
    element_mappings.insert("StartDate".to_string(), "StartDate".to_string());
    element_mappings.insert("EndDate".to_string(), "EndDate".to_string());
    element_mappings.insert("Price".to_string(), "Price".to_string());
    element_mappings.insert("PriceAmount".to_string(), "PriceAmount".to_string());
    element_mappings.insert("PriceCurrencyCode".to_string(), "PriceCurrencyCode".to_string());
    
    // New elements in ERN 4.2
    element_mappings.insert("PartyId".to_string(), "PartyId".to_string());
    element_mappings.insert("PartyReference".to_string(), "PartyReference".to_string());
    element_mappings.insert("DetailedHashSum".to_string(), "DetailedHashSum".to_string());
    element_mappings.insert("PreviewDetails".to_string(), "PreviewDetails".to_string());
    element_mappings.insert("UsageType".to_string(), "UsageType".to_string());
    
    VersionSpec {
        version: DdexVersion::Ern42,
        namespace: "http://ddex.net/xml/ern/42".to_string(),
        schema_location: Some("http://ddex.net/xml/ern/42 http://ddex.net/xml/ern/42/release-notification.xsd".to_string()),
        message_schema_version_id: "ern/42".to_string(),
        supported_message_types: vec![
            "NewReleaseMessage".to_string(),
            "UpdateReleaseMessage".to_string(),
            "CatalogTransferMessage".to_string(),
            "PurgeReleaseMessage".to_string(),
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
            // Elements deprecated in 4.2
            "LegacyTechnicalDetails".to_string(),
            "BasicPrice".to_string(),
            "SimpleTerritory".to_string(),
        ],
        new_elements: vec![
            // Elements new in 4.2 compared to 3.8.2
            "MessageControlType".to_string(),
            "PartyId".to_string(),
            "PartyReference".to_string(),
            "TechnicalResourceDetails".to_string(),
            "AudioCodecType".to_string(),
            "BitRate".to_string(),
            "FileName".to_string(),
            "DetailedHashSum".to_string(),
            "PreviewDetails".to_string(),
            "UsageType".to_string(),
            "ReleaseReference".to_string(),
            "DealReference".to_string(),
            "Territory".to_string(),
            "EndDate".to_string(),
        ],
        namespace_prefixes,
    }
}

/// ERN 4.2 specific constraints and validation rules
pub struct Ern42Constraints {
    /// Maximum allowed resources per release
    pub max_resources_per_release: usize,
    /// Supported image formats
    pub supported_image_formats: Vec<String>,
    /// Supported audio formats
    pub supported_audio_formats: Vec<String>,
    /// Maximum deal complexity
    pub max_deal_terms: usize,
    /// Enhanced validation features
    pub enhanced_validation: bool,
}

impl Default for Ern42Constraints {
    fn default() -> Self {
        Self {
            max_resources_per_release: 500,
            supported_image_formats: vec![
                "JPEG".to_string(), 
                "PNG".to_string(), 
                "GIF".to_string(),
                "TIFF".to_string(),
            ],
            supported_audio_formats: vec![
                "MP3".to_string(), 
                "WAV".to_string(), 
                "FLAC".to_string(),
                "AAC".to_string(),
                "OGG".to_string(),
            ],
            max_deal_terms: 50,
            enhanced_validation: true,
        }
    }
}

/// Get ERN 4.2 namespace mappings
pub fn get_namespace_mappings() -> IndexMap<String, String> {
    let mut mappings = IndexMap::new();
    
    mappings.insert("ern".to_string(), "http://ddex.net/xml/ern/42".to_string());
    mappings.insert("avs".to_string(), "http://ddex.net/xml/avs/avs".to_string());
    mappings.insert("drm".to_string(), "http://ddex.net/xml/drm/drm".to_string());
    mappings.insert("mv".to_string(), "http://ddex.net/xml/mv/mv".to_string());
    
    mappings
}

/// Get ERN 4.2 specific XML template
pub fn get_xml_template() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/42" 
                  xmlns:avs="http://ddex.net/xml/avs/avs"
                  MessageSchemaVersionId="ern/42">
    <MessageHeader>
        <MessageId>{message_id}</MessageId>
        <MessageSender>
            <PartyName>{sender_name}</PartyName>
            <PartyId>{sender_id}</PartyId>
        </MessageSender>
        <MessageRecipient>
            <PartyName>{recipient_name}</PartyName>
            <PartyId>{recipient_id}</PartyId>
        </MessageRecipient>
        <MessageControlType>{control_type}</MessageControlType>
        <MessageCreatedDateTime>{created_datetime}</MessageCreatedDateTime>
    </MessageHeader>
    
    <ResourceList>
        <!-- Enhanced resources will be populated here -->
    </ResourceList>
    
    <ReleaseList>
        <!-- Enhanced releases will be populated here -->
    </ReleaseList>
    
    <DealList>
        <!-- Enhanced deals will be populated here -->
    </DealList>
</NewReleaseMessage>"#
}

/// ERN 4.2 specific element builders
pub mod builders {
    use super::*;
    use crate::ast::{Element, Node};
    
    /// Build ERN 4.2 enhanced message header
    pub fn build_enhanced_message_header(
        message_id: &str,
        sender_name: &str,
        sender_id: Option<&str>,
        recipient_name: &str,
        recipient_id: Option<&str>,
        control_type: Option<&str>,
        created_datetime: &str,
    ) -> Element {
        let mut header = Element::new("MessageHeader");
        
        // Message ID
        let mut msg_id = Element::new("MessageId");
        msg_id.add_text(message_id);
        header.add_child(msg_id);
        
        // Enhanced Message Sender
        let mut sender = Element::new("MessageSender");
        let mut sender_party = Element::new("PartyName");
        sender_party.add_text(sender_name);
        sender.add_child(sender_party);
        
        if let Some(sid) = sender_id {
            let mut sender_id_elem = Element::new("PartyId");
            sender_id_elem.add_text(sid);
            sender.add_child(sender_id_elem);
        }
        header.add_child(sender);
        
        // Enhanced Message Recipient
        let mut recipient = Element::new("MessageRecipient");
        let mut recipient_party = Element::new("PartyName");
        recipient_party.add_text(recipient_name);
        recipient.add_child(recipient_party);
        
        if let Some(rid) = recipient_id {
            let mut recipient_id_elem = Element::new("PartyId");
            recipient_id_elem.add_text(rid);
            recipient.add_child(recipient_id_elem);
        }
        header.add_child(recipient);
        
        // Message Control Type (new in 4.2)
        if let Some(control) = control_type {
            let mut control_elem = Element::new("MessageControlType");
            control_elem.add_text(control);
            header.add_child(control_elem);
        }
        
        // Created DateTime
        let mut created = Element::new("MessageCreatedDateTime");
        created.add_text(created_datetime);
        header.add_child(created);
        
        header
    }
    
    /// Build ERN 4.2 enhanced sound recording resource
    pub fn build_enhanced_sound_recording(
        resource_ref: &str,
        resource_id: &str,
        title: &str,
        artist: &str,
        isrc: &str,
        duration: &str,
        file_name: Option<&str>,
        codec: Option<&str>,
        bit_rate: Option<u32>,
    ) -> Element {
        let mut sound_recording = Element::new("SoundRecording");
        
        // Resource Reference
        let mut res_ref = Element::new("ResourceReference");
        res_ref.add_text(resource_ref);
        sound_recording.add_child(res_ref);
        
        // Type
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
        
        // Enhanced Technical Details (new in 4.2)
        if file_name.is_some() || codec.is_some() || bit_rate.is_some() {
            let mut tech_details = Element::new("TechnicalResourceDetails");
            
            if let Some(fname) = file_name {
                let mut file_elem = Element::new("FileName");
                file_elem.add_text(fname);
                tech_details.add_child(file_elem);
            }
            
            if let Some(codec_type) = codec {
                let mut codec_elem = Element::new("AudioCodecType");
                codec_elem.add_text(codec_type);
                tech_details.add_child(codec_elem);
            }
            
            if let Some(bitrate) = bit_rate {
                let mut bitrate_elem = Element::new("BitRate");
                bitrate_elem.add_text(&bitrate.to_string());
                tech_details.add_child(bitrate_elem);
            }
            
            sound_recording.add_child(tech_details);
        }
        
        sound_recording
    }
    
    /// Build ERN 4.2 enhanced release
    pub fn build_enhanced_release(
        release_ref: &str,
        release_id: &str,
        release_type: &str,
        title: &str,
        artist: &str,
        label: &str,
        upc: Option<&str>,
        release_date: Option<&str>,
        genre: Option<&str>,
        resource_refs: &[String],
    ) -> Element {
        let mut release = Element::new("Release");
        
        // Release Reference (enhanced in 4.2)
        let mut rel_ref = Element::new("ReleaseReference");
        rel_ref.add_text(release_ref);
        release.add_child(rel_ref);
        
        // Release ID
        let mut rel_id = Element::new("ReleaseId");
        rel_id.add_text(release_id);
        release.add_child(rel_id);
        
        // Release Type (more options in 4.2)
        let mut rel_type = Element::new("ReleaseType");
        rel_type.add_text(release_type);
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
        
        // UPC
        if let Some(upc_val) = upc {
            let mut upc_elem = Element::new("UPC");
            upc_elem.add_text(upc_val);
            release.add_child(upc_elem);
        }
        
        // Release Date
        if let Some(date) = release_date {
            let mut date_elem = Element::new("ReleaseDate");
            date_elem.add_text(date);
            release.add_child(date_elem);
        }
        
        // Genre
        if let Some(genre_val) = genre {
            let mut genre_elem = Element::new("Genre");
            genre_elem.add_text(genre_val);
            release.add_child(genre_elem);
        }
        
        // Enhanced Resource Group
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
    
    /// Build ERN 4.2 enhanced deal
    pub fn build_enhanced_deal(
        deal_ref: &str,
        commercial_model: &str,
        territories: &[String],
        start_date: Option<&str>,
        end_date: Option<&str>,
        price: Option<f64>,
        currency: Option<&str>,
        usage_type: Option<&str>,
        release_refs: &[String],
    ) -> Element {
        let mut deal = Element::new("ReleaseDeal");
        
        // Deal Reference (new in 4.2)
        let mut deal_ref_elem = Element::new("DealReference");
        deal_ref_elem.add_text(deal_ref);
        deal.add_child(deal_ref_elem);
        
        // Enhanced Deal Terms
        let mut deal_terms = Element::new("DealTerms");
        
        // Commercial Model Type
        let mut model_elem = Element::new("CommercialModelType");
        model_elem.add_text(commercial_model);
        deal_terms.add_child(model_elem);
        
        // Enhanced Territory handling
        for territory_code in territories {
            let mut territory = Element::new("Territory");
            let mut territory_elem = Element::new("TerritoryCode");
            territory_elem.add_text(territory_code);
            territory.add_child(territory_elem);
            deal_terms.add_child(territory);
        }
        
        // Enhanced Validity Period
        if start_date.is_some() || end_date.is_some() {
            let mut validity = Element::new("ValidityPeriod");
            
            if let Some(start) = start_date {
                let mut start_elem = Element::new("StartDate");
                start_elem.add_text(start);
                validity.add_child(start_elem);
            }
            
            if let Some(end) = end_date {
                let mut end_elem = Element::new("EndDate");
                end_elem.add_text(end);
                validity.add_child(end_elem);
            }
            
            deal_terms.add_child(validity);
        }
        
        // Enhanced Price
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
        
        // Usage Type (new in 4.2)
        if let Some(usage) = usage_type {
            let mut usage_elem = Element::new("UsageType");
            usage_elem.add_text(usage);
            deal_terms.add_child(usage_elem);
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

/// ERN 4.2 validation functions
pub mod validation {
    use super::*;
    use regex::Regex;
    use once_cell::sync::Lazy;
    
    // Enhanced regex patterns for ERN 4.2
    static ISRC_PATTERN_42: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[A-Z]{2}[A-Z0-9]{3}\d{7}$").unwrap()
    });
    
    static UPC_PATTERN_42: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\d{12}$").unwrap()
    });
    
    static DURATION_PATTERN_42: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^PT(?:\d+H)?(?:\d+M)?(?:\d+(?:\.\d+)?S)?$").unwrap()
    });
    
    static DATE_PATTERN_42: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap()
    });
    
    static DATETIME_PATTERN_42: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$").unwrap()
    });
    
    /// Enhanced validation for ERN 4.2
    pub fn validate_isrc(isrc: &str) -> bool {
        ISRC_PATTERN_42.is_match(isrc)
    }
    
    /// Enhanced UPC validation for ERN 4.2
    pub fn validate_upc(upc: &str) -> bool {
        UPC_PATTERN_42.is_match(upc)
    }
    
    /// Enhanced duration validation for ERN 4.2
    pub fn validate_duration(duration: &str) -> bool {
        DURATION_PATTERN_42.is_match(duration)
    }
    
    /// Enhanced date validation for ERN 4.2
    pub fn validate_date(date: &str) -> bool {
        DATE_PATTERN_42.is_match(date)
    }
    
    /// Enhanced datetime validation for ERN 4.2
    pub fn validate_datetime(datetime: &str) -> bool {
        DATETIME_PATTERN_42.is_match(datetime)
    }
    
    /// Enhanced territory code validation for ERN 4.2
    pub fn validate_territory_code(territory: &str) -> bool {
        // Extended territory list for 4.2
        matches!(territory,
            "US" | "GB" | "DE" | "FR" | "JP" | "CA" | "AU" | "IT" | "ES" | "NL" |
            "SE" | "NO" | "DK" | "FI" | "BR" | "MX" | "AR" | "IN" | "CN" | "KR" |
            "Worldwide" | "WorldwideExceptUS"
        )
    }
    
    /// Enhanced commercial model validation for ERN 4.2
    pub fn validate_commercial_model(model: &str) -> bool {
        matches!(model,
            "SubscriptionModel" |
            "PurchaseModel" |
            "AdSupportedModel" |
            "FreeOfChargeModel" |
            "StreamingModel" |
            "DownloadModel"
        )
    }
    
    /// Validate audio codec type for ERN 4.2
    pub fn validate_audio_codec(codec: &str) -> bool {
        matches!(codec,
            "MP3" | "AAC" | "FLAC" | "WAV" | "OGG" | "WMA" | "MP4" | "M4A"
        )
    }
    
    /// Validate usage type for ERN 4.2
    pub fn validate_usage_type(usage: &str) -> bool {
        matches!(usage,
            "Stream" | "Download" | "Preview" | "ConditionalDownload" |
            "DigitalPhonogramDelivery" | "UserMadeClip"
        )
    }
    
    /// Get all validation errors for an ERN 4.2 message
    pub fn validate_ern_42_message(xml_content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        // Check required namespace
        if !xml_content.contains("http://ddex.net/xml/ern/42") {
            errors.push("Missing ERN 4.2 namespace".to_string());
        }
        
        // Check message schema version ID
        if !xml_content.contains("ern/42") {
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
        
        // Check for enhanced elements that should be present
        if xml_content.contains("TechnicalResourceDetails") {
            if !xml_content.contains("AudioCodecType") && !xml_content.contains("FileName") {
                errors.push("TechnicalResourceDetails should contain AudioCodecType or FileName".to_string());
            }
        }
        
        errors
    }
}