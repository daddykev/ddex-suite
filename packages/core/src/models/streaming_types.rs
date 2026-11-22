//! Streaming-friendly types for incremental DDEX parsing
//!
//! This module provides builder patterns and streaming-optimized types
//! that can handle partial data during streaming XML parsing.

use super::common::{Copyright, Identifier, LocalizedString};
use super::graph::*;
use super::*;
use serde::{Deserialize, Serialize};

/// Error types for conversion operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversionError {
    MissingRequired(String),
    InvalidFormat {
        field: String,
        value: String,
        expected: String,
    },
    ValidationFailed(String),
    ReferenceNotResolved(String),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::MissingRequired(field) => {
                write!(f, "Missing required field: {}", field)
            }
            ConversionError::InvalidFormat {
                field,
                value,
                expected,
            } => {
                write!(
                    f,
                    "Invalid format for {}: got '{}', expected {}",
                    field, value, expected
                )
            }
            ConversionError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            ConversionError::ReferenceNotResolved(ref_id) => {
                write!(f, "Reference not resolved: {}", ref_id)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

/// Conversion trait for streaming types to core types
pub trait ToCore {
    type Output;
    fn to_core(self) -> Result<Self::Output, ConversionError>;
}

/// Validation trait for builders
pub trait Validate {
    fn validate(&self) -> Result<(), ConversionError>;
    fn is_complete(&self) -> bool;
}

/// Builder for Release with incremental population
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReleaseBuilder {
    pub release_reference: Option<String>,
    pub release_id: Vec<Identifier>,
    pub release_title: Vec<LocalizedString>,
    pub release_subtitle: Option<Vec<LocalizedString>>,
    pub release_type: Option<ReleaseType>,
    pub genre: Vec<Genre>,
    pub release_resource_reference_list: Vec<ReleaseResourceReference>,
    pub display_artist: Vec<Artist>,
    pub party_list: Vec<ReleaseParty>,
    pub release_date: Vec<ReleaseEvent>,
    pub territory_code: Vec<String>,
    pub excluded_territory_code: Vec<String>,
    pub attributes: Option<AttributeMap>,
    pub extensions: Option<Extensions>,
    pub comments: Option<Vec<Comment>>,

    // Streaming-specific fields
    pub pending_resource_refs: Vec<String>,
    pub pending_party_refs: Vec<String>,
    pub field_count: usize,
}

impl ReleaseBuilder {
    pub fn new(reference: String) -> Self {
        Self {
            release_reference: Some(reference),
            ..Default::default()
        }
    }

    pub fn add_title(&mut self, title: LocalizedString) {
        self.release_title.push(title);
        self.field_count += 1;
    }

    pub fn add_genre(&mut self, genre: Genre) {
        self.genre.push(genre);
        self.field_count += 1;
    }

    pub fn add_resource_reference(&mut self, resource_ref: String) {
        self.pending_resource_refs.push(resource_ref.clone());
        self.release_resource_reference_list
            .push(ReleaseResourceReference {
                resource_reference: resource_ref,
                sequence_number: Some(self.release_resource_reference_list.len() as i32 + 1),
                disc_number: Some(1),
                track_number: Some(self.release_resource_reference_list.len() as i32 + 1),
                side: None,
                is_hidden: false,
                is_bonus: false,
                extensions: None,
                comments: None,
            });
        self.field_count += 1;
    }

    pub fn add_artist(&mut self, artist: Artist) {
        self.display_artist.push(artist);
        self.field_count += 1;
    }

    pub fn set_release_type(&mut self, release_type: ReleaseType) {
        self.release_type = Some(release_type);
        self.field_count += 1;
    }

    pub fn add_release_date(&mut self, event: ReleaseEvent) {
        self.release_date.push(event);
        self.field_count += 1;
    }
}

impl Validate for ReleaseBuilder {
    fn validate(&self) -> Result<(), ConversionError> {
        if self.release_reference.is_none() {
            return Err(ConversionError::MissingRequired(
                "release_reference".to_string(),
            ));
        }
        if self.release_title.is_empty() {
            return Err(ConversionError::MissingRequired(
                "release_title".to_string(),
            ));
        }
        Ok(())
    }

    fn is_complete(&self) -> bool {
        self.release_reference.is_some() && !self.release_title.is_empty()
    }
}

impl ToCore for ReleaseBuilder {
    type Output = Release;

    fn to_core(self) -> Result<Self::Output, ConversionError> {
        self.validate()?;

        Ok(Release {
            release_reference: self.release_reference.unwrap(),
            release_id: self.release_id,
            release_title: self.release_title,
            release_subtitle: self.release_subtitle,
            release_type: self.release_type,
            genre: self.genre,
            release_resource_reference_list: self.release_resource_reference_list,
            display_artist: self.display_artist,
            party_list: self.party_list,
            release_date: self.release_date,
            territory_code: self.territory_code,
            excluded_territory_code: self.excluded_territory_code,
            attributes: self.attributes,
            extensions: self.extensions,
            comments: self.comments,
            display_title_text: Vec::new(),
            display_title: Vec::new(),
            display_artist_name: Vec::new(),
            resource_group: None,
            release_type_user_defined_value: None,
            parental_warning_type: Vec::new(),
            release_label_reference: Vec::new(),
            c_line: Vec::new(),
            label_name: Vec::new(),
            original_release_date: Vec::new(),
            p_line: Vec::new(),
            parental_warnings: Vec::new(),
        })
    }
}

/// Builder for Resource with incremental population
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceBuilder {
    pub resource_reference: Option<String>,
    pub resource_type: Option<ResourceType>,
    pub resource_id: Vec<Identifier>,
    pub reference_title: Vec<LocalizedString>,
    pub duration: Option<std::time::Duration>,
    pub technical_details: Vec<TechnicalDetails>,
    pub rights_controller: Vec<String>,
    pub p_line: Vec<Copyright>,
    pub c_line: Vec<Copyright>,
    pub extensions: Option<Extensions>,

    // Streaming-specific fields
    pub field_count: usize,
    pub duration_text: Option<String>, // For parsing duration from text
}

impl ResourceBuilder {
    pub fn new(reference: String) -> Self {
        Self {
            resource_reference: Some(reference),
            resource_type: Some(ResourceType::SoundRecording), // Default
            ..Default::default()
        }
    }

    pub fn add_title(&mut self, title: LocalizedString) {
        self.reference_title.push(title);
        self.field_count += 1;
    }

    pub fn set_duration_from_text(&mut self, duration_text: String) {
        self.duration_text = Some(duration_text.clone());
        // Try to parse duration
        if let Ok(seconds) = duration_text.parse::<u64>() {
            self.duration = Some(std::time::Duration::from_secs(seconds));
        }
        self.field_count += 1;
    }

    pub fn set_resource_type(&mut self, resource_type: ResourceType) {
        self.resource_type = Some(resource_type);
        self.field_count += 1;
    }

    pub fn add_identifier(&mut self, identifier: Identifier) {
        self.resource_id.push(identifier);
        self.field_count += 1;
    }

    pub fn add_technical_details(&mut self, details: TechnicalDetails) {
        self.technical_details.push(details);
        self.field_count += 1;
    }
}

impl Validate for ResourceBuilder {
    fn validate(&self) -> Result<(), ConversionError> {
        if self.resource_reference.is_none() {
            return Err(ConversionError::MissingRequired(
                "resource_reference".to_string(),
            ));
        }
        if self.reference_title.is_empty() {
            return Err(ConversionError::MissingRequired(
                "reference_title".to_string(),
            ));
        }
        Ok(())
    }

    fn is_complete(&self) -> bool {
        self.resource_reference.is_some() && !self.reference_title.is_empty()
    }
}

impl ToCore for ResourceBuilder {
    type Output = Resource;

    fn to_core(self) -> Result<Self::Output, ConversionError> {
        self.validate()?;

        Ok(Resource {
            resource_reference: self.resource_reference.unwrap(),
            resource_type: self.resource_type.unwrap_or(ResourceType::SoundRecording),
            resource_id: self.resource_id,
            reference_title: self.reference_title,
            duration: self.duration,
            technical_details: self.technical_details,
            rights_controller: self.rights_controller,
            p_line: self.p_line,
            c_line: self.c_line,
            extensions: self.extensions,
            display_title_text: Vec::new(),
            display_title: Vec::new(),
            display_artist_name: Vec::new(),
            display_artist: Vec::new(),
            type_value: None,
            type_user_defined_value: None,
            parental_warning_type: Vec::new(),
            genre: Vec::new(),
            label_name: Vec::new(),
            parental_warnings: Vec::new(),
        })
    }
}

/// Builder for Party with incremental population
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartyBuilder {
    pub party_id: Vec<Identifier>,
    pub isni: Option<String>,
    pub ipi: Option<String>,
    pub party_name: Vec<LocalizedString>,
    pub party_role: Vec<PartyRole>,
    pub contact_details: Option<ContactDetails>,

    // Streaming-specific fields
    pub party_reference: Option<String>, // For referencing
    pub field_count: usize,
}

impl PartyBuilder {
    pub fn new(reference: Option<String>) -> Self {
        Self {
            party_reference: reference,
            ..Default::default()
        }
    }

    pub fn add_name(&mut self, name: LocalizedString) {
        self.party_name.push(name);
        self.field_count += 1;
    }

    pub fn add_identifier(&mut self, identifier: Identifier) {
        self.party_id.push(identifier);
        self.field_count += 1;
    }

    pub fn add_role(&mut self, role: PartyRole) {
        self.party_role.push(role);
        self.field_count += 1;
    }

    pub fn set_isni(&mut self, isni: String) {
        self.isni = Some(isni);
        self.field_count += 1;
    }
}

impl Validate for PartyBuilder {
    fn validate(&self) -> Result<(), ConversionError> {
        if self.party_name.is_empty() {
            return Err(ConversionError::MissingRequired("party_name".to_string()));
        }
        Ok(())
    }

    fn is_complete(&self) -> bool {
        !self.party_name.is_empty()
    }
}

impl ToCore for PartyBuilder {
    type Output = Party;

    fn to_core(self) -> Result<Self::Output, ConversionError> {
        self.validate()?;

        Ok(Party {
            party_id: self.party_id,
            isni: self.isni,
            ipi: self.ipi,
            party_name: self.party_name,
            party_role: self.party_role,
            contact_details: self.contact_details,
            party_reference: None,
        })
    }
}

/// Builder for MessageHeader with incremental population
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageHeaderBuilder {
    pub message_id: Option<String>,
    pub message_type: Option<MessageType>,
    pub message_created_date_time: Option<chrono::DateTime<chrono::Utc>>,
    pub message_sender: Option<MessageSender>,
    pub message_recipient: Option<MessageRecipient>,
    pub message_control_type: Option<MessageControlType>,
    pub message_thread_id: Option<String>,
    pub attributes: Option<AttributeMap>,
    pub extensions: Option<Extensions>,
    pub comments: Option<Vec<Comment>>,

    // Streaming-specific fields
    pub field_count: usize,
    pub created_date_time_text: Option<String>, // For parsing
}

impl MessageHeaderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_message_id(&mut self, id: String) {
        self.message_id = Some(id);
        self.field_count += 1;
    }

    pub fn set_created_date_time_from_text(&mut self, date_text: String) {
        self.created_date_time_text = Some(date_text.clone());
        // Try to parse the date
        if let Ok(date_time) = chrono::DateTime::parse_from_rfc3339(&date_text) {
            self.message_created_date_time = Some(date_time.with_timezone(&chrono::Utc));
        }
        self.field_count += 1;
    }

    pub fn set_sender(&mut self, sender: MessageSender) {
        self.message_sender = Some(sender);
        self.field_count += 1;
    }

    pub fn set_recipient(&mut self, recipient: MessageRecipient) {
        self.message_recipient = Some(recipient);
        self.field_count += 1;
    }

    pub fn set_message_type(&mut self, msg_type: MessageType) {
        self.message_type = Some(msg_type);
        self.field_count += 1;
    }
}

