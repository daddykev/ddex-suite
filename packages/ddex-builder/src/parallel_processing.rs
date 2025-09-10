//! Parallel processing optimizations for DDEX Builder
//! 
//! This module provides parallel validation, resource processing, and XML generation
//! using rayon for CPU-bound operations to achieve sub-10ms build times.

use crate::ast::{Element, Node};
use crate::builder::{BuildRequest, ReleaseRequest, TrackRequest};
use crate::error::BuildError;
use crate::optimized_strings::BuildContext;
use crate::memory_optimization::BuildMemoryManager;
use indexmap::IndexMap;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Parallel build configuration
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Minimum items to trigger parallel processing
    pub parallel_threshold: usize,
    /// Maximum number of threads to use (None = use all available)
    pub max_threads: Option<usize>,
    /// Whether to use parallel validation
    pub parallel_validation: bool,
    /// Whether to use parallel XML generation
    pub parallel_xml_generation: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            parallel_threshold: 5, // Parallel processing for 5+ items
            max_threads: None,     // Use all available cores
            parallel_validation: true,
            parallel_xml_generation: true,
        }
    }
}

/// Parallel processor for DDEX builds
pub struct ParallelProcessor {
    config: ParallelConfig,
    thread_pool: rayon::ThreadPool,
}

impl ParallelProcessor {
    /// Create a new parallel processor
    pub fn new(config: ParallelConfig) -> Result<Self, BuildError> {
        let thread_pool = if let Some(max_threads) = config.max_threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build()
                .map_err(|e| BuildError::Parallel(e.to_string()))?
        } else {
            rayon::ThreadPoolBuilder::new()
                .build()
                .map_err(|e| BuildError::Parallel(e.to_string()))?
        };
        
