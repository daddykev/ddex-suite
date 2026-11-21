// src/streaming/element.rs
//! Parsed element types for streaming interface

use ddex_core::models::graph::*;
use ddex_core::models::versions::ERNVersion;
use ddex_core::models::Identifier;

/// Element yielded by streaming parser
#[derive(Debug, Clone)]
pub enum ParsedElement {
    /// Message header (yielded first)
    Header {
        sender: MessageSender,
        recipients: Vec<MessageRecipient>,
        message_id: Identifier,
        created_date_time: String,
        version: ERNVersion,
    },
    /// Complete release element
    Release(Release),
    /// Complete resource element
    Resource(Resource),
    /// Complete party element
    Party(Party),
    /// Complete deal element
    Deal(Deal),
    /// End of stream marker
    EndOfStream,
}

impl ParsedElement {
    /// Get the element type as a string
    pub fn element_type(&self) -> &'static str {
        match self {
            ParsedElement::Header { .. } => "Header",
            ParsedElement::Release(_) => "Release",
            ParsedElement::Resource(_) => "Resource",
            ParsedElement::Party(_) => "Party",
            ParsedElement::Deal(_) => "Deal",
            ParsedElement::EndOfStream => "EndOfStream",
        }
    }

    /// Estimate memory usage of this element
    pub fn memory_estimate(&self) -> usize {
        match self {
            ParsedElement::Header { .. } => std::mem::size_of::<MessageHeader>() + 1024,
            ParsedElement::Release(r) => estimate_release_size(r),
            ParsedElement::Resource(r) => estimate_resource_size(r),
            ParsedElement::Party(p) => estimate_party_size(p),
            ParsedElement::Deal(d) => estimate_deal_size(d),
            ParsedElement::EndOfStream => std::mem::size_of::<ParsedElement>(),
        }
    }

    /// Get a reference ID if applicable
    pub fn reference_id(&self) -> Option<&str> {
        match self {
            ParsedElement::Release(r) => Some(&r.release_reference),
            ParsedElement::Resource(r) => Some(&r.resource_reference),
            ParsedElement::Party(p) => p.party_id.first().map(|id| id.value.as_str()),
            ParsedElement::Deal(d) => d.deal_reference.as_deref(),
            _ => None,
        }
    }

    /// Check if this element is complete and ready for consumption
    pub fn is_complete(&self) -> bool {
        match self {
            ParsedElement::Header {
                sender, message_id, ..
            } => !sender.party_name.is_empty() && !message_id.value.is_empty(),
            ParsedElement::Release(r) => {
                !r.release_reference.is_empty() && !r.release_title.is_empty()
            }
            ParsedElement::Resource(r) => {
                !r.resource_reference.is_empty() && !r.reference_title.is_empty()
            }
            ParsedElement::Party(p) => !p.party_id.is_empty() && !p.party_name.is_empty(),
            // DealReference is optional per DDEX ERN spec (minOccurs="0").
            // A Deal is complete if it has release references linking it to releases.
            ParsedElement::Deal(d) => !d.deal_release_reference.is_empty(),
            ParsedElement::EndOfStream => true,
        }
    }
}

/// Rough memory estimation for release
fn estimate_release_size(release: &Release) -> usize {
    let mut size = std::mem::size_of::<Release>();
    size += release.release_reference.len();
    size += release.release_id.len() * std::mem::size_of::<Identifier>();
    size += release.release_title.len() * 100; // Estimate for LocalizedString
    size += release.display_artist.len() * std::mem::size_of::<Artist>();
    size += release
        .genre
        .iter()
        .map(|g| g.genre_text.len())
        .sum::<usize>();
    size += release.release_resource_reference_list.len()
        * std::mem::size_of::<ReleaseResourceReference>();
    size
}

/// Rough memory estimation for resource
fn estimate_resource_size(resource: &Resource) -> usize {
    let mut size = std::mem::size_of::<Resource>();
    size += resource.resource_reference.len();
    size += std::mem::size_of::<ResourceType>();
    size += resource.resource_id.len() * std::mem::size_of::<Identifier>();
    size += resource.reference_title.len() * 100; // Estimate for LocalizedString
    size
}

/// Rough memory estimation for party
fn estimate_party_size(party: &Party) -> usize {
    let mut size = std::mem::size_of::<Party>();
    size += party
        .party_id
        .iter()
        .map(|id| id.value.len())
        .sum::<usize>();
    size += party.party_name.len() * 100; // Estimate for LocalizedString
    size += party.party_id.len() * std::mem::size_of::<Identifier>();
    size
}

/// Rough memory estimation for deal
fn estimate_deal_size(deal: &Deal) -> usize {
    let mut size = std::mem::size_of::<Deal>();
    size += deal.deal_reference.as_ref().map_or(0, |r| r.len());
    size +=
        deal.deal_terms.commercial_model_type.len() * std::mem::size_of::<CommercialModelType>();
    size += deal.deal_terms.use_type.len() * std::mem::size_of::<UseType>();
    size += deal
        .deal_terms
        .territory_code
        .iter()
        .map(|t| t.len())
        .sum::<usize>();
    // Add DealTerms size estimate
    size += std::mem::size_of::<DealTerms>();
    size
}

/// Builder for ParsedElement::Header
#[derive(Debug, Default)]
pub struct HeaderBuilder {
    sender: Option<MessageSender>,
    recipients: Vec<MessageRecipient>,
    message_id: Option<Identifier>,
    created_date_time: Option<String>,
    version: Option<ERNVersion>,
}

impl HeaderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sender(mut self, sender: MessageSender) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn add_recipient(mut self, recipient: MessageRecipient) -> Self {
        self.recipients.push(recipient);
        self
    }

    pub fn message_id(mut self, id: Identifier) -> Self {
        self.message_id = Some(id);
        self
    }

    pub fn created_date_time(mut self, datetime: String) -> Self {
        self.created_date_time = Some(datetime);
        self
    }

    pub fn version(mut self, version: ERNVersion) -> Self {
        self.version = Some(version);
        self
    }

    pub fn build(self) -> Result<ParsedElement, String> {
        let sender = self.sender.ok_or("Missing sender")?;
        let message_id = self.message_id.ok_or("Missing message ID")?;
        let created_date_time = self.created_date_time.ok_or("Missing created date time")?;
        let version = self.version.ok_or("Missing version")?;

        Ok(ParsedElement::Header {
            sender,
            recipients: self.recipients,
            message_id,
            created_date_time,
            version,
        })
    }
}
