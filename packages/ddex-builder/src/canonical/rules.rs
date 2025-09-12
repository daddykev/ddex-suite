//! # Canonical XML Rules for DB-C14N/1.0
//! 
//! This module defines the canonical XML transformation rules for DDEX Builder,
//! including comprehensive namespace prefix locking, element ordering, and
//! support for all DDEX standards (ERN, AVS, MEAD, PIE, etc.).

use indexmap::IndexMap;
use ddex_core::namespace::NamespaceRegistry;
use std::collections::HashSet;

/// Fixed XML declaration for all canonical XML
pub const XML_DECLARATION: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>";

/// Comprehensive namespace prefix lock tables for all ERN versions
pub struct NamespacePrefixLock {
    /// Registry for namespace management
    registry: NamespaceRegistry,
    /// Locked prefixes for specific versions
    version_locks: IndexMap<String, IndexMap<String, String>>, // version -> (uri -> prefix)
    /// Reserved prefixes that cannot be used
    reserved_prefixes: HashSet<String>,
}

impl NamespacePrefixLock {
    /// Create new prefix lock manager
    pub fn new() -> Self {
        let mut lock = Self {
            registry: NamespaceRegistry::new(),
            version_locks: IndexMap::new(),
            reserved_prefixes: HashSet::new(),
        };
        
        lock.initialize_version_locks();
        lock
    }
    
    /// Initialize prefix locks for all ERN versions
    fn initialize_version_locks(&mut self) {
        // ERN 3.8.2 prefix locks
        let mut ern_382_prefixes = IndexMap::new();
        ern_382_prefixes.insert("http://ddex.net/xml/ern/382".to_string(), "ern".to_string());
        ern_382_prefixes.insert("http://ddex.net/xml/avs".to_string(), "avs".to_string());
        ern_382_prefixes.insert("http://ddex.net/xml/avs/avs".to_string(), "avs".to_string());
        ern_382_prefixes.insert("http://www.w3.org/2001/XMLSchema-instance".to_string(), "xsi".to_string());
        ern_382_prefixes.insert("http://www.w3.org/2001/XMLSchema".to_string(), "xs".to_string());
        ern_382_prefixes.insert("http://ddex.net/xml/gc".to_string(), "gc".to_string());
        self.version_locks.insert("3.8.2".to_string(), ern_382_prefixes.clone());
        self.version_locks.insert("382".to_string(), ern_382_prefixes);

        // ERN 4.2 prefix locks
        let mut ern_42_prefixes = IndexMap::new();
        ern_42_prefixes.insert("http://ddex.net/xml/ern/42".to_string(), "ern".to_string());
        ern_42_prefixes.insert("http://ddex.net/xml/avs".to_string(), "avs".to_string());
        ern_42_prefixes.insert("http://ddex.net/xml/avs/avs".to_string(), "avs".to_string());
        ern_42_prefixes.insert("http://www.w3.org/2001/XMLSchema-instance".to_string(), "xsi".to_string());
        ern_42_prefixes.insert("http://www.w3.org/2001/XMLSchema".to_string(), "xs".to_string());
        ern_42_prefixes.insert("http://ddex.net/xml/gc".to_string(), "gc".to_string());
        // Additional 4.2 namespaces
        ern_42_prefixes.insert("http://ddex.net/xml/mead/mead".to_string(), "mead".to_string());
        ern_42_prefixes.insert("http://ddex.net/xml/pie/pie".to_string(), "pie".to_string());
        self.version_locks.insert("4.2".to_string(), ern_42_prefixes.clone());
        self.version_locks.insert("42".to_string(), ern_42_prefixes);

        // ERN 4.3 prefix locks
        let mut ern_43_prefixes = IndexMap::new();
        ern_43_prefixes.insert("http://ddex.net/xml/ern/43".to_string(), "ern".to_string());
        ern_43_prefixes.insert("http://ddex.net/xml/avs".to_string(), "avs".to_string());
        ern_43_prefixes.insert("http://ddex.net/xml/avs/avs".to_string(), "avs".to_string());
        ern_43_prefixes.insert("http://www.w3.org/2001/XMLSchema-instance".to_string(), "xsi".to_string());
        ern_43_prefixes.insert("http://www.w3.org/2001/XMLSchema".to_string(), "xs".to_string());
        ern_43_prefixes.insert("http://ddex.net/xml/gc".to_string(), "gc".to_string());
        // Additional 4.3 namespaces
        ern_43_prefixes.insert("http://ddex.net/xml/mead/mead".to_string(), "mead".to_string());
        ern_43_prefixes.insert("http://ddex.net/xml/pie/pie".to_string(), "pie".to_string());
        ern_43_prefixes.insert("http://ddex.net/xml/rin/rin".to_string(), "rin".to_string());
        ern_43_prefixes.insert("http://ddex.net/xml/dsrf".to_string(), "dsrf".to_string());
        self.version_locks.insert("4.3".to_string(), ern_43_prefixes.clone());
        self.version_locks.insert("43".to_string(), ern_43_prefixes);
        
        // Mark all locked prefixes as reserved
        for prefixes in self.version_locks.values() {
            for prefix in prefixes.values() {
                self.reserved_prefixes.insert(prefix.clone());
            }
        }
    }
    
