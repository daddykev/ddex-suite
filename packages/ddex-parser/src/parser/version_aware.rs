// core/src/parser/version_aware.rs
//! Version-aware parsing with adaptation

use crate::error::ParseError;
use ddex_core::models::flat::ParsedERNMessage;
use ddex_core::models::graph::ERNMessage;
use crate::parser::detector::{ERNVersion, VersionDetector};
use crate::transform::version_adapter::VersionAdapter;
use std::io::{BufRead, Seek};

pub struct VersionAwareParser;

impl VersionAwareParser {
    pub fn parse<R: BufRead + Seek>(
        mut reader: R,
    ) -> Result<ParsedERNMessage, ParseError> {
        // Detect version
        let version = VersionDetector::detect(&mut reader)?;
        reader.seek(std::io::SeekFrom::Start(0))?;
        
        // Create version-specific adapter
        let adapter = VersionAdapter::new(version);
        
        // Parse with version awareness
        let graph = match version {
            ERNVersion::V3_8_2 => Self::parse_382(reader, adapter)?,
            ERNVersion::V4_2 => Self::parse_42(reader, adapter)?,
            ERNVersion::V4_3 => Self::parse_43(reader, adapter)?,
        };
        
        // Transform to flat model
        use crate::transform::flatten::Flattener;
        let flat = Flattener::flatten(graph.clone());
        
        Ok(ParsedERNMessage { graph, flat })
    }
    
    fn parse_382<R: BufRead>(
        reader: R,
        adapter: VersionAdapter,
    ) -> Result<ERNMessage, ParseError> {
        // Parse with 3.8.2 specific handling
        // Handle missing MessageThreadId
        // Convert single values to arrays
        // Add default values for missing 4.x fields
        todo!("Implement 3.8.2 parsing")
    }
    
    fn parse_42<R: BufRead>(
        reader: R,
        adapter: VersionAdapter,
    ) -> Result<ERNMessage, ParseError> {
        // Parse with 4.2 specific handling
        // Handle MessageAuditTrail
        // Process TechnicalInstantiation
        todo!("Implement 4.2 parsing")
    }
    
    fn parse_43<R: BufRead>(
        reader: R,
        adapter: VersionAdapter,
    ) -> Result<ERNMessage, ParseError> {
        // Parse with 4.3 specific handling
        // Handle ResourceGroups
        // Process ChapterInformation
        // Extended DealTerms
        todo!("Implement 4.3 parsing")
    }
}