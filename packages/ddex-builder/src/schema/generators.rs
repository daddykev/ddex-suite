//! Schema generators for specific DDEX structures

use super::*;
// Only import what we actually need for the generators

impl SchemaGenerator {
    /// Generate schema for BuildRequest
    pub(crate) fn build_request_schema(&self, warnings: &mut Vec<SchemaWarning>) -> Result<JsonSchema, BuildError> {
        let mut properties = IndexMap::new();
        let mut required = Vec::new();
        
        // Message header (always required)
        properties.insert("header".to_string(), self.message_header_schema(warnings)?);
        required.push("header".to_string());
        
        // Releases
        let release_schema = JsonSchema {
            schema_type: Some("array".to_string()),
            items: Some(Box::new(JsonSchema {
                reference: Some("#/$defs/ReleaseRequest".to_string()),
                ..Default::default()
            })),
            min_length: Some(1),
            description: Some("List of releases in this DDEX message".to_string()),
            ..Default::default()
        };
        properties.insert("releases".to_string(), release_schema);
        required.push("releases".to_string());
        
        // Deals (optional but common)
        let deal_schema = JsonSchema {
            schema_type: Some("array".to_string()),
            items: Some(Box::new(JsonSchema {
                reference: Some("#/$defs/DealRequest".to_string()),
                ..Default::default()
            })),
            description: Some("Commercial deals for the releases".to_string()),
            ..Default::default()
        };
        properties.insert("deals".to_string(), deal_schema);
        
        // Add profile-specific required fields
        self.apply_profile_requirements(&mut required, warnings);
        
        Ok(JsonSchema {
            schema: self.schema_draft_url(),
            title: Some("DDEX BuildRequest".to_string()),
            description: Some(format!(
                "Complete DDEX message request for ERN {} with {} profile",
                self.version_string(), self.profile_string()
            )),
            schema_type: Some("object".to_string()),
            properties: Some(properties),
            required: Some(required),
            additional_properties: Some(false),
            examples: if self.config.include_examples { 
                Some(vec![self.build_request_example()])
            } else { 
                None 
            },
            ..Default::default()
        })
    }
    
    /// Generate schema for ReleaseRequest
    pub(crate) fn release_request_schema(&self, warnings: &mut Vec<SchemaWarning>) -> Result<JsonSchema, BuildError> {
        let mut properties = IndexMap::new();
        let mut required = Vec::new();
        
        // Release ID (always required)
        properties.insert("release_id".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Unique identifier for the release".to_string()),
            min_length: Some(1),
            max_length: Some(50),
            pattern: Some("^[A-Za-z0-9][A-Za-z0-9_-]*$".to_string()),
            examples: if self.config.include_examples {
                Some(vec![json!("release-001"), json!("album-2024-001")])
            } else { None },
            ..Default::default()
        });
        required.push("release_id".to_string());
        
        // Release reference (for linking)
        if self.version_supports_references() {
            properties.insert("release_reference".to_string(), JsonSchema {
                schema_type: Some("string".to_string()),
                description: Some("Reference identifier for linking within message".to_string()),
                pattern: Some("^REL[0-9]+$".to_string()),
                examples: if self.config.include_examples {
                    Some(vec![json!("REL001"), json!("REL002")])
                } else { None },
                ..Default::default()
            });
        }
        
        // Title (always required)
        properties.insert("title".to_string(), JsonSchema {
            schema_type: Some("array".to_string()),
            items: Some(Box::new(JsonSchema {
                reference: Some("#/$defs/LocalizedString".to_string()),
                ..Default::default()
            })),
            min_length: Some(1),
            description: Some("Release title in one or more languages".to_string()),
            examples: if self.config.include_examples {
                Some(vec![json!([{"text": "Greatest Hits Album", "language_code": "en"}])])
            } else { None },
            ..Default::default()
        });
        required.push("title".to_string());
        
