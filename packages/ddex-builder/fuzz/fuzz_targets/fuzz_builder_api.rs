#![no_main]

use libfuzzer_sys::fuzz_target;
use ddex_builder::{Builder, DeterminismConfig};
use ddex_builder::builder::{BuildRequest, BuildOptions};
use ddex_builder::security::{InputValidator, SecurityConfig};
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    xml_content: String,
    preset_name: String,
    use_preset: bool,
}

fuzz_target!(|input: FuzzInput| {
    // Test Builder API with fuzzed inputs
    let mut builder = Builder::new();
    
    // Test preset application with fuzzed name
    if input.use_preset && !input.preset_name.is_empty() {
        let _ = builder.apply_preset(&input.preset_name, false);
    }
    
    // Test input validation
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);
    
    // Validate the XML content first
    if validator.validate_xml_content(&input.xml_content).is_ok() {
        // Only attempt building if validation passes
        let request = BuildRequest {
            source_xml: input.xml_content,
            output_format: ddex_builder::builder::OutputFormat::Xml,
            preset: None,
            validate_schema: false,
        };
        
        let options = BuildOptions::default();
        let _ = builder.build_internal(&request);
    }
});