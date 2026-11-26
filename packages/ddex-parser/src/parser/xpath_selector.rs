// src/parser/xpath_selector.rs
//! XPath-like selector functionality for efficient XML element selection

use crate::error::ParseError;
use quick_xml::{events::Event, Reader};
use std::collections::HashMap;
use std::io::BufRead;

/// XPath-like selector for extracting specific XML elements and values
#[derive(Debug, Clone)]
pub struct XPathSelector {
    /// Parsed path components
    path: Vec<PathComponent>,
    /// Enable namespace-aware matching
    namespace_aware: bool,
    /// Enable case-sensitive matching
    case_sensitive: bool,
    /// Maximum results to return (0 = unlimited)
    max_results: usize,
    /// Skip validation for better performance
    fast_mode: bool,
    /// Pre-compiled element name cache for performance
    #[allow(dead_code)] // Future optimization feature
    element_cache: std::collections::HashMap<String, String>,
}

/// Component of an XPath expression
#[derive(Debug, Clone, PartialEq)]
pub enum PathComponent {
    /// Element name (e.g., "Release")
    Element(String),
    /// Wildcard match any element (*)
    Wildcard,
    /// Descendant-or-self axis (//)
    DescendantOrSelf,
    /// Attribute selector (`[@attr="value"]`)
    AttributeFilter {
        element: String,
        attribute: String,
        value: Option<String>,
    },
    /// Index selector (`[1]`, `[2]`, etc.)
    IndexFilter { element: String, index: usize },
}

/// Result of XPath selection
#[derive(Debug, Clone)]
pub struct XPathResult {
    /// Selected values
    pub values: Vec<String>,
    /// Element paths where matches were found
    pub paths: Vec<String>,
    /// Attributes found at matching elements
    pub attributes: Vec<HashMap<String, String>>,
    /// Performance statistics
    pub stats: XPathStats,
}

/// Performance statistics for XPath selection
#[derive(Debug, Clone)]
pub struct XPathStats {
    pub elements_processed: usize,
    pub matches_found: usize,
    pub bytes_processed: usize,
    pub duration: std::time::Duration,
}

impl XPathSelector {
    /// Create a new XPath selector
    pub fn new(xpath: &str) -> Result<Self, ParseError> {
        let path = Self::parse_xpath(xpath)?;

        Ok(Self {
            path,
            namespace_aware: true,
            case_sensitive: false,
            max_results: 0,
            fast_mode: false,
            element_cache: HashMap::new(),
        })
    }

    /// Create selector for common DDEX patterns
    pub fn ddex_release_titles() -> Result<Self, ParseError> {
        Self::new("//Release/ReferenceTitle/TitleText")
    }

    /// Create selector for ISRC extraction
    pub fn ddex_isrcs() -> Result<Self, ParseError> {
        Self::new("//SoundRecordingId")
    }

    /// Create selector for release IDs
    pub fn ddex_release_ids() -> Result<Self, ParseError> {
        Self::new("//ReleaseId")
    }

    /// Set namespace awareness
    pub fn namespace_aware(mut self, enabled: bool) -> Self {
        self.namespace_aware = enabled;
        self
    }

    /// Set case sensitivity
    pub fn case_sensitive(mut self, enabled: bool) -> Self {
        self.case_sensitive = enabled;
        self
    }

    /// Set maximum number of results
    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    /// Enable fast mode (skip some validations for better performance)
    pub fn fast_mode(mut self, enabled: bool) -> Self {
        self.fast_mode = enabled;
        self
    }

