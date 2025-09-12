use ddex_parser::{DDEXParser, ParseOptions};
use ddex_builder::{DDEXBuilder, BuildOptions};
use ddex_core::ddex::message::ern43::NewReleaseMessage;
use std::fs;
use std::time::{Duration, Instant};
use std::path::Path;
use sysinfo::{System, SystemExt, ProcessExt, Pid};

#[derive(Debug, Clone)]
struct BenchmarkResult {
    file_name: String,
    file_size: u64,
    parse_time: Duration,
    build_time: Option<Duration>,
    memory_before: u64,
    memory_after: u64,
    memory_peak: u64,
    success: bool,
    error: Option<String>,
}

struct BenchmarkSuite {
    parser: DDEXParser,
    builder: DDEXBuilder,
    system: System,
}

impl BenchmarkSuite {
    fn new() -> Self {
        Self {
            parser: DDEXParser::new(),
            builder: DDEXBuilder::new(),
            system: System::new_all(),
        }
    }

    fn get_memory_usage(&mut self) -> u64 {
        self.system.refresh_processes();
        if let Some(process) = self.system.process(Pid::from(std::process::id() as usize)) {
            process.memory()
        } else {
            0
        }
    }

    fn benchmark_file(&mut self, file_path: &Path, enable_fidelity: bool) -> BenchmarkResult {
        let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
        let file_size = fs::metadata(file_path).unwrap().len();
        
        println!("📊 Benchmarking {} ({:.1}KB)...", 
                 file_name, file_size as f64 / 1024.0);

        let memory_before = self.get_memory_usage();
        let mut memory_peak = memory_before;

        // Parse benchmark
        let parse_start = Instant::now();
        let parse_result = if enable_fidelity {
            let options = ParseOptions {
                preserve_extensions: true,
                preserve_comments: true,
                preserve_original_structure: true,
                ..Default::default()
            };
            self.parser.parse_with_options(file_path, &options)
        } else {
            self.parser.parse(file_path)
        };
        let parse_time = parse_start.elapsed();

        // Check memory after parsing
        let memory_after_parse = self.get_memory_usage();
        memory_peak = memory_peak.max(memory_after_parse);

        let mut result = BenchmarkResult {
            file_name,
            file_size,
            parse_time,
            build_time: None,
            memory_before,
            memory_after: memory_after_parse,
            memory_peak,
            success: false,
            error: None,
        };

        match parse_result {
            Ok(message) => {
                result.success = true;
                
                // Build benchmark (if parsing succeeded)
                let build_start = Instant::now();
                let build_result = if enable_fidelity {
                    let options = BuildOptions {
                        canonicalize: true,
                        preserve_extensions: true,
                        validate: true,
                        ..Default::default()
                    };
                    self.builder.build_with_options(&message, &options)
                } else {
                    self.builder.build(&message)
                };
                let build_time = build_start.elapsed();
                result.build_time = Some(build_time);

                // Check final memory
                let memory_final = self.get_memory_usage();
                result.memory_after = memory_final;
                result.memory_peak = memory_peak.max(memory_final);

                match build_result {
                    Ok(_) => {
                        println!("  ✅ Parse: {:.2}ms, Build: {:.2}ms, Memory: {:.1}MB", 
                                 parse_time.as_secs_f64() * 1000.0,
                                 build_time.as_secs_f64() * 1000.0,
                                 memory_peak as f64 / (1024.0 * 1024.0));
                    }
                    Err(e) => {
                        result.error = Some(format!("Build failed: {}", e));
                        println!("  ⚠️  Parse: {:.2}ms, Build failed: {}", 
                                 parse_time.as_secs_f64() * 1000.0, e);
                    }
                }
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
        
        let file_content = fs::read_to_string(file_path).unwrap();
        
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
                match parser.parse_from_string(&file_content) {
                    Ok(_) => thread_start.elapsed(),
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
        println!("=" * 80);
        
        let mut successful_parses = 0;
        let mut successful_builds = 0;
        let mut total_parse_time = Duration::new(0, 0);
        let mut total_build_time = Duration::new(0, 0);
        
        println!("{:<15} {:>8} {:>12} {:>12} {:>12} {:>10}", 
                 "File", "Size", "Parse(ms)", "Build(ms)", "Memory(MB)", "Status");
        println!("-" * 80);
        
        for result in results {
            let size_str = if result.file_size > 1024 * 1024 {
                format!("{:.1}MB", result.file_size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.0}KB", result.file_size as f64 / 1024.0)
            };
            
            let parse_ms = result.parse_time.as_secs_f64() * 1000.0;
            let build_ms = result.build_time.map_or(0.0, |d| d.as_secs_f64() * 1000.0);
            let memory_mb = result.memory_peak as f64 / (1024.0 * 1024.0);
            
            let status = if result.success { "✅" } else { "❌" };
            
            println!("{:<15} {:>8} {:>12.2} {:>12.2} {:>12.1} {:>10}", 
                     result.file_name, size_str, parse_ms, build_ms, memory_mb, status);
            
            if result.success {
                successful_parses += 1;
                total_parse_time += result.parse_time;
                
                if let Some(build_time) = result.build_time {
                    successful_builds += 1;
                    total_build_time += build_time;
                }
            }
        }
        
        println!("-" * 80);
        println!("📈 PERFORMANCE SUMMARY:");
        println!("  Successful parses: {}/{}", successful_parses, results.len());
        println!("  Successful builds: {}/{}", successful_builds, results.len());
        
        if successful_parses > 0 {
            let avg_parse = total_parse_time.as_secs_f64() * 1000.0 / successful_parses as f64;
            println!("  Average parse time: {:.2}ms", avg_parse);
        }
        
        if successful_builds > 0 {
            let avg_build = total_build_time.as_secs_f64() * 1000.0 / successful_builds as f64;
            println!("  Average build time: {:.2}ms", avg_build);
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
    }
}

fn main() {
    println!("🚀 DDEX Suite Rust Performance Benchmark");
    println!("=" * 50);
    
    let mut suite = BenchmarkSuite::new();
    
    // Test files
    let test_files = [
        "test-data/1kb.xml",
        "test-data/5kb.xml", 
        "test-data/10kb.xml",
        "test-data/50kb.xml",
        "test-data/100kb.xml",
        "test-data/500kb.xml",
        "test-data/1mb.xml",
        "test-data/5mb.xml",
        "test-data/10mb.xml",
        "test-data/25mb.xml",
    ];
    
    let mut results = Vec::new();
    
    // Standard benchmarks (fidelity disabled)
    println!("\n📊 Testing with Standard Mode (fidelity disabled)...");
    for file_path in &test_files {
        let path = Path::new(file_path);
        if path.exists() {
            results.push(suite.benchmark_file(path, false));
        }
    }
    
    // Fidelity benchmarks
    println!("\n🔍 Testing with Perfect Fidelity Mode (fidelity enabled)...");
    for file_path in &test_files[..5] { // Only smaller files for fidelity mode
        let path = Path::new(file_path);
        if path.exists() {
            results.push(suite.benchmark_file(path, true));
        }
    }
    
    // Parallel processing test
    if let Some(medium_file) = test_files.iter().find(|f| Path::new(f).exists() && f.contains("1mb")) {
        let path = Path::new(medium_file);
        suite.run_parallel_benchmark(path, 4);
        suite.run_parallel_benchmark(path, 8);
    }
    
    // Generate report
    suite.generate_performance_report(&results);
    
    println!("\n🎉 Rust benchmark completed!");
}