    /// Get locked prefix for a namespace URI in a specific version
    pub fn get_locked_prefix(&self, uri: &str, version: &str) -> Option<&str> {
        self.version_locks
            .get(version)
            .and_then(|prefixes| prefixes.get(uri))
            .map(|prefix| prefix.as_str())
    }
    
    /// Get all locked prefixes for a version
    pub fn get_version_prefixes(&self, version: &str) -> Option<&IndexMap<String, String>> {
        self.version_locks.get(version)
    }
    
    /// Apply prefix deduplication algorithm
    pub fn deduplicate_prefixes(&self, declarations: &IndexMap<String, String>, version: &str) -> IndexMap<String, String> {
        let mut deduplicated = IndexMap::new();
        let locked_prefixes = self.get_version_prefixes(version).cloned().unwrap_or_default();
        
        // First pass: apply locked prefixes
        for (original_prefix, uri) in declarations {
            if let Some(locked_prefix) = locked_prefixes.get(uri) {
                deduplicated.insert(locked_prefix.clone(), uri.clone());
            } else {
                // Use original prefix if not locked
                deduplicated.insert(original_prefix.clone(), uri.clone());
            }
        }
        
        // Second pass: resolve conflicts
        let mut final_declarations = IndexMap::new();
        let mut used_prefixes = HashSet::new();
        
        for (prefix, uri) in deduplicated {
            if used_prefixes.contains(&prefix) {
                // Generate unique prefix
                let unique_prefix = self.generate_unique_prefix(&prefix, &used_prefixes);
                used_prefixes.insert(unique_prefix.clone());
                final_declarations.insert(unique_prefix, uri);
            } else {
                used_prefixes.insert(prefix.clone());
                final_declarations.insert(prefix, uri);
            }
        }
        
        final_declarations
    }
    
    /// Generate a unique prefix to avoid conflicts
    fn generate_unique_prefix(&self, base_prefix: &str, used_prefixes: &HashSet<String>) -> String {
        let mut counter = 1;
        loop {
            let candidate = format!("{}{}", base_prefix, counter);
            if !used_prefixes.contains(&candidate) && !self.reserved_prefixes.contains(&candidate) {
                return candidate;
            }
            counter += 1;
        }
    }
}

/// Comprehensive element ordering for all ERN versions
pub struct ElementOrder {
    /// Element ordering rules by version
    version_orders: IndexMap<String, IndexMap<String, Vec<String>>>, // version -> (parent -> children)
}

impl ElementOrder {
    /// Create new element order manager
    pub fn new() -> Self {
        let mut order = Self {
            version_orders: IndexMap::new(),
        };
        
        order.initialize_element_orders();
        order
    }
    
