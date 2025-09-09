//! AST generation from BuildRequest

pub mod xml_writer;

use crate::ast::{AST, Element};
use crate::builder::{BuildRequest, MessageHeaderRequest, ReleaseRequest};
use crate::error::BuildError;
use indexmap::IndexMap;
use uuid::Uuid;
use chrono::Utc;

pub struct ASTGenerator {
    version: String,
    id_counter: usize,
}

impl ASTGenerator {
    pub fn new(version: String) -> Self {
        Self {
            version,
            id_counter: 0,
        }
    }
    
    pub fn generate(&mut self, request: &BuildRequest) -> Result<AST, BuildError> {
        // Create root element based on version
        let mut root = self.create_root_element(&request.version)?;
        
        // Add MessageHeader
        let header = self.generate_message_header(&request.header)?;
        root.add_child(header);
        
        // Add UpdateIndicator (required)
        root.add_child(Element::new("UpdateIndicator").with_text("OriginalMessage"));
        
        // Add ResourceList
        if !request.releases.is_empty() {
            let resource_list = self.generate_resource_list(&request.releases)?;
            root.add_child(resource_list);
        }
        
        // Add ReleaseList
        if !request.releases.is_empty() {
            let release_list = self.generate_release_list(&request.releases)?;
            root.add_child(release_list);
        }
        
        // Add DealList
        if !request.deals.is_empty() {
            let deal_list = self.generate_deal_list(&request.deals)?;
            root.add_child(deal_list);
        }
        
        // Create AST with namespaces
        let namespaces = self.get_namespaces(&request.version);
        
        Ok(AST {
            root,
            namespaces,
            schema_location: self.get_schema_location(&request.version),
        })
    }
    
    fn create_root_element(&self, version: &str) -> Result<Element, BuildError> {
        let mut root = match version {
            "4.3" => Element::new("NewReleaseMessage"),
            "4.2" => Element::new("NewReleaseMessage"),
            "3.8.2" => Element::new("NewReleaseMessage"),
            _ => return Err(BuildError::InvalidFormat {
                field: "version".to_string(),
                message: format!("Unsupported version: {}", version),
            }),
        };
        
        // Add version attributes
        root.attributes.insert("MessageSchemaVersionId".to_string(), format!("ern/{}", version));
        root.attributes.insert("LanguageAndScriptCode".to_string(), "en".to_string());
        
        Ok(root)
    }
    
    fn generate_message_header(&mut self, header: &MessageHeaderRequest) -> Result<Element, BuildError> {
        let mut header_elem = Element::new("MessageHeader");
        
        // MessageId
        let message_id = header.message_id.clone()
            .unwrap_or_else(|| format!("MSG_{}", Uuid::new_v4()));
        header_elem.add_child(Element::new("MessageId").with_text(message_id));
        
        // MessageCreatedDateTime
        let created = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        header_elem.add_child(Element::new("MessageCreatedDateTime").with_text(created));
        
        // MessageSender
        let sender = self.generate_party("MessageSender", &header.message_sender)?;
        header_elem.add_child(sender);
        
        // MessageRecipient
        let recipient = self.generate_party("MessageRecipient", &header.message_recipient)?;
        header_elem.add_child(recipient);
        
        // MessageControlType (optional)
        if let Some(control_type) = &header.message_control_type {
            header_elem.add_child(Element::new("MessageControlType").with_text(control_type));
        }
        
        Ok(header_elem)
    }
    
    fn generate_party(&self, element_name: &str, party: &crate::builder::PartyRequest) -> Result<Element, BuildError> {
        let mut party_elem = Element::new(element_name);
        
        // PartyName
        for name in &party.party_name {
            let mut party_name = Element::new("PartyName");
            party_name.add_child(Element::new("FullName").with_text(&name.text));
            if let Some(lang) = &name.language_code {
                party_name.attributes.insert("LanguageAndScriptCode".to_string(), lang.clone());
            }
            party_elem.add_child(party_name);
        }
        
        // PartyId
        if let Some(id) = &party.party_id {
            party_elem.add_child(
                Element::new("PartyId").with_text(id)
            );
        }
        
        Ok(party_elem)
    }
    
    fn generate_resource_list(&mut self, releases: &[ReleaseRequest]) -> Result<Element, BuildError> {
        let mut resource_list = Element::new("ResourceList");
        
        // Generate SoundRecording for each track
        for release in releases {
            for (_idx, track) in release.tracks.iter().enumerate() {
                let mut sound_recording = Element::new("SoundRecording");
                
                // ResourceReference
                let resource_ref = format!("A{}", self.next_id());
                sound_recording.add_child(
                    Element::new("ResourceReference").with_text(&resource_ref)
                );
                
                // ResourceId
                if let Some(isrc) = &track.isrc {
                    let mut resource_id = Element::new("ResourceId");
                    resource_id.add_child(Element::new("ISRC").with_text(isrc));
                    sound_recording.add_child(resource_id);
                }
                
                // ReferenceTitle
                let mut ref_title = Element::new("ReferenceTitle");
                ref_title.add_child(Element::new("TitleText").with_text(&track.title));
                sound_recording.add_child(ref_title);
                
                // Duration (ISO 8601)
                let duration = format!("PT{}M{}S", track.duration / 60, track.duration % 60);
                sound_recording.add_child(Element::new("Duration").with_text(duration));
                
                resource_list.add_child(sound_recording);
            }
        }
        
        Ok(resource_list)
    }
    
