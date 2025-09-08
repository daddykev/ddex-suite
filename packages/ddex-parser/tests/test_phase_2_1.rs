#[cfg(test)]
mod tests {
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
    
    #[test]
    fn test_to_build_request_exists() {
        // Simple test that verifies to_build_request method exists
        // We'll just test that the code compiles, not create actual instances
        // since MessageHeader and other types don't implement Default
        
        // This test just verifies the method exists on the type
        fn _test_compile_check() {
            use ddex_core::models::graph::ERNMessage;
            
            // This function never gets called, it just checks that the method exists
            fn check_method(msg: ERNMessage) {
                let _build_req = msg.to_build_request();
            }
        }
        
        // Test passes if it compiles
        assert!(true);
    }
}