use crate::presets::DdexVersion;
use crate::versions::{ConversionOptions};
use indexmap::IndexMap;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use quick_xml::{Reader, Writer};
use std::io::Cursor;

#[derive(Debug, Clone)]
pub enum ConversionResult {
    Success {
        xml: String,
        report: ConversionReport,
    },
    Failure {
        error: String,
        report: ConversionReport,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversionReport {
    pub from_version: DdexVersion,
    pub to_version: DdexVersion,
    pub warnings: Vec<ConversionWarning>,
    pub elements_converted: usize,
    pub elements_dropped: usize,
    pub elements_added: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConversionWarningType {
    ElementRenamed,
    ElementDropped,
    ElementAdded,
    ValidationChanged,
    NamespaceChanged,
    FormatMigrated,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversionWarning {
    pub warning_type: ConversionWarningType,
    pub message: String,
    pub element: Option<String>,
}

pub struct VersionConverter {
    conversion_rules: IndexMap<(DdexVersion, DdexVersion), ConversionRules>,
}

#[derive(Debug, Clone)]
struct ConversionRules {
    element_mappings: IndexMap<String, ElementMapping>,
    namespace_mapping: NamespaceMapping,
    field_migrations: Vec<FieldMigration>,
    validation_changes: Vec<ValidationChange>,
}

#[derive(Debug, Clone)]
enum ElementMapping {
    Direct(String),
    Renamed(String),
    Split { into: Vec<String>, splitter: fn(&str) -> Vec<String> },
    Merge { from: Vec<String>, merger: fn(Vec<&str>) -> String },
    Deprecated { replacement: Option<String>, warning: String },
    New { default_value: Option<String> },
}

#[derive(Debug, Clone)]
struct NamespaceMapping {
    from: String,
    to: String,
    schema_version_from: String,
    schema_version_to: String,
}

#[derive(Debug, Clone)]
struct FieldMigration {
    element: String,
    field: String,
    migration_type: MigrationType,
}

#[derive(Debug, Clone)]
enum MigrationType {
    FormatChange { from_pattern: String, to_pattern: String },
    ValueMapping(IndexMap<String, String>),
    ValidationChange { old_rules: Vec<String>, new_rules: Vec<String> },
}

#[derive(Debug, Clone)]
struct ValidationChange {
    element: String,
    change_type: ValidationChangeType,
}

#[derive(Debug, Clone)]
enum ValidationChangeType {
    RequiredAdded(String),
    RequiredRemoved(String),
    OptionalAdded(String),
    OptionalRemoved(String),
    FormatChanged { field: String, old_format: String, new_format: String },
}

impl VersionConverter {
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
            ElementMapping::Direct("SoundRecording".to_string())
        );
        
        element_mappings.insert(
            "TechnicalSoundRecordingDetails".to_string(),
            ElementMapping::Renamed("TechnicalDetails".to_string())
        );
        
        element_mappings.insert(
            "CommercialModelType".to_string(),
            ElementMapping::New { default_value: Some("SubscriptionModel".to_string()) }
        );

        element_mappings.insert(
            "Territory".to_string(),
            ElementMapping::Direct("Territory".to_string())
        );

        let rules = ConversionRules {
            element_mappings,
            namespace_mapping: NamespaceMapping {
                from: "http://ddex.net/xml/ern/382".to_string(),
                to: "http://ddex.net/xml/ern/42".to_string(),
                schema_version_from: "ern/382".to_string(),
                schema_version_to: "ern/42".to_string(),
            },
            field_migrations: vec![
                FieldMigration {
                    element: "Duration".to_string(),
                    field: "value".to_string(),
                    migration_type: MigrationType::FormatChange {
                        from_pattern: r"^PT\d+S$".to_string(),
                        to_pattern: r"^PT(\d+H)?(\d+M)?\d+(\.\d+)?S$".to_string(),
                    },
                },
            ],
            validation_changes: vec![
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

        self.conversion_rules.insert((DdexVersion::Ern382, DdexVersion::Ern42), rules);
    }

    fn add_42_to_43_rules(&mut self) {
        let mut element_mappings = IndexMap::new();
        
        element_mappings.insert(
            "SoundRecording".to_string(),
            ElementMapping::Direct("SoundRecording".to_string())
        );
        
        element_mappings.insert(
            "VideoResource".to_string(),
            ElementMapping::New { default_value: None }
        );
        
        element_mappings.insert(
            "HashSum".to_string(),
            ElementMapping::Direct("HashSum".to_string())
        );

        let rules = ConversionRules {
            element_mappings,
            namespace_mapping: NamespaceMapping {
                from: "http://ddex.net/xml/ern/42".to_string(),
                to: "http://ddex.net/xml/ern/43".to_string(),
                schema_version_from: "ern/42".to_string(),
                schema_version_to: "ern/43".to_string(),
            },
            field_migrations: vec![
                FieldMigration {
                    element: "ISRC".to_string(),
                    field: "value".to_string(),
                    migration_type: MigrationType::ValidationChange {
                        old_rules: vec![r"^[A-Z]{2}[A-Z0-9]{3}\d{7}$".to_string()],
                        new_rules: vec![r"^[A-Z]{2}-?[A-Z0-9]{3}-?\d{2}-?\d{5}$".to_string()],
                    },
                },
            ],
            validation_changes: vec![
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

        self.conversion_rules.insert((DdexVersion::Ern42, DdexVersion::Ern43), rules);
    }

    fn add_43_to_42_rules(&mut self) {
        let mut element_mappings = IndexMap::new();
        
        element_mappings.insert(
            "SoundRecording".to_string(),
            ElementMapping::Direct("SoundRecording".to_string())
        );
        
        element_mappings.insert(
            "VideoResource".to_string(),
            ElementMapping::Deprecated { 
                replacement: None,
                warning: "VideoResource not supported in ERN 4.2, will be omitted".to_string()
            }
        );
        
        element_mappings.insert(
            "HashSum".to_string(),
            ElementMapping::Direct("HashSum".to_string())
        );

        let rules = ConversionRules {
            element_mappings,
            namespace_mapping: NamespaceMapping {
                from: "http://ddex.net/xml/ern/43".to_string(),
                to: "http://ddex.net/xml/ern/42".to_string(),
                schema_version_from: "ern/43".to_string(),
                schema_version_to: "ern/42".to_string(),
            },
            field_migrations: vec![],
            validation_changes: vec![
                ValidationChange {
                    element: "SoundRecording".to_string(),
                    change_type: ValidationChangeType::RequiredRemoved("ProprietaryId".to_string()),
                },
            ],
        };

        self.conversion_rules.insert((DdexVersion::Ern43, DdexVersion::Ern42), rules);
    }

    fn add_42_to_382_rules(&mut self) {
        let mut element_mappings = IndexMap::new();
        
        element_mappings.insert(
            "SoundRecording".to_string(),
            ElementMapping::Direct("SoundRecording".to_string())
        );
        
        element_mappings.insert(
            "TechnicalDetails".to_string(),
            ElementMapping::Renamed("TechnicalSoundRecordingDetails".to_string())
        );
        
        element_mappings.insert(
            "CommercialModelType".to_string(),
            ElementMapping::Deprecated { 
                replacement: None,
                warning: "CommercialModelType not supported in ERN 3.8.2, will be omitted".to_string()
            }
        );

        let rules = ConversionRules {
            element_mappings,
            namespace_mapping: NamespaceMapping {
                from: "http://ddex.net/xml/ern/42".to_string(),
                to: "http://ddex.net/xml/ern/382".to_string(),
                schema_version_from: "ern/42".to_string(),
                schema_version_to: "ern/382".to_string(),
            },
            field_migrations: vec![],
            validation_changes: vec![
                ValidationChange {
                    element: "SoundRecording".to_string(),
                    change_type: ValidationChangeType::OptionalRemoved("HashSum".to_string()),
                },
            ],
        };

        self.conversion_rules.insert((DdexVersion::Ern42, DdexVersion::Ern382), rules);
    }

    pub fn convert(&self, xml_content: &str, from_version: DdexVersion, to_version: DdexVersion, options: Option<ConversionOptions>) -> ConversionResult {
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
                error: format!("No conversion path found from {:?} to {:?}", from_version, to_version),
                report,
            }
        }
    }

    fn find_conversion_path(&self, from: DdexVersion, to: DdexVersion) -> Option<Vec<DdexVersion>> {
        if let Some(_) = self.conversion_rules.get(&(from, to)) {
            return Some(vec![from, to]);
        }

        // Check for multi-step conversions
        match (from, to) {
            (DdexVersion::Ern382, DdexVersion::Ern43) => {
                Some(vec![DdexVersion::Ern382, DdexVersion::Ern42, DdexVersion::Ern43])
            }
            (DdexVersion::Ern43, DdexVersion::Ern382) => {
                Some(vec![DdexVersion::Ern43, DdexVersion::Ern42, DdexVersion::Ern382])
            }
            _ => None,
        }
    }

    fn execute_conversion_path(&self, xml_content: &str, path: &[DdexVersion], options: ConversionOptions, report: &mut ConversionReport) -> ConversionResult {
        let mut current_xml = xml_content.to_string();
        
        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];
            
            match self.convert_single_step(&current_xml, from, to, &options, report) {
                ConversionResult::Success { xml, report: step_report } => {
                    current_xml = xml;
                    report.warnings.extend(step_report.warnings);
                    report.elements_converted += step_report.elements_converted;
                    report.elements_dropped += step_report.elements_dropped;
                    report.elements_added += step_report.elements_added;
                }
                ConversionResult::Failure { error, .. } => {
                    return ConversionResult::Failure { error, report: report.clone() };
                }
            }
        }

        ConversionResult::Success {
            xml: current_xml,
            report: report.clone(),
        }
    }

    fn convert_single_step(&self, xml_content: &str, from: DdexVersion, to: DdexVersion, options: &ConversionOptions, report: &mut ConversionReport) -> ConversionResult {
        let rules = match self.conversion_rules.get(&(from, to)) {
            Some(rules) => rules,
            None => return ConversionResult::Failure {
                error: format!("No direct conversion rules from {:?} to {:?}", from, to),
                report: report.clone(),
            },
        };

        match self.transform_xml(xml_content, rules, options) {
            Ok((transformed_xml, conversion_warnings)) => {
                report.warnings.extend(conversion_warnings);
                ConversionResult::Success {
                    xml: transformed_xml,
                    report: report.clone(),
                }
            }
            Err(error) => ConversionResult::Failure {
                error: error.to_string(),
                report: report.clone(),
            }
        }
    }

    fn transform_xml(&self, xml_content: &str, rules: &ConversionRules, options: &ConversionOptions) -> Result<(String, Vec<ConversionWarning>), Box<dyn std::error::Error>> {
        let mut reader = Reader::from_str(xml_content);
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let mut warnings = Vec::new();
        let mut buf = Vec::new();
        let mut elements_stack = Vec::new();
        let mut skip_element = false;
        let mut skip_depth = 0;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    elements_stack.push(element_name.clone());

                    if skip_element {
                        skip_depth += 1;
                        continue;
                    }

                    match rules.element_mappings.get(&element_name) {
                        Some(ElementMapping::Direct(new_name)) => {
                            let mut new_element = BytesStart::new(new_name);
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    new_element.push_attribute(attr);
                                }
                            }
                            self.update_namespace_attributes(&mut new_element, &rules.namespace_mapping);
                            writer.write_event(Event::Start(new_element))?;
                        }
                        Some(ElementMapping::Renamed(new_name)) => {
                            let mut new_element = BytesStart::new(new_name);
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    new_element.push_attribute(attr);
                                }
                            }
                            self.update_namespace_attributes(&mut new_element, &rules.namespace_mapping);
                            writer.write_event(Event::Start(new_element))?;
                            warnings.push(ConversionWarning {
                                warning_type: ConversionWarningType::ElementRenamed,
                                message: format!("Element '{}' renamed to '{}'", element_name, new_name),
                                element: Some(element_name),
                            });
                        }
                        Some(ElementMapping::Deprecated { replacement: _, warning }) => {
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
                            self.update_namespace_attributes(&mut cloned_element, &rules.namespace_mapping);
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
        Ok((transformed_xml, warnings))
    }

