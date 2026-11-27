//! # Output configuration
//!
//! [`OutputConfig`] selects how `DDEXBuilder::build` writes its output
//! (`BuildOptions::output`):
//!
//! - [`CanonMode::DbC14n`] (the default): DB-C14N/1.0 canonical output (fixed namespace
//!   prefixes, sorted attributes, two-space indentation, elements in XSD order).
//! - [`CanonMode::Pretty`]: the generator's indented output, not canonicalized, indented
//!   with [`OutputConfig::indent_char`] × [`OutputConfig::indent_width`].
//!
//! The builder reads no clock or random source and uses no unordered iteration on the build
//! path (enforced by clippy lints). The same request and options produce byte-identical
//! output with the same crate version; repeated builds are tested to be byte-identical.
//!
//! Before 0.6.0 this was the `determinism` module and its `DeterminismConfig`.
//!
//! ```rust
//! use ddex_builder::output::{CanonMode, IndentChar, OutputConfig};
//!
//! let options = ddex_builder::BuildOptions {
//!     output: OutputConfig {
//!         canon_mode: CanonMode::Pretty,
//!         indent_char: IndentChar::Tab,
//!         indent_width: 1,
//!         ..Default::default()
//!     },
//!     ..Default::default()
//! };
//! # let _ = options;
//! ```

use serde::{Deserialize, Serialize};

/// How `DDEXBuilder::build` writes its output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Canonicalization mode
    pub canon_mode: CanonMode,
    /// Character used for indentation (`CanonMode::Pretty` only; DB-C14N indents with two
    /// spaces)
    pub indent_char: IndentChar,
    /// Number of indent characters per level (`CanonMode::Pretty` only)
    pub indent_width: usize,
    /// Return a one-line banner naming the builder version and DB-C14N version in
    /// `BuildResult::generator_banner` (it is not written into the XML)
    pub emit_generator_banner: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            canon_mode: CanonMode::DbC14n,
            indent_char: IndentChar::Space,
            indent_width: 2,
            emit_generator_banner: false,
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
