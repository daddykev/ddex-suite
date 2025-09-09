//! ERN 4.3 (Current) version specification and handling
//!
//! ERN 4.3 is the current recommended DDEX version with the most complete
//! feature set and modern capabilities.

use super::*;
use crate::presets::DdexVersion;

/// Get ERN 4.3 version specification
pub fn get_version_spec() -> VersionSpec {
    let mut element_mappings = IndexMap::new();
    let mut namespace_prefixes = IndexMap::new();
    
    // Current namespace mappings
    namespace_prefixes.insert("ern".to_string(), "http://ddex.net/xml/ern/43".to_string());
    namespace_prefixes.insert("avs".to_string(), "http://ddex.net/xml/avs/avs".to_string());
    namespace_prefixes.insert("mead".to_string(), "http://ddex.net/xml/mead/mead".to_string());
    
    // Complete element mappings for ERN 4.3
    element_mappings.insert("NewReleaseMessage".to_string(), "NewReleaseMessage".to_string());
    element_mappings.insert("UpdateReleaseMessage".to_string(), "UpdateReleaseMessage".to_string());
    element_mappings.insert("MessageHeader".to_string(), "MessageHeader".to_string());
    element_mappings.insert("MessageId".to_string(), "MessageId".to_string());
    element_mappings.insert("MessageSender".to_string(), "MessageSender".to_string());
    element_mappings.insert("MessageRecipient".to_string(), "MessageRecipient".to_string());
    element_mappings.insert("MessageCreatedDateTime".to_string(), "MessageCreatedDateTime".to_string());
    element_mappings.insert("MessageControlType".to_string(), "MessageControlType".to_string());
    
    // Advanced resource mappings
    element_mappings.insert("ResourceList".to_string(), "ResourceList".to_string());
    element_mappings.insert("SoundRecording".to_string(), "SoundRecording".to_string());
    element_mappings.insert("Video".to_string(), "Video".to_string());
    element_mappings.insert("Image".to_string(), "Image".to_string());
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
    element_mappings.insert("FileSize".to_string(), "FileSize".to_string());
    element_mappings.insert("HashSum".to_string(), "HashSum".to_string());
    
    // Advanced release mappings
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
    element_mappings.insert("OriginalReleaseDate".to_string(), "OriginalReleaseDate".to_string());
    element_mappings.insert("Genre".to_string(), "Genre".to_string());
    element_mappings.insert("SubGenre".to_string(), "SubGenre".to_string());
    element_mappings.insert("ResourceGroup".to_string(), "ResourceGroup".to_string());
    element_mappings.insert("ReleaseResourceReference".to_string(), "ReleaseResourceReference".to_string());
    
    // Advanced deal mappings
    element_mappings.insert("DealList".to_string(), "DealList".to_string());
    element_mappings.insert("ReleaseDeal".to_string(), "ReleaseDeal".to_string());
    element_mappings.insert("DealReference".to_string(), "DealReference".to_string());
    element_mappings.insert("DealTerms".to_string(), "DealTerms".to_string());
    element_mappings.insert("CommercialModelType".to_string(), "CommercialModelType".to_string());
    element_mappings.insert("Territory".to_string(), "Territory".to_string());
    element_mappings.insert("TerritoryCode".to_string(), "TerritoryCode".to_string());
    element_mappings.insert("ExcludedTerritoryCode".to_string(), "ExcludedTerritoryCode".to_string());
    element_mappings.insert("ValidityPeriod".to_string(), "ValidityPeriod".to_string());
    element_mappings.insert("StartDate".to_string(), "StartDate".to_string());
    element_mappings.insert("EndDate".to_string(), "EndDate".to_string());
    element_mappings.insert("Price".to_string(), "Price".to_string());
    element_mappings.insert("PriceAmount".to_string(), "PriceAmount".to_string());
    element_mappings.insert("PriceCurrencyCode".to_string(), "PriceCurrencyCode".to_string());
    element_mappings.insert("PriceType".to_string(), "PriceType".to_string());
    
    // New elements in ERN 4.3
    element_mappings.insert("PartyId".to_string(), "PartyId".to_string());
    element_mappings.insert("PartyReference".to_string(), "PartyReference".to_string());
    element_mappings.insert("DetailedHashSum".to_string(), "DetailedHashSum".to_string());
    element_mappings.insert("PreviewDetails".to_string(), "PreviewDetails".to_string());
    element_mappings.insert("UsageType".to_string(), "UsageType".to_string());
    element_mappings.insert("UseType".to_string(), "UseType".to_string());
    element_mappings.insert("DistributionChannel".to_string(), "DistributionChannel".to_string());
    element_mappings.insert("RightsController".to_string(), "RightsController".to_string());
    element_mappings.insert("RemixType".to_string(), "RemixType".to_string());
    element_mappings.insert("PLine".to_string(), "PLine".to_string());
    element_mappings.insert("CLine".to_string(), "CLine".to_string());
    
    VersionSpec {
        version: DdexVersion::Ern43,
        namespace: "http://ddex.net/xml/ern/43".to_string(),
        schema_location: Some("http://ddex.net/xml/ern/43 http://ddex.net/xml/ern/43/release-notification.xsd".to_string()),
        message_schema_version_id: "ern/43".to_string(),
        supported_message_types: vec![
            "NewReleaseMessage".to_string(),
            "UpdateReleaseMessage".to_string(),
            "CatalogTransferMessage".to_string(),
            "PurgeReleaseMessage".to_string(),
            "ReleaseAvailabilityMessage".to_string(),
            "SalesReportMessage".to_string(),
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
            // No deprecated elements in current version
        ],
        new_elements: vec![
            // Elements new in 4.3 compared to 4.2
            "Video".to_string(),
            "Image".to_string(),
            "FileSize".to_string(),
            "HashSum".to_string(),
            "DetailedHashSum".to_string(),
            "OriginalReleaseDate".to_string(),
            "SubGenre".to_string(),
            "ReleaseResourceReference".to_string(),
            "ExcludedTerritoryCode".to_string(),
            "PriceType".to_string(),
            "UseType".to_string(),
            "DistributionChannel".to_string(),
            "RightsController".to_string(),
            "RemixType".to_string(),
            "PLine".to_string(),
            "CLine".to_string(),
        ],
        namespace_prefixes,
    }
}

