// core/src/bin/bench_report.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub system: SystemInfo,
    pub results: Vec<BenchmarkResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub cpu: String,
    pub memory_gb: f32,
    pub rust_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub file_size_bytes: u64,
    pub parse_time_ms: f64,
    pub memory_used_mb: f64,
    pub mode: String,
    pub percentiles: Percentiles,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Percentiles {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report = BenchmarkReport {
        timestamp: chrono::Utc::now(),
        system: SystemInfo {
            os: std::env::consts::OS.to_string(),
            cpu: "AWS m7g.large (2 vCPU)".to_string(),
            memory_gb: 8.0,
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
        },
        results: vec![],
    };
    
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::create_dir_all("benchmarks/results")?;
    std::fs::write("benchmarks/results/report.json", json)?;
    
    println!("Benchmark report generated at benchmarks/results/report.json");
    Ok(())
}