impl Validate for MessageHeaderBuilder {
    fn validate(&self) -> Result<(), ConversionError> {
        if self.message_id.is_none() {
            return Err(ConversionError::MissingRequired("message_id".to_string()));
        }
        if self.message_created_date_time.is_none() {
            return Err(ConversionError::MissingRequired(
                "message_created_date_time".to_string(),
            ));
        }
        if self.message_sender.is_none() {
            return Err(ConversionError::MissingRequired(
                "message_sender".to_string(),
            ));
        }
        if self.message_recipient.is_none() {
            return Err(ConversionError::MissingRequired(
                "message_recipient".to_string(),
            ));
        }
        Ok(())
    }

    fn is_complete(&self) -> bool {
        self.message_id.is_some()
            && self.message_created_date_time.is_some()
            && self.message_sender.is_some()
            && self.message_recipient.is_some()
    }
}

impl ToCore for MessageHeaderBuilder {
    type Output = MessageHeader;

    fn to_core(self) -> Result<Self::Output, ConversionError> {
        self.validate()?;

        Ok(MessageHeader {
            message_id: self.message_id.unwrap(),
            message_type: self.message_type.unwrap_or(MessageType::NewReleaseMessage),
            message_created_date_time: self.message_created_date_time.unwrap(),
            message_sender: self.message_sender.unwrap(),
            message_recipient: self.message_recipient.unwrap(),
            message_control_type: self.message_control_type,
            message_thread_id: self.message_thread_id,
            attributes: self.attributes,
            extensions: self.extensions,
            comments: self.comments,
        })
    }
}