        Ok(Self {
            config,
            thread_pool,
        })
    }
    
    /// Process a build request with parallel optimizations
    pub fn process_build_parallel(
        &self,
        request: &BuildRequest,
        context: &mut BuildContext,
        memory_manager: &BuildMemoryManager,
    ) -> Result<ParallelBuildResult, BuildError> {
        let start_time = Instant::now();
        
        // Determine if we should use parallel processing
        let total_tracks: usize = request.releases.iter()
            .map(|r| r.tracks.len())
            .sum();
        
        let use_parallel = total_tracks >= self.config.parallel_threshold;
        
        let result = if use_parallel {
            self.process_parallel_impl(request, context, memory_manager)?
        } else {
            self.process_sequential_impl(request, context, memory_manager)?
        };
        
        let processing_time = start_time.elapsed();
        
        Ok(ParallelBuildResult {
            elements: result,
            processing_time,
            used_parallel: use_parallel,
            thread_count: if use_parallel { 
                self.thread_pool.current_num_threads() 
            } else { 
                1 
            },
            total_tracks,
        })
    }
    
    /// Parallel implementation for large builds
    fn process_parallel_impl(
        &self,
        request: &BuildRequest,
        _context: &mut BuildContext,
        _memory_manager: &BuildMemoryManager,
    ) -> Result<Vec<ProcessedElement>, BuildError> {
        self.thread_pool.install(|| {
            // Process releases in parallel
            let processed_releases: Result<Vec<_>, BuildError> = request.releases
                .par_iter()
                .map(|release| self.process_release_parallel(release))
                .collect();
            
            let releases = processed_releases?;
            
            // Combine results
            Ok(releases.into_iter().flatten().collect())
        })
    }
    
    /// Sequential implementation for small builds
    fn process_sequential_impl(
        &self,
        request: &BuildRequest,
        _context: &mut BuildContext,
        _memory_manager: &BuildMemoryManager,
    ) -> Result<Vec<ProcessedElement>, BuildError> {
        let mut results = Vec::new();
        
        for release in &request.releases {
            let processed = self.process_release_sequential(release)?;
            results.extend(processed);
        }
        
        Ok(results)
    }
    
    /// Process a single release in parallel
    fn process_release_parallel(&self, release: &ReleaseRequest) -> Result<Vec<ProcessedElement>, BuildError> {
        // Process tracks in parallel if there are enough of them
        if release.tracks.len() >= self.config.parallel_threshold {
            let processed_tracks: Result<Vec<_>, BuildError> = release.tracks
                .par_iter()
                .map(|track| self.process_track(track))
                .collect();
            
            let tracks = processed_tracks?;
            
            // Create release element
            let release_element = ProcessedElement {
                name: "Release".to_string(),
                processing_time: std::time::Duration::from_nanos(1), // Minimal for structure
                element_count: 1 + tracks.len(),
            };
            
            let mut result = vec![release_element];
            result.extend(tracks);
            Ok(result)
        } else {
            self.process_release_sequential(release)
        }
    }
    
    /// Process a single release sequentially
    fn process_release_sequential(&self, release: &ReleaseRequest) -> Result<Vec<ProcessedElement>, BuildError> {
        let mut results = Vec::new();
        
        // Process release
        let release_element = ProcessedElement {
            name: "Release".to_string(),
            processing_time: std::time::Duration::from_nanos(1),
            element_count: 1,
        };
        results.push(release_element);
        
        // Process tracks
        for track in &release.tracks {
            results.push(self.process_track(track)?);
        }
        
        Ok(results)
    }
    
    /// Process a single track
    fn process_track(&self, track: &TrackRequest) -> Result<ProcessedElement, BuildError> {
        let start_time = Instant::now();
        
        // Simulate track processing work
        // In reality, this would do validation, resource linking, etc.
        let _validated = self.validate_track(track)?;
        
        let processing_time = start_time.elapsed();
        
        Ok(ProcessedElement {
            name: format!("Track_{}", track.track_id),
            processing_time,
            element_count: 1,
        })
    }
    
    /// Validate a track (can be called in parallel)
    fn validate_track(&self, track: &TrackRequest) -> Result<ValidatedTrack, BuildError> {
        // ISRC validation
        if track.isrc.len() != 12 {
            return Err(BuildError::Validation(format!(
                "Invalid ISRC length for track {}: expected 12 characters, got {}", 
                track.track_id, 
                track.isrc.len()
            )));
        }
        
        // Duration validation (basic ISO 8601 check)
        if !track.duration.starts_with("PT") {
            return Err(BuildError::Validation(format!(
                "Invalid duration format for track {}: must start with 'PT'", 
                track.track_id
            )));
        }
        
        // Title validation
        if track.title.trim().is_empty() {
            return Err(BuildError::Validation(format!(
                "Track title cannot be empty for track {}", 
                track.track_id
            )));
        }
        
        Ok(ValidatedTrack {
            track_id: track.track_id.clone(),
            isrc: track.isrc.clone(),
            title: track.title.clone(),
            duration: track.duration.clone(),
            artist: track.artist.clone(),
        })
    }
    
    /// Parallel XML section generation for large elements
    pub fn generate_xml_sections_parallel(
        &self,
        elements: &[Element],
        context: &Arc<Mutex<BuildContext>>,
    ) -> Result<Vec<String>, BuildError> {
        if elements.len() < self.config.parallel_threshold || !self.config.parallel_xml_generation {
            return self.generate_xml_sections_sequential(elements, context);
        }
        
        self.thread_pool.install(|| {
            elements.par_iter()
                .map(|element| {
                    // Each thread gets its own temporary context to avoid contention
                    let mut local_context = BuildContext::new();
                    
                    // Generate XML for this element
                    self.element_to_xml_string(element, &mut local_context)
                })
                .collect()
        })
    }
    
    /// Sequential XML section generation
    fn generate_xml_sections_sequential(
        &self,
        elements: &[Element],
        context: &Arc<Mutex<BuildContext>>,
    ) -> Result<Vec<String>, BuildError> {
        let mut results = Vec::with_capacity(elements.len());
        
        for element in elements {
            let mut context = context.lock().unwrap();
            let xml = self.element_to_xml_string(element, &mut context)?;
            results.push(xml);
        }
        
        Ok(results)
    }
    
    /// Convert element to XML string (simplified for example)
    fn element_to_xml_string(&self, element: &Element, context: &mut BuildContext) -> Result<String, BuildError> {
        // Get buffer from context
        let mut buffer = context.get_xml_buffer(256);
        
        buffer.push('<');
        buffer.push_str(&element.name);
        
        // Add attributes
        for (key, value) in &element.attributes {
            buffer.push_str(&format!(" {}=\"{}\"", key, value));
        }
        
        if element.children.is_empty() {
            buffer.push_str("/>");
        } else {
            buffer.push('>');
            
            // Handle children (simplified)
            for child in &element.children {
                match child {
                    Node::Text(text) => buffer.push_str(text),
                    Node::Element(child_element) => {
                        let child_xml = self.element_to_xml_string(child_element, context)?;
                        buffer.push_str(&child_xml);
                    }
                    Node::Comment(comment) => {
                        buffer.push_str(&format!("<!-- {} -->", comment));
                    }
                }
            }
            
            buffer.push_str(&format!("</{}>", element.name));
        }
        
        let result = buffer.clone();
        context.return_xml_buffer(buffer);
        
        Ok(result)
    }
    
    /// Parallel validation of multiple items
    pub fn validate_items_parallel<T, F>(
        &self,
        items: &[T],
        validator: F,
    ) -> Result<Vec<ValidationResult>, BuildError>
    where
        T: Send + Sync,
        F: Fn(&T) -> Result<(), BuildError> + Send + Sync,
    {
        if items.len() < self.config.parallel_threshold || !self.config.parallel_validation {
            return self.validate_items_sequential(items, validator);
        }
        
        let validation_results: Vec<ValidationResult> = self.thread_pool.install(|| {
            items.par_iter()
                .map(|item| {
                    let start_time = Instant::now();
                    let result = validator(item);
                    let processing_time = start_time.elapsed();
                    
                    ValidationResult {
                        success: result.is_ok(),
                        error: result.err(),
                        processing_time,
                    }
                })
                .collect()
        });
        
        // Check for errors
        for result in &validation_results {
            if !result.success {
                if let Some(ref err) = result.error {
                    return Err(err.clone());
                }
            }
        }
        
        Ok(validation_results)
    }
    
    /// Sequential validation fallback
    fn validate_items_sequential<T, F>(
        &self,
        items: &[T],
        validator: F,
    ) -> Result<Vec<ValidationResult>, BuildError>
    where
        F: Fn(&T) -> Result<(), BuildError>,
    {
        let mut results = Vec::with_capacity(items.len());
        
        for item in items {
            let start_time = Instant::now();
            let result = validator(item);
            let processing_time = start_time.elapsed();
            
            results.push(ValidationResult {
                success: result.is_ok(),
                error: result.err(),
                processing_time,
            });
        }
        
        Ok(results)
    }
}