        // Artist (always required)
        properties.insert("artist".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Primary artist or band name".to_string()),
            min_length: Some(1),
            max_length: Some(200),
            examples: if self.config.include_examples {
                Some(vec![json!("The Beatles"), json!("Various Artists")])
            } else { None },
            ..Default::default()
        });
        required.push("artist".to_string());
        
        // Label
        properties.insert("label".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Record label name".to_string()),
            max_length: Some(200),
            examples: if self.config.include_examples {
                Some(vec![json!("Universal Music"), json!("Independent Records")])
            } else { None },
            ..Default::default()
        });
        
        // Release date
        properties.insert("release_date".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            format: Some("date".to_string()),
            description: Some("Original release date in YYYY-MM-DD format".to_string()),
            pattern: Some("^\\d{4}-\\d{2}-\\d{2}$".to_string()),
            examples: if self.config.include_examples {
                Some(vec![json!("2024-03-15"), json!("2023-12-01")])
            } else { None },
            ..Default::default()
        });
        
        // UPC (Universal Product Code)
        properties.insert("upc".to_string(), self.upc_schema());
        
        // Tracks (always required for album profiles)
        properties.insert("tracks".to_string(), JsonSchema {
            schema_type: Some("array".to_string()),
            items: Some(Box::new(JsonSchema {
                reference: Some("#/$defs/TrackRequest".to_string()),
                ..Default::default()
            })),
            min_length: if matches!(self.profile, MessageProfile::AudioSingle | MessageProfile::VideoSingle) { 
                Some(1) 
            } else { 
                Some(1) 
            },
            description: Some("List of tracks/resources in this release".to_string()),
            ..Default::default()
        });
        required.push("tracks".to_string());
        
        // Resource references (for advanced linking)
        if self.version_supports_references() {
            properties.insert("resource_references".to_string(), JsonSchema {
                schema_type: Some("array".to_string()),
                items: Some(Box::new(JsonSchema {
                    schema_type: Some("string".to_string()),
                    pattern: Some("^R[0-9]+$".to_string()),
                    ..Default::default()
                })),
                description: Some("References to resources for complex linking scenarios".to_string()),
                ..Default::default()
            });
        }
        
        // Apply version-specific validation
        self.apply_version_conditionals(&mut properties, warnings);
        
        Ok(JsonSchema {
            title: Some("DDEX ReleaseRequest".to_string()),
            description: Some("Release information for DDEX message".to_string()),
            schema_type: Some("object".to_string()),
            properties: Some(properties),
            required: Some(required),
            additional_properties: Some(false),
            ..Default::default()
        })
    }
    
    /// Generate schema for TrackRequest
    pub(crate) fn track_request_schema(&self, _warnings: &mut Vec<SchemaWarning>) -> Result<JsonSchema, BuildError> {
        let mut properties = IndexMap::new();
        let mut required = Vec::new();
        
        // Track ID
        properties.insert("track_id".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Unique identifier for the track".to_string()),
            min_length: Some(1),
            max_length: Some(50),
            pattern: Some("^[A-Za-z0-9][A-Za-z0-9_-]*$".to_string()),
            examples: if self.config.include_examples {
                Some(vec![json!("track-001"), json!("song-amazing")])
            } else { None },
            ..Default::default()
        });
        required.push("track_id".to_string());
        
        // Resource reference
        if self.version_supports_references() {
            properties.insert("resource_reference".to_string(), JsonSchema {
                schema_type: Some("string".to_string()),
                description: Some("Reference identifier for linking within message".to_string()),
                pattern: Some("^R[0-9]+$".to_string()),
                examples: if self.config.include_examples {
                    Some(vec![json!("R001"), json!("R002")])
                } else { None },
                ..Default::default()
            });
        }
        
        // ISRC (always required)
        properties.insert("isrc".to_string(), self.isrc_schema());
        required.push("isrc".to_string());
        
        // Title
        properties.insert("title".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Track title".to_string()),
            min_length: Some(1),
            max_length: Some(200),
            examples: if self.config.include_examples {
                Some(vec![json!("Amazing Song"), json!("Track Title Here")])
            } else { None },
            ..Default::default()
        });
        required.push("title".to_string());
        
        // Duration (ISO 8601 format)
        properties.insert("duration".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Track duration in ISO 8601 format (PT#M#S)".to_string()),
            pattern: Some("^PT(?:\\d+H)?(?:\\d+M)?(?:\\d+(?:\\.\\d+)?S)?$".to_string()),
            examples: if self.config.include_examples {
                Some(vec![json!("PT3M45S"), json!("PT4M12.5S")])
            } else { None },
            ..Default::default()
        });
        required.push("duration".to_string());
        
        // Artist
        properties.insert("artist".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Track artist (may differ from release artist)".to_string()),
            min_length: Some(1),
            max_length: Some(200),
            examples: if self.config.include_examples {
                Some(vec![json!("The Beatles"), json!("John Lennon")])
            } else { None },
            ..Default::default()
        });
        required.push("artist".to_string());
        
        Ok(JsonSchema {
            title: Some("DDEX TrackRequest".to_string()),
            description: Some("Track/resource information for DDEX message".to_string()),
            schema_type: Some("object".to_string()),
            properties: Some(properties),
            required: Some(required),
            additional_properties: Some(false),
            ..Default::default()
        })
    }
    
    /// Generate schema for DealRequest
    pub(crate) fn deal_request_schema(&self, _warnings: &mut Vec<SchemaWarning>) -> Result<JsonSchema, BuildError> {
        let mut properties = IndexMap::new();
        let mut required = Vec::new();
        
        // Deal reference
        if self.version_supports_references() {
            properties.insert("deal_reference".to_string(), JsonSchema {
                schema_type: Some("string".to_string()),
                description: Some("Reference identifier for the deal".to_string()),
                pattern: Some("^DEAL[0-9]+$".to_string()),
                examples: if self.config.include_examples {
                    Some(vec![json!("DEAL001"), json!("DEAL002")])
                } else { None },
                ..Default::default()
            });
        }
        
        // Deal terms
        properties.insert("deal_terms".to_string(), JsonSchema {
            reference: Some("#/$defs/DealTerms".to_string()),
            ..Default::default()
        });
        required.push("deal_terms".to_string());
        
        // Release references
        properties.insert("release_references".to_string(), JsonSchema {
            schema_type: Some("array".to_string()),
            items: Some(Box::new(JsonSchema {
                schema_type: Some("string".to_string()),
                description: Some("Reference to a release".to_string()),
                ..Default::default()
            })),
            min_length: Some(1),
            description: Some("References to releases covered by this deal".to_string()),
            ..Default::default()
        });
        required.push("release_references".to_string());
        
        Ok(JsonSchema {
            title: Some("DDEX DealRequest".to_string()),
            description: Some("Commercial deal information for DDEX message".to_string()),
            schema_type: Some("object".to_string()),
            properties: Some(properties),
            required: Some(required),
            additional_properties: Some(false),
            ..Default::default()
        })
    }
    
    /// Generate schema for MessageHeader
    pub(crate) fn message_header_schema(&self, _warnings: &mut Vec<SchemaWarning>) -> Result<JsonSchema, BuildError> {
        let mut properties = IndexMap::new();
        let mut required = Vec::new();
        
        // Message ID
        properties.insert("message_id".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Unique message identifier".to_string()),
            min_length: Some(1),
            max_length: Some(50),
            examples: if self.config.include_examples {
                Some(vec![json!("MSG-001"), json!("RELEASE-2024-001")])
            } else { None },
            ..Default::default()
        });
        
        // Message sender
        properties.insert("message_sender".to_string(), JsonSchema {
            reference: Some("#/$defs/Party".to_string()),
            ..Default::default()
        });
        required.push("message_sender".to_string());
        
        // Message recipient
        properties.insert("message_recipient".to_string(), JsonSchema {
            reference: Some("#/$defs/Party".to_string()),
            ..Default::default()
        });
        required.push("message_recipient".to_string());
        
        // Message control type
        properties.insert("message_control_type".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            enum_values: Some(vec![
                json!("NewReleaseMessage"),
                json!("UpdateReleaseMessage"),
                json!("CatalogTransferMessage")
            ]),
            description: Some("Type of DDEX message being sent".to_string()),
            ..Default::default()
        });
        
        // Message created date time
        properties.insert("message_created_date_time".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            format: Some("date-time".to_string()),
            description: Some("Timestamp when message was created (ISO 8601)".to_string()),
            examples: if self.config.include_examples {
                Some(vec![json!("2024-03-15T10:30:00Z")])
            } else { None },
            ..Default::default()
        });
        
        Ok(JsonSchema {
            title: Some("DDEX MessageHeader".to_string()),
            description: Some("Message header information for DDEX communication".to_string()),
            schema_type: Some("object".to_string()),
            properties: Some(properties),
            required: Some(required),
            additional_properties: Some(false),
            ..Default::default()
        })
    }
    
    /// Generate common type definitions
    pub(crate) fn common_type_definitions(&self, _warnings: &mut Vec<SchemaWarning>) -> Result<IndexMap<String, JsonSchema>, BuildError> {
        let mut definitions = IndexMap::new();
        
        // LocalizedString
        definitions.insert("LocalizedString".to_string(), JsonSchema {
            title: Some("Localized String".to_string()),
            description: Some("Text with optional language code".to_string()),
            schema_type: Some("object".to_string()),
            properties: Some({
                let mut props = IndexMap::new();
                props.insert("text".to_string(), JsonSchema {
                    schema_type: Some("string".to_string()),
                    description: Some("The text content".to_string()),
                    min_length: Some(1),
                    ..Default::default()
                });
                props.insert("language_code".to_string(), JsonSchema {
                    schema_type: Some("string".to_string()),
                    description: Some("ISO 639 language code".to_string()),
                    pattern: Some("^[a-z]{2}(-[A-Z]{2})?$".to_string()),
                    examples: if self.config.include_examples {
                        Some(vec![json!("en"), json!("en-US"), json!("fr")])
                    } else { None },
                    ..Default::default()
                });
                props
            }),
            required: Some(vec!["text".to_string()]),
            additional_properties: Some(false),
            ..Default::default()
        });
        
        // Party
        definitions.insert("Party".to_string(), JsonSchema {
            title: Some("Party".to_string()),
            description: Some("Organization or individual in DDEX communication".to_string()),
            schema_type: Some("object".to_string()),
            properties: Some({
                let mut props = IndexMap::new();
                props.insert("party_name".to_string(), JsonSchema {
                    schema_type: Some("array".to_string()),
                    items: Some(Box::new(JsonSchema {
                        reference: Some("#/$defs/LocalizedString".to_string()),
                        ..Default::default()
                    })),
                    min_length: Some(1),
                    description: Some("Party name in one or more languages".to_string()),
                    ..Default::default()
                });
                props.insert("party_id".to_string(), JsonSchema {
                    schema_type: Some("string".to_string()),
                    description: Some("Optional party identifier".to_string()),
                    ..Default::default()
                });
                props.insert("party_reference".to_string(), JsonSchema {
                    schema_type: Some("string".to_string()),
                    description: Some("Optional party reference for message linking".to_string()),
                    ..Default::default()
                });
                props
            }),
            required: Some(vec!["party_name".to_string()]),
            additional_properties: Some(false),
            ..Default::default()
        });
        
        // DealTerms
        definitions.insert("DealTerms".to_string(), JsonSchema {
            title: Some("Deal Terms".to_string()),
            description: Some("Commercial terms for a deal".to_string()),
            schema_type: Some("object".to_string()),
            properties: Some({
                let mut props = IndexMap::new();
                props.insert("commercial_model_type".to_string(), JsonSchema {
                    schema_type: Some("string".to_string()),
                    enum_values: Some(vec![
                        json!("SubscriptionModel"),
                        json!("PurchaseModel"),
                        json!("AdSupportedModel"),
                        json!("FreeOfChargeModel")
                    ]),
                    description: Some("Type of commercial model".to_string()),
                    ..Default::default()
                });
                props.insert("territory_code".to_string(), JsonSchema {
                    schema_type: Some("array".to_string()),
                    items: Some(Box::new(JsonSchema {
                        schema_type: Some("string".to_string()),
                        pattern: Some("^[A-Z]{2}|Worldwide$".to_string()),
                        description: Some("ISO 3166 country code or 'Worldwide'".to_string()),
                        ..Default::default()
                    })),
                    description: Some("Territories where deal applies".to_string()),
                    examples: if self.config.include_examples {
                        Some(vec![json!(["US", "CA"]), json!(["Worldwide"])])
                    } else { None },
                    ..Default::default()
                });
                props.insert("start_date".to_string(), JsonSchema {
                    schema_type: Some("string".to_string()),
                    format: Some("date".to_string()),
                    description: Some("Deal start date".to_string()),
                    pattern: Some("^\\d{4}-\\d{2}-\\d{2}$".to_string()),
                    ..Default::default()
                });
                props
            }),
            required: Some(vec!["commercial_model_type".to_string(), "territory_code".to_string()]),
            additional_properties: Some(false),
            ..Default::default()
        });
        
        Ok(definitions)
    }
    
    /// Generate flat release schema (simplified structure)
    pub(crate) fn flat_release_schema(&self, _warnings: &mut Vec<SchemaWarning>) -> Result<JsonSchema, BuildError> {
        let mut properties = IndexMap::new();
        let mut required = Vec::new();
        
        // Flattened release properties
        properties.insert("release_id".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Release identifier".to_string()),
            ..Default::default()
        });
        required.push("release_id".to_string());
        
        properties.insert("title".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Release title".to_string()),
            ..Default::default()
        });
        required.push("title".to_string());
        
        properties.insert("artist".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Primary artist".to_string()),
            ..Default::default()
        });
        required.push("artist".to_string());
        
        properties.insert("upc".to_string(), self.upc_schema());
        properties.insert("release_date".to_string(), JsonSchema {
            schema_type: Some("string".to_string()),
            format: Some("date".to_string()),
            ..Default::default()
        });
        
        // Flattened track information
        properties.insert("tracks".to_string(), JsonSchema {
            schema_type: Some("array".to_string()),
            items: Some(Box::new(JsonSchema {
                schema_type: Some("object".to_string()),
                properties: Some({
                    let mut track_props = IndexMap::new();
                    track_props.insert("isrc".to_string(), self.isrc_schema());
                    track_props.insert("title".to_string(), JsonSchema {
                        schema_type: Some("string".to_string()),
                        ..Default::default()
                    });
                    track_props.insert("duration".to_string(), JsonSchema {
                        schema_type: Some("string".to_string()),
                        pattern: Some("^PT(?:\\d+H)?(?:\\d+M)?(?:\\d+(?:\\.\\d+)?S)?$".to_string()),
                        ..Default::default()
                    });
                    track_props
                }),
                required: Some(vec!["isrc".to_string(), "title".to_string(), "duration".to_string()]),
                ..Default::default()
            })),
            ..Default::default()
        });
        required.push("tracks".to_string());
        
        Ok(JsonSchema {
            title: Some("Flat Release Schema".to_string()),
            description: Some("Simplified, flattened release structure".to_string()),
            schema_type: Some("object".to_string()),
            properties: Some(properties),
            required: Some(required),
            additional_properties: Some(false),
            ..Default::default()
        })
    }
    
    // Validation schemas for common DDEX codes
    
    fn isrc_schema(&self) -> JsonSchema {
        JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("International Standard Recording Code".to_string()),
            pattern: Some("^[A-Z]{2}[A-Z0-9]{3}\\d{7}$".to_string()),
            min_length: Some(12),
            max_length: Some(12),
            examples: if self.config.include_examples {
                Some(vec![json!("USRC17607839"), json!("GBUM71505078")])
            } else { None },
            ..Default::default()
        }
    }
    
    fn upc_schema(&self) -> JsonSchema {
        JsonSchema {
            schema_type: Some("string".to_string()),
            description: Some("Universal Product Code (12 digits)".to_string()),
            pattern: Some("^\\d{12}$".to_string()),
            min_length: Some(12),
            max_length: Some(12),
            examples: if self.config.include_examples {
                Some(vec![json!("123456789012"), json!("987654321098")])
            } else { None },
            ..Default::default()
        }
    }
    
    // Helper methods
    
    fn version_supports_references(&self) -> bool {
        // All supported versions have reference support
        true
    }
    
    fn apply_profile_requirements(&self, required: &mut Vec<String>, warnings: &mut Vec<SchemaWarning>) {
        match self.profile {
            MessageProfile::AudioAlbum | MessageProfile::VideoAlbum => {
                // Albums typically require deals
                if !required.contains(&"deals".to_string()) {
                    warnings.push(SchemaWarning {
                        code: "PROFILE_RECOMMENDATION".to_string(),
                        message: "Album profiles typically include deal information".to_string(),
                        field_path: Some("deals".to_string()),
                        suggestion: Some("Consider including deals array for album releases".to_string()),
                    });
                }
            },
            MessageProfile::AudioSingle | MessageProfile::VideoSingle => {
                // Singles may have simpler requirements
            },
            MessageProfile::Mixed => {
                // Mixed profile has flexible requirements
            }
        }
    }
    
    fn apply_version_conditionals(&self, _properties: &mut IndexMap<String, JsonSchema>, _warnings: &mut Vec<SchemaWarning>) {
        if !self.config.version_conditionals {
            return;
        }
        
        // Add version-specific conditional logic
        match self.version {
            DdexVersion::Ern43 => {
                // ERN 4.3 specific features
            },
            DdexVersion::Ern42 => {
                // ERN 4.2 may have different constraints
            },
            DdexVersion::Ern41 => {
                // ERN 4.1 legacy support
            },
            DdexVersion::Ern382 => {
                // ERN 3.8.2 legacy support
            }
        }
    }
    
    fn build_request_example(&self) -> JsonValue {
        json!({
            "header": {
                "message_id": "MSG-2024-001",
                "message_sender": {
                    "party_name": [{"text": "Independent Label Records"}]
                },
                "message_recipient": {
                    "party_name": [{"text": "Digital Service Provider"}]
                },
                "message_created_date_time": "2024-03-15T10:30:00Z"
            },
            "releases": [{
                "release_id": "album-001",
                "title": [{"text": "Greatest Hits Collection"}],
                "artist": "The Example Band",
                "label": "Independent Label Records",
                "release_date": "2024-04-01",
                "upc": "123456789012",
                "tracks": [{
                    "track_id": "track-001",
                    "isrc": "USRC17607839",
                    "title": "Amazing Song",
                    "duration": "PT3M45S",
                    "artist": "The Example Band"
                }]
            }]
        })
    }
}