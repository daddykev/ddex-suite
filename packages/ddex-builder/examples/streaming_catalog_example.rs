//! Streaming Catalog Example
//! 
//! This example demonstrates how to use the streaming builder to process large catalogs
//! of releases efficiently with batching, memory management, and progress tracking.

use ddex_builder::{Builder, BuildRequest, BuildOptions};
use ddex_builder::presets::{DdexVersion, MessageProfile};
use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::time::sleep;

// Mock streaming components (in real implementation, these would be in the main library)
mod streaming {
    use super::*;
    use ddex_builder::error::BuildError;
    
    #[derive(Clone)]
    pub struct StreamingConfig {
        pub batch_size: usize,
        pub max_memory_mb: usize,
        pub enable_compression: bool,
        pub parallel_processing: bool,
        pub checkpoint_interval: usize,
    }
    
    pub struct StreamingBuilder {
        config: StreamingConfig,
        processed_count: usize,
        builder: Builder,
    }
    
    pub struct BatchResult {
        pub total_processed: usize,
        pub success_count: usize,
        pub error_count: usize,
        pub processing_time: Duration,
        pub memory_used_mb: f64,
    }
    
    pub struct ProgressInfo {
        pub processed: usize,
        pub total: usize,
        pub rate_per_sec: f64,
        pub estimated_time_remaining: Duration,
    }
    
    impl StreamingBuilder {
        pub fn new(config: StreamingConfig) -> Self {
            Self {
                config,
                processed_count: 0,
                builder: Builder::new(),
            }
        }
        
        pub async fn process_catalog_batch(&mut self, requests: Vec<CatalogEntry>) -> Result<BatchResult, BuildError> {
            let start_time = Instant::now();
            let batch_size = requests.len();
            let mut success_count = 0;
            let mut error_count = 0;
            
            for (index, entry) in requests.into_iter().enumerate() {
                match self.process_single_entry(entry).await {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        eprintln!("Error processing entry {}: {:?}", index, e);
                        error_count += 1;
                    }
                }
                
                // Simulate processing delay
                if self.config.parallel_processing {
                    sleep(Duration::from_millis(1)).await;
                } else {
                    sleep(Duration::from_millis(5)).await;
                }
            }
            
            self.processed_count += success_count;
            let processing_time = start_time.elapsed();
            
            Ok(BatchResult {
                total_processed: batch_size,
                success_count,
                error_count,
                processing_time,
                memory_used_mb: 64.0 + (batch_size as f64 * 0.1), // Mock memory usage
            })
        }
        
        async fn process_single_entry(&mut self, entry: CatalogEntry) -> Result<String, BuildError> {
            // Convert catalog entry to build request
            let build_request = self.create_build_request_from_entry(entry)?;
            
            // Process with builder (simplified for example)
            let result = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<NewReleaseMessage>\n  <MessageId>{}</MessageId>\n  <Title>{}</Title>\n</NewReleaseMessage>", 
                               build_request.catalog_id, build_request.title);
            
            Ok(result)
        }
        
        fn create_build_request_from_entry(&self, entry: CatalogEntry) -> Result<SimpleBuildRequest, BuildError> {
            Ok(SimpleBuildRequest {
                catalog_id: entry.catalog_id,
                title: entry.title,
                artist: entry.artist,
                label: entry.label,
                release_date: entry.release_date,
                tracks: entry.tracks,
            })
        }
        
        pub fn get_progress_info(&self, total_expected: usize) -> ProgressInfo {
            let rate = if self.processed_count > 0 {
                self.processed_count as f64 / 60.0 // Mock: assume 1 minute processing time
            } else {
                0.0
            };
            
            let remaining = total_expected - self.processed_count;
            let estimated_time = if rate > 0.0 {
                Duration::from_secs((remaining as f64 / rate) as u64)
            } else {
                Duration::from_secs(0)
            };
            
            ProgressInfo {
                processed: self.processed_count,
                total: total_expected,
                rate_per_sec: rate,
                estimated_time_remaining: estimated_time,
            }
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct CatalogEntry {
        pub catalog_id: String,
        pub title: String,
        pub artist: String,
        pub label: String,
        pub release_date: String,
        pub genre: String,
        pub tracks: Vec<TrackInfo>,
        pub upc: Option<String>,
        pub territories: Vec<String>,
    }
    
    #[derive(Debug, Clone)]
    pub struct TrackInfo {
        pub track_number: u32,
        pub title: String,
        pub artist: String,
        pub isrc: Option<String>,
        pub duration: String,
    }
    
    struct SimpleBuildRequest {
        catalog_id: String,
        title: String,
        artist: String,
        label: String,
        release_date: String,
        tracks: Vec<TrackInfo>,
    }
}