    fn generate_release_list(&mut self, releases: &[ReleaseRequest]) -> Result<Element, BuildError> {
        let mut release_list = Element::new("ReleaseList");
        
        for (_idx, release) in releases.iter().enumerate() {
            let mut release_elem = Element::new("Release");
            
            // ReleaseReference
            let release_ref = format!("R{}", self.next_id());
            release_elem.add_child(
                Element::new("ReleaseReference").with_text(&release_ref)
            );
            
            // ReleaseId
            let mut release_id = Element::new("ReleaseId");
            release_id.add_child(Element::new("ICPN").with_text(&release.release_id));
            release_elem.add_child(release_id);
            
            // ReferenceTitle
            for title in &release.title {
                let mut ref_title = Element::new("ReferenceTitle");
                ref_title.add_child(Element::new("TitleText").with_text(&title.text));
                if let Some(lang) = &title.language_code {
                    ref_title.attributes.insert("LanguageAndScriptCode".to_string(), lang.clone());
                }
                release_elem.add_child(ref_title);
            }
            
            // ReleaseResourceReferenceList
            let mut resource_ref_list = Element::new("ReleaseResourceReferenceList");
            for (track_idx, _track) in release.tracks.iter().enumerate() {
                let mut rr_ref = Element::new("ReleaseResourceReference");
                rr_ref.add_child(Element::new("SequenceNumber").with_text((track_idx + 1).to_string()));
                rr_ref.add_child(Element::new("ReleaseResourceReference").with_text(format!("A{}", track_idx + 1)));
                resource_ref_list.add_child(rr_ref);
            }
            release_elem.add_child(resource_ref_list);
            
            // Add required DisplayArtist
            let mut display_artist = Element::new("DisplayArtist");
            display_artist.add_child(
                Element::new("PartyName")
                    .with_child(Element::new("FullName").with_text(&release.artist))
            );
            release_elem.add_child(display_artist);
            
            release_list.add_child(release_elem);
        }
        
        Ok(release_list)
    }
    
    fn generate_deal_list(&mut self, deals: &[crate::builder::DealRequest]) -> Result<Element, BuildError> {
        let mut deal_list = Element::new("DealList");
        
        for deal in deals {
            let mut release_deal = Element::new("ReleaseDeal");
            
            // DealReleaseReference
            release_deal.add_child(Element::new("DealReleaseReference").with_text("R1"));
            
            // Deal
            let mut deal_elem = Element::new("Deal");
            
            // DealTerms
            let mut deal_terms = Element::new("DealTerms");
            
            // TerritoryCode
            for territory in &deal.territories {
                deal_terms.add_child(Element::new("TerritoryCode").with_text(territory));
            }
            
            // ValidityPeriod
            if deal.start_date.is_some() || deal.end_date.is_some() {
                let mut validity = Element::new("ValidityPeriod");
                if let Some(start) = &deal.start_date {
                    validity.add_child(Element::new("StartDate").with_text(start));
                }
                if let Some(end) = &deal.end_date {
                    validity.add_child(Element::new("EndDate").with_text(end));
                }
                deal_terms.add_child(validity);
            }
            
            deal_elem.add_child(deal_terms);
            release_deal.add_child(deal_elem);
            deal_list.add_child(release_deal);
        }
        
        Ok(deal_list)
    }
    
    fn get_namespaces(&self, version: &str) -> IndexMap<String, String> {
        let mut namespaces = IndexMap::new();
        
        match version {
            "4.3" => {
                namespaces.insert("ern".to_string(), "http://ddex.net/xml/ern/43".to_string());
                namespaces.insert("xsi".to_string(), "http://www.w3.org/2001/XMLSchema-instance".to_string());
            }
            "4.2" => {
                namespaces.insert("ern".to_string(), "http://ddex.net/xml/ern/42".to_string());
                namespaces.insert("xsi".to_string(), "http://www.w3.org/2001/XMLSchema-instance".to_string());
            }
            "3.8.2" => {
                namespaces.insert("ern".to_string(), "http://ddex.net/xml/ern/382".to_string());
                namespaces.insert("xsi".to_string(), "http://www.w3.org/2001/XMLSchema-instance".to_string());
            }
            _ => {}
        }
        
        namespaces
    }
    
    fn get_schema_location(&self, version: &str) -> Option<String> {
        match version {
            "4.3" => Some("http://ddex.net/xml/ern/43 http://ddex.net/xml/ern/43/release-notification.xsd".to_string()),
            "4.2" => Some("http://ddex.net/xml/ern/42 http://ddex.net/xml/ern/42/release-notification.xsd".to_string()),
            "3.8.2" => Some("http://ddex.net/xml/ern/382 http://ddex.net/xml/ern/382/release-notification.xsd".to_string()),
            _ => None,
        }
    }
    
    fn next_id(&mut self) -> usize {
        self.id_counter += 1;
        self.id_counter
    }
}

// Extension trait for Element builder pattern
impl Element {
    pub fn with_child(mut self, child: Element) -> Self {
        self.add_child(child);
        self
    }
}