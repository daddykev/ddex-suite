//! Optimized string handling for DDEX Builder performance
//! 
//! This module provides string interning, Cow optimization, and memory-efficient
//! string operations to reduce allocations and improve build performance.

use std::borrow::Cow;
use indexmap::IndexMap;
use std::sync::Arc;
use once_cell::sync::Lazy;
use smartstring::{SmartString, LazyCompact};
use string_cache::{DefaultAtom, Atom};
use indexmap::IndexSet;

/// High-performance string type for small strings
pub type FastString = SmartString<LazyCompact>;

/// Static string cache for common DDEX values
static COMMON_STRINGS: Lazy<IndexMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = IndexMap::new();
    
    // Common DDEX versions
    map.insert("4.3", "4.3");
    map.insert("4.2", "4.2");
    map.insert("4.1", "4.1");
    
    // Common message types
    map.insert("NewReleaseMessage", "NewReleaseMessage");
    map.insert("PurgeReleaseMessage", "PurgeReleaseMessage");
    map.insert("LiveMessage", "LiveMessage");
    
    // Common roles
    map.insert("MainArtist", "MainArtist");
    map.insert("FeaturedArtist", "FeaturedArtist");
    map.insert("Producer", "Producer");
    map.insert("Composer", "Composer");
    map.insert("Performer", "Performer");
    map.insert("Engineer", "Engineer");
    map.insert("Mixer", "Mixer");
    
    // Common resource types
    map.insert("SoundRecording", "SoundRecording");
    map.insert("Video", "Video");
    map.insert("Image", "Image");
    map.insert("Text", "Text");
    
    // Common release types
    map.insert("Single", "Single");
    map.insert("Album", "Album");
    map.insert("EP", "EP");
    map.insert("Compilation", "Compilation");
    
    // Common genres
    map.insert("Rock", "Rock");
    map.insert("Pop", "Pop");
    map.insert("Electronic", "Electronic");
    map.insert("Hip-Hop", "Hip-Hop");
    map.insert("Classical", "Classical");
    map.insert("Jazz", "Jazz");
    map.insert("Country", "Country");
    map.insert("R&B", "R&B");
    map.insert("Folk", "Folk");
    map.insert("Alternative", "Alternative");
    
    // Common language codes
    map.insert("en", "en");
    map.insert("es", "es");
    map.insert("fr", "fr");
    map.insert("de", "de");
    map.insert("it", "it");
    map.insert("pt", "pt");
    map.insert("ja", "ja");
    map.insert("ko", "ko");
    map.insert("zh", "zh");
    
    // Common territory codes
    map.insert("US", "US");
    map.insert("GB", "GB");
    map.insert("CA", "CA");
    map.insert("AU", "AU");
    map.insert("DE", "DE");
    map.insert("FR", "FR");
    map.insert("JP", "JP");
    map.insert("KR", "KR");
    
    // Common commercial models
    map.insert("SubscriptionModel", "SubscriptionModel");
    map.insert("PermanentDownload", "PermanentDownload");
    map.insert("AdSupportedModel", "AdSupportedModel");
    map.insert("ConditionalDownload", "ConditionalDownload");
    
    // Common prefixes for copyright
    map.insert("℗ ", "℗ ");
    map.insert("© ", "© ");
    
    map
});

/// String interner for repeated values during build process
#[derive(Debug, Default)]
pub struct StringInterner {
    /// Interned strings storage
    strings: IndexSet<Arc<str>>,
    /// Quick lookup for atoms
    atoms: IndexMap<String, DefaultAtom>,
}

impl StringInterner {
    /// Create a new string interner
    pub fn new() -> Self {
        Self {
            strings: IndexSet::new(),
            atoms: IndexMap::new(),
        }
    }
    
    /// Intern a string, returning a reference to the interned version
    pub fn intern(&mut self, s: &str) -> Arc<str> {
        // Check static cache first
        if let Some(&static_str) = COMMON_STRINGS.get(s) {
            return Arc::from(static_str);
        }
        
        // Check if already interned
        if let Some(existing) = self.strings.get(s) {
            return existing.clone();
        }
        
        // Intern new string
        let arc_str: Arc<str> = Arc::from(s);
        self.strings.insert(arc_str.clone());
        arc_str
    }
    
    /// Intern as an atom for even better performance on repeated lookups
    pub fn intern_atom(&mut self, s: String) -> DefaultAtom {
        if let Some(atom) = self.atoms.get(&s) {
            return atom.clone();
        }
        
        let atom = DefaultAtom::from(s.as_str());
        self.atoms.insert(s, atom.clone());
        atom
    }
    
    /// Get memory usage statistics
    pub fn memory_usage(&self) -> usize {
        self.strings.iter()
            .map(|s| s.len() + std::mem::size_of::<Arc<str>>())
            .sum::<usize>()
            + self.atoms.len() * std::mem::size_of::<DefaultAtom>()
    }
    