    /// Initialize element orders for all ERN versions
    fn initialize_element_orders(&mut self) {
        // ERN 3.8.2 element orders
        let mut ern_382_order = IndexMap::new();
        self.add_common_orders(&mut ern_382_order);
        self.add_ern_382_specific_orders(&mut ern_382_order);
        self.version_orders.insert("3.8.2".to_string(), ern_382_order.clone());
        self.version_orders.insert("382".to_string(), ern_382_order);

        // ERN 4.2 element orders
        let mut ern_42_order = IndexMap::new();
        self.add_common_orders(&mut ern_42_order);
        self.add_ern_42_specific_orders(&mut ern_42_order);
        self.version_orders.insert("4.2".to_string(), ern_42_order.clone());
        self.version_orders.insert("42".to_string(), ern_42_order);

        // ERN 4.3 element orders
        let mut ern_43_order = IndexMap::new();
        self.add_common_orders(&mut ern_43_order);
        self.add_ern_43_specific_orders(&mut ern_43_order);
        self.version_orders.insert("4.3".to_string(), ern_43_order.clone());
        self.version_orders.insert("43".to_string(), ern_43_order);
    }
    
    /// Add common element orders across all versions
    fn add_common_orders(&self, order: &mut IndexMap<String, Vec<String>>) {
        // Message header - common to all versions
        order.insert("MessageHeader".to_string(), vec![
            "MessageId".to_string(),
            "MessageType".to_string(),
            "MessageCreatedDateTime".to_string(),
            "MessageSender".to_string(),
            "MessageRecipient".to_string(),
            "MessageControlType".to_string(),
        ]);
        
        // Party - common structure
        order.insert("Party".to_string(), vec![
            "PartyReference".to_string(),
            "PartyId".to_string(),
            "PartyName".to_string(),
            "PartyType".to_string(),
        ]);
        
        // Release - base structure
        order.insert("Release".to_string(), vec![
            "ReleaseReference".to_string(),
            "ReleaseId".to_string(),
            "ReferenceTitle".to_string(),
            "ReleaseType".to_string(),
            "ReleaseResourceReferenceList".to_string(),
            "ReleaseDetailsByTerritory".to_string(),
        ]);
        
        // Deal - base structure
        order.insert("Deal".to_string(), vec![
            "DealReference".to_string(),
            "DealTerms".to_string(),
            "DealReleaseReference".to_string(),
        ]);
    }
    
    /// Add ERN 3.8.2 specific orders
    fn add_ern_382_specific_orders(&self, order: &mut IndexMap<String, Vec<String>>) {
        // SoundRecording for 3.8.2
        order.insert("SoundRecording".to_string(), vec![
            "SoundRecordingType".to_string(),
            "SoundRecordingId".to_string(),
            "ReferenceTitle".to_string(),
            "Duration".to_string(),
            "CreationDate".to_string(),
            "SoundRecordingDetailsByTerritory".to_string(),
        ]);
    }
    
    /// Add ERN 4.2 specific orders
    fn add_ern_42_specific_orders(&self, order: &mut IndexMap<String, Vec<String>>) {
        // MessageHeader with audit trail for 4.2
        order.insert("MessageHeader".to_string(), vec![
            "MessageId".to_string(),
            "MessageType".to_string(),
            "MessageCreatedDateTime".to_string(),
            "MessageSender".to_string(),
            "MessageRecipient".to_string(),
            "MessageControlType".to_string(),
            "MessageAuditTrail".to_string(),
        ]);
        
        // Enhanced SoundRecording for 4.2
        order.insert("SoundRecording".to_string(), vec![
            "SoundRecordingType".to_string(),
            "SoundRecordingId".to_string(),
            "ReferenceTitle".to_string(),
            "DisplayTitle".to_string(),
            "Duration".to_string(),
            "CreationDate".to_string(),
            "MasteredDate".to_string(),
            "SoundRecordingDetailsByTerritory".to_string(),
        ]);
    }
    
