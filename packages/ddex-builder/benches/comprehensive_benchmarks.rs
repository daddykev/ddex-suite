//! Comprehensive Benchmarks for DDEX Builder
//! 
//! This module provides detailed performance benchmarks to track performance
//! regressions and optimize critical code paths.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use ddex_builder::Builder;
use std::time::Duration;

/// Benchmark building simple DDEX messages
fn bench_simple_builds(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_builds");
    
    // Test different ERN versions
    let versions = ["3.8.2", "4.2", "4.3"];
    
    for version in versions.iter() {
        group.bench_with_input(BenchmarkId::new("simple_release", version), version, |b, version| {
            let builder = Builder::new();
            let simple_request = create_simple_build_request(version);
            
            b.iter(|| {
                // This would use the actual builder API
                build_simple_release(&builder, &simple_request)
            });
        });
    }
    
    group.finish();
}

/// Benchmark building complex DDEX messages with many tracks
fn bench_complex_builds(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_builds");
    group.significance_level(0.1).sample_size(50); // Smaller sample for complex operations
    
    let track_counts = [10, 50, 100, 500, 1000];
    
    for &track_count in track_counts.iter() {
        group.throughput(Throughput::Elements(track_count as u64));
        group.bench_with_input(BenchmarkId::new("multi_track", track_count), &track_count, |b, &track_count| {
            let builder = Builder::new();
            let complex_request = create_complex_build_request(track_count);
            
            b.iter(|| {
                build_complex_release(&builder, &complex_request)
            });
        });
    }
    
    group.finish();
}

/// Benchmark canonicalization performance
fn bench_canonicalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("canonicalization");
    
    let xml_sizes = [
        ("1KB", create_xml_content(1024)),
        ("10KB", create_xml_content(10 * 1024)),
        ("100KB", create_xml_content(100 * 1024)),
        ("1MB", create_xml_content(1024 * 1024)),
    ];
    
    for (size_name, xml_content) in xml_sizes.iter() {
        group.throughput(Throughput::Bytes(xml_content.len() as u64));
        group.bench_with_input(BenchmarkId::new("canonicalize", size_name), xml_content, |b, xml| {
            b.iter(|| {
                canonicalize_xml(xml)
            });
        });
    }
    
    group.finish();
}

/// Benchmark deterministic ID generation
fn bench_id_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("id_generation");
    
    let content_sizes = [100, 1000, 10000, 100000];
    
    for &size in content_sizes.iter() {
        let content = "x".repeat(size);
        
        group.bench_with_input(BenchmarkId::new("generate_id", size), &content, |b, content| {
            b.iter(|| {
                generate_deterministic_id(content)
            });
        });
    }
    
    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    group.measurement_time(Duration::from_secs(10)); // Longer measurement for memory patterns
    
    group.bench_function("large_structure_allocation", |b| {
        b.iter(|| {
            allocate_large_ddex_structure()
        });
    });
    
    group.bench_function("many_small_allocations", |b| {
        b.iter(|| {
            allocate_many_small_structures()
        });
    });
    
    group.finish();
}

/// Benchmark concurrent building
fn bench_concurrent_builds(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_builds");
    group.measurement_time(Duration::from_secs(15)); // Longer for concurrent tests
    
    let concurrency_levels = [1, 2, 4, 8, 16];
    
    for &concurrency in concurrency_levels.iter() {
        group.bench_with_input(BenchmarkId::new("concurrent", concurrency), &concurrency, |b, &concurrency| {
            b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
                build_concurrently(concurrency).await
            });
        });
    }
    
    group.finish();
}

/// Benchmark large catalog processing
fn bench_large_catalog(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_catalog");
    group.measurement_time(Duration::from_secs(30)); // Extended time for large operations
    group.sample_size(10); // Fewer samples for large operations
    
    let catalog_sizes = [
        ("small_catalog", 100),   // 100 releases
        ("medium_catalog", 1000), // 1K releases  
        ("large_catalog", 5000),  // 5K releases (scaled down from 10K for benchmarking)
    ];
    
    for (name, size) in catalog_sizes.iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("process_catalog", name), size, |b, &size| {
            b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
                process_large_catalog(size).await
            });
        });
    }
    
    group.finish();
}

/// Benchmark regression tests - track performance over time
fn bench_regression_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_tracking");
    
    // Standard benchmark scenarios that should remain stable
    group.bench_function("baseline_simple_build", |b| {
        let builder = Builder::new();
        let request = create_baseline_request();
        
        b.iter(|| {
            build_baseline_release(&builder, &request)
        });
    });
    
    group.bench_function("baseline_complex_build", |b| {
        let builder = Builder::new();
        let request = create_baseline_complex_request();
        
        b.iter(|| {
            build_baseline_complex_release(&builder, &request)
        });
    });
    
    group.bench_function("baseline_canonicalization", |b| {
        let xml = create_baseline_xml();
        
        b.iter(|| {
            canonicalize_xml(&xml)
        });
    });
    
    group.finish();
}

// Helper functions for benchmarking

