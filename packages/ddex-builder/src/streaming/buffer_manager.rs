//! Buffer management for streaming DDEX XML output
//! 
//! Handles chunked writing to disk/network with automatic flushing
//! and memory-bounded operations.

use std::io::{Write as IoWrite, Result as IoResult};
use std::collections::VecDeque;

/// Callback type for flush events
pub type FlushCallback = Box<dyn Fn(usize) + Send + Sync>;

/// Configuration for buffer management
#[derive(Debug, Clone)]
pub struct BufferConfig {
    /// Maximum buffer size before automatic flush
    pub max_buffer_size: usize,
    /// Number of buffers to keep in memory
    pub buffer_count: usize,
    /// Whether to enable compression
    pub enable_compression: bool,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 1024 * 1024, // 1MB per buffer
            buffer_count: 10, // Up to 10MB total
            enable_compression: false, // Disabled by default for simplicity
        }
    }
}

/// Manages buffered writing with automatic flushing and memory limits
pub struct BufferManager<W: IoWrite> {
    writer: W,
    config: BufferConfig,
    
    // Buffer management
    buffers: VecDeque<Vec<u8>>,
    current_buffer: Vec<u8>,
    
    // Statistics
    total_bytes_written: usize,
    total_flushes: usize,
    peak_buffer_size: usize,
    
    // Callbacks
    flush_callback: Option<FlushCallback>,
}

impl<W: IoWrite> BufferManager<W> {
    /// Create a new buffer manager with default configuration
    pub fn new(writer: W, max_buffer_size: usize) -> IoResult<Self> {
        let config = BufferConfig {
            max_buffer_size,
            ..BufferConfig::default()
        };
        Self::new_with_config(writer, config)
    }
    
    /// Create a new buffer manager with custom configuration
    pub fn new_with_config(writer: W, config: BufferConfig) -> IoResult<Self> {
        let buffer_capacity = config.max_buffer_size;
        Ok(BufferManager {
            writer,
            config,
            buffers: VecDeque::new(),
            current_buffer: Vec::with_capacity(buffer_capacity),
            total_bytes_written: 0,
            total_flushes: 0,
            peak_buffer_size: 0,
            flush_callback: None,
        })
    }
    
    /// Set a callback to be called when buffers are flushed
    pub fn set_flush_callback(&mut self, callback: FlushCallback) {
        self.flush_callback = Some(callback);
    }
    
    /// Write a chunk of data to the buffer
    pub fn write_chunk(&mut self, data: &[u8]) -> IoResult<()> {
        // If this chunk would overflow current buffer, flush it first
        if self.current_buffer.len() + data.len() > self.config.max_buffer_size {
            self.flush_current_buffer()?;
        }
        
        // If chunk is larger than max buffer size, write it directly
        if data.len() > self.config.max_buffer_size {
            self.write_directly(data)?;
            return Ok(());
        }
        
        // Add to current buffer
        self.current_buffer.extend_from_slice(data);
        
        // Update peak memory usage
        let current_memory = self.current_memory_usage();
        if current_memory > self.peak_buffer_size {
            self.peak_buffer_size = current_memory;
        }
        
        // Check if we need to flush due to buffer count limit
        if self.buffers.len() >= self.config.buffer_count {
            self.flush_oldest_buffer()?;
        }
        
        Ok(())
    }
    
    /// Flush the current buffer to the queue
    pub fn flush_current_buffer(&mut self) -> IoResult<()> {
        if !self.current_buffer.is_empty() {
            let buffer = std::mem::replace(
                &mut self.current_buffer,
                Vec::with_capacity(self.config.max_buffer_size)
            );
            self.buffers.push_back(buffer);
            
            // If we have too many buffers, flush the oldest one
            if self.buffers.len() > self.config.buffer_count {
                self.flush_oldest_buffer()?;
            }
        }
        Ok(())
    }
    
    /// Flush the oldest buffer to the writer
    pub fn flush_oldest_buffer(&mut self) -> IoResult<()> {
        if let Some(buffer) = self.buffers.pop_front() {
            self.write_buffer(&buffer)?;
            
            // Call flush callback if set
            if let Some(ref callback) = self.flush_callback {
                callback(buffer.len());
            }
        }
        Ok(())
    }
    
    /// Flush all buffers to the writer
    pub fn flush_all(&mut self) -> IoResult<()> {
        // Flush current buffer to queue first
        self.flush_current_buffer()?;
        
        // Flush all queued buffers
        while !self.buffers.is_empty() {
            self.flush_oldest_buffer()?;
        }
        
        // Ensure writer is flushed
        self.writer.flush()?;
        
        Ok(())
    }
    
    /// Write data directly to the writer without buffering
    fn write_directly(&mut self, data: &[u8]) -> IoResult<()> {
        // First flush any existing buffers to maintain order
        self.flush_all()?;
        
        // Write directly
        self.write_buffer(data)?;
        
        Ok(())
    }
    