/// Result of parallel build processing
#[derive(Debug)]
pub struct ParallelBuildResult {
    pub elements: Vec<ProcessedElement>,
    pub processing_time: std::time::Duration,
    pub used_parallel: bool,
    pub thread_count: usize,
    pub total_tracks: usize,
}

impl ParallelBuildResult {
    /// Check if build met performance targets
    pub fn meets_performance_target(&self) -> bool {
        match self.total_tracks {
            1 => self.processing_time.as_millis() < 5,      // Single track: <5ms
            2..=20 => self.processing_time.as_millis() < 10, // Album: <10ms
            _ => self.processing_time.as_millis() < 50,      // Large: <50ms
        }
    }
    
    /// Get performance summary
    pub fn performance_summary(&self) -> String {
        format!(
            "Processed {} tracks in {:.2}ms using {} thread(s) ({}parallel)",
            self.total_tracks,
            self.processing_time.as_millis(),
            self.thread_count,
            if self.used_parallel { "" } else { "non-" }
        )
    }
}

/// Processed element information
#[derive(Debug)]
pub struct ProcessedElement {
    pub name: String,
    pub processing_time: std::time::Duration,
    pub element_count: usize,
}

/// Validated track data
#[derive(Debug)]
struct ValidatedTrack {
    pub track_id: String,
    pub isrc: String,
    pub title: String,
    pub duration: String,
    pub artist: String,
}

/// Validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub success: bool,
    pub error: Option<BuildError>,
    pub processing_time: std::time::Duration,
}

/// Workload analyzer to determine optimal parallel strategy
pub struct WorkloadAnalyzer;

impl WorkloadAnalyzer {
    /// Analyze a build request and suggest parallel configuration
    pub fn analyze_workload(request: &BuildRequest) -> WorkloadAnalysis {
        let total_releases = request.releases.len();
        let total_tracks: usize = request.releases.iter()
            .map(|r| r.tracks.len())
            .sum();
        
        let max_tracks_per_release = request.releases.iter()
            .map(|r| r.tracks.len())
            .max()
            .unwrap_or(0);
        
        let complexity_score = Self::calculate_complexity_score(request);
        
        WorkloadAnalysis {
            total_releases,
            total_tracks,
            max_tracks_per_release,
            complexity_score,
            recommended_config: Self::recommend_config(total_tracks, complexity_score),
        }
    }
    
