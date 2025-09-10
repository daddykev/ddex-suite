#![no_main]

use libfuzzer_sys::fuzz_target;
use ddex_builder::security::{InputValidator, SecurityConfig};
use ddex_builder::error::BuildError;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, handling invalid UTF-8 gracefully
    if let Ok(xml_str) = std::str::from_utf8(data) {
        // Test security validation of XML input
        let config = SecurityConfig::default();
        let validator = InputValidator::new(config);
        
        // Fuzz XML content validation (XXE prevention)
        let _ = validator.validate_xml_content(xml_str);
        
        // Test quick-xml parsing with potential malicious input
        let mut reader = quick_xml::Reader::from_str(xml_str);
        reader.config_mut().expand_empty_elements = false;
        reader.config_mut().trim_text(true);
        
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => {},
                Err(_) => break, // Invalid XML, expected
            }
            buf.clear();
        }
    }
});