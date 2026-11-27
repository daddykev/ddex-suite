//! Default output configuration.

#[test]
fn test_output_config() {
    use ddex_builder::OutputConfig;

    let config = OutputConfig::default();
    assert_eq!(config.canon_mode, ddex_builder::output::CanonMode::DbC14n);
    assert_eq!(config.indent_width, 2);
}