    /// Clear the interner (useful for long-running processes)
    pub fn clear(&mut self) {
        self.strings.clear();
        self.atoms.clear();
    }
}

/// Optimized string for DDEX data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptimizedString {
    /// Static string reference (zero allocation)
    Static(&'static str),
    /// Interned string (shared allocation)
    Interned(Arc<str>),
    /// Small string optimization
    Small(FastString),
    /// Atom for very frequent lookups
    Atom(DefaultAtom),
}

impl OptimizedString {
    /// Create from a string, choosing the most efficient representation
    pub fn new(s: &str) -> Self {
        // Check if it's a common static string
        if let Some(&static_str) = COMMON_STRINGS.get(s) {
            return OptimizedString::Static(static_str);
        }
        
        // Use small string optimization for short strings
        if s.len() <= 23 {  // SmartString threshold
            OptimizedString::Small(FastString::from(s))
        } else {
            // For longer strings, we'll need interning context
            OptimizedString::Small(FastString::from(s))
        }
    }
    
    /// Create from an interned string
    pub fn interned(s: Arc<str>) -> Self {
        OptimizedString::Interned(s)
    }
    
    /// Create from an atom
    pub fn atom(atom: DefaultAtom) -> Self {
        OptimizedString::Atom(atom)
    }
    
    /// Get the string value
    pub fn as_str(&self) -> &str {
        match self {
            OptimizedString::Static(s) => s,
            OptimizedString::Interned(s) => s,
            OptimizedString::Small(s) => s,
            OptimizedString::Atom(atom) => atom,
        }
    }
    
    /// Get memory footprint in bytes
    pub fn memory_footprint(&self) -> usize {
        match self {
            OptimizedString::Static(_) => 0, // No allocation
            OptimizedString::Interned(_) => std::mem::size_of::<Arc<str>>(),
            OptimizedString::Small(s) => s.capacity(),
            OptimizedString::Atom(_) => std::mem::size_of::<DefaultAtom>(),
        }
    }
}

impl AsRef<str> for OptimizedString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for OptimizedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Cow-optimized string for contexts where we may or may not own the data
pub type CowString = Cow<'static, str>;

/// Optimized localized string that minimizes allocations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimizedLocalizedString {
    pub text: OptimizedString,
    pub language_code: Option<OptimizedString>,
}

impl OptimizedLocalizedString {
    /// Create a new optimized localized string
    pub fn new(text: &str, language_code: Option<&str>) -> Self {
        Self {
            text: OptimizedString::new(text),
            language_code: language_code.map(OptimizedString::new),
        }
    }
    
    /// Memory footprint of this localized string
    pub fn memory_footprint(&self) -> usize {
        self.text.memory_footprint() 
            + self.language_code.as_ref()
                .map(|lc| lc.memory_footprint())
                .unwrap_or(0)
    }
}

/// Buffer pool for XML generation to reduce allocations
#[derive(Debug, Default)]
pub struct BufferPool {
    buffers: Vec<String>,
    current_size: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            current_size: 0,
        }
    }
    
    /// Get a buffer from the pool, or create one if none available
    pub fn get_buffer(&mut self, estimated_size: usize) -> String {
        match self.buffers.pop() {
            Some(mut buffer) => {
                buffer.clear();
                if buffer.capacity() < estimated_size {
                    buffer.reserve(estimated_size - buffer.capacity());
                }
                buffer
            }
            None => {
                self.current_size += estimated_size;
                String::with_capacity(estimated_size)
            }
        }
    }
    
    /// Return a buffer to the pool
    pub fn return_buffer(&mut self, buffer: String) {
        if buffer.capacity() <= 8192 {  // Don't keep huge buffers
            self.buffers.push(buffer);
        }
    }
    
    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.current_size + self.buffers.iter()
            .map(|b| b.capacity())
            .sum::<usize>()
    }
    
    /// Clear the pool
    pub fn clear(&mut self) {
        self.buffers.clear();
        self.current_size = 0;
    }
}

/// Build context that manages optimized strings and buffers
#[derive(Debug, Default)]
pub struct BuildContext {
    /// String interner for repeated values
    pub interner: StringInterner,
    /// Buffer pool for XML generation
    pub buffer_pool: BufferPool,
    /// Statistics
    pub stats: BuildStats,
}

impl BuildContext {
    /// Create a new build context
    pub fn new() -> Self {
        Self {
            interner: StringInterner::new(),
            buffer_pool: BufferPool::new(),
            stats: BuildStats::default(),
        }
    }
    
