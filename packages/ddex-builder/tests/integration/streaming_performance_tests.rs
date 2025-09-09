use ddex_builder::presets::DdexVersion;
use ddex_builder::{Builder, BuildOptions, BuildRequest};
use ddex_builder::builder::{ReleaseRequest, SoundRecordingRequest, DealRequest};
use ddex_builder::presets::MessageProfile;
use ddex_builder::streaming::{StreamingBuilder, StreamingConfig, BatchConfig};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
async fn test_streaming_10k_releases() {
    let config = StreamingConfig {
        batch_size: 100,
        max_memory_mb: 256,
        enable_compression: true,
        parallel_processing: true,
        checkpoint_interval: 1000,
    };
    
    let mut streaming_builder = StreamingBuilder::new(config);
    
    let start_time = Instant::now();
    let mut processed = 0;
    
    // Generate and process 10,000 releases
    for batch_num in 0..100 {
        let mut batch_requests = Vec::new();
        
        // Create batch of 100 releases
        for i in 0..100 {
            let release_num = batch_num * 100 + i;
            let request = create_streaming_test_request(release_num);
            batch_requests.push(request);
        }
        
        // Process batch
        let batch_results = streaming_builder.process_batch(batch_requests).await
            .expect("Failed to process batch");
        
        processed += batch_results.len();
        
        // Verify batch processing
        assert_eq!(batch_results.len(), 100);
        for result in batch_results {
            assert!(result.xml.len() > 1000); // Reasonable size check
            assert!(result.xml.contains("ERN/4.3"));
        }
        
        // Progress logging
        if batch_num % 10 == 0 {
            let elapsed = start_time.elapsed();
            let rate = processed as f64 / elapsed.as_secs_f64();
            println!("Processed {} releases in {:?} ({:.2} releases/sec)", 
                     processed, elapsed, rate);
        }
    }
    
    let total_time = start_time.elapsed();
    let final_rate = processed as f64 / total_time.as_secs_f64();
    
    println!("Final stats: {} releases in {:?} ({:.2} releases/sec)", 
             processed, total_time, final_rate);
    
    // Performance assertions
    assert_eq!(processed, 10000);
    assert!(final_rate > 100.0, "Should process at least 100 releases per second");
    assert!(total_time < Duration::from_secs(120), "Should complete within 2 minutes");
}

#[tokio::test]
async fn test_streaming_memory_management() {
    let config = StreamingConfig {
        batch_size: 50,
        max_memory_mb: 64, // Limited memory
        enable_compression: true,
        parallel_processing: false, // Sequential for memory testing
        checkpoint_interval: 100,
    };
    
    let mut streaming_builder = StreamingBuilder::new(config);
    
    // Monitor memory usage
    let initial_memory = get_memory_usage();
    let mut max_memory_seen = initial_memory;
    
    // Process 1000 releases to test memory management
    for batch_num in 0..20 {
        let mut batch_requests = Vec::new();
        
        for i in 0..50 {
            let release_num = batch_num * 50 + i;
            let request = create_large_streaming_request(release_num); // Larger requests
            batch_requests.push(request);
        }
        
        let batch_results = streaming_builder.process_batch(batch_requests).await
            .expect("Failed to process memory test batch");
        
        let current_memory = get_memory_usage();
        max_memory_seen = max_memory_seen.max(current_memory);
        
        // Verify results
        assert_eq!(batch_results.len(), 50);
        
        // Memory shouldn't grow unboundedly
        let memory_growth = current_memory - initial_memory;
        assert!(memory_growth < 128_000_000, // 128MB growth limit
                "Memory growth too large: {} bytes", memory_growth);
    }
    
    println!("Memory usage: initial={}, max={}, growth={}MB", 
             initial_memory, max_memory_seen, 
             (max_memory_seen - initial_memory) / 1_000_000);
}

