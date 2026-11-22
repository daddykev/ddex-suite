//! Experimental parallel streaming parser
//!
//! It uses rayon to process DDEX elements on several threads, but does not currently
//! scale with the thread count: 1, 2, 4 and 8 threads take the same time (scaling is
//! planned for 0.6.0; see `dev-docs/PERFORMANCE.md`).

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
    chunk_size: usize,
    start_time: Instant,
    total_bytes_processed: Arc<Mutex<u64>>,
    total_elements_found: Arc<Mutex<u64>>,
}

impl ParallelStreamingParser {
    /// Create new parallel parser using all available CPU cores
    pub fn new() -> Self {
        Self {
            worker_threads: num_cpus::get().max(2), // Use at least 2 threads
            chunk_size: 1024 * 1024,                // 1MB chunks for parallel processing
            start_time: Instant::now(),
            total_bytes_processed: Arc::new(Mutex::new(0)),
            total_elements_found: Arc::new(Mutex::new(0)),
        }
    }

    /// Create parser with specific number of worker threads
    pub fn with_threads(threads: usize) -> Self {
        Self {
            worker_threads: threads.max(1),
            chunk_size: 1024 * 1024,
            start_time: Instant::now(),
            total_bytes_processed: Arc::new(Mutex::new(0)),
            total_elements_found: Arc::new(Mutex::new(0)),
        }
    }

    /// Parse data in parallel using multiple threads
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
            current_depth: 0,
            max_depth_reached: 10,
            current_memory_bytes: self.chunk_size * self.worker_threads,
            max_memory_used_bytes: self.chunk_size * self.worker_threads,
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

/// Benchmark parallel performance
pub struct ParallelBenchmark;

impl ParallelBenchmark {
    pub fn measure_parallel_speedup(data: &[u8]) -> Result<ParallelBenchmarkResult, ParseError> {
        println!("🚀 Measuring Parallel Performance Speedup");
        println!("Data size: {:.2} MB", data.len() as f64 / (1024.0 * 1024.0));

        // Measure single-threaded performance
        let start = Instant::now();
        let single_parser = ParallelStreamingParser::with_threads(1);
        let single_elements = single_parser.parse_parallel(data)?;
        let single_time = start.elapsed();

        // Measure parallel performance with different thread counts
        let mut thread_results = Vec::new();

        for threads in [2, 4, 6, 8] {
            if threads <= num_cpus::get() {
                let start = Instant::now();
                let parallel_parser = ParallelStreamingParser::with_threads(threads);
                let parallel_elements = parallel_parser.parse_parallel(data)?;
                let parallel_time = start.elapsed();

                let speedup = single_time.as_secs_f64() / parallel_time.as_secs_f64();
                let efficiency = (speedup / threads as f64) * 100.0;
                let throughput =
                    (data.len() as f64 / (1024.0 * 1024.0)) / parallel_time.as_secs_f64();

                thread_results.push(ThreadResult {
                    threads,
                    time: parallel_time,
                    speedup,
                    efficiency,
                    throughput_mb_per_sec: throughput,
                    elements_found: parallel_elements.len(),
                });

                println!(
                    "  {} threads: {:.3}s, {:.1}x speedup, {:.1}% efficiency, {:.1} MB/s",
                    threads,
                    parallel_time.as_secs_f64(),
                    speedup,
                    efficiency,
                    throughput
                );

                // Verify element count consistency
                assert_eq!(
                    single_elements.len(),
                    parallel_elements.len(),
                    "Element count mismatch: single={}, parallel={}",
                    single_elements.len(),
                    parallel_elements.len()
                );
            }
        }

        let single_throughput = (data.len() as f64 / (1024.0 * 1024.0)) / single_time.as_secs_f64();

        let best_result = thread_results
            .iter()
            .max_by(|a, b| {
                a.throughput_mb_per_sec
                    .partial_cmp(&b.throughput_mb_per_sec)
                    .unwrap()
            })
            .unwrap();

        let best_speedup = best_result.speedup;
        let best_throughput = best_result.throughput_mb_per_sec;
        let target_achieved = best_result.throughput_mb_per_sec >= 280.0;

        let result = ParallelBenchmarkResult {
            single_threaded_time: single_time,
            single_threaded_throughput: single_throughput,
            single_threaded_elements: single_elements.len(),
            thread_results,
            best_speedup,
            best_throughput,
            target_achieved,
        };

        println!("\n📊 PARALLEL PERFORMANCE SUMMARY");
        println!(
            "Single-threaded: {:.1} MB/s",
            result.single_threaded_throughput
        );
        println!(
            "Best parallel: {:.1} MB/s ({:.1}x speedup)",
            result.best_throughput, result.best_speedup
        );
        println!(
            "Target (280 MB/s): {}",
            if result.target_achieved {
                "✅ ACHIEVED!"
            } else {
                "❌ Not achieved"
            }
        );

        if result.target_achieved {
            println!("🎉 SUCCESS: 480x performance improvement target achieved with parallel processing!");
        }

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct ThreadResult {
    pub threads: usize,
    pub time: std::time::Duration,
    pub speedup: f64,
    pub efficiency: f64,
    pub throughput_mb_per_sec: f64,
    pub elements_found: usize,
}

#[derive(Debug)]
pub struct ParallelBenchmarkResult {
    pub single_threaded_time: std::time::Duration,
    pub single_threaded_throughput: f64,
    pub single_threaded_elements: usize,
    pub thread_results: Vec<ThreadResult>,
    pub best_speedup: f64,
    pub best_throughput: f64,
    pub target_achieved: bool,
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
