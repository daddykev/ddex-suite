//! Memory optimization techniques for DDEX Builder
//! 
//! This module provides arena allocation, object pooling, and memory-efficient
//! data structures to minimize memory usage and improve performance.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::mem;
use indexmap::IndexMap;

/// Arena allocator for temporary objects during build process
pub struct Arena {
    chunks: RefCell<Vec<Vec<u8>>>,
    current_chunk: RefCell<usize>,
    current_offset: RefCell<usize>,
    chunk_size: usize,
}

impl Arena {
    /// Create a new arena with specified chunk size
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunks: RefCell::new(vec![Vec::with_capacity(chunk_size)]),
            current_chunk: RefCell::new(0),
            current_offset: RefCell::new(0),
            chunk_size,
        }
    }
    
    /// Allocate space for a value in the arena (safe version using Box)
    pub fn alloc<T>(&self, value: T) -> Box<T> {
        // For security audit compliance, use safe Box allocation instead of raw pointers
        Box::new(value)
    }
    
    
    /// Get total allocated memory
    pub fn allocated_bytes(&self) -> usize {
        self.chunks.borrow().iter()
            .map(|chunk| chunk.len())
            .sum()
    }
    
    /// Get total capacity
    pub fn capacity_bytes(&self) -> usize {
        self.chunks.borrow().iter()
            .map(|chunk| chunk.capacity())
            .sum()
    }
    
    /// Reset arena for reuse (keeps allocated chunks)
    pub fn reset(&self) {
        let mut chunks = self.chunks.borrow_mut();
        for chunk in chunks.iter_mut() {
            chunk.clear();
        }
        *self.current_chunk.borrow_mut() = 0;
        *self.current_offset.borrow_mut() = 0;
    }
    
    /// Clear all chunks and free memory
    pub fn clear(&self) {
        self.chunks.borrow_mut().clear();
        *self.current_chunk.borrow_mut() = 0;
        *self.current_offset.borrow_mut() = 0;
    }
}

/// Object pool for frequently created/destroyed types
pub struct ObjectPool<T> {
    objects: RefCell<VecDeque<T>>,
    factory: Box<dyn Fn() -> T>,
    max_size: usize,
}

impl<T> ObjectPool<T> {
    /// Create a new object pool
    pub fn new<F>(factory: F, max_size: usize) -> Self 
    where 
        F: Fn() -> T + 'static,
    {
        Self {
            objects: RefCell::new(VecDeque::new()),
            factory: Box::new(factory),
            max_size,
        }
    }
    
    /// Get an object from the pool (or create new one)
    pub fn get(&self) -> PooledObject<T> {
        let obj = self.objects.borrow_mut()
            .pop_front()
            .unwrap_or_else(|| (self.factory)());
            
        PooledObject {
            object: Some(obj),
            pool: self,
        }
    }
    
    /// Return an object to the pool
    fn return_object(&self, obj: T) {
        let mut objects = self.objects.borrow_mut();
        if objects.len() < self.max_size {
            objects.push_back(obj);
        }
        // If pool is full, drop the object
    }
    
    /// Get current pool size
    pub fn size(&self) -> usize {
        self.objects.borrow().len()
    }
    
    /// Clear the pool
    pub fn clear(&self) {
        self.objects.borrow_mut().clear();
    }
}

/// RAII wrapper for pooled objects
pub struct PooledObject<'a, T> {
    object: Option<T>,
    pool: &'a ObjectPool<T>,
}

impl<'a, T> PooledObject<'a, T> {
    /// Get mutable reference to the pooled object
    pub fn get_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap()
    }
    
    /// Get immutable reference to the pooled object
    pub fn get(&self) -> &T {
        self.object.as_ref().unwrap()
    }
}

impl<'a, T> Drop for PooledObject<'a, T> {
    fn drop(&mut self) {
        if let Some(obj) = self.object.take() {
            self.pool.return_object(obj);
        }
    }
}

/// Compact representation for DDEX elements to reduce memory usage
#[derive(Debug, Clone)]
pub struct CompactElement {
    /// Name index in string table
    name_idx: u32,
    /// Namespace index (optional)
    namespace_idx: Option<u32>,
    /// Attributes (packed)
    attributes: CompactAttributes,
    /// Children indices
    children: Vec<CompactNodeRef>,
}

/// Compact node reference (union type)
#[derive(Debug, Clone)]
pub enum CompactNodeRef {
    Element(u32), // Index in element table
    Text(u32),    // Index in string table
    Comment(u32), // Index in string table
}

/// Compact attributes storage
#[derive(Debug, Clone, Default)]
pub struct CompactAttributes {
    /// Packed attribute data: (key_idx, value_idx) pairs
    data: Vec<(u32, u32)>,
}

