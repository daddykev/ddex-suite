// core/src/transform/graph.rs
// Remove unused imports and variables
use crate::error::ParseError;
use crate::parser::namespace_detector::NamespaceContext;
use crate::parser::xml_validator::XmlValidator;
use crate::transform::{ern38, ern4};
use ddex_core::models::graph::{
    ERNMessage, MessageControlType, MessageHeader, MessageRecipient, MessageSender, MessageType,
    Release,
};
use ddex_core::models::versions::ERNVersion;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::BufRead;

/// Which list the main loop is currently inside (ERN 4.x dispatch).
#[derive(Clone, Copy)]
struct Ern4Context {
    in_release_list: bool,
    in_resource_list: bool,
    in_party_list: bool,
}

/// Where ERN 4.x elements are collected.
struct Ern4Sinks<'a> {
    releases: &'a mut Vec<Release>,
    track_releases: &'a mut Vec<ddex_core::models::graph::TrackRelease>,
    resources: &'a mut Vec<ddex_core::models::graph::Resource>,
    parties: &'a mut Vec<ddex_core::models::graph::Party>,
}

pub struct GraphBuilder {
    version: ERNVersion,
}

impl GraphBuilder {
    pub fn new(version: ERNVersion) -> Self {
        Self { version }
    }

    pub fn build_from_xml<R: BufRead + std::io::Seek>(
        &self,
        reader: R,
    ) -> Result<ERNMessage, ParseError> {
        self.build_from_xml_with_security_config(
            reader,
            &crate::parser::security::SecurityConfig::default(),
        )
    }

