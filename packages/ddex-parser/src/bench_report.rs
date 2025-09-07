// core/src/bench_report.rs
//! Benchmark report generation

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

impl BenchmarkReport {
    pub fn generate() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            system: SystemInfo {
                os: std::env::consts::OS.to_string(),
                cpu: "AWS m7g.large (2 vCPU)".to_string(),
                memory_gb: 8.0,
                rust_version: env!("CARGO_PKG_VERSION").to_string(),
            },
            results: vec![],
        }
    }
    
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}