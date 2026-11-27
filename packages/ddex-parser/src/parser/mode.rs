// core/src/parser/mode.rs
//! Parser mode selection (DOM vs Stream)
//!
//! The DOM path is the complete parser and is used unless [`ParseMode::Stream`] is requested
//! explicitly. (`ParseMode::Auto` and `ModeSelector`, which only resolved to `Dom` since 0.5.0,
//! were removed in 0.6.0.)

/// Parser mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    /// The complete parser (every field), for any input size. The default.
    Dom,
    /// **Experimental**, opt-in only: the memory-bounded streaming path. It reads a subset
    /// of the fields (no territory details, dates, P/C lines, genres, labels, track layout,
    /// ...) and is never selected automatically.
    Stream,
}

impl Default for ParseMode {
    fn default() -> Self {
        Self::Dom
    }
}