    /// Add ERN 4.3 specific orders
    fn add_ern_43_specific_orders(&self, order: &mut IndexMap<String, Vec<String>>) {
        // MessageHeader with audit trail for 4.3
        order.insert("MessageHeader".to_string(), vec![
            "MessageId".to_string(),
            "MessageType".to_string(),
            "MessageCreatedDateTime".to_string(),
            "MessageSender".to_string(),
            "MessageRecipient".to_string(),
            "MessageControlType".to_string(),
            "MessageAuditTrail".to_string(),
        ]);
        
        // Enhanced SoundRecording for 4.3
        order.insert("SoundRecording".to_string(), vec![
            "SoundRecordingType".to_string(),
            "SoundRecordingId".to_string(),
            "ReferenceTitle".to_string(),
            "DisplayTitle".to_string(),
            "DisplayTitleText".to_string(),
            "Duration".to_string(),
            "CreationDate".to_string(),
            "MasteredDate".to_string(),
            "OriginalResourceReleaseDate".to_string(),
            "SoundRecordingDetailsByTerritory".to_string(),
        ]);
        
        // Video for 4.3
        order.insert("Video".to_string(), vec![
            "VideoType".to_string(),
            "VideoId".to_string(),
            "ReferenceTitle".to_string(),
            "DisplayTitle".to_string(),
            "Duration".to_string(),
            "CreationDate".to_string(),
            "VideoDetailsByTerritory".to_string(),
        ]);
        
        // Image for 4.3
        order.insert("Image".to_string(), vec![
            "ImageType".to_string(),
            "ImageId".to_string(),
            "ReferenceTitle".to_string(),
            "DisplayTitle".to_string(),
            "CreationDate".to_string(),
            "ImageDetailsByTerritory".to_string(),
        ]);
    }
    
    /// Get element order for a parent element in a specific version
    pub fn get_element_order(&self, parent_element: &str, version: &str) -> Option<&Vec<String>> {
        self.version_orders
            .get(version)
            .and_then(|orders| orders.get(parent_element))
    }
    
    /// Get all element orders for a version
    pub fn get_version_orders(&self, version: &str) -> Option<&IndexMap<String, Vec<String>>> {
        self.version_orders.get(version)
    }
}

/// AVS namespace handling
pub struct AVSNamespaceHandler {
    /// AVS-specific namespaces
    avs_namespaces: IndexMap<String, String>,
}

impl AVSNamespaceHandler {
    /// Create new AVS namespace handler
    pub fn new() -> Self {
        let mut handler = Self {
            avs_namespaces: IndexMap::new(),
        };
        
        // Initialize AVS namespaces
        handler.avs_namespaces.insert("http://ddex.net/xml/avs".to_string(), "avs".to_string());
        handler.avs_namespaces.insert("http://ddex.net/xml/avs/avs".to_string(), "avs".to_string());
        
        handler
    }
    
    /// Check if a namespace is AVS-related
    pub fn is_avs_namespace(&self, uri: &str) -> bool {
        self.avs_namespaces.contains_key(uri)
    }
    
    /// Get AVS prefix for a namespace
    pub fn get_avs_prefix(&self, uri: &str) -> Option<&str> {
        self.avs_namespaces.get(uri).map(|prefix| prefix.as_str())
    }
}

/// Complete namespace management for canonical XML
pub struct CanonicalNamespaceManager {
    /// Namespace prefix lock
    prefix_lock: NamespacePrefixLock,
    /// Element order manager
    element_order: ElementOrder,
    /// AVS namespace handler
    avs_handler: AVSNamespaceHandler,
}

impl CanonicalNamespaceManager {
    /// Create new canonical namespace manager
    pub fn new() -> Self {
        Self {
            prefix_lock: NamespacePrefixLock::new(),
            element_order: ElementOrder::new(),
            avs_handler: AVSNamespaceHandler::new(),
        }
    }
    