    pub fn build_from_xml_with_security_config<R: BufRead + std::io::Seek>(
        &self,
        mut reader: R,
        _security_config: &crate::parser::security::SecurityConfig,
    ) -> Result<ERNMessage, ParseError> {
        let mut xml_reader = Reader::from_reader(&mut reader);

        // Enable strict XML validation
        // Text is kept exactly as written (whitespace-only text between elements is
        // formatting and never read as a value).
        xml_reader.config_mut().trim_text(false);
        xml_reader.config_mut().check_end_names = true;
        xml_reader.config_mut().expand_empty_elements = false;

        // Parse the actual header from XML
        let message_header = self.parse_header_from_xml(&mut xml_reader)?;

        // Reset reader to start for main parsing loop
        reader.seek(std::io::SeekFrom::Start(0))?;
        xml_reader = Reader::from_reader(&mut reader);
        // Text is kept exactly as written (whitespace-only text between elements is
        // formatting and never read as a value).
        xml_reader.config_mut().trim_text(false);
        xml_reader.config_mut().check_end_names = true;
        xml_reader.config_mut().expand_empty_elements = false;

        let mut validator = XmlValidator::strict();
        let mut releases = Vec::new();
        let mut resources = Vec::new(); // Made mutable to collect parsed resources
        let mut parties = Vec::new();
        let mut track_releases = Vec::new();
        let mut deals = Vec::new(); // Made mutable to collect parsed deals
                                    // ERN 4.x structures (TrackRelease, PartyList, ResourceGroup trees, display texts)
                                    // are read by `ern4`; 3.8.x keeps its original code path unchanged.
        let is_ern4 = matches!(self.version, ERNVersion::V4_2 | ERNVersion::V4_3);
        let mut in_party_list = false;
        // ERN 4.x root attributes (LanguageAndScriptCode, AvsVersionId, ...)
        let mut root_attributes = None;

        // Parse with XML validation and depth tracking
        let mut buf = Vec::new();
        let mut in_release_list = false;
        let mut in_resource_list = false;
        let mut in_deal_list = false;

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(ref event) => {
                    // Validate XML structure
                    validator.validate_event(event, &xml_reader)?;

                    // Check depth limit
                    if validator.get_depth() > 100 {
                        return Err(ParseError::DepthLimitExceeded {
                            depth: validator.get_depth(),
                            limit: 100,
                        });
                    }

                    if is_ern4 && root_attributes.is_none() {
                        if let Event::Start(ref e) = event {
                            root_attributes = Some(ern4::root_attributes(e)?);
                        }
                    }

                    match event {
                        Event::Start(ref e)
                            if is_ern4
                                && self.handle_ern4_start(
                                    e,
                                    &mut xml_reader,
                                    &mut validator,
                                    Ern4Context {
                                        in_release_list,
                                        in_resource_list,
                                        in_party_list,
                                    },
                                    Ern4Sinks {
                                        releases: &mut releases,
                                        track_releases: &mut track_releases,
                                        resources: &mut resources,
                                        parties: &mut parties,
                                    },
                                )? => {}
                        Event::Start(ref e) => {
                            match e.name().as_ref() {
                                b"PartyList" if is_ern4 => in_party_list = true,
                                b"ReleaseList" => in_release_list = true,
                                b"ResourceList" => in_resource_list = true,
                                b"DealList" => in_deal_list = true,
                                b"Release" if in_release_list => {
                                    let node =
                                        ern4::read_subtree(&mut xml_reader, &mut validator, e)?;
                                    releases.push(ern38::release(&node, self.version));
                                }
                                name @ (b"SoundRecording" | b"Video" | b"Image" | b"Text"
                                | b"SheetMusic")
                                    if in_resource_list =>
                                {
                                    use ddex_core::models::graph::ResourceType;
                                    let resource_type = match name {
                                        b"SoundRecording" => ResourceType::SoundRecording,
                                        b"Video" => ResourceType::Video,
                                        b"Image" => ResourceType::Image,
                                        b"Text" => ResourceType::Text,
                                        _ => ResourceType::SheetMusic,
                                    };
                                    // Record every resource type so releases can resolve their
                                    // references (e.g. cover art) instead of failing or guessing.
                                    let node =
                                        ern4::read_subtree(&mut xml_reader, &mut validator, e)?;
                                    resources.push(ern38::resource(
                                        &node,
                                        resource_type,
                                        self.version,
                                    ));
                                }
                                b"ReleaseDeal" if in_deal_list => {
                                    // A ReleaseDeal holds one or more Deals, all linked to its
                                    // DealReleaseReferences
                                    deals.extend(self.parse_release_deal(
                                        &mut xml_reader,
                                        &mut validator,
                                        e,
                                    )?);
                                }
                                _ => {}
                            }
                        }
                        Event::End(ref e) => match e.name().as_ref() {
                            b"PartyList" => in_party_list = false,
                            b"ReleaseList" => in_release_list = false,
                            b"ResourceList" => in_resource_list = false,
                            b"DealList" => in_deal_list = false,
                            _ => {}
                        },
                        Event::Eof => break,
                        _ => {}
                    }
                }
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error: {}", e)));
                }
            }
            buf.clear();
        }

        Ok(ERNMessage {
            message_header,
            parties,
            resources,
            releases,
            deals,
            version: self.version,
            profile: None,
            message_audit_trail: None,
            extensions: None,
            legacy_extensions: None,
            comments: None,
            attributes: root_attributes,
            track_releases,
        })
    }

    /// Handle an ERN 4.x start tag that `ern4` owns (Release, TrackRelease, Party, resources).
    /// Returns `false` for anything else so the shared loop handles it.
    fn handle_ern4_start<R: BufRead>(
        &self,
        e: &quick_xml::events::BytesStart,
        reader: &mut Reader<R>,
        validator: &mut XmlValidator,
        ctx: Ern4Context,
        sinks: Ern4Sinks<'_>,
    ) -> Result<bool, ParseError> {
        use ddex_core::models::graph::ResourceType;
        let local = e.local_name();
        let name = local.as_ref();
        if ctx.in_release_list && (name == b"Release" || name == b"TrackRelease") {
            let node = ern4::read_subtree(reader, validator, e)?;
            if name == b"Release" {
                sinks.releases.push(ern4::release(&node, self.version));
            } else if let Some(tr) = ern4::track_release(&node) {
                sinks.track_releases.push(tr);
            }
            return Ok(true);
        }
        if ctx.in_party_list && name == b"Party" {
            let node = ern4::read_subtree(reader, validator, e)?;
            sinks.parties.push(ern4::party(&node));
            return Ok(true);
        }
        if ctx.in_resource_list {
            let resource_type = match name {
                b"SoundRecording" => ResourceType::SoundRecording,
                b"Video" => ResourceType::Video,
                b"Image" => ResourceType::Image,
                b"Text" => ResourceType::Text,
                b"SheetMusic" => ResourceType::SheetMusic,
                _ => return Ok(false),
            };
            let node = ern4::read_subtree(reader, validator, e)?;
            sinks
                .resources
                .push(ern4::resource(&node, resource_type, self.version));
            return Ok(true);
        }
        Ok(false)
    }

    /// Build graph model from XML with namespace context
    pub fn build_from_xml_with_context<R: BufRead + std::io::Seek>(
        &self,
        reader: R,
        _context: NamespaceContext,
    ) -> Result<ERNMessage, ParseError> {
        self.build_from_xml_with_context_and_security(
            reader,
            _context,
            &crate::parser::security::SecurityConfig::default(),
        )
    }

    pub fn build_from_xml_with_context_and_security<R: BufRead + std::io::Seek>(
        &self,
        reader: R,
        _context: NamespaceContext,
        security_config: &crate::parser::security::SecurityConfig,
    ) -> Result<ERNMessage, ParseError> {
        // For now, delegate to the security-aware method
        // In the future, this would use the namespace context for proper element resolution
        self.build_from_xml_with_security_config(reader, security_config)
    }

    /// Read `MessageHeader` (first pass). Values follow the XSD whitespace rules: text
    /// (`xs:string`) exactly as written, `MessageCreatedDateTime` (`xs:dateTime`) collapsed
    /// before parsing. Missing values keep the historical fallbacks (message id
    /// `MSG_<version>`, creation time "now").
    fn parse_header_from_xml<R: BufRead>(
        &self,
        reader: &mut Reader<R>,
    ) -> Result<MessageHeader, ParseError> {
        use chrono::Utc;
        use ddex_core::models::common::{Identifier, IdentifierType, LocalizedString};

        let mut buf = Vec::new();
        let node = loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"MessageHeader" => {
                    let start = e.to_owned();
                    let mut validator = XmlValidator::strict();
                    validator.validate_event(&Event::Start(start.clone()), reader)?;
                    break Some(ern4::read_subtree(reader, &mut validator, &start)?);
                }
                Ok(Event::Eof) => break None,
                Err(e) => {
                    return Err(ParseError::XmlError(format!(
                        "XML parsing error in header: {}",
                        e
                    )))
                }
                _ => {}
            }
            buf.clear();
        };
        let node = node.unwrap_or_default();

        // PartyId values and PartyName/FullName (or, simplified, PartyName's own text) of a
        // MessageSender / MessageRecipient.
        let party =
            |p: &ern4::XNode, ids: &mut Vec<Identifier>, names: &mut Vec<LocalizedString>| {
                ids.extend(p.children_named("PartyId").filter_map(|id| {
                    Some(Identifier {
                        id_type: IdentifierType::Proprietary,
                        namespace: None,
                        value: id.value()?,
                    })
                }));
                for pn in p.children_named("PartyName") {
                    let full: Vec<_> = pn.children_named("FullName").collect();
                    if full.is_empty() {
                        names.extend(pn.value().map(LocalizedString::new));
                    } else {
                        names.extend(
                            full.into_iter()
                                .filter_map(ern4::XNode::value)
                                .map(LocalizedString::new),
                        );
                    }
                }
            };
        let (mut sender_ids, mut sender_names) = (Vec::new(), Vec::new());
        for p in node.children_named("MessageSender") {
            party(p, &mut sender_ids, &mut sender_names);
        }
        let (mut recipient_ids, mut recipient_names) = (Vec::new(), Vec::new());
        for p in node.children_named("MessageRecipient") {
            party(p, &mut recipient_ids, &mut recipient_names);
        }

        Ok(MessageHeader {
            message_id: node
                .child_value("MessageId")
                .unwrap_or_else(|| format!("MSG_{:?}", self.version)),
            message_type: MessageType::NewReleaseMessage,
            message_created_date_time: node
                .child_value("MessageCreatedDateTime")
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t.trim()).ok())
                .map(|t| t.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            message_sender: MessageSender {
                party_id: sender_ids,
                party_name: sender_names,
                trading_name: None,
                extensions: None,
                attributes: None,
                comments: None,
            },
            message_recipient: MessageRecipient {
                party_id: recipient_ids,
                party_name: recipient_names,
                trading_name: None,
                extensions: None,
                attributes: None,
                comments: None,
            },
            message_control_type: node.child_value("MessageControlType").map(|t| {
                match t.as_str() {
                    "LiveMessage" => MessageControlType::LiveMessage,
                    "TestMessage" => MessageControlType::TestMessage,
                    _ => MessageControlType::Other(t),
                }
            }),
            message_thread_id: node.child_value("MessageThreadId"),
            extensions: None,
            attributes: None,
            comments: None,
        })
    }

    /// Parse a `ReleaseDeal` (whose start tag `start` was just consumed) into its `Deal`s.
    ///
    /// ERN 3.8.x and 4.x share the structure
    /// `ReleaseDeal { DealReleaseReference+, Deal+ { DealReference?, DealTerms } }`.
    /// Each `Deal` becomes its own graph `Deal` with its own terms, linked to every
    /// `DealReleaseReference` of the enclosing `ReleaseDeal`. A simplified layout with
    /// `DealReference` / `DealTerms` directly under `ReleaseDeal` yields a single deal.
    pub(crate) fn parse_release_deal<R: BufRead>(
        &self,
        reader: &mut Reader<R>,
        validator: &mut crate::parser::xml_validator::XmlValidator,
        start: &quick_xml::events::BytesStart,
    ) -> Result<Vec<ddex_core::models::graph::Deal>, ParseError> {
        let node = ern4::read_subtree(reader, validator, start)?;
        Ok(release_deal(&node))
    }
}

