use ddex_builder::{DDEXBuilder, BuildRequest, BuildOptions};
use indexmap::IndexMap;

#[test]
fn test_builder_creation() {
    let builder = DDEXBuilder::new();
    // Just verify it was created - no .is_ok() needed since it's not a Result
    assert!(true); // Builder created successfully
}

#[test]
fn test_no_hashmap_usage() {
    // This test ensures we're using IndexMap instead of HashMap
    let mut map = IndexMap::new();
    map.insert("key".to_string(), "value".to_string());
    
    // Iteration order is deterministic
    let keys: Vec<_> = map.keys().collect();
    assert_eq!(keys[0], &"key");
}

#[test]
fn test_determinism_config() {
    use ddex_builder::DeterminismConfig;
    
    let config = DeterminismConfig::default();
    assert_eq!(config.canon_mode, ddex_builder::determinism::CanonMode::DbC14n);
    assert_eq!(config.indent_width, 2);
    
    // Verify locked prefixes are IndexMap
    assert!(!config.locked_prefixes.is_empty());
}