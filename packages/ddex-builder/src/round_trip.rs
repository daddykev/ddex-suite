//! Parse → Build → Parse round trips.
//!
//! [`verify_round_trip`] parses a message, maps it to a build request
//! ([`request_from_parsed`]), builds it, parses the result again and compares the two
//! parsed messages on the fields the request carries ([`compare_flat`]). The builder does
//! not depend on the parser, so the caller passes the parse function (for example
//! `ddex_parser::DDEXParser::parse`). Round trips do not carry everything: data the build
//! request cannot carry yet (contributors, P and C lines, images, ...) is not rebuilt and not
//! compared.

pub mod parsed;

pub use parsed::{compare_flat, request_from_parsed};

use crate::builder::{BuildOptions, DDEXBuilder};
use crate::error::BuildError;
use ddex_core::models::flat::ParsedERNMessage;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

/// The outcome of [`verify_round_trip`]. Every field is computed; there is no score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundTripVerification {
    /// No differences on the compared fields.
    pub success: bool,
    /// `what: original -> rebuilt` for each compared field that differs.
    pub differences: Vec<String>,
    /// The XML built from the original message.
    pub rebuilt_xml: String,
    /// Building the same request twice gave byte-identical XML.
    pub deterministic: bool,
    /// The rebuilt XML is unchanged by DB-C14N/1.0 canonicalization.
    pub canonical: bool,
    /// The SHA-256 of the rebuilt XML equals that of the XML built again from the re-parsed
    /// message (a second round trip changes nothing).
    pub checksums_match: bool,
}

fn sha256(xml: &str) -> String {
    hex::encode(Sha256::digest(xml.as_bytes()))
}

/// The build target for a parsed message: its own version (ERN 3.8.1 is built as 3.8.2).
fn target_version(parsed: &ParsedERNMessage) -> &'static str {
    match parsed.graph.version.as_str() {
        "3.8.1" => "3.8.2",
        version => version,
    }
}

/// Parse → Build → Parse round trip of `original_xml` (see the module docs). `parse` is
/// the parser's parse function; its error becomes `BuildError::InvalidFormat`.
pub fn verify_round_trip<F>(
    original_xml: &str,
    parse: F,
) -> Result<RoundTripVerification, BuildError>
where
    F: Fn(&str) -> Result<ParsedERNMessage, String>,
{
    let parse_xml = |what: &str, xml: &str| {
        parse(xml).map_err(|message| BuildError::InvalidFormat {
            field: what.to_string(),
            message,
        })
    };
    let original = parse_xml("original_xml", original_xml)?;
    let version = target_version(&original);
    let request = request_from_parsed(&original, version)?;
    let builder = DDEXBuilder::new();
    let rebuilt_xml = builder.build(request.clone(), BuildOptions::default())?.xml;
    let again = builder.build(request, BuildOptions::default())?.xml;
    let reparsed = parse_xml("rebuilt XML", &rebuilt_xml)?;
    let differences = compare_flat(&original.flat, &reparsed.flat);

    let second = builder
        .build(
            request_from_parsed(&reparsed, version)?,
            BuildOptions::default(),
        )?
        .xml;
    let canonical = crate::canonical::DB_C14N::new()
        .canonicalize(&rebuilt_xml)
        .map(|c| c == rebuilt_xml)
        .unwrap_or(false);

    Ok(RoundTripVerification {
        success: differences.is_empty(),
        differences,
        deterministic: again == rebuilt_xml,
        canonical,
        checksums_match: sha256(&second) == sha256(&rebuilt_xml),
        rebuilt_xml,
    })
}

/// Round-trip tester: [`verify_round_trip`] plus a DB-C14N comparison of the original and
/// rebuilt XML. (Before 0.6.0 it took fidelity options, which it never applied.)
#[derive(Debug, Clone, Default)]
pub struct RoundTripTester;

impl RoundTripTester {
    /// Create a new round-trip tester
    pub fn new() -> Self {
        Self
    }

    /// XML → Parse → Build → Parse → compare ([`verify_round_trip`]). `success` means no
    /// differences on the compared fields; `byte_identical` means the original and rebuilt
    /// XML are identical after DB-C14N (rare: the build request cannot carry everything).
    pub fn test_round_trip_with<F>(
        &self,
        original_xml: &str,
        parse: F,
    ) -> Result<RoundTripResult, BuildError>
    where
        F: Fn(&str) -> Result<ParsedERNMessage, String>,
    {
        let start_time = Instant::now();
        let verification = verify_round_trip(original_xml, parse)?;
        let c14n = crate::canonical::DB_C14N::new();
        let byte_identical = match (
            c14n.canonicalize(original_xml),
            c14n.canonicalize(&verification.rebuilt_xml),
        ) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        };
        Ok(RoundTripResult {
            success: verification.success,
            original_xml: original_xml.to_string(),
            rebuilt_xml: verification.rebuilt_xml,
            byte_identical,
            differences: verification.differences,
            test_time: start_time.elapsed(),
        })
    }
}

/// Round-trip test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundTripResult {
    /// Whether round-trip was successful
    pub success: bool,
    /// Original XML input
    pub original_xml: String,
    /// XML after build process
    pub rebuilt_xml: String,
    /// Whether XMLs are byte-identical after canonicalization
    pub byte_identical: bool,
    /// Differences found (if any)
    pub differences: Vec<String>,
    /// Time taken for round-trip test
    pub test_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_parse_error_is_reported() {
        let error = verify_round_trip("<a/>", |_| Err("not DDEX".to_string())).unwrap_err();
        assert!(error.to_string().contains("not DDEX"), "{error}");
    }
}
