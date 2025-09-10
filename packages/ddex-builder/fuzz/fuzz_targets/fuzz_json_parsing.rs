#![no_main]

use libfuzzer_sys::fuzz_target;
use ddex_builder::security::{InputValidator, SecurityConfig};

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, handling invalid UTF-8 gracefully
    if let Ok(json_str) = std::str::from_utf8(data) {
        // Test security validation of JSON input
        let config = SecurityConfig::default();
        let validator = InputValidator::new(config);
        
        // Fuzz JSON content validation
        let _ = validator.validate_json_content(json_str);
        
        // Test serde_json parsing with potential malicious input
        let _: Result<serde_json::Value, _> = serde_json::from_str(json_str);
    }
});