/// Map a `ReleaseDeal` element (see [`GraphBuilder::parse_release_deal`]).
fn release_deal(node: &ern4::XNode) -> Vec<ddex_core::models::graph::Deal> {
    use ddex_core::models::common::ValidityPeriod;
    use ddex_core::models::graph::{CommercialModelType, Deal, DealTerms, UseType};

    // DealReleaseReference is an xs:IDREF: collapsed.
    let release_refs: Vec<String> = node
        .children_named("DealReleaseReference")
        .filter_map(ern4::XNode::reference)
        .collect();

    let deal = |d: &ern4::XNode| -> Deal {
        let terms = d.child("DealTerms");
        let in_terms = |name: &'static str| {
            terms
                .into_iter()
                .flat_map(move |t| t.children_named(name))
                .filter_map(ern4::XNode::value)
        };
        // UseType: 4.x DealTerms/UseType, 3.8.x DealTerms/Usage/UseType.
        let use_types = terms
            .into_iter()
            .flat_map(|t| {
                t.children_named("UseType").chain(
                    t.children_named("Usage")
                        .flat_map(|u| u.children_named("UseType")),
                )
            })
            .filter_map(ern4::XNode::value)
            .map(|v| match v.as_str() {
                "Stream" => UseType::Stream,
                "Download" => UseType::Download,
                "OnDemandStream" => UseType::OnDemandStream,
                "NonInteractiveStream" => UseType::NonInteractiveStream,
                _ => UseType::Other(v),
            })
            .collect();
        // ValidityPeriod: StartDate / EndDate; with several periods the last date given wins
        // (as before).
        let periods: Vec<_> = terms
            .into_iter()
            .flat_map(|t| t.children_named("ValidityPeriod"))
            .collect();
        let date_of = |name: &str| {
            periods
                .iter()
                .filter_map(|p| p.child_value(name))
                .filter_map(|v| deal_date(&v))
                .last()
        };
        let (start_date, end_date) = (date_of("StartDate"), date_of("EndDate"));
        Deal {
            deal_reference: d.child_value("DealReference"),
            deal_release_reference: release_refs.clone(),
            deal_terms: DealTerms {
                validity_period: (!periods.is_empty()).then_some(ValidityPeriod {
                    start_date,
                    end_date,
                }),
                start_date,
                end_date,
                territory_code: in_terms("TerritoryCode").collect(),
                excluded_territory_code: Vec::new(),
                distribution_channel: Vec::new(),
                excluded_distribution_channel: Vec::new(),
                commercial_model_type: in_terms("CommercialModelType")
                    .map(|v| match v.as_str() {
                        "PayAsYouGoModel" => CommercialModelType::PayAsYouGoModel,
                        "SubscriptionModel" => CommercialModelType::SubscriptionModel,
                        "AdSupportedModel" => CommercialModelType::AdSupportedModel,
                        _ => CommercialModelType::Other(v),
                    })
                    .collect(),
                use_type: use_types,
                price_information: Vec::new(),
                wholesale_price: Vec::new(),
                suggested_retail_price: Vec::new(),
                pre_order_date: None,
                pre_order_preview_date: None,
                instant_gratification_date: None,
                takedown_date: None,
            },
        }
    };

    let mut deals = Vec::new();
    // Simplified layout: deal values directly under <ReleaseDeal>.
    if node.child("DealReference").is_some() || node.child("DealTerms").is_some() {
        deals.push(deal(node));
    }
    deals.extend(node.children_named("Deal").map(deal));
    deals
}