    /// Optimize a string using the context's interner
    pub fn optimize_string(&mut self, s: &str) -> OptimizedString {
        self.stats.strings_processed += 1;
        
        // Track if we use static cache
        if COMMON_STRINGS.contains_key(s) {
            self.stats.static_cache_hits += 1;
            return OptimizedString::new(s);
        }
        
        // Check if worth interning (repeated strings)
        if s.len() > 23 {  // Beyond small string optimization
            let interned = self.interner.intern(s);
            self.stats.interned_strings += 1;
            OptimizedString::interned(interned)
        } else {
            OptimizedString::new(s)
        }
    }
    
    /// Get a buffer for XML generation
    pub fn get_xml_buffer(&mut self, estimated_size: usize) -> String {
        self.stats.buffers_requested += 1;
        self.buffer_pool.get_buffer(estimated_size)
    }
    
    /// Return a buffer to the pool
    pub fn return_xml_buffer(&mut self, buffer: String) {
        self.buffer_pool.return_buffer(buffer);
    }
    
    /// Get memory usage statistics
    pub fn memory_usage(&self) -> MemoryUsage {
        MemoryUsage {
            interner_bytes: self.interner.memory_usage(),
            buffer_pool_bytes: self.buffer_pool.memory_usage(),
            total_bytes: self.interner.memory_usage() + self.buffer_pool.memory_usage(),
        }
    }
    
    /// Reset context for next build (keeps caches)
    pub fn reset_for_next_build(&mut self) {
        // Don't clear interner - strings likely to be reused
        self.buffer_pool.clear();
        self.stats = BuildStats::default();
    }
    
    /// Full reset including caches
    pub fn full_reset(&mut self) {
        self.interner.clear();
        self.buffer_pool.clear();
        self.stats = BuildStats::default();
    }
}

/// Build statistics for performance monitoring
#[derive(Debug, Default, Clone)]
pub struct BuildStats {
    pub strings_processed: usize,
    pub static_cache_hits: usize,
    pub interned_strings: usize,
    pub buffers_requested: usize,
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub interner_bytes: usize,
    pub buffer_pool_bytes: usize,
    pub total_bytes: usize,
}

/// Pre-calculate buffer sizes for different build types
pub mod buffer_sizes {
    /// Estimated XML output sizes
    pub const SINGLE_TRACK_XML: usize = 8_192;      // ~8KB
    pub const ALBUM_12_TRACKS_XML: usize = 65_536;  // ~64KB  
    pub const COMPILATION_100_TRACKS_XML: usize = 524_288; // ~512KB
    
    /// Buffer overhead factors
    pub const BUFFER_OVERHEAD_FACTOR: f32 = 1.2; // 20% overhead for safety
    
    /// Calculate estimated buffer size for track count
    pub fn estimated_xml_size(track_count: usize) -> usize {
        let base_size = match track_count {
            1 => SINGLE_TRACK_XML,
            2..=20 => ALBUM_12_TRACKS_XML,
            _ => COMPILATION_100_TRACKS_XML,
        };
        
        // Scale linearly for track count
        let scaled = if track_count <= 20 {
            (base_size * track_count / 12).max(SINGLE_TRACK_XML)
        } else {
            COMPILATION_100_TRACKS_XML * track_count / 100
        };
        
        (scaled as f32 * BUFFER_OVERHEAD_FACTOR) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimized_string_static_cache() {
        let s = OptimizedString::new("MainArtist");
        match s {
            OptimizedString::Static(val) => assert_eq!(val, "MainArtist"),
            _ => panic!("Expected static string"),
        }
        
        // Should be zero allocation
        assert_eq!(s.memory_footprint(), 0);
    }
    
    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();
        
        let s1 = interner.intern("Custom Artist Name");
        let s2 = interner.intern("Custom Artist Name");
        
        // Should be same Arc
        assert_eq!(s1.as_ptr(), s2.as_ptr());
    }
    
    #[test]
    fn test_buffer_pool() {
        let mut pool = BufferPool::new();
        
        let mut buffer = pool.get_buffer(1024);
        buffer.push_str("test content");
        
        assert!(buffer.capacity() >= 1024);
        
        pool.return_buffer(buffer);
        
        let buffer2 = pool.get_buffer(512);
        assert!(buffer2.is_empty());
        assert!(buffer2.capacity() >= 1024); // Reused larger buffer
    }
    
    #[test]
    fn test_buffer_size_estimation() {
        assert_eq!(buffer_sizes::estimated_xml_size(1), 
                   (buffer_sizes::SINGLE_TRACK_XML as f32 * buffer_sizes::BUFFER_OVERHEAD_FACTOR) as usize);
        
        assert_eq!(buffer_sizes::estimated_xml_size(12),
                   (buffer_sizes::ALBUM_12_TRACKS_XML as f32 * buffer_sizes::BUFFER_OVERHEAD_FACTOR) as usize);
        
        // Large compilation should scale
        let size_100 = buffer_sizes::estimated_xml_size(100);
        let size_200 = buffer_sizes::estimated_xml_size(200);
        assert!(size_200 > size_100);
    }
}