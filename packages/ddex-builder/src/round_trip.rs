//! Parse → Build → Parse round trips.
//!
//! [`verify_round_trip`] parses a message, maps it to a build request
//! ([`request_from_parsed`]), builds it, parses the result again and compares the two
//! parsed messages on the fields the request carries ([`compare_flat`]). The builder does
//! not depend on the parser, so the caller passes the parse function (for example
//! `ddex_parser::DDEXParser::parse`). Round trips are not lossless in 0.5.x: data the build
//! request cannot carry yet (contributors, P and C lines, images, ...) is not rebuilt and not
//! compared.

pub mod parsed;

pub use parsed::{compare_flat, request_from_parsed};

use crate::builder::{BuildOptions, DDEXBuilder};
use crate::{error::BuildError, FidelityOptions};
use ddex_core::models::flat::ParsedERNMessage;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

/// `HashMap` for the serde-derived structs below, which a field-level allow cannot cover
/// (the derived impls are separate items). Same type as `HashMap<K, V>`.
#[allow(
    clippy::disallowed_types,
    reason = "H7: round-trip statistics; not build output"
)]
type UnorderedMap<K, V> = std::collections::HashMap<K, V>;

/// The error of an API that 0.5.x does not implement (it used to return a made-up result).
pub(crate) fn not_implemented(api: &str, detail: &str) -> BuildError {
    BuildError::Other(format!("{api} is not implemented in 0.5.x: {detail}"))
}

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
    let canonical =
        crate::canonical::DB_C14N::new(crate::determinism::DeterminismConfig::default())
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

/// Round-trip tester for the fidelity options
pub struct RoundTripTester {
    fidelity_options: FidelityOptions,
}

impl RoundTripTester {
    /// Create a new round-trip tester with the specified fidelity options
    pub fn new(fidelity_options: FidelityOptions) -> Self {
        Self { fidelity_options }
    }

    /// The fidelity options (not applied by the build in 0.5.x).
    pub fn fidelity_options(&self) -> &FidelityOptions {
        &self.fidelity_options
    }

    /// Not implemented in 0.5.x: the builder has no parser. Use
    /// [`RoundTripTester::test_round_trip_with`] (or [`verify_round_trip`]) with the
    /// parser's parse function. Before 0.5.1 this returned success without building.
    pub fn test_round_trip(&self, _original_xml: &str) -> Result<RoundTripResult, BuildError> {
        Err(not_implemented(
            "RoundTripTester::test_round_trip",
            "the builder has no parser; call test_round_trip_with(xml, parse) or \
             ddex_builder::round_trip::verify_round_trip with ddex_parser's parse",
        ))
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
        let c14n = crate::canonical::DB_C14N::new(crate::determinism::DeterminismConfig::default());
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

    /// Not implemented in 0.5.x. Before 0.5.1 its "preserved" counts were copies of the
    /// totals and its score depended only on the options, not on any output.
    pub fn analyze_fidelity(&self, _original_xml: &str) -> Result<FidelityAnalysis, BuildError> {
        Err(not_implemented(
            "RoundTripTester::analyze_fidelity",
            "preservation is not measured; use verify_round_trip for the differences",
        ))
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

/// Comprehensive fidelity analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityAnalysis {
    /// Element analysis
    pub element_analysis: ElementAnalysis,
    /// Attribute analysis  
    pub attribute_analysis: AttributeAnalysis,
    /// Comment analysis
    pub comment_analysis: CommentAnalysis,
    /// Extension analysis
    pub extension_analysis: ExtensionAnalysis,
    /// Namespace analysis
    pub namespace_analysis: NamespaceAnalysis,
    /// Time taken for analysis
    pub analysis_time: Duration,
    /// Overall fidelity score (0.0 - 1.0)
    pub overall_score: f64,
}

/// Element preservation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementAnalysis {
    /// Total number of elements
    pub total_elements: usize,
    /// Elements by type/name
    pub elements_by_type: UnorderedMap<String, usize>,
    /// Unknown elements (not in schema)
    pub unknown_elements: usize,
    /// Elements preserved after round-trip
    pub preserved_elements: usize,
}

/// Attribute preservation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeAnalysis {
    /// Total number of attributes
    pub total_attributes: usize,
    /// Attributes by element type
    pub attributes_by_element: UnorderedMap<String, usize>,
    /// Unknown attributes (not in schema)
    pub unknown_attributes: usize,
    /// Attributes preserved after round-trip
    pub preserved_attributes: usize,
}

/// Comment preservation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentAnalysis {
    /// Total number of comments
    pub total_comments: usize,
    /// Document-level comments
    pub document_level_comments: usize,
    /// Element-level comments
    pub element_level_comments: usize,
    /// Inline comments
    pub inline_comments: usize,
    /// Comments preserved after round-trip
    pub preserved_comments: usize,
}

/// Extension preservation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionAnalysis {
    /// Total number of extensions
    pub total_extensions: usize,
    /// Extension namespaces found
    pub extension_namespaces: UnorderedMap<String, String>,
    /// Known extensions (recognized patterns)
    pub known_extensions: usize,
    /// Unknown extensions
    pub unknown_extensions: usize,
    /// Extensions preserved after round-trip
    pub preserved_extensions: usize,
}

/// Namespace preservation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceAnalysis {
    /// Total number of namespaces
    pub total_namespaces: usize,
    /// Prefixed namespaces
    pub prefixed_namespaces: UnorderedMap<String, String>,
    /// Default namespace (if any)
    pub default_namespace: Option<String>,
    /// Namespaces preserved after round-trip
    pub preserved_namespaces: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apis_without_a_parser_are_not_implemented() {
        let tester = RoundTripTester::new(FidelityOptions::default());
        let error = tester.test_round_trip("<a/>").unwrap_err().to_string();
        assert!(error.contains("not implemented in 0.5.x"), "{error}");
        let error = tester.analyze_fidelity("<a/>").unwrap_err().to_string();
        assert!(error.contains("not implemented in 0.5.x"), "{error}");
    }

    #[test]
    fn a_parse_error_is_reported() {
        let error = verify_round_trip("<a/>", |_| Err("not DDEX".to_string())).unwrap_err();
        assert!(error.to_string().contains("not DDEX"), "{error}");
    }
}