fn create_simple_build_request(version: &str) -> SimpleBuildRequest {
    SimpleBuildRequest {
        version: version.to_string(),
        track_count: 1,
        complexity: "simple".to_string(),
    }
}

fn create_complex_build_request(track_count: usize) -> ComplexBuildRequest {
    ComplexBuildRequest {
        track_count,
        complexity: "complex".to_string(),
    }
}

fn create_xml_content(size: usize) -> String {
    let base_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>BENCH_TEST</MessageId>
    <Description>{}</Description>
  </MessageHeader>
</ern:NewReleaseMessage>"#;
    
    let filler_content = "x".repeat(size.saturating_sub(300)); // Subtract base XML size
    base_xml.replace("{}", &filler_content)
}

fn build_simple_release(_builder: &Builder, _request: &SimpleBuildRequest) -> String {
    // Placeholder implementation
    format!("<?xml version=\"1.0\"?><SimpleRelease>{}</SimpleRelease>", _request.version)
}

fn build_complex_release(_builder: &Builder, request: &ComplexBuildRequest) -> String {
    // Placeholder implementation - simulate building with many tracks
    let mut xml = String::with_capacity(request.track_count * 200); // Pre-allocate
    xml.push_str("<?xml version=\"1.0\"?><ComplexRelease>");
    
    for i in 0..request.track_count {
        xml.push_str(&format!("<Track id=\"{}\">Track {}</Track>", i, i));
    }
    
    xml.push_str("</ComplexRelease>");
    xml
}

fn canonicalize_xml(xml: &str) -> String {
    // Placeholder canonicalization - in real implementation would use DB-C14N/1.0
    xml.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_deterministic_id(content: &str) -> String {
    // Placeholder ID generation
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("ID_{:016x}", hasher.finish())
}

fn allocate_large_ddex_structure() -> LargeStructure {
    LargeStructure {
        releases: (0..1000).map(|i| format!("Release {}", i)).collect(),
        resources: (0..2000).map(|i| format!("Resource {}", i)).collect(),
        metadata: (0..500).map(|i| (format!("key_{}", i), format!("value_{}", i))).collect(),
    }
}

fn allocate_many_small_structures() -> Vec<SmallStructure> {
    (0..10000).map(|i| SmallStructure {
        id: format!("id_{}", i),
        name: format!("name_{}", i),
    }).collect()
}

async fn build_concurrently(concurrency: usize) -> Vec<String> {
    let mut handles = Vec::new();
    
    for i in 0..concurrency {
        let handle = tokio::spawn(async move {
            let builder = Builder::new();
            let request = create_simple_build_request("4.3");
            build_simple_release(&builder, &request)
        });
        handles.push(handle);
    }
    
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    
    results
}

async fn process_large_catalog(size: usize) -> String {
    // Simulate processing a large catalog
    let mut catalog = String::with_capacity(size * 1000);
    catalog.push_str("<?xml version=\"1.0\"?><LargeCatalog>");
    
    for i in 0..size {
        catalog.push_str(&format!(
            "<Release id=\"{}\" title=\"Release {}\" tracks=\"5\"/>", 
            i, i
        ));
    }
    
    catalog.push_str("</LargeCatalog>");
    catalog
}

fn create_baseline_request() -> SimpleBuildRequest {
    SimpleBuildRequest {
        version: "4.3".to_string(),
        track_count: 1,
        complexity: "baseline".to_string(),
    }
}

fn create_baseline_complex_request() -> ComplexBuildRequest {
    ComplexBuildRequest {
        track_count: 100,
        complexity: "baseline_complex".to_string(),
    }
}

fn create_baseline_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>BASELINE_001</MessageId>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_BASELINE_001</ReleaseId>
      <Title>Baseline Test Release</Title>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#.to_string()
}

fn build_baseline_release(_builder: &Builder, _request: &SimpleBuildRequest) -> String {
    "<?xml version=\"1.0\"?><BaselineRelease>Standard</BaselineRelease>".to_string()
}

fn build_baseline_complex_release(_builder: &Builder, request: &ComplexBuildRequest) -> String {
    format!("<?xml version=\"1.0\"?><BaselineComplex tracks=\"{}\">Standard</BaselineComplex>", 
        request.track_count)
}

// Benchmark data structures
#[derive(Debug, Clone)]
struct SimpleBuildRequest {
    version: String,
    track_count: usize,
    complexity: String,
}

#[derive(Debug, Clone)]
struct ComplexBuildRequest {
    track_count: usize,
    complexity: String,
}

#[derive(Debug)]
struct LargeStructure {
    releases: Vec<String>,
    resources: Vec<String>,
    metadata: Vec<(String, String)>,
}

#[derive(Debug)]
struct SmallStructure {
    id: String,
    name: String,
}

// Criterion benchmark groups
criterion_group!(
    benches,
    bench_simple_builds,
    bench_complex_builds,
    bench_canonicalization,
    bench_id_generation,
    bench_memory_allocation,
    bench_concurrent_builds,
    bench_large_catalog,
    bench_regression_tracking
);

criterion_main!(benches);