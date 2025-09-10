//! Testing utilities for integration tests

use sha2::{Sha256, Digest};
use std::io::Read;
use std::path::Path;
use std::fs;

/// Calculate SHA-256 hash of a string
pub fn hash_string(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

/// Calculate SHA-256 hash of a file
pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hex::encode(hasher.finalize()))
}

/// Normalize XML for comparison (remove whitespace variations)
pub fn normalize_xml(xml: &str) -> String {
    xml.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("")
}

/// Compare two XML strings semantically (ignoring formatting differences)
pub fn xml_semantically_equal(xml1: &str, xml2: &str) -> bool {
    normalize_xml(xml1) == normalize_xml(xml2)
}

/// Extract key elements from XML for structural comparison
pub fn extract_xml_structure(xml: &str) -> XMLStructure {
    let mut structure = XMLStructure::default();
    
    // Extract message ID
    if let Some(start) = xml.find("<MessageId>") {
        if let Some(end) = xml.find("</MessageId>") {
            if start < end {
                structure.message_id = Some(xml[start + 11..end].to_string());
            }
        }
    }
    
    // Count releases
    structure.release_count = xml.matches("<Release>").count();
    
    // Count sound recordings
    structure.sound_recording_count = xml.matches("<SoundRecording>").count();
    
    // Extract version
    if let Some(start) = xml.find("MessageSchemaVersionId=\"") {
        if let Some(end) = xml[start + 24..].find("\"") {
            structure.version = Some(xml[start + 24..start + 24 + end].to_string());
        }
    }
    
    // Check if it's a valid DDEX message
    structure.is_ddex = xml.contains("ern:NewReleaseMessage") || xml.contains("xmlns:ern");
    
    structure
}

/// XML structure representation for comparison
#[derive(Debug, Default, PartialEq, Eq)]
pub struct XMLStructure {
    pub message_id: Option<String>,
    pub release_count: usize,
    pub sound_recording_count: usize,
    pub version: Option<String>,
    pub is_ddex: bool,
}

impl XMLStructure {
    /// Check if this structure is compatible with another for round-trip testing
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.message_id == other.message_id
            && self.release_count == other.release_count
            && self.sound_recording_count == other.sound_recording_count
            && self.version == other.version
            && self.is_ddex == other.is_ddex
    }
}

/// Measure execution time of a function
pub fn measure_time<F, R>(f: F) -> (R, std::time::Duration)
where
    F: FnOnce() -> R,
{
    let start = std::time::Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// Format duration in a human-readable way
pub fn format_duration(duration: std::time::Duration) -> String {
    let total_micros = duration.as_micros();
    
    if total_micros < 1_000 {
        format!("{}μs", total_micros)
    } else if total_micros < 1_000_000 {
        format!("{:.2}ms", total_micros as f64 / 1_000.0)
    } else {
        format!("{:.2}s", duration.as_secs_f64())
    }
}

/// Test result with performance metrics
#[derive(Debug)]
pub struct TestResult<T> {
    pub result: T,
    pub parse_time: std::time::Duration,
    pub build_time: std::time::Duration,
    pub total_time: std::time::Duration,
}

impl<T> TestResult<T> {
    pub fn new(result: T, parse_time: std::time::Duration, build_time: std::time::Duration) -> Self {
        let total_time = parse_time + build_time;
        Self {
            result,
            parse_time,
            build_time,
            total_time,
        }
    }
}

/// XML diff utility for debugging failed tests
pub fn create_xml_diff(original: &str, generated: &str) -> String {
    use similar::{ChangeTag, TextDiff};
    
    let diff = TextDiff::from_lines(original, generated);
    let mut result = String::new();
    
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        result.push_str(&format!("{}{}", sign, change));
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hash_string() {
        let data = "test data";
        let hash = hash_string(data);
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
        
        // Same input should produce same hash
        assert_eq!(hash, hash_string(data));
    }

    #[test]
    fn test_normalize_xml() {
        let xml = r#"  <root>
            <child>value</child>
        </root>  "#;
        let normalized = normalize_xml(xml);
        assert_eq!(normalized, "<root><child>value</child></root>");
    }

    #[test]
    fn test_xml_structure_extraction() {
        let xml = r#"<ern:NewReleaseMessage MessageSchemaVersionId="ern/43">
            <MessageId>TEST_001</MessageId>
            <Release></Release>
            <SoundRecording></SoundRecording>
            <SoundRecording></SoundRecording>
        </ern:NewReleaseMessage>"#;
        
        let structure = extract_xml_structure(xml);
        assert_eq!(structure.message_id, Some("TEST_001".to_string()));
        assert_eq!(structure.release_count, 1);
        assert_eq!(structure.sound_recording_count, 2);
        assert_eq!(structure.version, Some("ern/43".to_string()));
        assert!(structure.is_ddex);
    }

    #[test]
    fn test_measure_time() {
        let (result, duration) = measure_time(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        assert!(duration >= std::time::Duration::from_millis(10));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(std::time::Duration::from_micros(500)), "500μs");
        assert_eq!(format_duration(std::time::Duration::from_millis(15)), "15.00ms");
        assert_eq!(format_duration(std::time::Duration::from_secs(2)), "2.00s");
    }
}