    fn update_namespace_attributes(&self, element: &mut BytesStart, namespace_mapping: &NamespaceMapping) {
        // Update xmlns attributes to new namespace
        let mut attrs_to_update = Vec::new();
        
        for (i, attr_result) in element.attributes().enumerate() {
            if let Ok(attr) = attr_result {
                let key = String::from_utf8_lossy(attr.key.as_ref());
                let value = String::from_utf8_lossy(&attr.value);
                
                if key == "xmlns" && value == namespace_mapping.from {
                    attrs_to_update.push((i, "xmlns".to_string(), namespace_mapping.to.clone()));
                } else if key.starts_with("xmlns:") && value == namespace_mapping.from {
                    attrs_to_update.push((i, key.to_string(), namespace_mapping.to.clone()));
                }
            }
        }

        // Apply namespace updates
        for (_, key, new_value) in attrs_to_update {
            element.extend_attributes(std::iter::once((key.as_str(), new_value.as_str())));
        }
    }

    pub fn get_supported_conversions(&self) -> Vec<(DdexVersion, DdexVersion)> {
        self.conversion_rules.keys().cloned().collect()
    }

    pub fn can_convert(&self, from: DdexVersion, to: DdexVersion) -> bool {
        self.find_conversion_path(from, to).is_some()
    }
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
        assert_eq!(path, Some(vec![DdexVersion::Ern382, DdexVersion::Ern42, DdexVersion::Ern43]));
    }

    #[test]
    fn test_same_version_conversion() {
        let converter = VersionConverter::new();
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><test>content</test>"#;
        let result = converter.convert(xml, DdexVersion::Ern42, DdexVersion::Ern42, None);
        
        match result {
            ConversionResult::Success { xml: result_xml, .. } => {
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