    /// Calculate complexity score for the build
    fn calculate_complexity_score(request: &BuildRequest) -> f32 {
        let mut score = 0.0;
        
        // Base score from track count
        let total_tracks: usize = request.releases.iter()
            .map(|r| r.tracks.len())
            .sum();
        score += total_tracks as f32 * 1.0;
        
        // Add complexity for multiple releases
        score += request.releases.len() as f32 * 0.5;
        
        // Add complexity for deals
        score += request.deals.len() as f32 * 2.0; // Deals are more complex
        
        // Add complexity for extensions
        if request.extensions.is_some() {
            score += 1.0;
        }
        
        score
    }
    
    /// Recommend parallel configuration based on workload
    fn recommend_config(total_tracks: usize, complexity_score: f32) -> ParallelConfig {
        let parallel_threshold = if complexity_score > 20.0 {
            3 // Lower threshold for complex builds
        } else if total_tracks > 50 {
            5 // Higher threshold for simple large builds
        } else {
            10 // Even higher threshold for small builds
        };
        
        let max_threads = if total_tracks > 100 {
            None // Use all available cores for very large builds
        } else if total_tracks > 20 {
            Some(num_cpus::get().min(4)) // Cap at 4 cores for medium builds
        } else {
            Some(2) // Only 2 cores for small builds
        };
        
        ParallelConfig {
            parallel_threshold,
            max_threads,
            parallel_validation: complexity_score > 10.0,
            parallel_xml_generation: total_tracks > 15,
        }
    }
}

/// Workload analysis result
#[derive(Debug)]
pub struct WorkloadAnalysis {
    pub total_releases: usize,
    pub total_tracks: usize,
    pub max_tracks_per_release: usize,
    pub complexity_score: f32,
    pub recommended_config: ParallelConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::{LocalizedStringRequest, PartyRequest, MessageHeaderRequest};
    
    #[test]
    fn test_parallel_processor_creation() {
        let config = ParallelConfig::default();
        let processor = ParallelProcessor::new(config);
        assert!(processor.is_ok());
    }
    
    #[test]
    fn test_workload_analysis() {
        let request = BuildRequest {
            header: MessageHeaderRequest {
                message_id: Some("TEST_001".to_string()),
                message_sender: PartyRequest {
                    party_name: vec![LocalizedStringRequest {
                        text: "Test Sender".to_string(),
                        language_code: None,
                    }],
                    party_id: Some("SENDER_001".to_string()),
                    party_reference: None,
                },
                message_recipient: PartyRequest {
                    party_name: vec![LocalizedStringRequest {
                        text: "Test Recipient".to_string(),
                        language_code: None,
                    }],
                    party_id: Some("RECIPIENT_001".to_string()),
                    party_reference: None,
                },
                message_control_type: None,
                message_created_date_time: None,
            },
            version: "4.3".to_string(),
            profile: None,
            releases: vec![],
            deals: vec![],
            extensions: None,
        };
        
        let analysis = WorkloadAnalyzer::analyze_workload(&request);
        assert_eq!(analysis.total_tracks, 0);
        assert_eq!(analysis.total_releases, 0);
    }
    
    #[test]
    fn test_track_validation() {
        let config = ParallelConfig::default();
        let processor = ParallelProcessor::new(config).unwrap();
        
        let valid_track = TrackRequest {
            track_id: "T001".to_string(),
            resource_reference: Some("A001".to_string()),
            isrc: "USRC17607839".to_string(), // 12 chars
            title: "Test Track".to_string(),
            duration: "PT3M30S".to_string(),
            artist: "Test Artist".to_string(),
        };
        
        let result = processor.validate_track(&valid_track);
        assert!(result.is_ok());
        
        let invalid_track = TrackRequest {
            track_id: "T002".to_string(),
            resource_reference: None,
            isrc: "INVALID".to_string(), // Too short
            title: "".to_string(), // Empty
            duration: "3:30".to_string(), // Wrong format
            artist: "Test Artist".to_string(),
        };
        
        let result = processor.validate_track(&invalid_track);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_performance_target_checking() {
        let result = ParallelBuildResult {
            elements: vec![],
            processing_time: std::time::Duration::from_millis(3),
            used_parallel: false,
            thread_count: 1,
            total_tracks: 1,
        };
        
        assert!(result.meets_performance_target()); // 3ms < 5ms target for single track
        
        let slow_result = ParallelBuildResult {
            elements: vec![],
            processing_time: std::time::Duration::from_millis(15),
            used_parallel: true,
            thread_count: 4,
            total_tracks: 12,
        };
        
        assert!(!slow_result.meets_performance_target()); // 15ms > 10ms target for album
    }
}