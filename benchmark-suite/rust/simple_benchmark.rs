use std::fs;
use std::time::{Duration, Instant};
use std::path::Path;
use std::io::BufReader;
use sysinfo::{System, Pid};

// Import the actual DDEX Suite modules directly
use ddex_parser::DDEXParser;

#[derive(Debug, Clone)]
struct BenchmarkResult {
    file_name: String,
    file_size: u64,
    parse_time: Duration,
    memory_usage: u64,
    success: bool,
    error: Option<String>,
}

struct SimpleBenchmarkSuite {
    parser: DDEXParser,
    system: System,
}

impl SimpleBenchmarkSuite {
    fn new() -> Self {
        Self {
            parser: DDEXParser::new(),
            system: System::new_all(),
        }
    }

    fn get_memory_usage(&mut self) -> u64 {
        use sysinfo::ProcessesToUpdate;
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        if let Some(process) = self.system.process(Pid::from(std::process::id() as usize)) {
            process.memory()
        } else {
            0
        }
    }

    fn benchmark_file(&mut self, file_path: &Path) -> BenchmarkResult {
        let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
        let file_size = fs::metadata(file_path).unwrap().len();
        
        println!("📊 Benchmarking {} ({:.1}KB)...", 
                 file_name, file_size as f64 / 1024.0);

        let memory_before = self.get_memory_usage();

        // Parse benchmark using file reader
        let parse_start = Instant::now();
        let file = fs::File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let parse_result = self.parser.parse(reader);
        let parse_time = parse_start.elapsed();

        let memory_after = self.get_memory_usage();

        let mut result = BenchmarkResult {
            file_name,
            file_size,
            parse_time,
            memory_usage: memory_after.saturating_sub(memory_before),
            success: false,
            error: None,
        };

        match parse_result {
            Ok(_message) => {
                result.success = true;
                println!("  ✅ Parse: {:.2}ms, Memory: {:.1}MB", 
                         parse_time.as_secs_f64() * 1000.0,
                         result.memory_usage as f64 / (1024.0 * 1024.0));
            }
            Err(e) => {
                result.error = Some(format!("Parse failed: {}", e));
                println!("  ❌ Parse failed: {}", e);
            }
        }

        result
    }

    fn run_parallel_benchmark(&mut self, file_path: &Path, num_threads: usize) -> Vec<Duration> {
        use rayon::prelude::*;
        
        println!("🚀 Testing parallel processing with {} threads...", num_threads);
        
        // Configure rayon thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();

        let start = Instant::now();
        let results: Vec<_> = pool.install(|| {
            (0..num_threads).into_par_iter().map(|_| {
                let thread_start = Instant::now();
                let parser = DDEXParser::new();
                
                // Parse the file in each thread
                match fs::File::open(file_path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        match parser.parse(reader) {
                            Ok(_) => thread_start.elapsed(),
                            Err(_) => Duration::from_millis(0),
                        }
                    },
                    Err(_) => Duration::from_millis(0),
                }
            }).collect()
        });
        let total_time = start.elapsed();
        
        println!("  Total time: {:.2}ms, Average per thread: {:.2}ms", 
                 total_time.as_secs_f64() * 1000.0,
                 results.iter().map(|d| d.as_secs_f64() * 1000.0).sum::<f64>() / results.len() as f64);
        
        results
    }

    fn generate_performance_report(&self, results: &[BenchmarkResult]) {
        println!("\n🎯 RUST PERFORMANCE BENCHMARK REPORT");
        println!("{}", "=".repeat(80));
        
        let mut successful_parses = 0;
        let mut total_parse_time = Duration::new(0, 0);
        let mut total_memory = 0u64;
        
        println!("{:<15} {:>8} {:>12} {:>12} {:>10}", 
                 "File", "Size", "Parse(ms)", "Memory(MB)", "Status");
        println!("{}", "-".repeat(80));
        
        for result in results {
            let size_str = if result.file_size > 1024 * 1024 {
                format!("{:.1}MB", result.file_size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.0}KB", result.file_size as f64 / 1024.0)
            };
            
            let parse_ms = result.parse_time.as_secs_f64() * 1000.0;
            let memory_mb = result.memory_usage as f64 / (1024.0 * 1024.0);
            
            let status = if result.success { "✅" } else { "❌" };
            
            println!("{:<15} {:>8} {:>12.2} {:>12.1} {:>10}", 
                     result.file_name, size_str, parse_ms, memory_mb, status);
            
            if result.success {
                successful_parses += 1;
                total_parse_time += result.parse_time;
                total_memory += result.memory_usage;
            }
        }
        
        println!("{}", "-".repeat(80));
        println!("📈 PERFORMANCE SUMMARY:");
        println!("  Successful parses: {}/{}", successful_parses, results.len());
        
        if successful_parses > 0 {
            let avg_parse = total_parse_time.as_secs_f64() * 1000.0 / successful_parses as f64;
            let avg_memory = total_memory as f64 / (1024.0 * 1024.0) / successful_parses as f64;
            println!("  Average parse time: {:.2}ms", avg_parse);
            println!("  Average memory usage: {:.1}MB", avg_memory);
        }
        
        // Check targets
        let target_results: Vec<_> = results.iter()
            .filter(|r| r.file_size >= 10 * 1024 * 1024) // 10MB+ files
            .collect();
            
        if !target_results.is_empty() {
            let avg_10mb_parse = target_results.iter()
                .filter(|r| r.success)
                .map(|r| r.parse_time.as_secs_f64() * 1000.0)
                .sum::<f64>() / target_results.len() as f64;
                
            println!("  Average 10MB+ parse time: {:.2}ms (target: <50ms) {}", 
                     avg_10mb_parse,
                     if avg_10mb_parse < 50.0 { "✅" } else { "❌" });
        }

        // Performance throughput calculation
        let total_data: u64 = results.iter()
            .filter(|r| r.success)
            .map(|r| r.file_size)
            .sum();
        let total_time_sec = total_parse_time.as_secs_f64();
        
        if total_time_sec > 0.0 {
            let throughput_mb_s = (total_data as f64 / (1024.0 * 1024.0)) / total_time_sec;
            println!("  Overall throughput: {:.1} MB/s", throughput_mb_s);
        }
    }
}

fn main() {
    println!("🚀 DDEX Suite Rust Performance Benchmark (Simplified)");
    println!("{}", "=".repeat(60));
    
    let mut suite = SimpleBenchmarkSuite::new();
    
    // Test files
    let test_files = [
        "../test-data/1kb.xml",
        "../test-data/5kb.xml", 
        "../test-data/10kb.xml",
        "../test-data/50kb.xml",
        "../test-data/100kb.xml",
        "../test-data/500kb.xml",
        "../test-data/1mb.xml",
        "../test-data/5mb.xml",
        "../test-data/10mb.xml",
        "../test-data/25mb.xml",
    ];
    
    let mut results = Vec::new();
    
    // Standard benchmarks
    println!("\n📊 Testing parse performance...");
    for file_path in &test_files {
        let path = Path::new(file_path);
        if path.exists() {
            results.push(suite.benchmark_file(path));
        }
    }
    
    // Parallel processing test
    if let Some(medium_file) = test_files.iter().find(|f| Path::new(f).exists() && f.contains("1mb")) {
        let path = Path::new(medium_file);
        println!("\n🔄 Testing parallel processing capabilities...");
        suite.run_parallel_benchmark(path, 4);
        suite.run_parallel_benchmark(path, 8);
    }
    
    // Generate report
    suite.generate_performance_report(&results);
    
    println!("\n🎉 Rust benchmark completed!");
}