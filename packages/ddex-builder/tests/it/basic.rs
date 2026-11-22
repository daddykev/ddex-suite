//! Default determinism configuration.

#[test]
fn test_determinism_config() {
    use ddex_builder::DeterminismConfig;

    let config = DeterminismConfig::default();
    assert_eq!(
        config.canon_mode,
        ddex_builder::determinism::CanonMode::DbC14n
    );
    assert_eq!(config.indent_width, 2);

    // Verify locked prefixes are IndexMap
    assert!(!config.locked_prefixes.is_empty());
}