    /// Select elements matching the XPath expression
    pub fn select<R: BufRead>(&self, reader: R) -> Result<XPathResult, ParseError> {
        let start_time = std::time::Instant::now();
        let mut xml_reader = Reader::from_reader(crate::xml_text::Checked::new(reader));
        xml_reader.config_mut().trim_text(true);

        // Performance optimizations
        if self.fast_mode {
            xml_reader.config_mut().check_end_names = false;
            xml_reader.config_mut().check_comments = false;
        }

        let mut results = Vec::new();
        let mut paths = Vec::new();
        let mut attributes = Vec::new();
        let mut buf = Vec::new();
        let mut current_path = Vec::new();
        let mut capture_context: Vec<CaptureContext> = Vec::new();
        let mut elements_processed = 0;

        let mut text_run = crate::xml_text::TextRun::default();
        loop {
            let event = xml_reader.read_event_into(&mut buf);
            crate::xml_text::check(xml_reader.get_ref())?;
            // One text value per run of Text / GeneralRef events (one Text event before
            // quick-xml 0.38), handled before the event that ends the run, as before.
            if let Ok(ref ev) = event {
                if text_run.push(ev) {
                    buf.clear();
                    continue;
                }
            }
            if let Some(raw) = text_run.take() {
                if !capture_context.is_empty() {
                    // Use utf8_utils for proper UTF-8 handling
                    let current_pos = xml_reader.buffer_position() as usize;
                    let text = crate::utf8_utils::handle_escaped_text(&raw, current_pos)?
                        .trim()
                        .to_string();

                    if !text.is_empty() {
                        let context = capture_context.last().unwrap();
                        results.push(text);
                        paths.push(context.path.clone());
                        attributes.push(context.attributes.clone());

                        if self.max_results > 0 && results.len() >= self.max_results {
                            break;
                        }
                    }
                }
            }
            match event {
                Ok(Event::Start(ref e)) => {
                    elements_processed += 1;
                    let element_name = self.extract_element_name(e.name().as_ref())?;
                    current_path.push(element_name.clone());

                    // Extract attributes for potential filtering
                    let mut attr_map = HashMap::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        attr_map.insert(key, value);
                    }

                    // Check if this element matches our selector with attribute filtering
                    if self.matches_path_with_attributes(&current_path, &attr_map) {
                        capture_context.push(CaptureContext {
                            path: current_path.join("/"),
                            attributes: attr_map,
                            capture_text: true,
                        });
                    }
                }
                Ok(Event::End(_)) => {
                    current_path.pop();

                    // End any active capture
                    if !capture_context.is_empty() {
                        capture_context.pop();
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    elements_processed += 1;
                    let element_name = self.extract_element_name(e.name().as_ref())?;
                    current_path.push(element_name);

                    // Check for match on empty element
                    if self.matches_path(&current_path) {
                        let mut attr_map = HashMap::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            attr_map.insert(key, value);
                        }

                        // For empty elements, capture attribute values or empty string
                        let value = self.get_main_attribute(&attr_map).unwrap_or_default();

                        results.push(value);
                        paths.push(current_path.join("/"));
                        attributes.push(attr_map);

                        if self.max_results > 0 && results.len() >= self.max_results {
                            break;
                        }
                    }

                    current_path.pop();
                }
                Ok(Event::CData(ref e)) => {
                    if !capture_context.is_empty() {
                        let text = String::from_utf8_lossy(e).trim().to_string();
                        if !text.is_empty() {
                            let context = capture_context.last().unwrap();
                            results.push(text);
                            paths.push(context.path.clone());
                            attributes.push(context.attributes.clone());

                            if self.max_results > 0 && results.len() >= self.max_results {
                                break;
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error: {}", e)));
                }
                _ => {} // Skip other events
            }
            buf.clear();
        }

        Ok(XPathResult {
            values: results.clone(),
            paths,
            attributes,
            stats: XPathStats {
                elements_processed,
                matches_found: results.len(),
                bytes_processed: xml_reader.buffer_position() as usize,
                duration: start_time.elapsed(),
            },
        })
    }

    /// Parse XPath expression into path components
    fn parse_xpath(xpath: &str) -> Result<Vec<PathComponent>, ParseError> {
        let mut components = Vec::new();
        let mut parts = Vec::new();

        // Split by '/' but handle '//' specially
        let mut chars = xpath.chars().peekable();
        let mut current = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '/' => {
                    if chars.peek() == Some(&'/') {
                        chars.next(); // consume second '/'
                        if !current.is_empty() {
                            parts.push(current);
                            current = String::new();
                        }
                        parts.push("//".to_string());
                    } else if !current.is_empty() {
                        parts.push(current);
                        current = String::new();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        // Parse each part into components
        for part in parts {
            if part.is_empty() {
                continue;
            } else if part == "//" {
                components.push(PathComponent::DescendantOrSelf);
            } else if part == "*" {
                components.push(PathComponent::Wildcard);
            } else if part.contains('[') && part.contains(']') {
                // Parse attribute or index filters
                let (element, filter) = Self::parse_filter(&part)?;

                if filter.starts_with('@') {
                    // Attribute filter [@attr] or [@attr="value"]
                    let attr_expr = &filter[1..]; // Remove @
                    if let Some(eq_pos) = attr_expr.find('=') {
                        let attr_name = attr_expr[..eq_pos].to_string();
                        let attr_value = attr_expr[eq_pos + 1..]
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();
                        components.push(PathComponent::AttributeFilter {
                            element,
                            attribute: attr_name,
                            value: Some(attr_value),
                        });
                    } else {
                        components.push(PathComponent::AttributeFilter {
                            element,
                            attribute: attr_expr.to_string(),
                            value: None,
                        });
                    }
                } else if let Ok(index) = filter.parse::<usize>() {
                    // Index filter [1], [2], etc.
                    components.push(PathComponent::IndexFilter { element, index });
                } else {
                    return Err(ParseError::XmlError(format!(
                        "Invalid filter expression: [{}]",
                        filter
                    )));
                }
            } else {
                components.push(PathComponent::Element(part));
            }
        }

        Ok(components)
    }

    /// Parse filter expression like `element[filter]`
    fn parse_filter(input: &str) -> Result<(String, String), ParseError> {
        if let Some(bracket_start) = input.find('[') {
            if let Some(bracket_end) = input.rfind(']') {
                let element = input[..bracket_start].to_string();
                let filter = input[bracket_start + 1..bracket_end].to_string();
                return Ok((element, filter));
            }
        }

        Err(ParseError::XmlError(format!(
            "Invalid filter syntax: {}",
            input
        )))
    }

    /// Check if current path matches the selector
    fn matches_path(&self, current: &[String]) -> bool {
        self.match_components(&self.path, current, 0, 0, &HashMap::new())
    }

    /// Check if current path matches with attribute filtering
    fn matches_path_with_attributes(
        &self,
        current: &[String],
        attributes: &HashMap<String, String>,
    ) -> bool {
        self.match_components(&self.path, current, 0, 0, attributes)
    }

    /// Recursively match path components against current path
    fn match_components(
        &self,
        components: &[PathComponent],
        current: &[String],
        comp_idx: usize,
        path_idx: usize,
        attributes: &HashMap<String, String>,
    ) -> bool {
        // If we've matched all components, success
        if comp_idx >= components.len() {
            return true;
        }

        // If we've run out of path but still have components, no match
        if path_idx >= current.len() {
            return false;
        }

        match &components[comp_idx] {
            PathComponent::Element(name) => {
                if self.element_matches(name, &current[path_idx]) {
                    // Exact match, advance both
                    self.match_components(
                        components,
                        current,
                        comp_idx + 1,
                        path_idx + 1,
                        attributes,
                    )
                } else {
                    false
                }
            }
            PathComponent::Wildcard => {
                // Wildcard matches any element, advance both
                self.match_components(components, current, comp_idx + 1, path_idx + 1, attributes)
            }
            PathComponent::DescendantOrSelf => {
                // Try matching the next component at any remaining position
                for i in path_idx..current.len() {
                    if self.match_components(components, current, comp_idx + 1, i, attributes) {
                        return true;
                    }
                }
                false
            }
            PathComponent::AttributeFilter {
                element,
                attribute,
                value,
            } => {
                if self.element_matches(element, &current[path_idx]) {
                    // Check attribute filtering
                    if let Some(attr_value) = attributes.get(attribute) {
                        if let Some(expected_value) = value {
                            // Attribute must have specific value
                            if expected_value == attr_value {
                                self.match_components(
                                    components,
                                    current,
                                    comp_idx + 1,
                                    path_idx + 1,
                                    attributes,
                                )
                            } else {
                                false
                            }
                        } else {
                            // Attribute just needs to exist
                            self.match_components(
                                components,
                                current,
                                comp_idx + 1,
                                path_idx + 1,
                                attributes,
                            )
                        }
                    } else {
                        false // Attribute doesn't exist
                    }
                } else {
                    false
                }
            }
            PathComponent::IndexFilter { element, index } => {
                if self.element_matches(element, &current[path_idx]) {
                    // For index filtering, we'd need to count elements at this level
                    // For now, just match the first occurrence (index 1)
                    if *index == 1 {
                        self.match_components(
                            components,
                            current,
                            comp_idx + 1,
                            path_idx + 1,
                            attributes,
                        )
                    } else {
                        // More sophisticated index tracking would be needed
                        self.match_components(
                            components,
                            current,
                            comp_idx + 1,
                            path_idx + 1,
                            attributes,
                        )
                    }
                } else {
                    false
                }
            }
        }
    }

    /// Check if element name matches, considering namespace and case sensitivity
    fn element_matches(&self, pattern: &str, actual: &str) -> bool {
        let actual_local = if self.namespace_aware {
            // Extract local name after ':'
            actual.split(':').next_back().unwrap_or(actual)
        } else {
            actual
        };

        if self.case_sensitive {
            pattern == actual_local
        } else {
            pattern.eq_ignore_ascii_case(actual_local)
        }
    }

    /// Extract element name from QName bytes
    fn extract_element_name(&self, qname: &[u8]) -> Result<String, ParseError> {
        let name_str = std::str::from_utf8(qname)
            .map_err(|_| ParseError::IoError("Invalid UTF-8 in element name".to_string()))?;

        Ok(name_str.to_string())
    }

    /// Get main attribute value (common patterns like Namespace, value, etc.)
    fn get_main_attribute(&self, attributes: &HashMap<String, String>) -> Option<String> {
        // Try common attribute names
        for attr_name in &["value", "Namespace", "id", "ref"] {
            if let Some(value) = attributes.get(*attr_name) {
                return Some(value.clone());
            }
        }

        // Return first attribute value if any
        attributes.values().next().cloned()
    }
}

/// Context for capturing element content
#[derive(Debug, Clone)]
struct CaptureContext {
    path: String,
    attributes: HashMap<String, String>,
    #[allow(dead_code)] // Future text capture feature
    capture_text: bool,
}

/// Convenience functions for common DDEX XPath patterns
impl XPathSelector {
    /// Select all release titles
    pub fn select_release_titles<R: BufRead>(reader: R) -> Result<Vec<String>, ParseError> {
        let selector = Self::ddex_release_titles()?;
        let result = selector.select(reader)?;
        Ok(result.values)
    }

    /// Select all ISRCs
    pub fn select_isrcs<R: BufRead>(reader: R) -> Result<Vec<String>, ParseError> {
        let selector = Self::ddex_isrcs()?;
        let result = selector.select(reader)?;
        Ok(result.values)
    }

    /// Select elements with custom XPath
    pub fn select_with_xpath<R: BufRead>(
        reader: R,
        xpath: &str,
    ) -> Result<Vec<String>, ParseError> {
        let selector = Self::new(xpath)?;
        let result = selector.select(reader)?;
        Ok(result.values)
    }

    /// Batch selection for multiple XPath expressions
    pub fn select_multiple<R: BufRead>(
        reader: R,
        xpaths: &[&str],
    ) -> Result<Vec<Vec<String>>, ParseError> {
        let mut selectors = Vec::new();
        for xpath in xpaths {
            selectors.push(Self::new(xpath)?.fast_mode(true));
        }

        let mut xml_reader = Reader::from_reader(crate::xml_text::Checked::new(reader));
        xml_reader.config_mut().trim_text(true);
        xml_reader.config_mut().check_end_names = false;
        xml_reader.config_mut().check_comments = false;

        let mut all_results: Vec<Vec<String>> = vec![Vec::new(); selectors.len()];
        let mut buf = Vec::new();
        let mut current_path = Vec::new();
        let mut capture_contexts: Vec<Vec<CaptureContext>> = vec![Vec::new(); selectors.len()];

        let mut text_run = crate::xml_text::TextRun::default();
        loop {
            let event = xml_reader.read_event_into(&mut buf);
            crate::xml_text::check(xml_reader.get_ref())?;
            // One text value per run of Text / GeneralRef events (one Text event before
            // quick-xml 0.38), handled before the event that ends the run, as before.
            if let Ok(ref ev) = event {
                if text_run.push(ev) {
                    buf.clear();
                    continue;
                }
            }
            if let Some(raw) = text_run.take() {
                // Use utf8_utils for proper UTF-8 handling
                let current_pos = xml_reader.buffer_position() as usize;
                let text = crate::utf8_utils::handle_escaped_text(&raw, current_pos)?
                    .trim()
                    .to_string();

                if !text.is_empty() {
                    for (i, contexts) in capture_contexts.iter().enumerate() {
                        if !contexts.is_empty() {
                            all_results[i].push(text.clone());
                        }
                    }
                }
            }
            match event {
                Ok(Event::Start(ref e)) => {
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    current_path.push(element_name.clone());

                    // Extract attributes once for all selectors
                    let mut attr_map = HashMap::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        attr_map.insert(key, value);
                    }

                    // Check against all selectors
                    for (i, selector) in selectors.iter().enumerate() {
                        if selector.matches_path_with_attributes(&current_path, &attr_map) {
                            capture_contexts[i].push(CaptureContext {
                                path: current_path.join("/"),
                                attributes: attr_map.clone(),
                                capture_text: true,
                            });
                        }
                    }
                }
                Ok(Event::End(_)) => {
                    current_path.pop();
                    for contexts in &mut capture_contexts {
                        if !contexts.is_empty() {
                            contexts.pop();
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error: {}", e)));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(all_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_xpath_parsing() {
        let selector = XPathSelector::new("//Release/Title").expect("Valid XPath");
        assert_eq!(selector.path.len(), 3);

        match &selector.path[0] {
            PathComponent::DescendantOrSelf => {}
            _ => panic!("Expected DescendantOrSelf"),
        }

        match &selector.path[1] {
            PathComponent::Element(name) => assert_eq!(name, "Release"),
            _ => panic!("Expected Element(Release)"),
        }
    }

    #[test]
    fn test_simple_element_selection() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:MessageHeader>
                <ern:MessageId>MSG001</ern:MessageId>
            </ern:MessageHeader>
            <ern:ReleaseList>
                <ern:Release>
                    <ern:ReleaseId>REL001</ern:ReleaseId>
                    <ern:ReferenceTitle>
                        <ern:TitleText>My Album Title</ern:TitleText>
                    </ern:ReferenceTitle>
                </ern:Release>
            </ern:ReleaseList>
        </ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let selector = XPathSelector::new("//TitleText").expect("Valid XPath");
        let result = selector.select(cursor).expect("Selection should work");

        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values[0], "My Album Title");
        assert!(result.stats.elements_processed > 0);
    }

    #[test]
    fn test_wildcard_selection() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <root>
            <section>
                <item>Value 1</item>
                <item>Value 2</item>
            </section>
            <section>
                <item>Value 3</item>
            </section>
        </root>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let selector = XPathSelector::new("//*/item").expect("Valid XPath");
        let result = selector.select(cursor).expect("Selection should work");

        assert_eq!(result.values.len(), 3);
        assert!(result.values.contains(&"Value 1".to_string()));
        assert!(result.values.contains(&"Value 2".to_string()));
        assert!(result.values.contains(&"Value 3".to_string()));
    }

    #[test]
    fn test_descendant_or_self() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <root>
            <level1>
                <level2>
                    <target>Deep Value</target>
                </level2>
            </level1>
            <target>Shallow Value</target>
        </root>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let selector = XPathSelector::new("//target").expect("Valid XPath");
        let result = selector.select(cursor).expect("Selection should work");

        assert_eq!(result.values.len(), 2);
        assert!(result.values.contains(&"Deep Value".to_string()));
        assert!(result.values.contains(&"Shallow Value".to_string()));
    }

    #[test]
    fn test_max_results_limit() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <root>
            <item>1</item>
            <item>2</item>
            <item>3</item>
            <item>4</item>
            <item>5</item>
        </root>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let selector = XPathSelector::new("//item")
            .expect("Valid XPath")
            .max_results(3);
        let result = selector.select(cursor).expect("Selection should work");

        assert_eq!(result.values.len(), 3);
        assert_eq!(result.stats.matches_found, 3);
    }

    #[test]
    fn test_namespace_awareness() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:Release>
                <ern:ReleaseId>REL001</ern:ReleaseId>
            </ern:Release>
        </ern:NewReleaseMessage>"#;

        // With namespace awareness (default)
        let cursor1 = Cursor::new(xml.as_bytes());
        let selector1 = XPathSelector::new("//ReleaseId")
            .expect("Valid XPath")
            .namespace_aware(true);
        let result1 = selector1.select(cursor1).expect("Selection should work");
        assert_eq!(result1.values.len(), 1);

        // Without namespace awareness
        let cursor2 = Cursor::new(xml.as_bytes());
        let selector2 = XPathSelector::new("//ReleaseId")
            .expect("Valid XPath")
            .namespace_aware(false);
        let result2 = selector2.select(cursor2).expect("Selection should work");
        assert_eq!(result2.values.len(), 0); // Won't match ern:ReleaseId
    }

    #[test]
    fn test_case_sensitivity() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <root>
            <ReleaseId>REL001</ReleaseId>
            <releaseid>REL002</releaseid>
        </root>"#;

        // Case insensitive (default)
        let cursor1 = Cursor::new(xml.as_bytes());
        let selector1 = XPathSelector::new("//releaseid")
            .expect("Valid XPath")
            .case_sensitive(false);
        let result1 = selector1.select(cursor1).expect("Selection should work");
        assert_eq!(result1.values.len(), 2); // Matches both

        // Case sensitive
        let cursor2 = Cursor::new(xml.as_bytes());
        let selector2 = XPathSelector::new("//releaseid")
            .expect("Valid XPath")
            .case_sensitive(true);
        let result2 = selector2.select(cursor2).expect("Selection should work");
        assert_eq!(result2.values.len(), 1); // Only exact match
    }

    #[test]
    fn test_ddex_convenience_methods() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:ReleaseList>
                <ern:Release>
                    <ern:ReferenceTitle>
                        <ern:TitleText>Album Title</ern:TitleText>
                    </ern:ReferenceTitle>
                </ern:Release>
            </ern:ReleaseList>
            <ern:ResourceList>
                <ern:SoundRecording>
                    <ern:SoundRecordingId Namespace="ISRC">USRC17607839</ern:SoundRecordingId>
                </ern:SoundRecording>
            </ern:ResourceList>
        </ern:NewReleaseMessage>"#;

        // Test release titles
        let cursor1 = Cursor::new(xml.as_bytes());
        let titles = XPathSelector::select_release_titles(cursor1).expect("Should find titles");
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0], "Album Title");

        // Test ISRCs
        let cursor2 = Cursor::new(xml.as_bytes());
        let isrcs = XPathSelector::select_isrcs(cursor2).expect("Should find ISRCs");
        assert_eq!(isrcs.len(), 1);
        assert_eq!(isrcs[0], "USRC17607839");
    }

    #[test]
    fn test_empty_elements_with_attributes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <root>
            <element value="test1"/>
            <element value="test2">content</element>
            <element/>
        </root>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let selector = XPathSelector::new("//element").expect("Valid XPath");
        let result = selector.select(cursor).expect("Selection should work");

        // Should find 3 elements: 2 with values, 1 with content
        assert_eq!(result.values.len(), 3);

        // Check that we captured both attribute values and text content
        assert!(result
            .values
            .iter()
            .any(|v| v == "test1" || v == "test2" || v == "content"));
    }

    #[test]
    fn test_performance_stats() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <root>
            <item>1</item>
            <item>2</item>
            <item>3</item>
        </root>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let selector = XPathSelector::new("//item").expect("Valid XPath");
        let result = selector.select(cursor).expect("Selection should work");

        assert_eq!(result.stats.matches_found, 3);
        assert!(result.stats.elements_processed >= 4); // root + 3 items
        assert!(result.stats.bytes_processed > 0);
        assert!(result.stats.duration.as_nanos() > 0);
    }

    #[test]
    fn test_attribute_filtering() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <root>
            <item type="audio">Audio Item</item>
            <item type="video">Video Item</item>
            <item>No Type</item>
        </root>"#;

        // Test attribute existence
        let cursor1 = Cursor::new(xml.as_bytes());
        let selector1 = XPathSelector::new("//item[@type]").expect("Valid XPath");
        let result1 = selector1.select(cursor1).expect("Selection should work");
        assert_eq!(result1.values.len(), 2); // Only items with type attribute

        // Test attribute value matching
        let cursor2 = Cursor::new(xml.as_bytes());
        let selector2 = XPathSelector::new("//item[@type='audio']").expect("Valid XPath");
        let result2 = selector2.select(cursor2).expect("Selection should work");
        assert_eq!(result2.values.len(), 1);
        assert_eq!(result2.values[0], "Audio Item");
    }
}
