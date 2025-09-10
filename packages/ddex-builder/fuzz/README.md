# DDEX Builder Fuzzing

This directory contains fuzzing targets for security testing of the DDEX Builder.

## Setup

Fuzzing requires the nightly Rust toolchain:

```bash
rustup toolchain install nightly
rustup default nightly
```

## Targets

- `fuzz_xml_parsing` - Tests XML parsing with malicious inputs, XXE prevention
- `fuzz_json_parsing` - Tests JSON parsing with deep nesting and malicious content
- `fuzz_builder_api` - Tests Builder API with arbitrary inputs and presets
- `fuzz_version_conversion` - Tests DDEX version conversion with fuzzed parameters

## Running

```bash
# Run a specific target for 60 seconds
cargo fuzz run fuzz_xml_parsing -- -max_total_time=60

# Run with custom corpus
cargo fuzz run fuzz_xml_parsing corpus/

# List all targets
cargo fuzz list
```

## Note

The fuzzing infrastructure is set up but may require nightly toolchain configuration on some systems. The fuzz targets are ready for security testing once the toolchain issues are resolved.