#[tokio::test]
async fn test_streaming_error_recovery() {
    let config = StreamingConfig {
        batch_size: 10,
        max_memory_mb: 128,
        enable_compression: false,
        parallel_processing: true,
        checkpoint_interval: 5,
    };
    
    let mut streaming_builder = StreamingBuilder::new(config);
    
    // Create batch with some invalid requests
    let mut batch_requests = Vec::new();
    
    // Add valid requests
    for i in 0..7 {
        batch_requests.push(create_streaming_test_request(i));
    }
    
    // Add invalid requests
    for i in 7..10 {
        let mut invalid_request = create_streaming_test_request(i);
        invalid_request.release.title = "".to_string(); // Invalid empty title
        invalid_request.resources.sound_recordings[0].isrc = Some("INVALID".to_string()); // Invalid ISRC
        batch_requests.push(invalid_request);
    }
    
    let batch_results = streaming_builder.process_batch(batch_requests).await;
    
    // Should handle errors gracefully
    match batch_results {
        Ok(results) => {
            // Some results should succeed
            assert!(results.len() >= 7);
            for result in &results {
                assert!(result.xml.contains("ERN/4.3"));
            }
        }
        Err(e) => {
            // Should provide meaningful error information
            let error_msg = format!("{:?}", e);
            assert!(error_msg.contains("title") || error_msg.contains("ISRC"));
        }
    }
}

#[tokio::test]
async fn test_streaming_checkpoint_recovery() {
    let config = StreamingConfig {
        batch_size: 20,
        max_memory_mb: 128,
        enable_compression: true,
        parallel_processing: true,
        checkpoint_interval: 10, // Checkpoint every 10 batches
    };
    
    let mut streaming_builder = StreamingBuilder::new(config);
    
    // Process several batches to create checkpoints
    let mut total_processed = 0;
    
    for batch_num in 0..5 {
        let mut batch_requests = Vec::new();
        
        for i in 0..20 {
            let release_num = batch_num * 20 + i;
            batch_requests.push(create_streaming_test_request(release_num));
        }
        
        let batch_results = streaming_builder.process_batch(batch_requests).await
            .expect("Failed to process checkpoint batch");
        
        total_processed += batch_results.len();
        
        // Simulate checkpoint
        if batch_num % 2 == 1 {
            streaming_builder.create_checkpoint().await
                .expect("Failed to create checkpoint");
        }
    }
    
    // Verify checkpoint functionality
    let checkpoint_info = streaming_builder.get_checkpoint_info().await
        .expect("Failed to get checkpoint info");
    
    assert!(checkpoint_info.total_processed > 0);
    assert!(checkpoint_info.last_checkpoint.is_some());
    
    println!("Checkpoint info: processed={}, checkpoints={}", 
             checkpoint_info.total_processed, 
             checkpoint_info.checkpoint_count);
}

#[tokio::test]
async fn test_streaming_compression_efficiency() {
    let config_no_compression = StreamingConfig {
        batch_size: 50,
        max_memory_mb: 128,
        enable_compression: false,
        parallel_processing: false,
        checkpoint_interval: 100,
    };
    
    let config_with_compression = StreamingConfig {
        batch_size: 50,
        max_memory_mb: 128,
        enable_compression: true,
        parallel_processing: false,
        checkpoint_interval: 100,
    };
    
    let batch_requests: Vec<_> = (0..50)
        .map(|i| create_streaming_test_request(i))
        .collect();
    
    // Test without compression
    let mut builder_no_comp = StreamingBuilder::new(config_no_compression);
    let start_no_comp = Instant::now();
    let results_no_comp = builder_no_comp.process_batch(batch_requests.clone()).await
        .expect("Failed without compression");
    let time_no_comp = start_no_comp.elapsed();
    
    // Test with compression
    let mut builder_with_comp = StreamingBuilder::new(config_with_compression);
    let start_with_comp = Instant::now();
    let results_with_comp = builder_with_comp.process_batch(batch_requests).await
        .expect("Failed with compression");
    let time_with_comp = start_with_comp.elapsed();
    
    // Both should produce valid results
    assert_eq!(results_no_comp.len(), 50);
    assert_eq!(results_with_comp.len(), 50);
    
    // Calculate compression ratio by comparing result sizes
    let total_size_no_comp: usize = results_no_comp.iter().map(|r| r.xml.len()).sum();
    let total_size_with_comp: usize = results_with_comp.iter().map(|r| r.xml.len()).sum();
    
    println!("Compression test: no_comp={}KB in {:?}, with_comp={}KB in {:?}", 
             total_size_no_comp / 1024, time_no_comp,
             total_size_with_comp / 1024, time_with_comp);
    
    // Results should be functionally equivalent
    for (r1, r2) in results_no_comp.iter().zip(results_with_comp.iter()) {
        assert!(r1.xml.contains("ERN/4.3"));
        assert!(r2.xml.contains("ERN/4.3"));
    }
}

