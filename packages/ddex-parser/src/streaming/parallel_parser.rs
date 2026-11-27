//! Experimental parallel streaming parser
//!
//! It processes the document on one thread whatever the thread count: 1, 2, 4 and 8 threads
//! take the same time (scaling is planned for 0.6.0; see `dev-docs/PERFORMANCE.md`).

#[allow(dead_code)] // Experimental parallel streaming parser
use crate::error::ParseError;
use crate::streaming::fast_zero_copy::FastZeroCopyParser;
use crate::streaming::{WorkingStreamingElement, WorkingStreamingStats};
use ddex_core::models::versions::ERNVersion;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::time::Instant;
// use crossbeam_channel::{bounded, Receiver, Sender}; // For future streaming implementation

/// Parallel streaming parser for multi-core processing
pub struct ParallelStreamingParser {
    worker_threads: usize,
    start_time: Instant,
    total_bytes_processed: Arc<Mutex<u64>>,
    total_elements_found: Arc<Mutex<u64>>,
}

impl ParallelStreamingParser {
    /// Create new parallel parser using all available CPU cores
    pub fn new() -> Self {
        Self {
            worker_threads: num_cpus::get().max(2), // Use at least 2 threads
            start_time: Instant::now(),
            total_bytes_processed: Arc::new(Mutex::new(0)),
            total_elements_found: Arc::new(Mutex::new(0)),
        }
    }

    /// Create parser with specific number of worker threads
    pub fn with_threads(threads: usize) -> Self {
        Self {
            worker_threads: threads.max(1),
            start_time: Instant::now(),
            total_bytes_processed: Arc::new(Mutex::new(0)),
            total_elements_found: Arc::new(Mutex::new(0)),
        }
    }

    /// The configured thread count. It is not used in 0.5.x: `parse_parallel` runs on one
    /// thread (a parallel iterator that scales is planned for 0.6.0).
    pub fn worker_threads(&self) -> usize {
        self.worker_threads
    }

    /// Parse data (on one thread in 0.5.x, whatever the thread count)
    pub fn parse_parallel(&self, data: &[u8]) -> Result<Vec<WorkingStreamingElement>, ParseError> {
        // For now, use optimized single-threaded processing to ensure consistency
        // The performance gain comes from the fast zero-copy parser optimizations
        self.parse_single_threaded(data)
    }

    /// Single-threaded fallback for small files
    fn parse_single_threaded(
        &self,
        data: &[u8],
    ) -> Result<Vec<WorkingStreamingElement>, ParseError> {
        let mut parser = FastZeroCopyParser::new();
        let mut elements = parser.parse_chunk(data)?;

        // Update statistics
        {
            let mut bytes = self.total_bytes_processed.lock().unwrap();
            *bytes += data.len() as u64;
        }
        {
            let mut count = self.total_elements_found.lock().unwrap();
            *count += elements.len() as u64;
        }

        elements.push(WorkingStreamingElement::EndOfStream {
            stats: self.get_stats(),
        });

        Ok(elements)
    }

    /// Get current performance statistics
    pub fn get_stats(&self) -> WorkingStreamingStats {
        let elapsed = self.start_time.elapsed();
        let bytes_processed = *self.total_bytes_processed.lock().unwrap();
        let elements_found = *self.total_elements_found.lock().unwrap();

        let throughput = if elapsed.as_secs_f64() > 0.0 {
            (bytes_processed as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64()
        } else {
            0.0
        };

        WorkingStreamingStats {
            bytes_processed,
            elements_yielded: elements_found as usize,
            // Depth and memory are not tracked by this parser (before 0.5.1, a constant 10
            // and chunk size x threads; before 0.6.0, 0).
            current_depth: None,
            current_memory_bytes: None,
            max_memory_used_bytes: None,
            elapsed_time: elapsed,
            throughput_mb_per_sec: throughput,
        }
    }
}

impl Default for ParallelStreamingParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parallel streaming iterator for processing large files
pub struct ParallelStreamingIterator<R: BufRead> {
    // The input is read to the end in the constructor; only its type is kept.
    _reader: std::marker::PhantomData<R>,
    parser: ParallelStreamingParser,
    buffer: Vec<u8>,
    finished: bool,
    elements_queue: Vec<WorkingStreamingElement>,
    current_index: usize,
}

impl<R: BufRead> ParallelStreamingIterator<R> {
    pub fn new(mut reader: R, _version: ERNVersion) -> Self {
        // Read all data into buffer for parallel processing
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer);

        Self {
            _reader: std::marker::PhantomData,
            parser: ParallelStreamingParser::new(),
            buffer,
            finished: false,
            elements_queue: Vec::new(),
            current_index: 0,
        }
    }

    pub fn with_threads(mut reader: R, _version: ERNVersion, threads: usize) -> Self {
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer);

        Self {
            _reader: std::marker::PhantomData,
            parser: ParallelStreamingParser::with_threads(threads),
            buffer,
            finished: false,
            elements_queue: Vec::new(),
            current_index: 0,
        }
    }

    pub fn stats(&self) -> WorkingStreamingStats {
        self.parser.get_stats()
    }
}

impl<R: BufRead> Iterator for ParallelStreamingIterator<R> {
    type Item = Result<WorkingStreamingElement, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // Process all data if we haven't yet
        if self.elements_queue.is_empty() && self.current_index == 0 {
            match self.parser.parse_parallel(&self.buffer) {
                Ok(elements) => {
                    self.elements_queue = elements;
                }
                Err(e) => {
                    self.finished = true;
                    return Some(Err(e));
                }
            }
        }

        // Return next element from queue
        if self.current_index < self.elements_queue.len() {
            let element = self.elements_queue[self.current_index].clone();
            self.current_index += 1;

            // Check if this is the last element
            if matches!(element, WorkingStreamingElement::EndOfStream { .. }) {
                self.finished = true;
            }

            Some(Ok(element))
        } else {
            self.finished = true;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parallel_basic_functionality() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>PAR-TEST-MSG</MessageId>
        <CreatedDateTime>2024-01-01T00:00:00Z</CreatedDateTime>
    </MessageHeader>
    <Release ReleaseReference="PAR-REL-001">
        <ReferenceTitle>
            <TitleText>Parallel Test Release</TitleText>
        </ReferenceTitle>
    </Release>
</ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let iterator = ParallelStreamingIterator::new(cursor, ERNVersion::V4_3);

        let elements: Result<Vec<_>, _> = iterator.collect();
        assert!(elements.is_ok(), "Parallel parsing should work");

        let elements = elements.unwrap();
        assert!(!elements.is_empty(), "Should find elements");

        let has_header = elements
            .iter()
            .any(|e| matches!(e, WorkingStreamingElement::MessageHeader { .. }));
        let has_release = elements
            .iter()
            .any(|e| matches!(e, WorkingStreamingElement::Release { .. }));

        assert!(has_header, "Should find message header");
        assert!(has_release, "Should find release");

        println!("✅ Parallel parser basic test passed!");
    }
}
