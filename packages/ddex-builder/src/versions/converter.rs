use super::DdexVersion;
use crate::versions::ConversionOptions;
use indexmap::IndexMap;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;

/// Result of version conversion operation
#[derive(Debug, Clone)]
pub enum ConversionResult {
    /// Successful conversion
    Success {
        /// Converted XML content
        xml: String,
        /// Conversion report with warnings
        report: ConversionReport,
    },
    /// Conversion failed
    Failure {
        /// Error description
        error: String,
        /// Partial conversion report
        report: ConversionReport,
    },
}

/// Report of conversion process between DDEX versions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversionReport {
    /// Source DDEX version
    pub from_version: DdexVersion,
    /// Target DDEX version
    pub to_version: DdexVersion,
    /// List of conversion warnings
    pub warnings: Vec<ConversionWarning>,
    /// Number of elements successfully converted
    pub elements_converted: usize,
    /// Number of elements dropped (not supported in target version)
    pub elements_dropped: usize,
    /// Number of elements added (required in target version)
    pub elements_added: usize,
}

/// Type of conversion warning
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ConversionWarningType {
    /// Element was renamed in target version
    ElementRenamed,
    /// Element not supported in target version
    ElementDropped,
    /// Element added with default value
    ElementAdded,
    /// Validation rules changed between versions
    ValidationChanged,
    /// Namespace changes required
    NamespaceChanged,
    /// Format migration performed
    FormatMigrated,
}

/// Warning generated during conversion
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversionWarning {
    /// Type of warning
    pub warning_type: ConversionWarningType,
    /// Warning description
    pub message: String,
    /// Element affected if applicable
    pub element: Option<String>,
}

/// Handles conversion between different DDEX versions
pub struct VersionConverter {
    conversion_rules: IndexMap<(DdexVersion, DdexVersion), ConversionRules>,
}