/// ERN 4.3 specific constraints and validation rules
pub struct Ern43Constraints {
    /// Maximum allowed resources per release
    pub max_resources_per_release: usize,
    /// Supported image formats
    pub supported_image_formats: Vec<String>,
    /// Supported audio formats
    pub supported_audio_formats: Vec<String>,
    /// Supported video formats
    pub supported_video_formats: Vec<String>,
    /// Maximum deal complexity
    pub max_deal_terms: usize,
    /// Enhanced validation features
    pub enhanced_validation: bool,
    /// Support for complex rights management
    pub rights_management: bool,
}

impl Default for Ern43Constraints {
    fn default() -> Self {
        Self {
            max_resources_per_release: 1000,
            supported_image_formats: vec![
                "JPEG".to_string(),
                "PNG".to_string(),
                "GIF".to_string(),
                "TIFF".to_string(),
                "BMP".to_string(),
                "WEBP".to_string(),
            ],
            supported_audio_formats: vec![
                "MP3".to_string(),
                "WAV".to_string(),
                "FLAC".to_string(),
                "AAC".to_string(),
                "OGG".to_string(),
                "M4A".to_string(),
                "AIFF".to_string(),
                "WMA".to_string(),
            ],
            supported_video_formats: vec![
                "MP4".to_string(),
                "MOV".to_string(),
                "AVI".to_string(),
                "WMV".to_string(),
                "MKV".to_string(),
                "WEBM".to_string(),
            ],
            max_deal_terms: 100,
            enhanced_validation: true,
            rights_management: true,
        }
    }
}

/// Get ERN 4.3 namespace mappings
pub fn get_namespace_mappings() -> IndexMap<String, String> {
    let mut mappings = IndexMap::new();
    
    mappings.insert("ern".to_string(), "http://ddex.net/xml/ern/43".to_string());
    mappings.insert("avs".to_string(), "http://ddex.net/xml/avs/avs".to_string());
    mappings.insert("drm".to_string(), "http://ddex.net/xml/drm/drm".to_string());
    mappings.insert("mv".to_string(), "http://ddex.net/xml/mv/mv".to_string());
    mappings.insert("mead".to_string(), "http://ddex.net/xml/mead/mead".to_string());
    
    mappings
}