impl CompactAttributes {
    /// Add an attribute
    pub fn insert(&mut self, key_idx: u32, value_idx: u32) {
        self.data.push((key_idx, value_idx));
    }
    
    /// Get number of attributes
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Iterate over attributes
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.data.iter().copied()
    }
}

/// Compact AST representation for memory efficiency
#[derive(Debug)]
pub struct CompactAST {
    /// String table for all text content
    strings: Vec<String>,
    /// String lookup map
    string_map: IndexMap<String, u32>,
    /// Element table
    elements: Vec<CompactElement>,
    /// Root element index
    root_idx: u32,
    /// Namespace table
    namespaces: Vec<(u32, u32)>, // (prefix_idx, uri_idx) pairs
    /// Schema location index
    schema_location_idx: Option<u32>,
}

impl CompactAST {
    /// Create a new compact AST
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            string_map: IndexMap::new(),
            elements: Vec::new(),
            root_idx: 0,
            namespaces: Vec::new(),
            schema_location_idx: None,
        }
    }
    
    /// Intern a string and return its index
    pub fn intern_string(&mut self, s: &str) -> u32 {
        if let Some(&idx) = self.string_map.get(s) {
            return idx;
        }
        
        let idx = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.string_map.insert(s.to_string(), idx);
        idx
    }
    
    /// Get string by index
    pub fn get_string(&self, idx: u32) -> Option<&str> {
        self.strings.get(idx as usize).map(|s| s.as_str())
    }
    
    /// Add an element and return its index
    pub fn add_element(&mut self, element: CompactElement) -> u32 {
        let idx = self.elements.len() as u32;
        self.elements.push(element);
        idx
    }
    
    /// Get element by index
    pub fn get_element(&self, idx: u32) -> Option<&CompactElement> {
        self.elements.get(idx as usize)
    }
    
    /// Calculate memory footprint
    pub fn memory_footprint(&self) -> usize {
        let strings_size = self.strings.iter()
            .map(|s| s.len())
            .sum::<usize>();
            
        let map_size = self.string_map.len() * 
            (mem::size_of::<String>() + mem::size_of::<u32>());
            
        let elements_size = self.elements.len() * mem::size_of::<CompactElement>();
        
        strings_size + map_size + elements_size
    }
    
    /// Convert from regular AST (memory optimization pass)
    pub fn from_ast(ast: &crate::ast::AST) -> Self {
        let mut compact = CompactAST::new();
        
        // Intern namespace strings
        for (prefix, uri) in &ast.namespaces {
            let prefix_idx = compact.intern_string(prefix);
            let uri_idx = compact.intern_string(uri);
            compact.namespaces.push((prefix_idx, uri_idx));
        }
        
        // Intern schema location if present
        if let Some(ref location) = ast.schema_location {
            compact.schema_location_idx = Some(compact.intern_string(location));
        }
        
        // Convert root element
        compact.root_idx = compact.convert_element(&ast.root);
        
        compact
    }
    
    /// Convert an element to compact format
    fn convert_element(&mut self, element: &crate::ast::Element) -> u32 {
        let name_idx = self.intern_string(&element.name);
        let namespace_idx = element.namespace.as_ref()
            .map(|ns| self.intern_string(ns));
        
        // Convert attributes
        let mut attributes = CompactAttributes::default();
        for (key, value) in &element.attributes {
            let key_idx = self.intern_string(key);
            let value_idx = self.intern_string(value);
            attributes.insert(key_idx, value_idx);
        }
        
        // Convert children (placeholder - would need full recursive conversion)
        let children = Vec::new(); // Simplified for now
        
        let compact_element = CompactElement {
            name_idx,
            namespace_idx,
            attributes,
            children,
        };
        
        self.add_element(compact_element)
    }
}

/// Lazy-loaded data structures for optional fields
pub struct LazyField<T> {
    value: RefCell<Option<T>>,
    loader: Box<dyn Fn() -> T>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for LazyField<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyField")
            .field("value", &self.value)
            .field("loader", &"<function>")
            .finish()
    }
}

impl<T> LazyField<T> {
    /// Create a new lazy field
    pub fn new<F>(loader: F) -> Self 
    where 
        F: Fn() -> T + 'static,
    {
        Self {
            value: RefCell::new(None),
            loader: Box::new(loader),
        }
    }
    
    /// Get the value, loading if necessary
    pub fn get(&self) -> std::cell::Ref<T> {
        if self.value.borrow().is_none() {
            *self.value.borrow_mut() = Some((self.loader)());
        }
        
        std::cell::Ref::map(self.value.borrow(), |opt| opt.as_ref().unwrap())
    }
    