    /// Write a buffer to the underlying writer
    fn write_buffer(&mut self, buffer: &[u8]) -> IoResult<()> {
        if self.config.enable_compression {
            // TODO: Implement compression if needed
            self.writer.write_all(buffer)?;
        } else {
            self.writer.write_all(buffer)?;
        }
        
        self.total_bytes_written += buffer.len();
        self.total_flushes += 1;
        
        Ok(())
    }
    
    /// Get current memory usage in bytes
    pub fn current_memory_usage(&self) -> usize {
        let buffered_size: usize = self.buffers.iter().map(|b| b.len()).sum();
        buffered_size + self.current_buffer.len()
    }
    
    /// Get current buffer size
    pub fn current_buffer_size(&self) -> usize {
        self.current_buffer.len()
    }
    
    /// Get total bytes written so far
    pub fn total_bytes_written(&self) -> usize {
        self.total_bytes_written
    }
    
    /// Get peak buffer size reached
    pub fn peak_buffer_size(&self) -> usize {
        self.peak_buffer_size
    }
    
    /// Get total number of flushes performed
    pub fn total_flushes(&self) -> usize {
        self.total_flushes
    }
    
    /// Get number of buffers currently queued
    pub fn queued_buffer_count(&self) -> usize {
        self.buffers.len()
    }
    
    /// Check if buffers are near capacity
    pub fn is_near_capacity(&self) -> bool {
        self.current_buffer.len() > (self.config.max_buffer_size * 3 / 4) ||
        self.buffers.len() >= (self.config.buffer_count * 3 / 4)
    }
    
    /// Get buffer statistics
    pub fn get_stats(&self) -> BufferStats {
        BufferStats {
            current_memory_usage: self.current_memory_usage(),
            peak_memory_usage: self.peak_buffer_size,
            total_bytes_written: self.total_bytes_written,
            total_flushes: self.total_flushes,
            queued_buffers: self.buffers.len(),
            current_buffer_size: self.current_buffer.len(),
            is_near_capacity: self.is_near_capacity(),
        }
    }
}

/// Statistics about buffer usage
#[derive(Debug, Clone)]
pub struct BufferStats {
    pub current_memory_usage: usize,
    pub peak_memory_usage: usize,
    pub total_bytes_written: usize,
    pub total_flushes: usize,
    pub queued_buffers: usize,
    pub current_buffer_size: usize,
    pub is_near_capacity: bool,
}

impl<W: IoWrite> Drop for BufferManager<W> {
    /// Ensure all buffers are flushed when dropped
    fn drop(&mut self) {
        let _ = self.flush_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_basic_buffering() {
        let output = Vec::new();
        let cursor = Cursor::new(output);
        let mut buffer_manager = BufferManager::new(cursor, 100).unwrap();
        
        // Write some data
        buffer_manager.write_chunk(b"Hello, ").unwrap();
        buffer_manager.write_chunk(b"World!").unwrap();
        
        assert_eq!(buffer_manager.current_buffer_size(), 13);
        assert_eq!(buffer_manager.total_bytes_written(), 0); // Not flushed yet
        
        // Flush all
        buffer_manager.flush_all().unwrap();
        
        assert_eq!(buffer_manager.total_bytes_written(), 13);
        let output = buffer_manager.writer.clone().into_inner();
        assert_eq!(output, b"Hello, World!");
    }
    
    #[test]
    fn test_automatic_flushing() {
        let output = Vec::new();
        let cursor = Cursor::new(output);
        let mut buffer_manager = BufferManager::new(cursor, 10).unwrap(); // Small buffer
        
        // Write data that exceeds buffer size
        buffer_manager.write_chunk(b"This is a longer string").unwrap();
        
        // Should have been written directly
        assert!(buffer_manager.total_bytes_written() > 0);
    }
    
    #[test]
    fn test_buffer_stats() {
        let output = Vec::new();
        let cursor = Cursor::new(output);
        let mut buffer_manager = BufferManager::new(cursor, 100).unwrap();
        
        buffer_manager.write_chunk(b"test data").unwrap();
        
        let stats = buffer_manager.get_stats();
        assert_eq!(stats.current_buffer_size, 9);
        assert_eq!(stats.total_bytes_written, 0);
        assert_eq!(stats.queued_buffers, 0);
    }
    
    #[test] 
    fn test_flush_callback() {
        use std::sync::{Arc, Mutex};
        
        let output = Vec::new();
        let cursor = Cursor::new(output);
        let mut buffer_manager = BufferManager::new(cursor, 10).unwrap();
        
        let flush_count = Arc::new(Mutex::new(0));
        let flush_count_clone = flush_count.clone();
        
        buffer_manager.set_flush_callback(Box::new(move |_size| {
            let mut count = flush_count_clone.lock().unwrap();
            *count += 1;
        }));
        
        // Write enough to trigger flush
        buffer_manager.write_chunk(b"This will trigger a flush").unwrap();
        buffer_manager.flush_all().unwrap();
        
        let count = *flush_count.lock().unwrap();
        assert!(count > 0);
    }
}