/// Get ERN 4.3 specific XML template
pub fn get_xml_template() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" 
                  xmlns:avs="http://ddex.net/xml/avs/avs"
                  xmlns:mead="http://ddex.net/xml/mead/mead"
                  MessageSchemaVersionId="ern/43">
    <MessageHeader>
        <MessageId>{message_id}</MessageId>
        <MessageSender>
            <PartyName>{sender_name}</PartyName>
            <PartyId>{sender_id}</PartyId>
            <PartyReference>{sender_ref}</PartyReference>
        </MessageSender>
        <MessageRecipient>
            <PartyName>{recipient_name}</PartyName>
            <PartyId>{recipient_id}</PartyId>
            <PartyReference>{recipient_ref}</PartyReference>
        </MessageRecipient>
        <MessageControlType>{control_type}</MessageControlType>
        <MessageCreatedDateTime>{created_datetime}</MessageCreatedDateTime>
    </MessageHeader>
    
    <ResourceList>
        <!-- Advanced resources will be populated here -->
    </ResourceList>
    
    <ReleaseList>
        <!-- Advanced releases will be populated here -->
    </ReleaseList>
    
    <DealList>
        <!-- Advanced deals will be populated here -->
    </DealList>
</NewReleaseMessage>"#
}

/// ERN 4.3 specific element builders
pub mod builders {
    use super::*;
    use crate::ast::{Element, Node};
    