    /// Apply complete canonical transformation
    pub fn canonicalize_namespaces(
        &self, 
        declarations: &IndexMap<String, String>, 
        version: &str
    ) -> IndexMap<String, String> {
        // Apply prefix locking and deduplication
        let locked_declarations = self.prefix_lock.deduplicate_prefixes(declarations, version);
        
        // Sort declarations alphabetically by prefix
        let mut sorted_declarations: Vec<_> = locked_declarations.into_iter().collect();
        sorted_declarations.sort_by(|a, b| a.0.cmp(&b.0));
        
        sorted_declarations.into_iter().collect()
    }
    
    /// Get element order for canonicalization
    pub fn get_canonical_element_order(&self, parent: &str, version: &str) -> Option<&Vec<String>> {
        self.element_order.get_element_order(parent, version)
    }
    
    /// Check if namespace requires special AVS handling
    pub fn requires_avs_handling(&self, uri: &str) -> bool {
        self.avs_handler.is_avs_namespace(uri)
    }
}

impl Default for NamespacePrefixLock {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ElementOrder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AVSNamespaceHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CanonicalNamespaceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for backward compatibility

/// Get namespace prefix table for a specific ERN version
pub fn get_namespace_prefixes(version: &str) -> IndexMap<String, String> {
    let lock = NamespacePrefixLock::new();
    lock.get_version_prefixes(version).cloned().unwrap_or_default()
}

/// Get element order for a specific ERN version
pub fn get_element_order(version: &str) -> IndexMap<String, Vec<String>> {
    let order = ElementOrder::new();
    order.get_version_orders(version).cloned().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_prefix_lock() {
        let lock = NamespacePrefixLock::new();
        
        assert_eq!(
            lock.get_locked_prefix("http://ddex.net/xml/ern/43", "4.3"),
            Some("ern")
        );
        assert_eq!(
            lock.get_locked_prefix("http://ddex.net/xml/avs", "4.3"),
            Some("avs")
        );
    }

    #[test]
    fn test_prefix_deduplication() {
        let lock = NamespacePrefixLock::new();
        
        let mut declarations = IndexMap::new();
        declarations.insert("custom_ern".to_string(), "http://ddex.net/xml/ern/43".to_string());
        declarations.insert("avs".to_string(), "http://ddex.net/xml/avs".to_string());
        
        let deduplicated = lock.deduplicate_prefixes(&declarations, "4.3");
        
        // Should use locked prefix for ERN
        assert_eq!(deduplicated.get("ern"), Some(&"http://ddex.net/xml/ern/43".to_string()));
        assert_eq!(deduplicated.get("avs"), Some(&"http://ddex.net/xml/avs".to_string()));
    }

    #[test]
    fn test_element_order() {
        let order = ElementOrder::new();
        
        let message_order = order.get_element_order("MessageHeader", "4.3");
        assert!(message_order.is_some());
        
        let order_vec = message_order.unwrap();
        assert_eq!(order_vec[0], "MessageId");
        assert_eq!(order_vec[1], "MessageType");
    }

    #[test]
    fn test_avs_namespace_handler() {
        let handler = AVSNamespaceHandler::new();
        
        assert!(handler.is_avs_namespace("http://ddex.net/xml/avs"));
        assert_eq!(handler.get_avs_prefix("http://ddex.net/xml/avs"), Some("avs"));
        assert!(!handler.is_avs_namespace("http://ddex.net/xml/ern/43"));
    }

    #[test]
    fn test_canonical_namespace_manager() {
        let manager = CanonicalNamespaceManager::new();
        
        let mut declarations = IndexMap::new();
        declarations.insert("z_ern".to_string(), "http://ddex.net/xml/ern/43".to_string());
        declarations.insert("a_avs".to_string(), "http://ddex.net/xml/avs".to_string());
        
        let canonical = manager.canonicalize_namespaces(&declarations, "4.3");
        
        // Should be sorted alphabetically and use locked prefixes
        let keys: Vec<_> = canonical.keys().collect();
        assert!(keys.len() >= 2);
        // Should contain locked prefixes
        assert!(canonical.contains_key("ern"));
        assert!(canonical.contains_key("avs"));
    }
}