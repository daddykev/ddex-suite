// core/src/transform/version_adapter.rs
//! Version-specific transformation and adaptation

use crate::error::ParseError;
use ddex_core::models::versions::ERNVersion;
use ddex_core::models::versions::VersionDifferences;

pub struct VersionAdapter {
    _version: ERNVersion,
    _differences: VersionDifferences,
}

impl VersionAdapter {
    pub fn new(version: ERNVersion) -> Self {
        Self {
            _version: version,
            _differences: VersionDifferences::for_version(version),
        }
    }

    /// Not implemented in 0.5.x: returns an error (before 0.5.1 it ignored the XML and
    /// returned a header with the message ID "PLACEHOLDER" and the current time). The
    /// parser reads headers with `DDEXParser::parse`.
    pub fn adapt_message_header(
        &self,
        _xml_data: &[u8],
    ) -> Result<ddex_core::models::graph::MessageHeader, ParseError> {
        Err(not_implemented("VersionAdapter::adapt_message_header"))
    }

    /// Not implemented in 0.5.x: returns an error (before 0.5.1 it ignored the XML and
    /// returned empty deal terms). The parser reads deals with `DDEXParser::parse`.
    pub fn adapt_deal_terms(
        &self,
        _xml_data: &[u8],
    ) -> Result<ddex_core::models::graph::DealTerms, ParseError> {
        Err(not_implemented("VersionAdapter::adapt_deal_terms"))
    }
}

fn not_implemented(api: &str) -> ParseError {
    ParseError::InvalidValue {
        field: api.to_string(),
        value: "not implemented in 0.5.x (use DDEXParser::parse)".to_string(),
    }
}
