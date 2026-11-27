#![no_main]

use libfuzzer_sys::fuzz_target;
use ddex_builder::{Builder, DeterminismConfig};
use ddex_builder::builder::{BuildRequest, BuildOptions};
use ddex_builder::security::{InputValidator, SecurityConfig};
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    xml_content: String,
}

fuzz_target!(|input: FuzzInput| {
    // Test Builder API with fuzzed inputs
    let builder = Builder::new();
    
    // Test input validation
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);
    
    // Validate the XML content first
    if validator.validate_xml_content(&input.xml_content).is_ok() {
        // Only attempt building if validation passes
        let request = BuildRequest {
            source_xml: input.xml_content,
            output_format: ddex_builder::builder::OutputFormat::Xml,
            validate_schema: false,
            ..Default::default()
        };
        
        let options = BuildOptions::default();
        let _ = builder.build_internal(&request);
    }
});