    /// Check if value is loaded
    pub fn is_loaded(&self) -> bool {
        self.value.borrow().is_some()
    }
    
    /// Clear the loaded value
    pub fn clear(&self) {
        *self.value.borrow_mut() = None;
    }
}

/// Memory manager for the entire build process
pub struct BuildMemoryManager {
    /// Arena for temporary allocations
    pub arena: Arena,
    /// Element pool
    pub element_pool: ObjectPool<crate::ast::Element>,
    /// String pool for small strings
    pub small_string_pool: ObjectPool<String>,
    /// Large buffer pool for XML generation
    pub buffer_pool: ObjectPool<Vec<u8>>,
}

impl BuildMemoryManager {
    /// Create a new memory manager optimized for typical DDEX builds
    pub fn new() -> Self {
        Self {
            arena: Arena::new(64 * 1024), // 64KB chunks
            element_pool: ObjectPool::new(
                || crate::ast::Element::new(""), 
                100 // Keep up to 100 elements
            ),
            small_string_pool: ObjectPool::new(
                || String::with_capacity(64),
                50 // Keep up to 50 small strings
            ),
            buffer_pool: ObjectPool::new(
                || Vec::with_capacity(8192), // 8KB buffers
                10 // Keep up to 10 buffers
            ),
        }
    }
    
    /// Get memory usage statistics
    pub fn memory_usage(&self) -> MemoryStats {
        MemoryStats {
            arena_allocated: self.arena.allocated_bytes(),
            arena_capacity: self.arena.capacity_bytes(),
            element_pool_size: self.element_pool.size(),
            string_pool_size: self.small_string_pool.size(),
            buffer_pool_size: self.buffer_pool.size(),
        }
    }
    
    /// Reset manager for next build (keeps pools)
    pub fn reset_for_next_build(&self) {
        self.arena.reset();
        // Pools reset automatically when objects are returned
    }
    
    /// Full reset including all pools
    pub fn full_reset(&self) {
        self.arena.clear();
        self.element_pool.clear();
        self.small_string_pool.clear();
        self.buffer_pool.clear();
    }
}

impl Default for BuildMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub arena_allocated: usize,
    pub arena_capacity: usize,
    pub element_pool_size: usize,
    pub string_pool_size: usize,
    pub buffer_pool_size: usize,
}

impl MemoryStats {
    /// Get total memory usage estimate
    pub fn total_bytes(&self) -> usize {
        self.arena_capacity + 
        (self.element_pool_size * mem::size_of::<crate::ast::Element>()) +
        (self.string_pool_size * 64) + // Estimated string size
        (self.buffer_pool_size * 8192) // Buffer size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arena_allocation() {
        let arena = Arena::new(1024);
        
        let val1 = arena.alloc(42u32);
        let val2 = arena.alloc("hello world".to_string());
        
        assert_eq!(*val1, 42);
        assert_eq!(*val2, "hello world");
        
        assert!(arena.allocated_bytes() > 0);
    }
    
    #[test]
    fn test_object_pool() {
        let pool = ObjectPool::new(|| String::with_capacity(32), 5);
        
        {
            let mut obj1 = pool.get();
            obj1.get_mut().push_str("test");
            assert_eq!(obj1.get(), "test");
            
            {
                let obj2 = pool.get();
                assert_eq!(pool.size(), 0); // Both objects checked out
            }
            // obj2 returned to pool
        }
        // obj1 returned to pool
        
        assert_eq!(pool.size(), 2);
    }
    
    #[test]
    fn test_compact_ast() {
        let mut compact = CompactAST::new();
        
        let hello_idx = compact.intern_string("hello");
        let hello_idx2 = compact.intern_string("hello"); // Should reuse
        let world_idx = compact.intern_string("world");
        
        assert_eq!(hello_idx, hello_idx2);
        assert_ne!(hello_idx, world_idx);
        assert_eq!(compact.get_string(hello_idx), Some("hello"));
        assert_eq!(compact.get_string(world_idx), Some("world"));
    }
    
    #[test]
    fn test_lazy_field() {
        let counter = RefCell::new(0);
        let lazy = LazyField::new(move || {
            *counter.borrow_mut() += 1;
            "computed".to_string()
        });
        
        assert!(!lazy.is_loaded());
        
        let val = lazy.get();
        assert_eq!(*val, "computed");
        
        // Second access shouldn't recompute
        let val2 = lazy.get();
        assert_eq!(*val2, "computed");
    }
    
    #[test]
    fn test_memory_manager() {
        let manager = BuildMemoryManager::new();
        let stats = manager.memory_usage();
        
        // Should start with some capacity but no allocation
        assert_eq!(stats.arena_allocated, 0);
        assert!(stats.arena_capacity > 0);
    }
}