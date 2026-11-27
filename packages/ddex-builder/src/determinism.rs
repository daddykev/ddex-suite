//! # Output configuration
//!
//! [`DeterminismConfig`] selects how `DDEXBuilder::build` writes its output
//! (`BuildOptions::determinism`; the default is used when it is `None`):
//!
//! - [`CanonMode::DbC14n`] (the default): DB-C14N/1.0 canonical output (fixed namespace
//!   prefixes, sorted attributes, two-space indentation, elements in XSD order).
//! - [`CanonMode::Pretty`]: the generator's indented output, not canonicalized, indented
//!   with [`DeterminismConfig::indent_char`] × [`DeterminismConfig::indent_width`].
//!
//! Byte-for-byte determinism is being implemented for 0.6.0: a message ID, references and
//! `MessageCreatedDateTime` that the request leaves out are still generated (random UUIDs,
//! the current time), so two builds of such a request can differ. See `KNOWN_ISSUES.md`.
//!
//! Before 0.6.0 this module also had twelve configuration fields that no build read and a
//! runtime `DeterminismVerifier`; they were removed.
//!
//! ```rust
//! use ddex_builder::determinism::{CanonMode, DeterminismConfig, IndentChar};
//!
//! let config = DeterminismConfig {
//!     canon_mode: CanonMode::Pretty,
//!     indent_char: IndentChar::Tab,
//!     indent_width: 1,
//!     ..Default::default()
//! };
//! let options = ddex_builder::BuildOptions {
//!     determinism: Some(config),
//!     ..Default::default()
//! };
//! # let _ = options;
//! ```

use serde::{Deserialize, Serialize};

/// How `DDEXBuilder::build` writes its output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismConfig {
    /// Canonicalization mode
    pub canon_mode: CanonMode,
    /// Character used for indentation (`CanonMode::Pretty` only; DB-C14N indents with two
    /// spaces)
    pub indent_char: IndentChar,
    /// Number of indent characters per level (`CanonMode::Pretty` only)
    pub indent_width: usize,
    /// Return a one-line banner naming the builder version and DB-C14N version in
    /// `BuildResult::reproducibility_banner` (it is not written into the XML)
    pub emit_reproducibility_banner: bool,
}

impl Default for DeterminismConfig {
    fn default() -> Self {
        Self {
            canon_mode: CanonMode::DbC14n,
            indent_char: IndentChar::Space,
            indent_width: 2,
            emit_reproducibility_banner: false,
        }
    }
}

/// Canonicalization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonMode {
    /// DB-C14N/1.0 canonicalization
    DbC14n,
    /// Pretty printing (non-canonical)
    Pretty,
}

/// Indentation character
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndentChar {
    /// Space indentation
    Space,
    /// Tab indentation
    Tab,
}
