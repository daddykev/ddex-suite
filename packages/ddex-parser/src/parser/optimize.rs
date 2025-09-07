// core/src/parser/optimize.rs
//! Performance optimizations

use std::simd::{u8x32, SimdPartialEq};

/// Fast XML tag scanner using SIMD
pub struct FastScanner {
    buffer: Vec<u8>,
}

impl FastScanner {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(8192),
        }
    }
    
    /// Find next tag boundary using SIMD
    #[inline]
    pub fn find_next_tag(&self, data: &[u8], start: usize) -> Option<usize> {
        let needle = u8x32::splat(b'<');
        let chunks = data[start..].chunks_exact(32);
        let remainder = chunks.remainder();
        
        for (i, chunk) in chunks.enumerate() {
            let chunk_vec = u8x32::from_slice(chunk);
            let matches = chunk_vec.simd_eq(needle);
            
            if matches.any() {
                // Found a match, find exact position
                for (j, &byte) in chunk.iter().enumerate() {
                    if byte == b'<' {
                        return Some(start + i * 32 + j);
                    }
                }
            }
        }
        
        // Check remainder
        for (i, &byte) in remainder.iter().enumerate() {
            if byte == b'<' {
                return Some(start + data.len() - remainder.len() + i);
            }
        }
        
        None
    }
}

/// String interning for repeated values
pub struct StringInterner {
    strings: std::collections::HashMap<String, std::rc::Rc<str>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: std::collections::HashMap::with_capacity(1000),
        }
    }
    
    pub fn intern(&mut self, s: String) -> std::rc::Rc<str> {
        if let Some(existing) = self.strings.get(&s) {
            existing.clone()
        } else {
            let rc: std::rc::Rc<str> = s.clone().into();
            self.strings.insert(s, rc.clone());
            rc
        }
    }
}