use streaming::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🏭 DDEX Builder - Streaming Catalog Example");
    println!("Processing large music catalog with streaming builder...\n");
    
    // Configure streaming builder for high-performance processing
    let streaming_config = StreamingConfig {
        batch_size: 100,
        max_memory_mb: 512,
        enable_compression: true,
        parallel_processing: true,
        checkpoint_interval: 500,
    };
    
    let mut streaming_builder = StreamingBuilder::new(streaming_config.clone());
    
    println!("⚙️  Streaming Configuration:");
    println!("  📦 Batch size: {} releases", streaming_config.batch_size);
    println!("  💾 Memory limit: {} MB", streaming_config.max_memory_mb);
    println!("  🗜️  Compression: {}", if streaming_config.enable_compression { "Enabled" } else { "Disabled" });
    println!("  ⚡ Parallel processing: {}", if streaming_config.parallel_processing { "Enabled" } else { "Disabled" });
    println!("  ✅ Checkpoint interval: {} releases", streaming_config.checkpoint_interval);
    
    // Generate sample catalog data
    println!("\n📚 Generating sample catalog...");
    let total_catalog_size = 2500; // Process 2,500 releases
    let catalog_entries = generate_sample_catalog(total_catalog_size);
    
    println!("📊 Catalog generated: {} releases", catalog_entries.len());
    print_catalog_statistics(&catalog_entries);
    
    // Process catalog in batches
    println!("\n🚀 Starting streaming processing...");
    let start_time = Instant::now();
    let mut total_processed = 0;
    let mut total_errors = 0;
    let mut batch_number = 0;
    
    // Process in batches
    for batch in catalog_entries.chunks(streaming_config.batch_size) {
        batch_number += 1;
        
        println!("\n📦 Processing batch {} ({} releases)...", batch_number, batch.len());
        let batch_start = Instant::now();
        
        let batch_result = streaming_builder
            .process_catalog_batch(batch.to_vec())
            .await?;
        
        total_processed += batch_result.success_count;
        total_errors += batch_result.error_count;
        
        // Print batch results
        print_batch_results(batch_number, &batch_result);
        
        // Print progress
        let progress = streaming_builder.get_progress_info(total_catalog_size);
        print_progress_info(&progress);
        
        // Simulate checkpoint creation every 5 batches
        if batch_number % 5 == 0 {
            println!("💾 Creating checkpoint...");
            sleep(Duration::from_millis(100)).await; // Simulate checkpoint time
        }
    }
    
    let total_time = start_time.elapsed();
    
    // Print final results
    println!("\n🎉 Catalog processing completed!");
    print_final_summary(total_processed, total_errors, total_time, total_catalog_size);
    
    // Demonstrate catalog analytics
    println!("\n📈 Catalog Analytics:");
    analyze_processing_performance(total_processed, total_time);
    
    Ok(())
}

