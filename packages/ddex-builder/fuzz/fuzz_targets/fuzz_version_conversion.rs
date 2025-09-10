#![no_main]

use libfuzzer_sys::fuzz_target;
use ddex_builder::{Builder, DdexVersion};
use ddex_builder::versions::ConversionOptions;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct VersionConversionInput {
    xml_content: String,
    from_version: u8,
    to_version: u8,
}

// Map arbitrary u8 to valid DDEX versions
fn map_to_version(val: u8) -> DdexVersion {
    match val % 4 {
        0 => DdexVersion::Ern382,
        1 => DdexVersion::Ern383,
        2 => DdexVersion::Ern41,
        _ => DdexVersion::Ern43,
    }
}

fuzz_target!(|input: VersionConversionInput| {
    let builder = Builder::new();
    
    // Test version detection with fuzzed XML
    if !input.xml_content.is_empty() {
        let _ = builder.detect_version(&input.xml_content);
    }
    
    // Test version conversion with fuzzed parameters
    let from_version = map_to_version(input.from_version);
    let to_version = map_to_version(input.to_version);
    
    // Test version compatibility check
    let _ = builder.is_version_compatible(from_version, to_version);
    
    // Test version conversion if compatible and XML is valid
    if builder.is_version_compatible(from_version, to_version) 
        && !input.xml_content.is_empty() 
        && input.xml_content.len() < 10000 // Limit size for fuzzing performance
    {
        let options = Some(ConversionOptions::default());
        let _ = builder.convert_version(&input.xml_content, from_version, to_version, options);
    }
});