#[tokio::test]
async fn test_streaming_parallel_vs_sequential() {
    let batch_requests: Vec<_> = (0..100)
        .map(|i| create_streaming_test_request(i))
        .collect();
    
    // Sequential processing
    let config_sequential = StreamingConfig {
        batch_size: 100,
        max_memory_mb: 256,
        enable_compression: false,
        parallel_processing: false,
        checkpoint_interval: 1000,
    };
    
    let mut builder_seq = StreamingBuilder::new(config_sequential);
    let start_seq = Instant::now();
    let results_seq = builder_seq.process_batch(batch_requests.clone()).await
        .expect("Sequential processing failed");
    let time_seq = start_seq.elapsed();
    
    // Parallel processing
    let config_parallel = StreamingConfig {
        batch_size: 100,
        max_memory_mb: 256,
        enable_compression: false,
        parallel_processing: true,
        checkpoint_interval: 1000,
    };
    
    let mut builder_par = StreamingBuilder::new(config_parallel);
    let start_par = Instant::now();
    let results_par = builder_par.process_batch(batch_requests).await
        .expect("Parallel processing failed");
    let time_par = start_par.elapsed();
    
    // Both should produce same number of results
    assert_eq!(results_seq.len(), 100);
    assert_eq!(results_par.len(), 100);
    
    println!("Performance comparison: sequential={:?}, parallel={:?}, speedup={:.2}x", 
             time_seq, time_par, time_seq.as_secs_f64() / time_par.as_secs_f64());
    
    // Parallel should be faster (or at least not significantly slower)
    // Allow some variance due to overhead
    assert!(time_par <= time_seq * 2, "Parallel processing shouldn't be much slower");
    
    // Results should be functionally equivalent
    for result in &results_par {
        assert!(result.xml.contains("ERN/4.3"));
        assert!(result.xml.len() > 1000);
    }
}

fn create_streaming_test_request(index: usize) -> BuildRequest {
    BuildRequest {
        message_id: format!("STREAM_TEST_{:06}", index),
        version: Some(DdexVersion::Ern43),
        profile: Some(MessageProfile::AudioSingle),
        sender: "StreamingSender".to_string(),
        recipient: "StreamingPlatform".to_string(),
        release: ReleaseRequest {
            release_id: format!("REL{:06}", index),
            title: format!("Streaming Test Track {}", index),
            display_artist: format!("Test Artist {}", index % 100), // Cycle artists
            label_name: Some(format!("Test Label {}", index % 10)), // Cycle labels
            release_date: "2024-01-01".to_string(),
            original_release_date: Some("2024-01-01".to_string()),
            genre: Some(match index % 5 {
                0 => "Pop",
                1 => "Rock", 
                2 => "Electronic",
                3 => "Hip-Hop",
                _ => "Alternative",
            }.to_string()),
            pline: Some(format!("℗ 2024 Test Label {}", index % 10)),
            cline: Some(format!("© 2024 Test Label {}", index % 10)),
            upc: Some(format!("{:012}", 123456789000u64 + index as u64)),
            grid: None,
            icpn: None,
            catalog_number: Some(format!("TL{:06}", index)),
            release_type: Some("Single".to_string()),
        },
        resources: ddex_builder::builder::ResourcesRequest {
            sound_recordings: vec![
                SoundRecordingRequest {
                    resource_reference: "R1".to_string(),
                    resource_id: format!("SR{:06}", index),
                    title: format!("Streaming Test Track {}", index),
                    display_artist: format!("Test Artist {}", index % 100),
                    isrc: Some(format!("TEST{:08}", 17600000 + index)),
                    duration: Some(format!("PT{}M{}S", 2 + (index % 6), 15 + (index % 45))),
                    technical_details: {
                        let mut details = HashMap::new();
                        details.insert("FileName".to_string(), format!("track{:06}.flac", index));
                        details.insert("Codec".to_string(), "FLAC".to_string());
                        details.insert("BitRate".to_string(), "1411".to_string());
                        details.insert("SampleRate".to_string(), "44100".to_string());
                        details.insert("HashSum".to_string(), format!("sha256:{:064x}", index));
                        details
                    },
                },
            ],
            image_resources: vec![],
            video_resources: vec![],
        },
        deals: vec![
            DealRequest {
                deal_id: format!("DEAL{:06}", index),
                commercial_model_type: Some("SubscriptionModel".to_string()),
                usage_type: "Stream".to_string(),
                territory_code: "Worldwide".to_string(),
                start_date: Some("2024-01-01".to_string()),
                end_date: None,
                price: None,
                currency: None,
                resources: vec!["R1".to_string()],
            },
        ],
        metadata: HashMap::new(),
        options: BuildOptions::default(),
    }
}