/// Utility functions for creating streaming types from text data
pub mod builders {
    use super::*;

    pub fn create_localized_string(text: String, language: Option<String>) -> LocalizedString {
        LocalizedString {
            text,
            language_code: language,
            script: None,
        }
    }

    pub fn create_identifier(
        value: String,
        id_type: IdentifierType,
        namespace: Option<String>,
    ) -> Identifier {
        Identifier {
            id_type,
            namespace,
            value,
        }
    }

    pub fn create_genre(text: String, sub_genre: Option<String>) -> Genre {
        Genre {
            genre_text: text,
            sub_genre,
            attributes: None,
            extensions: None,
            comments: None,
            scope: Default::default(),
        }
    }

    pub fn create_artist(name: String, role: String, party_ref: Option<String>) -> Artist {
        Artist {
            party_reference: party_ref,
            artist_role: vec![role],
            display_artist_name: vec![create_localized_string(name, None)],
            sequence_number: None,
        }
    }

    pub fn create_message_sender(name: String, id_value: Option<String>) -> MessageSender {
        let mut party_ids = vec![];
        if let Some(id) = id_value {
            party_ids.push(create_identifier(id, IdentifierType::Proprietary, None));
        }

        MessageSender {
            party_id: party_ids,
            party_name: vec![create_localized_string(name, None)],
            trading_name: None,
            attributes: None,
            extensions: None,
            comments: None,
        }
    }

