//! ParseOptions presets.

use ddex_parser::parser::ParseOptions;

#[test]
fn test_parse_options_with_extensions() {
    let opts = ParseOptions::with_extensions();
    assert!(opts.include_raw_extensions);
    assert!(opts.include_comments);
    assert!(opts.preserve_unknown_elements);
}

#[test]
fn test_parse_options_for_round_trip() {
    let opts = ParseOptions::for_round_trip();
    assert!(opts.include_raw_extensions);
    assert!(opts.include_comments);
    assert!(opts.preserve_unknown_elements);
    assert!(!opts.resolve_references);
}
