//! Debug test for canonical implementation

use ddex_builder::canonical::*;
use ddex_builder::determinism::DeterminismConfig;

#[test]
fn debug_simple_canonicalization() {
    let config = DeterminismConfig::default();
    let canonicalizer = DB_C14N::new(config);

    let input = r#"<root z="z" a="a">
  <child>content</child>
</root>"#;

    println!("INPUT:\n{}", input);

    let result = canonicalizer.canonicalize(input).unwrap();
    println!("OUTPUT:\n{}", result);

    // Basic assertion to make this a valid test
    assert!(result.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
}