#[derive(Debug, Clone)]
struct ConversionRules {
    element_mappings: IndexMap<String, ElementMapping>,
    namespace_mapping: NamespaceMapping,
    _field_migrations: Vec<FieldMigration>,
    _validation_changes: Vec<ValidationChange>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ElementMapping {
    Direct(String),
    Renamed(String),
    /// These variants may be used in future versions
    Split {
        into: Vec<String>,
        splitter: fn(&str) -> Vec<String>,
    },
    /// These variants may be used in future versions
    Merge {
        from: Vec<String>,
        merger: fn(Vec<&str>) -> String,
    },
    Deprecated {
        replacement: Option<String>,
        warning: String,
    },
    New {
        default_value: Option<String>,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct NamespaceMapping {
    from: String,
    to: String,
    schema_version_from: String,
    schema_version_to: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct FieldMigration {
    element: String,
    field: String,
    migration_type: MigrationType,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum MigrationType {
    FormatChange {
        from_pattern: String,
        to_pattern: String,
    },
    ValueMapping(IndexMap<String, String>),
    ValidationChange {
        old_rules: Vec<String>,
        new_rules: Vec<String>,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ValidationChange {
    element: String,
    change_type: ValidationChangeType,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum ValidationChangeType {
    RequiredAdded(String),
    RequiredRemoved(String),
    OptionalAdded(String),
    OptionalRemoved(String),
    FormatChanged {
        field: String,
        old_format: String,
        new_format: String,
    },
}

impl VersionConverter {
    /// Create a new version converter
    pub fn new() -> Self {
        let mut converter = Self {
            conversion_rules: IndexMap::new(),
        };
        converter.initialize_conversion_rules();
        converter
    }

    fn initialize_conversion_rules(&mut self) {
        self.add_382_to_42_rules();
        self.add_42_to_43_rules();
        self.add_43_to_42_rules();
        self.add_42_to_382_rules();
    }

    fn add_382_to_42_rules(&mut self) {
        let mut element_mappings = IndexMap::new();

        element_mappings.insert(
            "SoundRecording".to_string(),
            ElementMapping::Direct("SoundRecording".to_string()),
        );

        element_mappings.insert(
            "TechnicalSoundRecordingDetails".to_string(),
            ElementMapping::Renamed("TechnicalDetails".to_string()),
        );

        element_mappings.insert(
            "CommercialModelType".to_string(),
            ElementMapping::New {
                default_value: Some("SubscriptionModel".to_string()),
            },
        );

        element_mappings.insert(
            "Territory".to_string(),
            ElementMapping::Direct("Territory".to_string()),
        );

        let rules = ConversionRules {
            element_mappings,
            namespace_mapping: NamespaceMapping {
                from: "http://ddex.net/xml/ern/382".to_string(),
                to: "http://ddex.net/xml/ern/42".to_string(),
                schema_version_from: "ern/382".to_string(),
                schema_version_to: "ern/42".to_string(),
            },
            _field_migrations: vec![FieldMigration {
                element: "Duration".to_string(),
                field: "value".to_string(),
                migration_type: MigrationType::FormatChange {
                    from_pattern: r"^PT\d+S$".to_string(),
                    to_pattern: r"^PT(\d+H)?(\d+M)?\d+(\.\d+)?S$".to_string(),
                },
            }],
            _validation_changes: vec![
                ValidationChange {
                    element: "SoundRecording".to_string(),
                    change_type: ValidationChangeType::OptionalAdded("HashSum".to_string()),
                },
                ValidationChange {
                    element: "TechnicalDetails".to_string(),
                    change_type: ValidationChangeType::OptionalAdded("BitRate".to_string()),
                },
            ],
        };

        self.conversion_rules
            .insert((DdexVersion::Ern382, DdexVersion::Ern42), rules);
    }

    fn add_42_to_43_rules(&mut self) {
        let mut element_mappings = IndexMap::new();

        element_mappings.insert(
            "SoundRecording".to_string(),
            ElementMapping::Direct("SoundRecording".to_string()),
        );

        element_mappings.insert(
            "VideoResource".to_string(),
            ElementMapping::New {
                default_value: None,
            },
        );

        element_mappings.insert(
            "HashSum".to_string(),
            ElementMapping::Direct("HashSum".to_string()),
        );

        let rules = ConversionRules {
            element_mappings,
            namespace_mapping: NamespaceMapping {
                from: "http://ddex.net/xml/ern/42".to_string(),
                to: "http://ddex.net/xml/ern/43".to_string(),
                schema_version_from: "ern/42".to_string(),
                schema_version_to: "ern/43".to_string(),
            },
            _field_migrations: vec![FieldMigration {
                element: "ISRC".to_string(),
                field: "value".to_string(),
                migration_type: MigrationType::ValidationChange {
                    old_rules: vec![r"^[A-Z]{2}[A-Z0-9]{3}\d{7}$".to_string()],
                    new_rules: vec![r"^[A-Z]{2}-?[A-Z0-9]{3}-?\d{2}-?\d{5}$".to_string()],
                },
            }],
            _validation_changes: vec![
                ValidationChange {
                    element: "SoundRecording".to_string(),
                    change_type: ValidationChangeType::RequiredAdded("ProprietaryId".to_string()),
                },
                ValidationChange {
                    element: "VideoResource".to_string(),
                    change_type: ValidationChangeType::OptionalAdded("Duration".to_string()),
                },
            ],
        };

        self.conversion_rules
            .insert((DdexVersion::Ern42, DdexVersion::Ern43), rules);
    }

    fn add_43_to_42_rules(&mut self) {
        let mut element_mappings = IndexMap::new();

        element_mappings.insert(
            "SoundRecording".to_string(),
            ElementMapping::Direct("SoundRecording".to_string()),
        );

        element_mappings.insert(
            "VideoResource".to_string(),
            ElementMapping::Deprecated {
                replacement: None,
                warning: "VideoResource not supported in ERN 4.2, will be omitted".to_string(),
            },
        );

        element_mappings.insert(
            "HashSum".to_string(),
            ElementMapping::Direct("HashSum".to_string()),
        );

        let rules = ConversionRules {
            element_mappings,
            namespace_mapping: NamespaceMapping {
                from: "http://ddex.net/xml/ern/43".to_string(),
                to: "http://ddex.net/xml/ern/42".to_string(),
                schema_version_from: "ern/43".to_string(),
                schema_version_to: "ern/42".to_string(),
            },
            _field_migrations: vec![],
            _validation_changes: vec![ValidationChange {
                element: "SoundRecording".to_string(),
                change_type: ValidationChangeType::RequiredRemoved("ProprietaryId".to_string()),
            }],
        };

        self.conversion_rules
            .insert((DdexVersion::Ern43, DdexVersion::Ern42), rules);
    }

    fn add_42_to_382_rules(&mut self) {
        let mut element_mappings = IndexMap::new();

        element_mappings.insert(
            "SoundRecording".to_string(),
            ElementMapping::Direct("SoundRecording".to_string()),
        );

        element_mappings.insert(
            "TechnicalDetails".to_string(),
            ElementMapping::Renamed("TechnicalSoundRecordingDetails".to_string()),
        );

        element_mappings.insert(
            "CommercialModelType".to_string(),
            ElementMapping::Deprecated {
                replacement: None,
                warning: "CommercialModelType not supported in ERN 3.8.2, will be omitted"
                    .to_string(),
            },
        );

        let rules = ConversionRules {
            element_mappings,
            namespace_mapping: NamespaceMapping {
                from: "http://ddex.net/xml/ern/42".to_string(),
                to: "http://ddex.net/xml/ern/382".to_string(),
                schema_version_from: "ern/42".to_string(),
                schema_version_to: "ern/382".to_string(),
            },
            _field_migrations: vec![],
            _validation_changes: vec![ValidationChange {
                element: "SoundRecording".to_string(),
                change_type: ValidationChangeType::OptionalRemoved("HashSum".to_string()),
            }],
        };

        self.conversion_rules
            .insert((DdexVersion::Ern42, DdexVersion::Ern382), rules);
    }

    /// Convert DDEX content between versions
    ///
    /// # Arguments
    /// * `xml_content` - Source XML content
    /// * `from_version` - Source DDEX version
    /// * `to_version` - Target DDEX version
    /// * `options` - Optional conversion configuration
    pub fn convert(
        &self,
        xml_content: &str,
        from_version: DdexVersion,
        to_version: DdexVersion,
        options: Option<ConversionOptions>,
    ) -> ConversionResult {
        let options = options.unwrap_or_default();
        let mut report = ConversionReport {
            from_version,
            to_version,
            warnings: Vec::new(),
            elements_converted: 0,
            elements_dropped: 0,
            elements_added: 0,
        };

        if from_version == to_version {
            return ConversionResult::Success {
                xml: xml_content.to_string(),
                report,
            };
        }

        let conversion_path = self.find_conversion_path(from_version, to_version);
        match conversion_path {
            Some(path) => self.execute_conversion_path(xml_content, &path, options, &mut report),
            None => ConversionResult::Failure {
                error: format!(
                    "No conversion path found from {:?} to {:?}",
                    from_version, to_version
                ),
                report,
            },
        }
    }

    fn find_conversion_path(&self, from: DdexVersion, to: DdexVersion) -> Option<Vec<DdexVersion>> {
        if let Some(_) = self.conversion_rules.get(&(from, to)) {
            return Some(vec![from, to]);
        }

        // Check for multi-step conversions
        match (from, to) {
            (DdexVersion::Ern382, DdexVersion::Ern43) => Some(vec![
                DdexVersion::Ern382,
                DdexVersion::Ern42,
                DdexVersion::Ern43,
            ]),
            (DdexVersion::Ern43, DdexVersion::Ern382) => Some(vec![
                DdexVersion::Ern43,
                DdexVersion::Ern42,
                DdexVersion::Ern382,
            ]),
            _ => None,
        }
    }

    fn execute_conversion_path(
        &self,
        xml_content: &str,
        path: &[DdexVersion],
        options: ConversionOptions,
        report: &mut ConversionReport,
    ) -> ConversionResult {
        let mut current_xml = xml_content.to_string();

        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];

            match self.convert_single_step(&current_xml, from, to, &options, report) {
                // The step updated `report` itself (its returned report is a copy of it).
                ConversionResult::Success { xml, .. } => {
                    current_xml = xml;
                }
                ConversionResult::Failure { error, .. } => {
                    return ConversionResult::Failure {
                        error,
                        report: report.clone(),
                    };
                }
            }
        }

        ConversionResult::Success {
            xml: current_xml,
            report: report.clone(),
        }
    }

    fn convert_single_step(
        &self,
        xml_content: &str,
        from: DdexVersion,
        to: DdexVersion,
        options: &ConversionOptions,
        report: &mut ConversionReport,
    ) -> ConversionResult {
        let rules = match self.conversion_rules.get(&(from, to)) {
            Some(rules) => rules,
            None => {
                return ConversionResult::Failure {
                    error: format!("No direct conversion rules from {:?} to {:?}", from, to),
                    report: report.clone(),
                }
            }
        };

        match self.transform_xml(xml_content, rules, options) {
            Ok((transformed_xml, conversion_warnings, counts)) => {
                report.warnings.extend(conversion_warnings);
                report.elements_converted += counts.converted;
                report.elements_dropped += counts.dropped;
                ConversionResult::Success {
                    xml: transformed_xml,
                    report: report.clone(),
                }
            }
            Err(error) => ConversionResult::Failure {
                error: error.to_string(),
                report: report.clone(),
            },
        }
    }

    fn transform_xml(
        &self,
        xml_content: &str,
        rules: &ConversionRules,
        options: &ConversionOptions,
    ) -> Result<(String, Vec<ConversionWarning>, ElementCounts), Box<dyn std::error::Error>> {
        // A reference to an undefined entity (`&foo;`, no DTD) is an error, not copied.
        ddex_core::xml_refs::check_document(xml_content.as_bytes())?;
        let mut reader = Reader::from_str(xml_content);
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let mut warnings = Vec::new();
        let mut buf = Vec::new();
        let mut elements_stack = Vec::new();
        let mut skip_element = false;
        let mut skip_depth = 0;
        let mut counts = ElementCounts::default();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    elements_stack.push(element_name.clone());

                    if skip_element {
                        skip_depth += 1;
                        continue;
                    }
                    if matches!(
                        rules.element_mappings.get(&element_name),
                        Some(ElementMapping::Deprecated { .. })
                    ) {
                        counts.dropped += 1;
                    } else {
                        counts.converted += 1;
                    }

                    match rules.element_mappings.get(&element_name) {
                        Some(ElementMapping::Direct(new_name)) => {
                            let mut new_element = BytesStart::new(new_name);
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    new_element.push_attribute(attr);
                                }
                            }
                            self.update_namespace_attributes(
                                &mut new_element,
                                &rules.namespace_mapping,
                            );
                            writer.write_event(Event::Start(new_element))?;
                        }
                        Some(ElementMapping::Renamed(new_name)) => {
                            let mut new_element = BytesStart::new(new_name);
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    new_element.push_attribute(attr);
                                }
                            }
                            self.update_namespace_attributes(
                                &mut new_element,
                                &rules.namespace_mapping,
                            );
                            writer.write_event(Event::Start(new_element))?;
                            warnings.push(ConversionWarning {
                                warning_type: ConversionWarningType::ElementRenamed,
                                message: format!(
                                    "Element '{}' renamed to '{}'",
                                    element_name, new_name
                                ),
                                element: Some(element_name),
                            });
                        }
                        Some(ElementMapping::Deprecated {
                            replacement: _,
                            warning,
                        }) => {
                            skip_element = true;
                            skip_depth = 1;
                            warnings.push(ConversionWarning {
                                warning_type: ConversionWarningType::ElementDropped,
                                message: warning.clone(),
                                element: Some(element_name),
                            });
                        }
                        Some(ElementMapping::New { .. }) => {
                            writer.write_event(Event::Start(e.clone()))?;
                        }
                        _ => {
                            let mut cloned_element = e.clone();
                            self.update_namespace_attributes(
                                &mut cloned_element,
                                &rules.namespace_mapping,
                            );
                            writer.write_event(Event::Start(cloned_element))?;
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    elements_stack.pop();

                    if skip_element {
                        skip_depth -= 1;
                        if skip_depth == 0 {
                            skip_element = false;
                        }
                        continue;
                    }

                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match rules.element_mappings.get(&element_name) {
                        Some(ElementMapping::Direct(new_name)) => {
                            writer.write_event(Event::End(BytesEnd::new(new_name)))?;
                        }
                        Some(ElementMapping::Renamed(new_name)) => {
                            writer.write_event(Event::End(BytesEnd::new(new_name)))?;
                        }
                        Some(ElementMapping::Deprecated { .. }) => {
                            // Skip deprecated elements
                        }
                        _ => {
                            writer.write_event(Event::End(e.clone()))?;
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if !skip_element {
                        writer.write_event(Event::Text(e.clone()))?;
                    }
                }
                // A reference in character data (its own event since quick-xml 0.38), written
                // back unchanged like the text around it.
                Ok(Event::GeneralRef(ref e)) => {
                    if !skip_element {
                        writer.write_event(Event::GeneralRef(e.clone()))?;
                    }
                }
                Ok(Event::Comment(ref e)) => {
                    if !skip_element && options.preserve_comments {
                        writer.write_event(Event::Comment(e.clone()))?;
                    }
                }
                Ok(Event::CData(ref e)) => {
                    if !skip_element {
                        writer.write_event(Event::CData(e.clone()))?;
                    }
                }
                Ok(Event::Decl(ref e)) => {
                    writer.write_event(Event::Decl(e.clone()))?;
                }
                Ok(Event::PI(ref e)) => {
                    if !skip_element {
                        writer.write_event(Event::PI(e.clone()))?;
                    }
                }
                Ok(Event::DocType(ref e)) => {
                    writer.write_event(Event::DocType(e.clone()))?;
                }
                Ok(Event::Empty(ref e)) => {
                    if !skip_element {
                        counts.converted += 1;
                        writer.write_event(Event::Empty(e.clone()))?;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("Error parsing XML: {}", e).into()),
            }
            buf.clear();
        }

        let result = writer.into_inner().into_inner();
        let transformed_xml = String::from_utf8(result)?;
        Ok((transformed_xml, warnings, counts))
    }

    /// Replaces the value of each namespace declaration (`xmlns`, `xmlns:*`) that names the
    /// source namespace with the target namespace. (Before 0.5.1 the new declaration was
    /// appended, so the element had the attribute twice: not well-formed XML.)
    fn update_namespace_attributes(
        &self,
        element: &mut BytesStart,
        namespace_mapping: &NamespaceMapping,
    ) {
        let attrs: Vec<(Vec<u8>, Vec<u8>)> = element
            .attributes()
            .flatten()
            .map(|attr| {
                let key = attr.key.as_ref().to_vec();
                let is_declaration = key == b"xmlns" || key.starts_with(b"xmlns:");
                let value =
                    if is_declaration && attr.value.as_ref() == namespace_mapping.from.as_bytes() {
                        namespace_mapping.to.as_bytes().to_vec()
                    } else {
                        attr.value.to_vec()
                    };
                (key, value)
            })
            .collect();
        let mut updated =
            BytesStart::new(String::from_utf8_lossy(element.name().as_ref()).into_owned());
        for (key, value) in &attrs {
            updated.push_attribute((key.as_slice(), value.as_slice()));
        }
        *element = updated;
    }

    /// Get list of supported version conversions
    pub fn get_supported_conversions(&self) -> Vec<(DdexVersion, DdexVersion)> {
        self.conversion_rules.keys().cloned().collect()
    }

    /// Check if conversion between versions is supported
    pub fn can_convert(&self, from: DdexVersion, to: DdexVersion) -> bool {
        self.find_conversion_path(from, to).is_some()
    }
}

/// Elements written (`converted`) and left out as deprecated (`dropped`) by one conversion
/// step; no step adds elements, so `ConversionReport::elements_added` stays 0.
#[derive(Debug, Default, Clone, Copy)]
struct ElementCounts {
    converted: usize,
    dropped: usize,
}

impl Default for VersionConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter_initialization() {
        let converter = VersionConverter::new();
        assert!(!converter.conversion_rules.is_empty());
    }

    #[test]
    fn test_direct_conversion_path() {
        let converter = VersionConverter::new();
        let path = converter.find_conversion_path(DdexVersion::Ern382, DdexVersion::Ern42);
        assert_eq!(path, Some(vec![DdexVersion::Ern382, DdexVersion::Ern42]));
    }

    #[test]
    fn test_multi_step_conversion_path() {
        let converter = VersionConverter::new();
        let path = converter.find_conversion_path(DdexVersion::Ern382, DdexVersion::Ern43);
        assert_eq!(
            path,
            Some(vec![
                DdexVersion::Ern382,
                DdexVersion::Ern42,
                DdexVersion::Ern43
            ])
        );
    }

    #[test]
    fn test_same_version_conversion() {
        let converter = VersionConverter::new();
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><test>content</test>"#;
        let result = converter.convert(xml, DdexVersion::Ern42, DdexVersion::Ern42, None);

        match result {
            ConversionResult::Success {
                xml: result_xml, ..
            } => {
                assert_eq!(result_xml, xml);
            }
            _ => panic!("Expected successful conversion for same version"),
        }
    }

    #[test]
    fn test_supported_conversions() {
        let converter = VersionConverter::new();
        let conversions = converter.get_supported_conversions();
        assert!(conversions.contains(&(DdexVersion::Ern382, DdexVersion::Ern42)));
        assert!(conversions.contains(&(DdexVersion::Ern42, DdexVersion::Ern43)));
        assert!(conversions.contains(&(DdexVersion::Ern43, DdexVersion::Ern42)));
        assert!(conversions.contains(&(DdexVersion::Ern42, DdexVersion::Ern382)));
    }
}