    pub fn create_message_recipient(name: String) -> MessageRecipient {
        MessageRecipient {
            party_id: vec![],
            party_name: vec![create_localized_string(name, None)],
            trading_name: None,
            attributes: None,
            extensions: None,
            comments: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::builders::*;
    use super::*;

    #[test]
    fn test_release_builder() {
        let mut builder = ReleaseBuilder::new("REL001".to_string());
        builder.add_title(create_localized_string(
            "Test Release".to_string(),
            Some("en".to_string()),
        ));
        builder.add_genre(create_genre("Rock".to_string(), None));
        builder.set_release_type(ReleaseType::Album);

        assert!(builder.is_complete());

        let release = builder.to_core().unwrap();
        assert_eq!(release.release_reference, "REL001");
        assert_eq!(release.release_title[0].text, "Test Release");
        assert_eq!(release.genre[0].genre_text, "Rock");
        assert_eq!(release.release_type, Some(ReleaseType::Album));
    }

    #[test]
    fn test_resource_builder() {
        let mut builder = ResourceBuilder::new("RES001".to_string());
        builder.add_title(create_localized_string("Test Track".to_string(), None));
        builder.set_duration_from_text("180".to_string());

        assert!(builder.is_complete());

        let resource = builder.to_core().unwrap();
        assert_eq!(resource.resource_reference, "RES001");
        assert_eq!(resource.reference_title[0].text, "Test Track");
        assert_eq!(resource.duration, Some(std::time::Duration::from_secs(180)));
    }

    #[test]
    fn test_message_header_builder() {
        let mut builder = MessageHeaderBuilder::new();
        builder.set_message_id("MSG001".to_string());
        builder.set_created_date_time_from_text("2023-01-01T00:00:00Z".to_string());
        builder.set_sender(create_message_sender(
            "Test Sender".to_string(),
            Some("SENDER001".to_string()),
        ));
        builder.set_recipient(create_message_recipient("Test Recipient".to_string()));

        assert!(builder.is_complete());

        let header = builder.to_core().unwrap();
        assert_eq!(header.message_id, "MSG001");
        assert_eq!(header.message_sender.party_name[0].text, "Test Sender");
    }

    #[test]
    fn test_validation_errors() {
        let builder = ReleaseBuilder::default(); // No reference or title
        assert!(!builder.is_complete());

        let result = builder.to_core();
        assert!(result.is_err());

        if let Err(ConversionError::MissingRequired(field)) = result {
            assert_eq!(field, "release_reference");
        } else {
            panic!("Expected MissingRequired error");
        }
    }

    #[test]
    fn test_builder_utility_functions() {
        let localized = create_localized_string("Test".to_string(), Some("en".to_string()));
        assert_eq!(localized.text, "Test");
        assert_eq!(localized.language_code, Some("en".to_string()));

        let identifier = create_identifier("TEST123".to_string(), IdentifierType::ISRC, None);
        assert_eq!(identifier.value, "TEST123");
        assert_eq!(identifier.id_type, IdentifierType::ISRC);

        let genre = create_genre("Rock".to_string(), Some("Alternative".to_string()));
        assert_eq!(genre.genre_text, "Rock");
        assert_eq!(genre.sub_genre, Some("Alternative".to_string()));
    }
}