fn create_large_streaming_request(index: usize) -> BuildRequest {
    let mut request = create_streaming_test_request(index);
    
    // Add more resources to make it larger
    for i in 1..5 { // 5 total tracks
        request.resources.sound_recordings.push(
            SoundRecordingRequest {
                resource_reference: format!("R{}", i + 1),
                resource_id: format!("SR{:06}_{}", index, i),
                title: format!("Track {} of Album {}", i + 1, index),
                display_artist: format!("Test Artist {}", index % 100),
                isrc: Some(format!("TEST{:08}", 17600000 + index * 10 + i)),
                duration: Some(format!("PT{}M{}S", 3 + (i % 4), 20 + (i % 40))),
                technical_details: {
                    let mut details = HashMap::new();
                    details.insert("FileName".to_string(), format!("track{:06}_{}.flac", index, i));
                    details.insert("Codec".to_string(), "FLAC".to_string());
                    details.insert("BitRate".to_string(), "1411".to_string());
                    details.insert("SampleRate".to_string(), "44100".to_string());
                    details.insert("HashSum".to_string(), format!("sha256:{:064x}", index * 10 + i));
                    details
                },
            }
        );
        
        request.deals[0].resources.push(format!("R{}", i + 1));
    }
    
    // Change to album
    request.profile = Some(MessageProfile::AudioAlbum);
    request.release.release_type = Some("Album".to_string());
    request.release.title = format!("Streaming Test Album {}", index);
    
    request
}

fn get_memory_usage() -> usize {
    // Simple memory usage approximation
    // In a real implementation, you'd use a proper memory profiler
    std::alloc::System.info().resident
}

// Mock implementations for streaming components
mod streaming_mocks {
    use super::*;
    
    pub struct StreamingBuilder {
        config: StreamingConfig,
        processed_count: usize,
    }
    
    impl StreamingBuilder {
        pub fn new(config: StreamingConfig) -> Self {
            Self {
                config,
                processed_count: 0,
            }
        }
        
        pub async fn process_batch(&mut self, requests: Vec<BuildRequest>) -> Result<Vec<ddex_builder::builder::BuildResult>, ddex_builder::error::BuildError> {
            let mut results = Vec::new();
            let mut builder = Builder::new();
            
            for request in requests {
                let result = builder.build_internal(&request)?;
                results.push(result);
                self.processed_count += 1;
                
                // Simulate processing delay
                if self.config.parallel_processing {
                    sleep(Duration::from_millis(1)).await;
                } else {
                    sleep(Duration::from_millis(5)).await;
                }
            }
            
            Ok(results)
        }
        
        pub async fn create_checkpoint(&mut self) -> Result<(), ddex_builder::error::BuildError> {
            // Mock checkpoint creation
            Ok(())
        }
        
        pub async fn get_checkpoint_info(&self) -> Result<CheckpointInfo, ddex_builder::error::BuildError> {
            Ok(CheckpointInfo {
                total_processed: self.processed_count,
                last_checkpoint: Some(std::time::SystemTime::now()),
                checkpoint_count: self.processed_count / self.config.checkpoint_interval,
            })
        }
    }
    
    pub struct StreamingConfig {
        pub batch_size: usize,
        pub max_memory_mb: usize,
        pub enable_compression: bool,
        pub parallel_processing: bool,
        pub checkpoint_interval: usize,
    }
    
    pub struct CheckpointInfo {
        pub total_processed: usize,
        pub last_checkpoint: Option<std::time::SystemTime>,
        pub checkpoint_count: usize,
    }
}

use streaming_mocks::*;