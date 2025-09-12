// core/src/parser/dom.rs
//! DOM-based parser for smaller DDEX files

use crate::error::ParseError;
use ddex_core::models::flat::ParsedERNMessage;
use ddex_core::models::graph::ERNMessage;
use ddex_core::models::versions::ERNVersion;
use crate::parser::ParseOptions;
use crate::parser::namespace_detector::{NamespaceDetector, NamespaceContext};
use crate::transform::{graph::GraphBuilder, flatten::Flattener};
use std::io::{BufRead, Seek, SeekFrom};
use std::time::Instant;

/// Parse using DOM for smaller files
pub fn parse_dom<R: BufRead + Seek>(
    mut reader: R,
    version: ERNVersion,
    options: ParseOptions,
) -> Result<ParsedERNMessage, ParseError> {
    let start = Instant::now();
    
    // Check timeout
    if !options.allow_blocking && options.timeout_ms > 0 {
        // Would implement timeout checking
    }
    
    // First pass: detect namespaces
    let mut namespace_detector = NamespaceDetector::new();
    let namespace_result = namespace_detector.detect_from_xml(&mut reader)?;
    let namespace_context = NamespaceContext::from_detection_result(namespace_result);
    
    // Reset reader for second pass
    reader.seek(SeekFrom::Start(0))?;
    
    // Build graph model from XML with namespace context
    let graph_builder = GraphBuilder::new(version);
    let graph = graph_builder.build_from_xml_with_context(reader, namespace_context)?;
    
    // Optionally resolve references
    let graph = if options.resolve_references {
        resolve_references(graph)?
    } else {
        graph
    };
    
    // Flatten to developer-friendly model
    let flat = Flattener::flatten(graph.clone());
    
    // Check elapsed time
    let elapsed = start.elapsed();
    if elapsed.as_millis() > options.timeout_ms as u128 {
        return Err(ParseError::Timeout {
            seconds: elapsed.as_secs(),
        });
    }
    
    Ok(ParsedERNMessage {
        graph,
        flat,
        extensions: None,
    })
}

fn resolve_references(message: ERNMessage) -> Result<ERNMessage, ParseError> {
    use crate::transform::resolve::ReferenceResolver;
    
    let mut resolver = ReferenceResolver::new();
    resolver.build_maps(&message);
    
    // Check for unresolved references
    let unresolved = resolver.validate_references(&message);
    if !unresolved.is_empty() {
        // Log warnings but don't fail
        for uref in unresolved {
            tracing::warn!(
                "Unresolved reference: {} -> {} at {}",
                uref.reference_type,
                uref.reference_value,
                uref.location
            );
        }
    }
    
    Ok(message)
}