/// A deal `StartDate` / `EndDate`: an RFC 3339 date-time, or a full `YYYY-MM-DD` date at
/// midnight UTC. Partial dates (`YYYY`, `YYYY-MM`) are not completed: `None`.
fn deal_date(text: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let text = text.trim();
    chrono::DateTime::parse_from_rfc3339(text)
        .map(|d| d.with_timezone(&chrono::Utc))
        .ok()
        .or_else(|| ern4::full_date(text))
}

// Helper function to parse duration strings
pub(crate) fn parse_duration(
    duration_str: &str,
) -> Result<std::time::Duration, std::time::Duration> {
    use std::time::Duration;
    // Handle ISO 8601 duration format (PT3M30S)
    if duration_str.starts_with("PT") {
        let duration_part = &duration_str[2..]; // Remove "PT"
        let mut total_seconds = 0u64;
        let mut current_number = String::new();

        for ch in duration_part.chars() {
            match ch {
                '0'..='9' | '.' => current_number.push(ch),
                'H' => {
                    if let Ok(hours) = current_number.parse::<f64>() {
                        total_seconds += (hours * 3600.0) as u64;
                    }
                    current_number.clear();
                }
                'M' => {
                    if let Ok(minutes) = current_number.parse::<f64>() {
                        total_seconds += (minutes * 60.0) as u64;
                    }
                    current_number.clear();
                }
                'S' => {
                    if let Ok(seconds) = current_number.parse::<f64>() {
                        total_seconds += seconds as u64;
                    }
                    current_number.clear();
                }
                _ => {}
            }
        }

        Ok(Duration::from_secs(total_seconds))
    } else {
        // Try to parse as plain seconds
        if let Ok(seconds) = duration_str.parse::<f64>() {
            Ok(Duration::from_secs_f64(seconds))
        } else {
            Err(Duration::from_secs(0)) // Return error as Duration (will be ignored)
        }
    }
}
