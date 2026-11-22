// src/streaming/iterator.rs
//! Iterator implementation for streaming DDEX parser

use super::{ParsedElement, StreamingConfig, StreamingDDEXParser, StreamingProgress};
use crate::error::ParseError;
use ddex_core::models::versions::ERNVersion;
use std::io::BufRead;

/// Iterator wrapper for streaming DDEX parser
pub struct DDEXStreamIterator<R: BufRead> {
    parser: StreamingDDEXParser<R>,
    finished: bool,
    error_state: Option<ParseError>,
}

impl<R: BufRead> DDEXStreamIterator<R> {
    /// Create new iterator from reader
    pub fn new(reader: R, version: ERNVersion) -> Self {
        Self {
            parser: StreamingDDEXParser::new(reader, version),
            finished: false,
            error_state: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(reader: R, version: ERNVersion, config: StreamingConfig) -> Self {
        Self {
            parser: StreamingDDEXParser::with_config(reader, version, config),
            finished: false,
            error_state: None,
        }
    }

    /// Add progress callback
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: FnMut(StreamingProgress) + Send + 'static,
    {
        self.parser = self.parser.with_progress_callback(callback);
        self
    }

    /// Get current parsing statistics
    pub fn stats(&self) -> IteratorStats {
        IteratorStats {
            bytes_processed: self.parser.bytes_processed,
            elements_yielded: self.parser.elements_yielded,
            current_depth: self.parser.context.current_depth,
            memory_usage: self.parser.current_memory,
            elapsed: self.parser.start_time.elapsed(),
            is_finished: self.finished,
            has_error: self.error_state.is_some(),
        }
    }

    /// Check if iterator has encountered an error
    pub fn has_error(&self) -> bool {
        self.error_state.is_some()
    }

    /// Get the last error if any
    pub fn last_error(&self) -> Option<&ParseError> {
        self.error_state.as_ref()
    }

    /// Reset error state (use with caution)
    pub fn clear_error(&mut self) {
        self.error_state = None;
    }

    /// Try to recover from a specific error type
    pub fn try_recover(&mut self) -> Result<(), ParseError> {
        if let Some(ref error) = self.error_state {
            match error {
                ParseError::XmlError { .. } => {
                    // For XML errors, we might be able to skip to next element
                    self.clear_error();
                    Ok(())
                }
                ParseError::SecurityViolation { .. } => {
                    // Security violations should not be recoverable
                    Err(error.clone())
                }
                _ => {
                    // Other errors might be recoverable
                    self.clear_error();
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    /// Consume iterator and collect all elements
    pub fn collect_all(self) -> Result<Vec<ParsedElement>, ParseError> {
        let mut elements = Vec::new();
        for result in self {
            match result {
                Ok(element) => {
                    if matches!(element, ParsedElement::EndOfStream) {
                        break;
                    }
                    elements.push(element);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(elements)
    }

    /// Collect only specific element types
    pub fn collect_releases(self) -> Result<Vec<ddex_core::models::graph::Release>, ParseError> {
        let mut releases = Vec::new();
        for result in self {
            match result {
                Ok(ParsedElement::Release(release)) => {
                    releases.push(release);
                }
                Ok(ParsedElement::EndOfStream) => break,
                Ok(_) => continue, // Skip other element types
                Err(e) => return Err(e),
            }
        }
        Ok(releases)
    }

    /// Collect only resources
    pub fn collect_resources(self) -> Result<Vec<ddex_core::models::graph::Resource>, ParseError> {
        let mut resources = Vec::new();
        for result in self {
            match result {
                Ok(ParsedElement::Resource(resource)) => {
                    resources.push(resource);
                }
                Ok(ParsedElement::EndOfStream) => break,
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(resources)
    }

    /// Skip to next element of specific type
    pub fn skip_to_next_release(
        &mut self,
    ) -> Result<Option<ddex_core::models::graph::Release>, ParseError> {
        for result in self {
            match result {
                Ok(ParsedElement::Release(release)) => {
                    return Ok(Some(release));
                }
                Ok(ParsedElement::EndOfStream) => {
                    return Ok(None);
                }
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(None)
    }
}

impl<R: BufRead> Iterator for DDEXStreamIterator<R> {
    type Item = Result<ParsedElement, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.error_state.is_some() {
            return None;
        }

        match self.parser.parse_next_element() {
            Ok(Some(element)) => {
                if matches!(element, ParsedElement::EndOfStream) {
                    self.finished = true;
                }
                Some(Ok(element))
            }
            Ok(None) => {
                self.finished = true;
                None
            }
            Err(e) => {
                self.error_state = Some(e.clone());
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

impl<R: BufRead> std::fmt::Debug for DDEXStreamIterator<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DDEXStreamIterator")
            .field("finished", &self.finished)
            .field("has_error", &self.error_state.is_some())
            .field("parser", &self.parser)
            .finish()
    }
}

/// Statistics about iterator state
#[derive(Debug, Clone)]
pub struct IteratorStats {
    pub bytes_processed: u64,
    pub elements_yielded: usize,
    pub current_depth: usize,
    pub memory_usage: usize,
    pub elapsed: std::time::Duration,
    pub is_finished: bool,
    pub has_error: bool,
}

impl IteratorStats {
    /// Get processing rate in bytes per second
    pub fn bytes_per_second(&self) -> f64 {
        if self.elapsed.as_secs_f64() > 0.0 {
            self.bytes_processed as f64 / self.elapsed.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Get element processing rate per second
    pub fn elements_per_second(&self) -> f64 {
        if self.elapsed.as_secs_f64() > 0.0 {
            self.elements_yielded as f64 / self.elapsed.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Get memory usage in MB
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage as f64 / (1024.0 * 1024.0)
    }

    /// Get throughput in MiB/s
    pub fn throughput_mibs(&self) -> f64 {
        if self.elapsed.as_secs_f64() > 0.0 {
            (self.bytes_processed as f64 / (1024.0 * 1024.0)) / self.elapsed.as_secs_f64()
        } else {
            0.0
        }
    }
}

/// Filtered iterator for specific element types
pub struct FilteredDDEXIterator<R: BufRead, F>
where
    F: Fn(&ParsedElement) -> bool,
{
    inner: DDEXStreamIterator<R>,
    filter: F,
}

impl<R: BufRead, F> FilteredDDEXIterator<R, F>
where
    F: Fn(&ParsedElement) -> bool,
{
    /// Create new filtered iterator
    pub fn new(inner: DDEXStreamIterator<R>, filter: F) -> Self {
        Self { inner, filter }
    }
}

impl<R: BufRead, F> Iterator for FilteredDDEXIterator<R, F>
where
    F: Fn(&ParsedElement) -> bool,
{
    type Item = Result<ParsedElement, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some(Ok(element)) => {
                    if (self.filter)(&element) || matches!(element, ParsedElement::EndOfStream) {
                        return Some(Ok(element));
                    }
                    // Continue to next element
                }
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
    }
}

/// Convenience functions for creating filtered iterators
impl<R: BufRead> DDEXStreamIterator<R> {
    /// Filter to only releases
    pub fn releases_only(self) -> FilteredDDEXIterator<R, impl Fn(&ParsedElement) -> bool> {
        FilteredDDEXIterator::new(self, |element| matches!(element, ParsedElement::Release(_)))
    }

    /// Filter to only resources
    pub fn resources_only(self) -> FilteredDDEXIterator<R, impl Fn(&ParsedElement) -> bool> {
        FilteredDDEXIterator::new(self, |element| {
            matches!(element, ParsedElement::Resource(_))
        })
    }

    /// Filter to only headers
    pub fn headers_only(self) -> FilteredDDEXIterator<R, impl Fn(&ParsedElement) -> bool> {
        FilteredDDEXIterator::new(self, |element| {
            matches!(element, ParsedElement::Header { .. })
        })
    }

    /// Filter with custom predicate
    pub fn filter<F>(self, filter: F) -> FilteredDDEXIterator<R, F>
    where
        F: Fn(&ParsedElement) -> bool,
    {
        FilteredDDEXIterator::new(self, filter)
    }
}

#[cfg(all(test, feature = "legacy-streaming-tests"))]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_iterator_stats() {
        let stats = IteratorStats {
            bytes_processed: 1024 * 1024, // 1MB
            elements_yielded: 10,
            current_depth: 5,
            memory_usage: 2 * 1024 * 1024, // 2MB
            elapsed: std::time::Duration::from_secs(1),
            is_finished: false,
            has_error: false,
        };

        assert_eq!(stats.bytes_per_second(), 1024.0 * 1024.0);
        assert_eq!(stats.elements_per_second(), 10.0);
        assert_eq!(stats.memory_usage_mb(), 2.0);
        assert_eq!(stats.throughput_mibs(), 1.0);
    }

    #[test]
    fn test_iterator_creation() {
        let xml = "<ERNMessage></ERNMessage>";
        let cursor = Cursor::new(xml.as_bytes());
        let iterator = DDEXStreamIterator::new(cursor, ERNVersion::V4_3);

        assert!(!iterator.finished);
        assert!(!iterator.has_error());
    }
}
