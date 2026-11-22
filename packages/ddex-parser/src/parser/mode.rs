// core/src/parser/mode.rs
//! Parser mode selection (DOM vs Stream)
//!
//! The DOM path is the complete parser and is used unless [`ParseMode::Stream`] is requested
//! explicitly. Since 0.5.0, `Auto` no longer switches to the streaming path for large inputs:
//! that path reads only a subset of the fields, so values silently disappeared for files
//! above the threshold.

use std::io::{BufRead, Seek};

/// Parser mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    /// The complete parser (every field), for any input size. The default.
    Dom,
    /// **Experimental**, opt-in only: the memory-bounded streaming path. It reads a subset
    /// of the fields (no territory details, dates, P/C lines, genres, labels, track layout,
    /// ...) and is never selected automatically.
    Stream,
    /// Kept for compatibility: resolves to [`ParseMode::Dom`] for every input size.
    Auto,
}

impl Default for ParseMode {
    fn default() -> Self {
        Self::Dom
    }
}

/// Mode selector for choosing parsing strategy
pub struct ModeSelector {
    threshold: u64,
}

impl Default for ModeSelector {
    fn default() -> Self {
        Self {
            threshold: 10 * 1024 * 1024, // 10MB default threshold
        }
    }
}

impl ModeSelector {
    pub fn new(threshold: u64) -> Self {
        Self { threshold }
    }

    /// The mode to use: `Stream` only when requested explicitly; `Dom` otherwise, whatever
    /// the input size (`Auto` never switches to the experimental streaming path).
    pub fn select_mode<R: BufRead + Seek>(
        &self,
        _reader: &mut R,
        mode: ParseMode,
    ) -> Result<ParseMode, std::io::Error> {
        Ok(self.select_mode_hint(None, mode))
    }

    /// Same as [`ModeSelector::select_mode`], without a reader.
    pub fn select_mode_hint(&self, _size_hint: Option<u64>, mode: ParseMode) -> ParseMode {
        match mode {
            ParseMode::Stream => ParseMode::Stream,
            ParseMode::Dom | ParseMode::Auto => ParseMode::Dom,
        }
    }

    /// The configured size threshold. Since 0.5.0 it no longer influences the mode.
    pub fn threshold(&self) -> u64 {
        self.threshold
    }
}