fn generate_sample_catalog(size: usize) -> Vec<CatalogEntry> {
    let genres = vec!["Pop", "Rock", "Electronic", "Hip-Hop", "Jazz", "Classical", "Country", "R&B", "Indie", "Alternative"];
    let labels = vec!["Universal Music", "Sony Music", "Warner Music", "Independent Records", "Indie Label", "Digital Records"];
    let territories = vec!["US", "GB", "CA", "AU", "DE", "FR", "JP", "BR"];
    
    (0..size)
        .map(|i| {
            let track_count = 3 + (i % 8); // 3-10 tracks per release
            let tracks = (1..=track_count)
                .map(|track_num| TrackInfo {
                    track_number: track_num as u32,
                    title: format!("Track {} of Release {}", track_num, i + 1),
                    artist: format!("Artist {}", (i % 100) + 1),
                    isrc: Some(format!("STRM{:08}", 20240000 + i * 10 + track_num)),
                    duration: format!("PT{}M{}S", 2 + (track_num % 4), 15 + ((i + track_num) % 45)),
                })
                .collect();
            
            CatalogEntry {
                catalog_id: format!("CAT{:06}", i + 1),
                title: format!("Streaming Release {}", i + 1),
                artist: format!("Artist {}", (i % 100) + 1),
                label: labels[i % labels.len()].to_string(),
                release_date: format!("2024-{:02}-{:02}", 1 + (i % 12), 1 + (i % 28)),
                genre: genres[i % genres.len()].to_string(),
                tracks,
                upc: Some(format!("{:012}", 123456000000u64 + i as u64)),
                territories: territories.iter().take(1 + (i % 4)).map(|s| s.to_string()).collect(),
            }
        })
        .collect()
}

fn print_catalog_statistics(catalog: &[CatalogEntry]) {
    let total_tracks: usize = catalog.iter().map(|e| e.tracks.len()).sum();
    let genres: std::collections::HashSet<_> = catalog.iter().map(|e| &e.genre).collect();
    let labels: std::collections::HashSet<_> = catalog.iter().map(|e| &e.label).collect();
    let artists: std::collections::HashSet<_> = catalog.iter().map(|e| &e.artist).collect();
    
    println!("  📊 Statistics:");
    println!("    🎵 Total tracks: {}", total_tracks);
    println!("    🎭 Unique genres: {}", genres.len());
    println!("    🏷️  Unique labels: {}", labels.len());
    println!("    🎤 Unique artists: {}", artists.len());
    println!("    ⚖️  Avg tracks per release: {:.1}", total_tracks as f64 / catalog.len() as f64);
}

fn print_batch_results(batch_number: usize, result: &BatchResult) {
    println!("  ✅ Batch {} completed:", batch_number);
    println!("    📊 Processed: {} releases", result.total_processed);
    println!("    ✅ Successful: {}", result.success_count);
    println!("    ❌ Errors: {}", result.error_count);
    println!("    ⏱️  Time: {:?}", result.processing_time);
    println!("    💾 Memory: {:.1} MB", result.memory_used_mb);
    
    let rate = result.success_count as f64 / result.processing_time.as_secs_f64();
    println!("    🚀 Rate: {:.1} releases/sec", rate);
    
    if result.error_count > 0 {
        let error_rate = (result.error_count as f64 / result.total_processed as f64) * 100.0;
        println!("    ⚠️  Error rate: {:.1}%", error_rate);
    }
}

fn print_progress_info(progress: &ProgressInfo) {
    let completion = (progress.processed as f64 / progress.total as f64) * 100.0;
    println!("  📈 Progress: {}/{} ({:.1}%)", progress.processed, progress.total, completion);
    println!("  🚀 Rate: {:.1} releases/sec", progress.rate_per_sec);
    if progress.estimated_time_remaining > Duration::from_secs(0) {
        println!("  ⏰ Est. remaining: {:?}", progress.estimated_time_remaining);
    }
    
    // Progress bar
    let progress_width = 40;
    let filled = ((completion / 100.0) * progress_width as f64) as usize;
    let empty = progress_width - filled;
    println!("  [{}{}] {:.1}%", 
             "█".repeat(filled), 
             "░".repeat(empty), 
             completion);
}