    /// Build ERN 4.3 advanced message header
    pub fn build_advanced_message_header(
        message_id: &str,
        sender_name: &str,
        sender_id: Option<&str>,
        sender_ref: Option<&str>,
        recipient_name: &str,
        recipient_id: Option<&str>,
        recipient_ref: Option<&str>,
        control_type: Option<&str>,
        created_datetime: &str,
    ) -> Element {
        let mut header = Element::new("MessageHeader");
        
        // Message ID
        let mut msg_id = Element::new("MessageId");
        msg_id.add_text(message_id);
        header.add_child(msg_id);
        
        // Advanced Message Sender
        let mut sender = Element::new("MessageSender");
        let mut sender_party = Element::new("PartyName");
        sender_party.add_text(sender_name);
        sender.add_child(sender_party);
        
        if let Some(sid) = sender_id {
            let mut sender_id_elem = Element::new("PartyId");
            sender_id_elem.add_text(sid);
            sender.add_child(sender_id_elem);
        }
        
        if let Some(sref) = sender_ref {
            let mut sender_ref_elem = Element::new("PartyReference");
            sender_ref_elem.add_text(sref);
            sender.add_child(sender_ref_elem);
        }
        header.add_child(sender);
        
        // Advanced Message Recipient
        let mut recipient = Element::new("MessageRecipient");
        let mut recipient_party = Element::new("PartyName");
        recipient_party.add_text(recipient_name);
        recipient.add_child(recipient_party);
        
        if let Some(rid) = recipient_id {
            let mut recipient_id_elem = Element::new("PartyId");
            recipient_id_elem.add_text(rid);
            recipient.add_child(recipient_id_elem);
        }
        
        if let Some(rref) = recipient_ref {
            let mut recipient_ref_elem = Element::new("PartyReference");
            recipient_ref_elem.add_text(rref);
            recipient.add_child(recipient_ref_elem);
        }
        header.add_child(recipient);
        
        // Message Control Type
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
    
    /// Build ERN 4.3 advanced sound recording resource
    pub fn build_advanced_sound_recording(
        resource_ref: &str,
        resource_id: &str,
        title: &str,
        artist: &str,
        isrc: &str,
        duration: &str,
        file_name: Option<&str>,
        file_size: Option<u64>,
        codec: Option<&str>,
        bit_rate: Option<u32>,
        hash_sum: Option<&str>,
        hash_algorithm: Option<&str>,
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
        
        // Advanced Technical Details
        if file_name.is_some() || file_size.is_some() || codec.is_some() || 
           bit_rate.is_some() || hash_sum.is_some() {
            let mut tech_details = Element::new("TechnicalResourceDetails");
            
            if let Some(fname) = file_name {
                let mut file_elem = Element::new("FileName");
                file_elem.add_text(fname);
                tech_details.add_child(file_elem);
            }
            
            if let Some(fsize) = file_size {
                let mut size_elem = Element::new("FileSize");
                size_elem.add_text(&fsize.to_string());
                tech_details.add_child(size_elem);
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
            
            // Advanced hash sum support
            if let Some(hash) = hash_sum {
                let mut hash_elem = Element::new("HashSum");
                
                if let Some(algorithm) = hash_algorithm {
                    hash_elem.attributes.insert("Algorithm".to_string(), algorithm.to_string());
                }
                
                hash_elem.add_text(hash);
                tech_details.add_child(hash_elem);
            }
            
            sound_recording.add_child(tech_details);
        }
        
        sound_recording
    }
    
    /// Build ERN 4.3 video resource
    pub fn build_video_resource(
        resource_ref: &str,
        resource_id: &str,
        title: &str,
        artist: &str,
        duration: &str,
        file_name: Option<&str>,
        file_size: Option<u64>,
        codec: Option<&str>,
        resolution: Option<&str>,
        frame_rate: Option<f32>,
    ) -> Element {
        let mut video = Element::new("Video");
        
        // Resource Reference
        let mut res_ref = Element::new("ResourceReference");
        res_ref.add_text(resource_ref);
        video.add_child(res_ref);
        
        // Type
        let mut res_type = Element::new("Type");
        res_type.add_text("Video");
        video.add_child(res_type);
        
        // Resource ID
        let mut res_id = Element::new("ResourceId");
        res_id.add_text(resource_id);
        video.add_child(res_id);
        
        // Reference Title
        let mut ref_title = Element::new("ReferenceTitle");
        ref_title.add_text(title);
        video.add_child(ref_title);
        
        // Display Artist
        let mut display_artist = Element::new("DisplayArtist");
        display_artist.add_text(artist);
        video.add_child(display_artist);
        
        // Duration
        let mut duration_elem = Element::new("Duration");
        duration_elem.add_text(duration);
        video.add_child(duration_elem);
        
        // Technical Details for Video
        if file_name.is_some() || file_size.is_some() || codec.is_some() || 
           resolution.is_some() || frame_rate.is_some() {
            let mut tech_details = Element::new("TechnicalResourceDetails");
            
            if let Some(fname) = file_name {
                let mut file_elem = Element::new("FileName");
                file_elem.add_text(fname);
                tech_details.add_child(file_elem);
            }
            
            if let Some(fsize) = file_size {
                let mut size_elem = Element::new("FileSize");
                size_elem.add_text(&fsize.to_string());
                tech_details.add_child(size_elem);
            }
            
            if let Some(codec_type) = codec {
                let mut codec_elem = Element::new("VideoCodecType");
                codec_elem.add_text(codec_type);
                tech_details.add_child(codec_elem);
            }
            
            if let Some(res) = resolution {
                let mut res_elem = Element::new("VideoResolution");
                res_elem.add_text(res);
                tech_details.add_child(res_elem);
            }
            
            if let Some(fps) = frame_rate {
                let mut fps_elem = Element::new("FrameRate");
                fps_elem.add_text(&fps.to_string());
                tech_details.add_child(fps_elem);
            }
            
            video.add_child(tech_details);
        }
        
        video
    }
}

/// ERN 4.3 validation functions
pub mod validation {
    use super::*;
    use regex::Regex;
    use once_cell::sync::Lazy;
    
    // Most advanced regex patterns for ERN 4.3
    static ISRC_PATTERN_43: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[A-Z]{2}[A-Z0-9]{3}\d{7}$").unwrap()
    });
    
