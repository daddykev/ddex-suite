// core/src/parser/mode.rs
//! Parser mode selection (DOM vs Stream)

use std::io::{BufRead, Seek, SeekFrom};

/// Parser mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    /// Use DOM parsing for smaller files
    Dom,
    /// Use streaming for larger files
    Stream,
    /// Automatically choose based on file size
    Auto,
}

impl Default for ParseMode {
    fn default() -> Self {
        Self::Auto
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
    
    /// Select parsing mode based on file size
    pub fn select_mode<R: BufRead + Seek>(&self, reader: &mut R, mode: ParseMode) -> Result<ParseMode, std::io::Error> {
        match mode {
            ParseMode::Dom | ParseMode::Stream => Ok(mode),
            ParseMode::Auto => {
                // Try to determine file size
                let current_pos = reader.stream_position()?;
                let size = reader.seek(SeekFrom::End(0))?;
                reader.seek(SeekFrom::Start(current_pos))?;
                
                if size > self.threshold {
                    Ok(ParseMode::Stream)
                } else {
                    Ok(ParseMode::Dom)
                }
            }
        }
    }
    
    /// Select mode without seeking (for non-seekable streams)
    pub fn select_mode_hint(&self, size_hint: Option<u64>, mode: ParseMode) -> ParseMode {
        match mode {
            ParseMode::Dom | ParseMode::Stream => mode,
            ParseMode::Auto => {
                if let Some(size) = size_hint {
                    if size > self.threshold {
                        ParseMode::Stream
                    } else {
                        ParseMode::Dom
                    }
                } else {
                    // Default to streaming for unknown size
                    ParseMode::Stream
                }
            }
        }
    }
}