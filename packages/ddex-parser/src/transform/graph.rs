// core/src/transform/graph.rs
// Remove unused imports and variables
use crate::error::ParseError;
use crate::parser::namespace_detector::NamespaceContext;
use crate::parser::xml_validator::XmlValidator;
use ddex_core::models::graph::{
    ERNMessage, MessageHeader, MessageRecipient, MessageSender, MessageType, Release,
};
use ddex_core::models::versions::ERNVersion;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::BufRead;

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
        xml_reader.config_mut().trim_text(true);
        xml_reader.config_mut().check_end_names = true;
        xml_reader.config_mut().expand_empty_elements = false;

        // Parse the actual header from XML
        let message_header = self.parse_header_from_xml(&mut xml_reader)?;

        // Reset reader to start for main parsing loop
        reader.seek(std::io::SeekFrom::Start(0))?;
        xml_reader = Reader::from_reader(&mut reader);
        xml_reader.config_mut().trim_text(true);
        xml_reader.config_mut().check_end_names = true;
        xml_reader.config_mut().expand_empty_elements = false;

        let mut validator = XmlValidator::strict();
        let mut releases = Vec::new();
        let mut resources = Vec::new(); // Made mutable to collect parsed resources
        let parties = Vec::new(); // Remove mut
        let mut deals = Vec::new(); // Made mutable to collect parsed deals

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

                    match event {
                        Event::Start(ref e) => {
                            match e.name().as_ref() {
                                b"ReleaseList" => in_release_list = true,
                                b"ResourceList" => in_resource_list = true,
                                b"DealList" => in_deal_list = true,
                                b"Release" if in_release_list => {
                                    // Create a minimal release and manually validate the end event
                                    releases.push(
                                        self.parse_minimal_release(
                                            &mut xml_reader,
                                            &mut validator,
                                        )?,
                                    );
                                }
                                name @ (b"SoundRecording" | b"Video" | b"Image" | b"Text" | b"SheetMusic")
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
                                    resources.push(self.parse_resource(
                                        &mut xml_reader,
                                        &mut validator,
                                        resource_type,
                                    )?);
                                }
                                b"ReleaseDeal" if in_deal_list => {
                                    // A ReleaseDeal holds one or more Deals, all linked to its
                                    // DealReleaseReferences
                                    deals.extend(
                                        self.parse_release_deal(
                                            &mut xml_reader,
                                            &mut validator,
                                        )?,
                                    );
                                }
                                _ => {}
                            }
                        }
                        Event::End(ref e) => {
                            match e.name().as_ref() {
                                b"ReleaseList" => in_release_list = false,
                                b"ResourceList" => in_resource_list = false,
                                b"DealList" => in_deal_list = false,
                                _ => {}
                            }
                        }
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
            attributes: None,
        })
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

    fn parse_header_from_xml<R: BufRead>(&self, reader: &mut Reader<R>) -> Result<MessageHeader, ParseError> {
        use chrono::Utc;
        use ddex_core::models::common::LocalizedString;

        let mut message_id = format!("MSG_{:?}", self.version); // fallback
        let mut message_thread_id: Option<String> = None;
        let mut message_created_date_time = Utc::now();
        let mut sender_party_names = Vec::new();
        let mut recipient_party_names = Vec::new();
        let mut sender_party_ids = Vec::new();
        let mut recipient_party_ids = Vec::new();

        let mut buf = Vec::new();
        let mut in_message_header = false;
        let mut in_message_sender = false;
        let mut in_message_recipient = false;
        let mut in_sender_party_name = false;
        let mut in_recipient_party_name = false;
        let mut current_text = String::new();

        // Parse until we exit MessageHeader or reach EOF
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"MessageHeader" => in_message_header = true,
                        b"MessageId" if in_message_header => current_text.clear(),
                        b"MessageThreadId" if in_message_header => current_text.clear(),
                        b"MessageCreatedDateTime" if in_message_header => current_text.clear(),
                        b"MessageSender" if in_message_header => in_message_sender = true,
                        b"MessageRecipient" if in_message_header => in_message_recipient = true,
                        b"PartyId" if in_message_sender => {
                            current_text.clear();
                        },
                        b"PartyId" if in_message_recipient => {
                            current_text.clear();
                        },
                        b"PartyName" if in_message_sender => {
                            in_sender_party_name = true;
                            current_text.clear();
                        },
                        b"PartyName" if in_message_recipient => {
                            in_recipient_party_name = true;
                            current_text.clear();
                        },
                        b"FullName" if in_sender_party_name || in_recipient_party_name => {
                            current_text.clear();
                        },
                        _ => {}
                    }
                },
                Ok(Event::Text(ref e)) => {
                    current_text.push_str(&e.unescape().unwrap_or_default());
                },
                Ok(Event::End(ref e)) => {
                    match e.name().as_ref() {
                        b"MessageHeader" => {
                            in_message_header = false;
                            break; // We're done parsing the header
                        },
                        b"MessageId" if in_message_header => {
                            if !current_text.trim().is_empty() {
                                message_id = current_text.trim().to_string();
                            }
                            current_text.clear();
                        },
                        b"MessageThreadId" if in_message_header => {
                            if !current_text.trim().is_empty() {
                                message_thread_id = Some(current_text.trim().to_string());
                            }
                            current_text.clear();
                        },
                        b"MessageCreatedDateTime" if in_message_header => {
                            // Try to parse the datetime, fall back to current time if invalid
                            if let Ok(parsed_time) = chrono::DateTime::parse_from_rfc3339(current_text.trim()) {
                                message_created_date_time = parsed_time.with_timezone(&Utc);
                            }
                            current_text.clear();
                        },
                        b"MessageSender" => in_message_sender = false,
                        b"MessageRecipient" => in_message_recipient = false,
                        b"PartyId" if in_message_sender => {
                            // Handle PartyId text content
                            if !current_text.trim().is_empty() {
                                use ddex_core::models::common::{Identifier, IdentifierType};
                                sender_party_ids.push(Identifier {
                                    id_type: IdentifierType::Proprietary,
                                    namespace: None,
                                    value: current_text.trim().to_string(),
                                });
                            }
                            current_text.clear();
                        },
                        b"PartyId" if in_message_recipient => {
                            // Handle PartyId text content
                            if !current_text.trim().is_empty() {
                                use ddex_core::models::common::{Identifier, IdentifierType};
                                recipient_party_ids.push(Identifier {
                                    id_type: IdentifierType::Proprietary,
                                    namespace: None,
                                    value: current_text.trim().to_string(),
                                });
                            }
                            current_text.clear();
                        },
                        b"PartyName" if in_message_sender => {
                            // Handle direct text content in PartyName (fallback for simplified format)
                            if !current_text.trim().is_empty() {
                                sender_party_names.push(LocalizedString::new(current_text.trim().to_string()));
                            }
                            current_text.clear();
                            in_sender_party_name = false;
                        },
                        b"PartyName" if in_message_recipient => {
                            // Handle direct text content in PartyName (fallback for simplified format)
                            if !current_text.trim().is_empty() {
                                recipient_party_names.push(LocalizedString::new(current_text.trim().to_string()));
                            }
                            current_text.clear();
                            in_recipient_party_name = false;
                        },
                        b"FullName" if in_sender_party_name => {
                            if !current_text.trim().is_empty() {
                                sender_party_names.push(LocalizedString::new(current_text.trim().to_string()));
                            }
                            current_text.clear();
                        },
                        b"FullName" if in_recipient_party_name => {
                            if !current_text.trim().is_empty() {
                                recipient_party_names.push(LocalizedString::new(current_text.trim().to_string()));
                            }
                            current_text.clear();
                        },
                        _ => {}
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error in header: {}", e)));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(MessageHeader {
            message_id,
            message_type: MessageType::NewReleaseMessage,
            message_created_date_time,
            message_sender: MessageSender {
                party_id: sender_party_ids,
                party_name: sender_party_names,
                trading_name: None,
                extensions: None,
                attributes: None,
                comments: None,
            },
            message_recipient: MessageRecipient {
                party_id: recipient_party_ids,
                party_name: recipient_party_names,
                trading_name: None,
                extensions: None,
                attributes: None,
                comments: None,
            },
            message_control_type: None,
            message_thread_id,
            extensions: None,
            attributes: None,
            comments: None,
        })
    }

    fn parse_minimal_release<R: BufRead>(
        &self,
        reader: &mut Reader<R>,
        validator: &mut crate::parser::xml_validator::XmlValidator,
    ) -> Result<Release, ParseError> {
        use ddex_core::models::common::{LocalizedString, Identifier, IdentifierType};
        use ddex_core::models::graph::{Artist, ReleaseResourceReference, ReleaseType};

        // Initialize all the fields we'll extract
        let mut release_reference = format!("R_{:?}", self.version); // fallback
        let mut release_ids = Vec::new();
        let mut release_titles = Vec::new();
        let mut release_type: Option<ReleaseType> = None;
        let mut display_artists = Vec::new();
        let mut resource_references = Vec::new();
        let mut current_text = String::new();

        // State tracking for nested elements
        let mut in_release_title = false;
        let mut in_title_text = false;
        let mut in_display_title_text = false;
        let mut in_release_type = false;
        let mut in_release_reference = false;
        let mut in_release_id = false;
        let mut in_icpn = false;
        let mut in_grin = false;
        let mut in_grid = false;
        let mut in_display_artist = false;
        let mut in_artist_party_name = false;
        let mut in_artist_full_name = false;
        let mut in_resource_reference_list = false;
        let mut in_resource_reference = false;

        // Parse the Release element and extract all real data
        let mut buf = Vec::new();
        let mut depth = 1;
        while depth > 0 {
            match reader.read_event_into(&mut buf) {
                Ok(ref event) => {
                    // Validate each event so the validator stack stays consistent
                    validator.validate_event(event, reader)?;

                    match event {
                        Event::Start(ref e) => {
                            depth += 1;
                            match e.name().as_ref() {
                                b"ReleaseReference" => {
                                    in_release_reference = true;
                                    current_text.clear();
                                },
                                b"ReleaseId" => in_release_id = true,
                                b"ICPN" if in_release_id => {
                                    in_icpn = true;
                                    current_text.clear();
                                },
                                b"GRIN" if in_release_id => {
                                    in_grin = true;
                                    current_text.clear();
                                },
                                b"GRid" if in_release_id => {
                                    in_grid = true;
                                    current_text.clear();
                                },
                                // ReleaseTitle (simplified format) or ReferenceTitle (3.8.x, direct child)
                                b"ReleaseTitle" => in_release_title = true,
                                b"ReferenceTitle" if depth == 2 => in_release_title = true,
                                b"TitleText" if in_release_title => {
                                    in_title_text = true;
                                    current_text.clear();
                                },
                                // DisplayTitleText (4.x, direct child)
                                b"DisplayTitleText" if depth == 2 => {
                                    in_display_title_text = true;
                                    current_text.clear();
                                },
                                b"ReleaseType" => {
                                    in_release_type = true;
                                    current_text.clear();
                                },
                                b"DisplayArtist" => in_display_artist = true,
                                b"PartyName" if in_display_artist => {
                                    in_artist_party_name = true;
                                },
                                b"FullName" if in_artist_party_name => {
                                    in_artist_full_name = true;
                                    current_text.clear();
                                },
                                b"ReleaseResourceReferenceList" => in_resource_reference_list = true,
                                b"ReleaseResourceReference" if in_resource_reference_list => {
                                    in_resource_reference = true;
                                    current_text.clear();
                                },
                                _ => {}
                            }
                        },
                        Event::Text(ref e) => {
                            if in_title_text || in_display_title_text || in_release_type || in_release_reference ||
                               in_icpn || in_grin || in_grid || in_artist_full_name || in_resource_reference {
                                current_text.push_str(&e.unescape().unwrap_or_default());
                            }
                        },
                        Event::End(ref e) => {
                            depth -= 1;
                            match e.name().as_ref() {
                                b"ReleaseReference" => {
                                    if !current_text.trim().is_empty() {
                                        release_reference = current_text.trim().to_string();
                                    }
                                    in_release_reference = false;
                                    current_text.clear();
                                },
                                b"ReleaseId" => in_release_id = false,
                                b"ICPN" if in_icpn => {
                                    if !current_text.trim().is_empty() {
                                        release_ids.push(Identifier {
                                            id_type: IdentifierType::UPC,
                                            namespace: None,
                                            value: current_text.trim().to_string(),
                                        });
                                    }
                                    in_icpn = false;
                                    current_text.clear();
                                },
                                b"GRIN" if in_grin => {
                                    if !current_text.trim().is_empty() {
                                        release_ids.push(Identifier {
                                            id_type: IdentifierType::GRid,
                                            namespace: None,
                                            value: current_text.trim().to_string(),
                                        });
                                    }
                                    in_grin = false;
                                    current_text.clear();
                                },
                                b"GRid" if in_grid => {
                                    if !current_text.trim().is_empty() {
                                        release_ids.push(Identifier {
                                            id_type: IdentifierType::GRID,
                                            namespace: None,
                                            value: current_text.trim().to_string(),
                                        });
                                    }
                                    in_grid = false;
                                    current_text.clear();
                                },
                                b"ReleaseTitle" | b"ReferenceTitle" => in_release_title = false,
                                b"TitleText" if in_title_text => {
                                    if !current_text.trim().is_empty() {
                                        release_titles.push(LocalizedString::new(current_text.trim().to_string()));
                                    }
                                    in_title_text = false;
                                    current_text.clear();
                                },
                                b"DisplayTitleText" if in_display_title_text => {
                                    if !current_text.trim().is_empty() {
                                        release_titles.push(LocalizedString::new(current_text.trim().to_string()));
                                    }
                                    in_display_title_text = false;
                                    current_text.clear();
                                },
                                b"ReleaseType" => {
                                    if !current_text.trim().is_empty() {
                                        release_type = match current_text.trim() {
                                            "Album" => Some(ReleaseType::Album),
                                            "Single" => Some(ReleaseType::Single),
                                            "EP" => Some(ReleaseType::EP),
                                            "Compilation" => Some(ReleaseType::Compilation),
                                            other => Some(ReleaseType::Other(other.to_string())),
                                        };
                                    }
                                    in_release_type = false;
                                    current_text.clear();
                                },
                                b"DisplayArtist" => in_display_artist = false,
                                b"PartyName" if in_artist_party_name => {
                                    in_artist_party_name = false;
                                },
                                b"FullName" if in_artist_full_name => {
                                    if !current_text.trim().is_empty() {
                                        display_artists.push(Artist {
                                            party_reference: None,
                                            artist_role: vec!["MainArtist".to_string()],
                                            display_artist_name: vec![LocalizedString::new(current_text.trim().to_string())],
                                            sequence_number: None,
                                        });
                                    }
                                    in_artist_full_name = false;
                                    current_text.clear();
                                },
                                b"ReleaseResourceReferenceList" => in_resource_reference_list = false,
                                b"ReleaseResourceReference" if in_resource_reference => {
                                    if !current_text.trim().is_empty() {
                                        resource_references.push(ReleaseResourceReference {
                                            resource_reference: current_text.trim().to_string(),
                                            sequence_number: None,
                                            disc_number: None,
                                            track_number: None,
                                            side: None,
                                            is_hidden: false,
                                            is_bonus: false,
                                            extensions: None,
                                            comments: None,
                                        });
                                    }
                                    in_resource_reference = false;
                                    current_text.clear();
                                },
                                _ => {}
                            }
                        },
                        Event::Eof => break,
                        _ => {}
                    }
                }
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error in release: {}", e)));
                }
            }
            buf.clear();
        }

        let release = Release {
            release_reference,
            release_id: release_ids,
            release_title: release_titles,
            release_subtitle: None,
            release_type,
            genre: Vec::new(),
            release_resource_reference_list: resource_references,
            display_artist: display_artists,
            party_list: Vec::new(),
            release_date: Vec::new(),
            territory_code: Vec::new(),
            excluded_territory_code: Vec::new(),
            extensions: None,
            attributes: None,
            comments: None,
        };

        Ok(release)
    }

    /// Parse a resource element (SoundRecording, Video, Image, Text or SheetMusic).
    ///
    /// Only values present in the XML are recorded: a resource without a title keeps an
    /// empty `reference_title`, and one without a duration keeps `duration: None`.
    fn parse_resource<R: BufRead>(
        &self,
        reader: &mut Reader<R>,
        validator: &mut crate::parser::xml_validator::XmlValidator,
        resource_type: ddex_core::models::graph::ResourceType,
    ) -> Result<ddex_core::models::graph::Resource, ParseError> {
        use ddex_core::models::common::{LocalizedString, Identifier, IdentifierType};
        use ddex_core::models::graph::Resource;
        use std::time::Duration;

        // Initialize all the fields we'll extract
        let mut resource_reference = format!("RES_{:?}", self.version); // fallback
        let mut resource_ids = Vec::new();
        let mut reference_titles = Vec::new();
        let mut duration: Option<Duration> = None;
        let mut current_text = String::new();

        // State tracking for nested elements
        let mut in_resource_reference = false;
        let mut in_sound_recording_id = false;
        let mut in_isrc = false;
        let mut in_title = false;
        let mut in_title_text = false;
        let mut in_display_title_text = false;
        let mut in_duration = false;
        let mut in_display_artist = false;
        let mut in_artist_party_name = false;
        let mut in_artist_full_name = false;

        // Parse the resource element and extract real data
        let mut buf = Vec::new();
        let mut depth = 1;
        while depth > 0 {
            match reader.read_event_into(&mut buf) {
                Ok(ref event) => {
                    // Validate each event so the validator stack stays consistent
                    validator.validate_event(event, reader)?;

                    match event {
                        Event::Start(ref e) => {
                            depth += 1;
                            match e.name().as_ref() {
                                b"ResourceReference" => {
                                    in_resource_reference = true;
                                    current_text.clear();
                                },
                                // SoundRecordingId / VideoId (3.8.x) or ResourceId (4.x)
                                b"SoundRecordingId" | b"VideoId" | b"ResourceId" => in_sound_recording_id = true,
                                b"ISRC" if in_sound_recording_id => {
                                    in_isrc = true;
                                    current_text.clear();
                                },
                                // ReferenceTitle (3.8.x, direct child) or Title (details by territory)
                                b"ReferenceTitle" if depth == 2 => in_title = true,
                                b"Title" => in_title = true,
                                b"TitleText" if in_title => {
                                    in_title_text = true;
                                    current_text.clear();
                                },
                                // DisplayTitleText (4.x, direct child)
                                b"DisplayTitleText" if depth == 2 => {
                                    in_display_title_text = true;
                                    current_text.clear();
                                },
                                b"Duration" => {
                                    in_duration = true;
                                    current_text.clear();
                                },
                                b"DisplayArtist" => in_display_artist = true,
                                b"PartyName" if in_display_artist => {
                                    in_artist_party_name = true;
                                },
                                b"FullName" if in_artist_party_name => {
                                    in_artist_full_name = true;
                                    current_text.clear();
                                },
                                _ => {}
                            }
                        },
                        Event::Text(ref e) => {
                            if in_resource_reference || in_isrc || in_title_text || in_display_title_text ||
                               in_duration || in_artist_full_name {
                                current_text.push_str(&e.unescape().unwrap_or_default());
                            }
                        },
                        Event::End(ref e) => {
                            depth -= 1;
                            match e.name().as_ref() {
                                b"ResourceReference" => {
                                    if !current_text.trim().is_empty() {
                                        resource_reference = current_text.trim().to_string();
                                    }
                                    in_resource_reference = false;
                                    current_text.clear();
                                },
                                b"SoundRecordingId" | b"VideoId" | b"ResourceId" => in_sound_recording_id = false,
                                b"ISRC" if in_isrc => {
                                    if !current_text.trim().is_empty() {
                                        resource_ids.push(Identifier {
                                            id_type: IdentifierType::ISRC,
                                            namespace: None,
                                            value: current_text.trim().to_string(),
                                        });
                                    }
                                    in_isrc = false;
                                    current_text.clear();
                                },
                                b"ReferenceTitle" | b"Title" => in_title = false,
                                b"TitleText" if in_title_text => {
                                    if !current_text.trim().is_empty() {
                                        reference_titles.push(LocalizedString::new(current_text.trim().to_string()));
                                    }
                                    in_title_text = false;
                                    current_text.clear();
                                },
                                b"DisplayTitleText" if in_display_title_text => {
                                    if !current_text.trim().is_empty() {
                                        reference_titles.push(LocalizedString::new(current_text.trim().to_string()));
                                    }
                                    in_display_title_text = false;
                                    current_text.clear();
                                },
                                b"Duration" => {
                                    if !current_text.trim().is_empty() {
                                        // Parse duration in ISO 8601 format (PT3M30S) or as seconds
                                        if let Ok(parsed_duration) = parse_duration(&current_text.trim()) {
                                            duration = Some(parsed_duration);
                                        }
                                    }
                                    in_duration = false;
                                    current_text.clear();
                                },
                                b"DisplayArtist" => in_display_artist = false,
                                b"PartyName" if in_artist_party_name => {
                                    in_artist_party_name = false;
                                },
                                b"FullName" if in_artist_full_name => {
                                    // For now, we'll store artist names in the reference_title
                                    // In a full implementation, we might want to track artists separately
                                    in_artist_full_name = false;
                                    current_text.clear();
                                },
                                _ => {}
                            }
                        },
                        Event::Eof => break,
                        _ => {}
                    }
                }
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error in resource: {}", e)));
                }
            }
            buf.clear();
        }

        let resource = Resource {
            resource_reference,
            resource_type,
            resource_id: resource_ids,
            reference_title: reference_titles,
            duration,
            technical_details: Vec::new(),
            rights_controller: Vec::new(),
            p_line: Vec::new(),
            c_line: Vec::new(),
            extensions: None,
        };

        Ok(resource)
    }

    /// Parse a `ReleaseDeal` into its `Deal`s.
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
    ) -> Result<Vec<ddex_core::models::graph::Deal>, ParseError> {
        use ddex_core::models::common::ValidityPeriod;
        use ddex_core::models::graph::{CommercialModelType, Deal, DealTerms, UseType};
        use chrono::{DateTime, Utc};

        /// Values collected for one deal.
        #[derive(Default)]
        struct DealAcc {
            deal_reference: Option<String>,
            territory_codes: Vec<String>,
            use_types: Vec<UseType>,
            commercial_model_types: Vec<CommercialModelType>,
            validity_period: Option<ValidityPeriod>,
            start_date: Option<DateTime<Utc>>,
            seen: bool,
        }

        impl DealAcc {
            fn into_deal(self, release_refs: &[String]) -> Deal {
                Deal {
                    deal_reference: self.deal_reference,
                    deal_release_reference: release_refs.to_vec(),
                    deal_terms: DealTerms {
                        validity_period: self.validity_period,
                        start_date: self.start_date,
                        end_date: None,
                        territory_code: self.territory_codes,
                        excluded_territory_code: Vec::new(),
                        distribution_channel: Vec::new(),
                        excluded_distribution_channel: Vec::new(),
                        commercial_model_type: self.commercial_model_types,
                        use_type: self.use_types,
                        price_information: Vec::new(),
                        wholesale_price: Vec::new(),
                        suggested_retail_price: Vec::new(),
                        pre_order_date: None,
                        pre_order_preview_date: None,
                        instant_gratification_date: None,
                        takedown_date: None,
                    },
                }
            }
        }

        let mut release_refs: Vec<String> = Vec::new();
        let mut finished: Vec<DealAcc> = Vec::new();
        // Deal-level values: either inside the current <Deal>, or directly under <ReleaseDeal>
        // for the simplified layout.
        let mut current = DealAcc::default();
        let mut in_deal_element = false;
        let mut current_text = String::new();

        // State tracking for nested elements
        let mut in_release_ref = false;
        let mut in_deal_reference = false;
        let mut in_deal_terms = false;
        let mut in_territory_code = false;
        let mut in_use_type = false;
        let mut in_commercial_model_type = false;
        let mut in_validity_period = false;
        let mut in_start_date = false;

        let mut buf = Vec::new();
        let mut depth = 1;
        while depth > 0 {
            match reader.read_event_into(&mut buf) {
                Ok(ref event) => {
                    // Validate each event so the validator stack stays consistent
                    validator.validate_event(event, reader)?;

                    match event {
                        Event::Start(ref e) => {
                            depth += 1;
                            match e.name().as_ref() {
                                b"DealReleaseReference" if depth == 2 => {
                                    in_release_ref = true;
                                    current_text.clear();
                                },
                                b"Deal" if depth == 2 => {
                                    // A new <Deal>; anything collected at ReleaseDeal level so far
                                    // belongs to the simplified layout and is kept separately.
                                    if current.seen {
                                        finished.push(std::mem::take(&mut current));
                                    }
                                    in_deal_element = true;
                                },
                                b"DealReference" => {
                                    in_deal_reference = true;
                                    current_text.clear();
                                },
                                b"DealTerms" => in_deal_terms = true,
                                b"TerritoryCode" if in_deal_terms => {
                                    in_territory_code = true;
                                    current_text.clear();
                                },
                                b"UseType" if in_deal_terms => {
                                    in_use_type = true;
                                    current_text.clear();
                                },
                                b"CommercialModelType" if in_deal_terms => {
                                    in_commercial_model_type = true;
                                    current_text.clear();
                                },
                                b"ValidityPeriod" if in_deal_terms => {
                                    in_validity_period = true;
                                },
                                b"StartDate" if in_validity_period => {
                                    in_start_date = true;
                                    current_text.clear();
                                },
                                _ => {}
                            }
                        },
                        Event::Text(ref e) => {
                            if in_release_ref || in_deal_reference || in_territory_code || in_use_type ||
                               in_commercial_model_type || in_start_date {
                                current_text.push_str(&e.unescape().unwrap_or_default());
                            }
                        },
                        Event::End(ref e) => {
                            depth -= 1;
                            match e.name().as_ref() {
                                b"DealReleaseReference" if in_release_ref => {
                                    if !current_text.trim().is_empty() {
                                        release_refs.push(current_text.trim().to_string());
                                    }
                                    in_release_ref = false;
                                    current_text.clear();
                                },
                                b"Deal" if in_deal_element && depth == 1 => {
                                    current.seen = true;
                                    finished.push(std::mem::take(&mut current));
                                    in_deal_element = false;
                                },
                                b"DealReference" => {
                                    if !current_text.trim().is_empty() {
                                        current.deal_reference = Some(current_text.trim().to_string());
                                        current.seen = true;
                                    }
                                    in_deal_reference = false;
                                    current_text.clear();
                                },
                                b"DealTerms" => {
                                    in_deal_terms = false;
                                    current.seen = true;
                                },
                                b"TerritoryCode" if in_territory_code => {
                                    if !current_text.trim().is_empty() {
                                        current.territory_codes.push(current_text.trim().to_string());
                                    }
                                    in_territory_code = false;
                                    current_text.clear();
                                },
                                b"UseType" if in_use_type => {
                                    if !current_text.trim().is_empty() {
                                        let use_type = match current_text.trim() {
                                            "Stream" => UseType::Stream,
                                            "Download" => UseType::Download,
                                            "OnDemandStream" => UseType::OnDemandStream,
                                            "NonInteractiveStream" => UseType::NonInteractiveStream,
                                            other => UseType::Other(other.to_string()),
                                        };
                                        current.use_types.push(use_type);
                                    }
                                    in_use_type = false;
                                    current_text.clear();
                                },
                                b"CommercialModelType" if in_commercial_model_type => {
                                    if !current_text.trim().is_empty() {
                                        let commercial_model = match current_text.trim() {
                                            "PayAsYouGoModel" => CommercialModelType::PayAsYouGoModel,
                                            "SubscriptionModel" => CommercialModelType::SubscriptionModel,
                                            "AdSupportedModel" => CommercialModelType::AdSupportedModel,
                                            other => CommercialModelType::Other(other.to_string()),
                                        };
                                        current.commercial_model_types.push(commercial_model);
                                    }
                                    in_commercial_model_type = false;
                                    current_text.clear();
                                },
                                b"ValidityPeriod" if in_validity_period => {
                                    current.validity_period = Some(ValidityPeriod {
                                        start_date: current.start_date,
                                        end_date: None, // Could be extended to parse EndDate if needed
                                    });
                                    in_validity_period = false;
                                },
                                b"StartDate" if in_start_date => {
                                    if !current_text.trim().is_empty() {
                                        if let Ok(parsed_date) = DateTime::parse_from_rfc3339(current_text.trim()) {
                                            current.start_date = Some(parsed_date.with_timezone(&Utc));
                                        }
                                    }
                                    in_start_date = false;
                                    current_text.clear();
                                },
                                _ => {}
                            }
                        },
                        Event::Eof => break,
                        _ => {}
                    }
                }
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error in release deal: {}", e)));
                }
            }
            buf.clear();
        }

        // Simplified layout: deal values directly under <ReleaseDeal>
        if current.seen {
            finished.push(current);
        }

        Ok(finished
            .into_iter()
            .map(|acc| acc.into_deal(&release_refs))
            .collect())
    }
}

// Helper function to parse duration strings
fn parse_duration(duration_str: &str) -> Result<std::time::Duration, std::time::Duration> {
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
                },
                'M' => {
                    if let Ok(minutes) = current_number.parse::<f64>() {
                        total_seconds += (minutes * 60.0) as u64;
                    }
                    current_number.clear();
                },
                'S' => {
                    if let Ok(seconds) = current_number.parse::<f64>() {
                        total_seconds += seconds as u64;
                    }
                    current_number.clear();
                },
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