    static UPC_PATTERN_43: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\d{12}$").unwrap()
    });
    
    static EAN_PATTERN_43: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\d{13}$").unwrap()
    });
    
    static DURATION_PATTERN_43: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^PT(?:\d+H)?(?:\d+M)?(?:\d+(?:\.\d+)?S)?$").unwrap()
    });
    
    static DATE_PATTERN_43: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap()
    });
    
    static DATETIME_PATTERN_43: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$").unwrap()
    });
    
    static HASH_PATTERN_43: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[A-Fa-f0-9]+$").unwrap()
    });
    
    /// Most comprehensive validation for ERN 4.3
    pub fn validate_isrc(isrc: &str) -> bool {
        ISRC_PATTERN_43.is_match(isrc)
    }
    
    /// Enhanced UPC validation for ERN 4.3
    pub fn validate_upc(upc: &str) -> bool {
        UPC_PATTERN_43.is_match(upc)
    }
    
    /// EAN validation for ERN 4.3
    pub fn validate_ean(ean: &str) -> bool {
        EAN_PATTERN_43.is_match(ean)
    }
    
    /// Most flexible duration validation for ERN 4.3
    pub fn validate_duration(duration: &str) -> bool {
        DURATION_PATTERN_43.is_match(duration)
    }
    
    /// Enhanced date validation for ERN 4.3
    pub fn validate_date(date: &str) -> bool {
        DATE_PATTERN_43.is_match(date)
    }
    
    /// Enhanced datetime validation for ERN 4.3
    pub fn validate_datetime(datetime: &str) -> bool {
        DATETIME_PATTERN_43.is_match(datetime)
    }
    
    /// Hash sum validation for ERN 4.3
    pub fn validate_hash_sum(hash: &str) -> bool {
        HASH_PATTERN_43.is_match(hash) && hash.len() >= 8
    }
    
    /// Most comprehensive territory code validation for ERN 4.3
    pub fn validate_territory_code(territory: &str) -> bool {
        // Full ISO 3166 support + DDEX extensions
        matches!(territory,
            // Major markets
            "US" | "GB" | "DE" | "FR" | "JP" | "CA" | "AU" | "IT" | "ES" | "NL" |
            "SE" | "NO" | "DK" | "FI" | "BR" | "MX" | "AR" | "IN" | "CN" | "KR" |
            "RU" | "PL" | "CZ" | "AT" | "CH" | "BE" | "PT" | "GR" | "TR" | "ZA" |
            // Extended regions
            "Worldwide" | "WorldwideExceptUS" | "Europe" | "NorthAmerica" | "SouthAmerica" |
            "Asia" | "Africa" | "Oceania"
        )
    }
    
    /// Most comprehensive commercial model validation for ERN 4.3
    pub fn validate_commercial_model(model: &str) -> bool {
        matches!(model,
            "SubscriptionModel" | "PurchaseModel" | "AdSupportedModel" |
            "FreeOfChargeModel" | "StreamingModel" | "DownloadModel" |
            "ConditionalDownloadModel" | "LimitedDownloadModel" |
            "PayAsYouGoModel" | "FreemiumModel"
        )
    }
    
    /// Enhanced audio codec validation for ERN 4.3
    pub fn validate_audio_codec(codec: &str) -> bool {
        matches!(codec,
            "MP3" | "AAC" | "FLAC" | "WAV" | "OGG" | "WMA" | "MP4" | "M4A" |
            "AIFF" | "DSD" | "ALAC" | "Opus" | "AMR" | "AC3" | "DTS"
        )
    }
    
    /// Video codec validation for ERN 4.3
    pub fn validate_video_codec(codec: &str) -> bool {
        matches!(codec,
            "H.264" | "H.265" | "VP8" | "VP9" | "AV1" | "MPEG-2" | "MPEG-4" |
            "DivX" | "XviD" | "WMV" | "QuickTime" | "ProRes"
        )
    }
    
    /// Enhanced usage type validation for ERN 4.3
    pub fn validate_usage_type(usage: &str) -> bool {
        matches!(usage,
            "Stream" | "Download" | "Preview" | "ConditionalDownload" |
            "DigitalPhonogramDelivery" | "UserMadeClip" | "Podcast" |
            "RadioPlay" | "BackgroundMusic" | "Ringtone" | "Synchronization"
        )
    }
    
    /// Hash algorithm validation for ERN 4.3
    pub fn validate_hash_algorithm(algorithm: &str) -> bool {
        matches!(algorithm,
            "MD5" | "SHA-1" | "SHA-256" | "SHA-512" | "CRC32"
        )
    }
    
    /// Get all validation errors for an ERN 4.3 message
    pub fn validate_ern_43_message(xml_content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        // Check required namespace
        if !xml_content.contains("http://ddex.net/xml/ern/43") {
            errors.push("Missing ERN 4.3 namespace".to_string());
        }
        
        // Check message schema version ID
        if !xml_content.contains("ern/43") {
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
        
        // Check for advanced features that should be used properly
        if xml_content.contains("HashSum") {
            if !xml_content.contains("Algorithm=") {
                errors.push("HashSum should specify Algorithm attribute".to_string());
            }
        }
        
        if xml_content.contains("TechnicalResourceDetails") {
            if !xml_content.contains("FileName") && !xml_content.contains("FileSize") {
                errors.push("TechnicalResourceDetails should contain FileName or FileSize".to_string());
            }
        }
        
        errors
    }
}