fn print_final_summary(processed: usize, errors: usize, total_time: Duration, catalog_size: usize) {
    println!("📊 Final Results:");
    println!("  ✅ Successfully processed: {} releases", processed);
    println!("  ❌ Errors encountered: {}", errors);
    println!("  ⏱️  Total processing time: {:?}", total_time);
    println!("  🚀 Average rate: {:.1} releases/sec", processed as f64 / total_time.as_secs_f64());
    
    let success_rate = (processed as f64 / catalog_size as f64) * 100.0;
    println!("  📈 Success rate: {:.1}%", success_rate);
    
    if processed > 0 {
        let avg_time_per_release = total_time.as_millis() as f64 / processed as f64;
        println!("  ⚡ Avg time per release: {:.1}ms", avg_time_per_release);
    }
}

fn analyze_processing_performance(processed: usize, total_time: Duration) {
    let rate = processed as f64 / total_time.as_secs_f64();
    
    println!("🔍 Performance Analysis:");
    
    // Performance classification
    if rate > 100.0 {
        println!("  🚀 Performance: Excellent ({}+ releases/sec)", rate as u32);
        println!("  💡 Suitable for: Large-scale catalog processing");
    } else if rate > 50.0 {
        println!("  ✅ Performance: Good ({:.0} releases/sec)", rate);
        println!("  💡 Suitable for: Medium catalog processing");
    } else if rate > 20.0 {
        println!("  ⚠️  Performance: Acceptable ({:.0} releases/sec)", rate);
        println!("  💡 Suitable for: Small catalog processing");
    } else {
        println!("  🐌 Performance: Needs optimization ({:.0} releases/sec)", rate);
        println!("  💡 Consider: Reducing batch size or enabling compression");
    }
    
    // Throughput projections
    println!("  📊 Throughput Projections:");
    println!("    • 1,000 releases: ~{:.0} seconds", 1000.0 / rate);
    println!("    • 10,000 releases: ~{:.1} minutes", (10000.0 / rate) / 60.0);
    println!("    • 100,000 releases: ~{:.1} hours", (100000.0 / rate) / 3600.0);
    
    // Memory efficiency (mock calculation)
    let estimated_memory_per_release = 0.5; // MB
    let total_memory_estimate = processed as f64 * estimated_memory_per_release;
    println!("  💾 Memory Efficiency:");
    println!("    • Estimated memory per release: {:.1} MB", estimated_memory_per_release);
    println!("    • Total memory processed: {:.1} MB", total_memory_estimate);
    
    // Recommendations
    println!("  💡 Optimization Recommendations:");
    if rate < 50.0 {
        println!("    • Enable parallel processing");
        println!("    • Increase batch size to 200-500");
        println!("    • Consider compression for large catalogs");
    } else {
        println!("    • Current configuration is well-optimized");
        println!("    • Consider scaling horizontally for larger catalogs");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_catalog_generation() {
        let catalog = generate_sample_catalog(100);
        
        assert_eq!(catalog.len(), 100);
        assert!(!catalog[0].title.is_empty());
        assert!(!catalog[0].artist.is_empty());
        assert!(!catalog[0].tracks.is_empty());
    }
    
    #[tokio::test]
    async fn test_streaming_builder() {
        let config = StreamingConfig {
            batch_size: 10,
            max_memory_mb: 64,
            enable_compression: false,
            parallel_processing: false,
            checkpoint_interval: 50,
        };
        
        let mut builder = StreamingBuilder::new(config);
        let catalog = generate_sample_catalog(10);
        
        let result = builder.process_catalog_batch(catalog).await.unwrap();
        
        assert_eq!(result.total_processed, 10);
        assert!(result.success_count <= 10);
        assert!(result.processing_time > Duration::from_millis(0));
    }
    
    #[test]
    fn test_progress_tracking() {
        let config = StreamingConfig {
            batch_size: 50,
            max_memory_mb: 128,
            enable_compression: true,
            parallel_processing: true,
            checkpoint_interval: 100,
        };
        
        let builder = StreamingBuilder::new(config);
        let progress = builder.get_progress_info(1000);
        
        assert_eq!(progress.total, 1000);
        assert!(progress.rate_per_sec >= 0.0);
    }
}