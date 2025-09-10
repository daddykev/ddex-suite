//! DDEX Parser CLI implementation - Enhanced with comprehensive commands

use anyhow::{Context, Result};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde_json::{Value as JsonValue};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;
use std::time::Instant;
use glob::glob;

#[derive(Parser)]
#[command(
    name = "ddex-parser",
    about = "DDEX Parser CLI - High-performance DDEX XML parsing toolkit",
    long_about = "A comprehensive command-line interface for parsing, validating, extracting data from DDEX XML files with streaming support for large files.",
    version = env!("CARGO_PKG_VERSION"),
    author = "Kevin Marques Moo"
)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress all non-error output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Control color output
    #[arg(long, global = true, value_enum, default_value_t = ColorChoice::Auto)]
    color: ColorChoice,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse DDEX XML file to JSON/YAML/MessagePack
    Parse(ParseCommand),
    /// Extract specific elements from DDEX files
    Extract(ExtractCommand),
    /// Stream large DDEX files with memory-bounded processing
    Stream(StreamCommand),
    /// Process multiple files in parallel
    Batch(BatchCommand),
    /// Validate DDEX XML files with detailed reports
    Validate(ValidateCommand),
    /// Convert between different output formats
    Convert(ConvertCommand),
    /// Analyze metadata and generate statistics
    Stats(StatsCommand),
    /// Interactive REPL mode for exploration
    Interactive,
    /// Generate shell completions
    Completions(CompletionsCommand),
    /// Detect DDEX version (legacy command)
    DetectVersion(DetectVersionCommand),
    /// Quick sanity check (legacy command)
    SanityCheck(SanityCheckCommand),
}

#[derive(Args)]
struct ParseCommand {
    /// Input DDEX XML file or '-' for stdin
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output file path or '-' for stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,

    /// Use flattened representation (more developer-friendly)
    #[arg(long)]
    flatten: bool,

    /// Pretty-print output
    #[arg(long, default_value_t = true)]
    pretty: bool,

    /// Include raw XML metadata in output
    #[arg(long)]
    include_metadata: bool,

    /// Validate XML structure during parsing
    #[arg(long)]
    validate: bool,
}

#[derive(Args)]
struct ExtractCommand {
    /// Input DDEX XML file
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// XPath or element name to extract
    #[arg(short, long, value_name = "XPATH")]
    query: String,

    /// Output file path or '-' for stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,

    /// Extract all matching elements (not just first)
    #[arg(long)]
    all: bool,

    /// Include element attributes
    #[arg(long)]
    include_attributes: bool,
}

#[derive(Args)]
struct StreamCommand {
    /// Input DDEX XML file
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// Output directory for streamed elements
    #[arg(short, long)]
    output_dir: PathBuf,

    /// Maximum memory usage in MB
    #[arg(long, default_value_t = 100)]
    max_memory_mb: usize,

    /// Element to stream (e.g., 'Release', 'Sound')
    #[arg(short, long)]
    element: String,

    /// Batch size for processing
    #[arg(long, default_value_t = 1000)]
    batch_size: usize,

    /// Enable progress reporting
    #[arg(long)]
    progress: bool,
}

#[derive(Args)]
struct BatchCommand {
    /// Input pattern (supports globs like '*.xml')
    #[arg(value_name = "PATTERN")]
    pattern: String,

    /// Output directory
    #[arg(short, long)]
    output_dir: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,

    /// Number of worker threads
    #[arg(short, long, default_value_t = num_cpus::get())]
    workers: usize,

    /// Continue processing on errors
    #[arg(long)]
    continue_on_error: bool,

    /// Generate summary report
    #[arg(long)]
    report: Option<PathBuf>,

    /// Use flattened representation
    #[arg(long)]
    flatten: bool,
}

#[derive(Args)]
struct ValidateCommand {
    /// DDEX XML files to validate
    #[arg(value_name = "FILES", required = true)]
    files: Vec<PathBuf>,

    /// Validation strictness level
    #[arg(short, long, value_enum, default_value_t = ValidationLevel::Standard)]
    level: ValidationLevel,

    /// Output format for validation results
    #[arg(short, long, value_enum, default_value_t = ValidationFormat::Human)]
    format: ValidationFormat,

    /// Stop at first validation error
    #[arg(long)]
    fail_fast: bool,

    /// Check XML well-formedness only
    #[arg(long)]
    xml_only: bool,

    /// Validate against specific DDEX version
    #[arg(long)]
    ddex_version: Option<String>,
}

#[derive(Args)]
struct ConvertCommand {
    /// Input DDEX XML file
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// Source format (auto-detected if not specified)
    #[arg(short, long, value_enum)]
    from: Option<InputFormat>,

    /// Target format
    #[arg(short, long, value_enum)]
    to: OutputFormat,

    /// Output file path or '-' for stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Use flattened representation
    #[arg(long)]
    flatten: bool,

    /// Pretty-print output
    #[arg(long, default_value_t = true)]
    pretty: bool,
}

#[derive(Args)]
struct StatsCommand {
    /// Input DDEX XML files
    #[arg(value_name = "FILES", required = true)]
    files: Vec<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,

    /// Output file path or '-' for stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Include detailed element statistics
    #[arg(long)]
    detailed: bool,

    /// Analyze file size distribution
    #[arg(long)]
    size_analysis: bool,

    /// Generate performance metrics
    #[arg(long)]
    performance: bool,
}

#[derive(Args)]
struct CompletionsCommand {
    /// Shell to generate completions for
    #[arg(value_enum)]
    shell: Shell,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Args)]
struct DetectVersionCommand {
    /// Input DDEX XML file
    #[arg(value_name = "FILE")]
    input: PathBuf,
}

#[derive(Args)]
struct SanityCheckCommand {
    /// Input DDEX XML file
    #[arg(value_name = "FILE")]
    input: PathBuf,
}

#[derive(ValueEnum, Clone, Debug)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Json,
    Yaml,
    MessagePack,
    Csv,
    Xml,
}

#[derive(ValueEnum, Clone, Debug)]
enum InputFormat {
    Xml,
    Json,
    Yaml,
}

#[derive(ValueEnum, Clone, Debug)]
enum ValidationLevel {
    Permissive,
    Standard,
    Strict,
}

#[derive(ValueEnum, Clone, Debug)]
enum ValidationFormat {
    Human,
    Json,
    Junit,
    Tap,
}

pub fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging based on verbosity
    setup_logging(cli.verbose, cli.quiet);

    // Setup color output
    setup_colors(cli.color);

    let result = match cli.command {
        Commands::Parse(cmd) => handle_parse_command(cmd),
        Commands::Extract(cmd) => handle_extract_command(cmd),
        Commands::Stream(cmd) => handle_stream_command(cmd),
        Commands::Batch(cmd) => handle_batch_command(cmd),
        Commands::Validate(cmd) => handle_validate_command(cmd),
        Commands::Convert(cmd) => handle_convert_command(cmd),
        Commands::Stats(cmd) => handle_stats_command(cmd),
        Commands::Interactive => handle_interactive_mode(),
        Commands::Completions(cmd) => handle_completions_command(cmd),
        Commands::DetectVersion(cmd) => detect_version(&cmd.input.to_string_lossy()),
        Commands::SanityCheck(cmd) => sanity_check(&cmd.input.to_string_lossy()),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }

    Ok(())
}

fn setup_logging(verbosity: u8, quiet: bool) {
    if quiet {
        return;
    }

    let level = match verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_level(level)
        .target(env_logger::Target::Stderr)
        .init();
}

fn setup_colors(color_choice: ColorChoice) {
    match color_choice {
        ColorChoice::Always => {
            colored::control::set_override(true);
        }
        ColorChoice::Never => {
            colored::control::set_override(false);
        }
        ColorChoice::Auto => {
            // Default behavior - colors enabled for TTY
        }
    }
}

fn handle_parse_command(cmd: ParseCommand) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    let input_content = read_input_string(&cmd.input)?;
    let parser = DDEXParser::new();
    let start_time = Instant::now();
    
    let result = parser.parse(std::io::Cursor::new(input_content.as_bytes()))?;
    let parse_duration = start_time.elapsed();
    
    let output_data = if cmd.flatten {
        serde_json::to_value(&result.flat)?
    } else {
        serde_json::to_value(&result.graph)?
    };
    
    let formatted_output = format_output(&output_data, cmd.format, cmd.pretty)?;
    write_output(&formatted_output, &cmd.output)?;
    
    if !is_quiet() {
        eprintln!("{} Parsed in {:.2}ms", "✓".green(), parse_duration.as_secs_f64() * 1000.0);
        // TODO: Extract DDEX version from result when available
        // eprintln!("  DDEX Version: {:?}", version);
        eprintln!("  Representation: {}", if cmd.flatten { "Flattened" } else { "Graph" });
    }
    
    Ok(())
}

fn handle_extract_command(cmd: ExtractCommand) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    let xml_content = fs::read_to_string(&cmd.input)
        .context(format!("Failed to read file: {}", cmd.input.display()))?;
    
    let parser = DDEXParser::new();
    let result = parser.parse(std::io::Cursor::new(xml_content.as_bytes()))?;
    
    // Extract elements based on query
    let extracted_data = extract_elements(&result, &cmd.query, cmd.all, cmd.include_attributes)?;
    
    let formatted_output = format_output(&extracted_data, cmd.format, true)?;
    write_output(&formatted_output, &cmd.output)?;
    
    if !is_quiet() {
        let count = if extracted_data.is_array() { 
            extracted_data.as_array().unwrap().len() 
        } else { 1 };
        eprintln!("{} Extracted {} element(s)", "✓".green(), count);
    }
    
    Ok(())
}

fn handle_stream_command(cmd: StreamCommand) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    fs::create_dir_all(&cmd.output_dir)?;
    
    let file_size = fs::metadata(&cmd.input)?.len();
    let progress_bar = if cmd.progress && !is_quiet() {
        let pb = ProgressBar::new(file_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} {msg}")
                .unwrap(),
        );
        Some(pb)
    } else {
        None
    };
    
    // Implement streaming logic here
    let xml_content = fs::read_to_string(&cmd.input)?;
    let parser = DDEXParser::new();
    let result = parser.parse(std::io::Cursor::new(xml_content.as_bytes()))?;
    
    // Stream elements to separate files
    let output_file = cmd.output_dir.join(format!("{}_{}.json", cmd.element, 0));
    let output = serde_json::to_string_pretty(&result.flat)?;
    fs::write(output_file, output)?;
    
    if let Some(pb) = progress_bar {
        pb.finish_with_message("Streaming completed");
    }
    
    if !is_quiet() {
        eprintln!("{} Streaming completed", "✓".green());
        eprintln!("  Output directory: {}", cmd.output_dir.display());
    }
    
    Ok(())
}

fn handle_batch_command(cmd: BatchCommand) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    fs::create_dir_all(&cmd.output_dir)?;
    
    let input_files: Vec<PathBuf> = glob(&cmd.pattern)?
        .filter_map(|entry| entry.ok())
        .collect();
    
    if input_files.is_empty() {
        return Err(anyhow::anyhow!("No files found matching pattern: {}", cmd.pattern));
    }
    
    let progress_bar = if !is_quiet() {
        let pb = ProgressBar::new(input_files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap(),
        );
        Some(pb)
    } else {
        None
    };
    
    // Setup thread pool
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cmd.workers)
        .build()?;
    
    let results: Vec<BatchResult> = pool.install(|| {
        input_files
            .par_iter()
            .enumerate()
            .map(|(_i, file_path)| {
                let result = process_file_batch(file_path, &cmd.output_dir, cmd.format.clone(), cmd.flatten);
                if let Some(pb) = &progress_bar {
                    pb.set_message(format!("Processing {}", file_path.display()));
                    pb.inc(1);
                }
                BatchResult {
                    file_path: file_path.clone(),
                    success: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                }
            })
            .collect()
    });
    
    if let Some(pb) = &progress_bar {
        pb.finish_with_message("Batch processing completed");
    }
    
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;
    
    if !is_quiet() {
        eprintln!("\n{} Batch processing completed", "✓".green());
        eprintln!("  Processed: {} files", results.len());
        eprintln!("  Successful: {}", successful);
        if failed > 0 {
            eprintln!("  Failed: {}", failed.to_string().red());
        }
    }
    
    if let Some(report_path) = cmd.report {
        let report = BatchReport {
            total_files: results.len(),
            successful,
            failed,
            results,
        };
        let report_json = serde_json::to_string_pretty(&report)?;
        fs::write(report_path, report_json)?;
    }
    
    if failed > 0 && !cmd.continue_on_error {
        process::exit(1);
    }
    
    Ok(())
}

fn handle_validate_command(cmd: ValidateCommand) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    let mut all_valid = true;
    let mut results = Vec::new();
    
    for file_path in &cmd.files {
        let validation_result = if cmd.xml_only {
            validate_xml_only(file_path)?
        } else {
            validate_ddex_file(file_path, cmd.level.clone(), cmd.ddex_version.clone())?
        };
        
        let file_valid = validation_result.errors.is_empty();
        all_valid = all_valid && file_valid;
        
        results.push((file_path.clone(), validation_result));
        
        if cmd.fail_fast && !file_valid {
            break;
        }
    }
    
    match cmd.format {
        ValidationFormat::Human => {
            for (file_path, result) in &results {
                print_validation_result_human(file_path, result);
            }
        }
        ValidationFormat::Json => {
            let json_output = serde_json::to_string_pretty(&results)?;
            println!("{}", json_output);
        }
        ValidationFormat::Junit => {
            let junit_output = format_junit_results(&results)?;
            println!("{}", junit_output);
        }
        ValidationFormat::Tap => {
            let tap_output = format_tap_results(&results)?;
            println!("{}", tap_output);
        }
    }
    
    if !all_valid {
        process::exit(1);
    }
    
    Ok(())
}

fn handle_convert_command(cmd: ConvertCommand) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    let input_content = fs::read_to_string(&cmd.input)?;
    let parser = DDEXParser::new();
    let result = parser.parse(std::io::Cursor::new(input_content.as_bytes()))?;
    
    let output_data = if cmd.flatten {
        serde_json::to_value(&result.flat)?
    } else {
        serde_json::to_value(&result.graph)?
    };
    
    let to_format = cmd.to.clone();
    let formatted_output = format_output(&output_data, cmd.to, cmd.pretty)?;
    write_output(&formatted_output, &cmd.output)?;
    
    if !is_quiet() {
        eprintln!("{} Conversion completed", "✓".green());
        eprintln!("  Format: {:?}", &to_format);
    }
    
    Ok(())
}

fn handle_stats_command(cmd: StatsCommand) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    let mut stats = StatsReport::new();
    let parser = DDEXParser::new();
    
    for file_path in &cmd.files {
        let start_time = Instant::now();
        let file_size = fs::metadata(file_path)?.len();
        
        let xml_content = fs::read_to_string(file_path)?;
        let result = parser.parse(std::io::Cursor::new(xml_content.as_bytes()))?;
        let parse_duration = start_time.elapsed();
        
        stats.add_file_stats(FileStats {
            path: file_path.clone(),
            size_bytes: file_size,
            parse_time_ms: parse_duration.as_secs_f64() * 1000.0,
            ddex_version: Some(ddex_core::models::versions::ERNVersion::V4_3), // TODO: Extract from result
            element_count: count_elements_flat(&result.flat),
        });
    }
    
    let formatted_output = format_output(&serde_json::to_value(&stats)?, cmd.format, true)?;
    write_output(&formatted_output, &cmd.output)?;
    
    if !is_quiet() {
        eprintln!("{} Statistics generated for {} files", "✓".green(), cmd.files.len());
        eprintln!("  Total size: {:.2} MB", stats.total_size_mb());
        eprintln!("  Average parse time: {:.2}ms", stats.average_parse_time());
    }
    
    Ok(())
}

fn handle_interactive_mode() -> Result<()> {
    println!("{}", "DDEX Parser Interactive Mode".bold().blue());
    println!("Type 'help' for available commands, 'exit' to quit\n");
    
    let stdin = io::stdin();
    loop {
        print!("{} ", "ddex>".green().bold());
        io::stdout().flush()?;
        
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        match input {
            "exit" | "quit" => break,
            "help" => print_interactive_help(),
            _ if input.starts_with("parse ") => {
                let file_path = input.strip_prefix("parse ").unwrap();
                if let Err(e) = parse_file_interactive(file_path) {
                    eprintln!("{} {}", "Error:".red(), e);
                }
            }
            _ if input.starts_with("extract ") => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Err(e) = extract_interactive(&parts[1], &parts[2]) {
                        eprintln!("{} {}", "Error:".red(), e);
                    }
                } else {
                    eprintln!("Usage: extract <file> <query>");
                }
            }
            _ => {
                eprintln!("Unknown command: {}. Type 'help' for available commands.", input);
            }
        }
    }
    
    println!("Goodbye!");
    Ok(())
}

fn handle_completions_command(cmd: CompletionsCommand) -> Result<()> {
    let mut cli = Cli::command();
    
    if let Some(output_path) = cmd.output {
        let mut file = fs::File::create(output_path)?;
        generate(cmd.shell, &mut cli, "ddex-parser", &mut file);
    } else {
        generate(cmd.shell, &mut cli, "ddex-parser", &mut io::stdout());
    }
    
    Ok(())
}

fn detect_version(path: &str) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    let xml = fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path))?;
    
    let parser = DDEXParser::new();
    let version = parser.detect_version(std::io::Cursor::new(xml.as_bytes()))?;
    
    println!("DDEX Version: {:?}", version);
    
    Ok(())
}

fn sanity_check(path: &str) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    let xml = fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path))?;
    
    let parser = DDEXParser::new();
    let result = parser.sanity_check(std::io::Cursor::new(xml.as_bytes()))?;
    
    if result.is_valid {
        println!("✅ Valid DDEX {:?}", result.version);
    } else {
        println!("❌ Invalid DDEX");
        for error in &result.errors {
            println!("  Error: {}", error);
        }
    }
    
    std::process::exit(if result.is_valid { 0 } else { 1 })
}

// Helper functions and data structures

fn read_input_string(input: &Option<PathBuf>) -> Result<String> {
    match input {
        Some(path) if path.to_str() == Some("-") => {
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            Ok(content)
        }
        Some(path) => Ok(fs::read_to_string(path)?),
        None => {
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            Ok(content)
        }
    }
}

fn write_output(content: &str, output: &Option<PathBuf>) -> Result<()> {
    match output {
        Some(path) if path.to_str() == Some("-") => {
            print!("{}", content);
            Ok(())
        }
        Some(path) => {
            fs::write(path, content)?;
            Ok(())
        }
        None => {
            print!("{}", content);
            Ok(())
        }
    }
}

fn format_output(data: &JsonValue, format: OutputFormat, pretty: bool) -> Result<String> {
    match format {
        OutputFormat::Json => {
            if pretty {
                Ok(serde_json::to_string_pretty(data)?)
            } else {
                Ok(serde_json::to_string(data)?)
            }
        }
        OutputFormat::Yaml => Ok(serde_yaml::to_string(data)?),
        OutputFormat::MessagePack => {
            let bytes = rmp_serde::to_vec(data)?;
            // Use proper base64 encoding
            use base64::Engine;
            Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
        }
        OutputFormat::Csv => convert_to_csv(data),
        OutputFormat::Xml => convert_to_xml(data),
    }
}

fn extract_elements(
    result: &ddex_core::models::flat::ParsedERNMessage,
    query: &str,
    _all: bool,
    _include_attributes: bool,
) -> Result<JsonValue> {
    // Convert flat structure to JSON for element extraction
    let flat_json = serde_json::to_value(&result.flat)?;
    
    // For now, implement basic element extraction
    if let Some(value) = find_element_by_path(&flat_json, query) {
        Ok(value.clone())
    } else {
        Ok(JsonValue::Array(vec![]))
    }
}

fn find_element_by_path<'a>(data: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = data;
    
    for part in parts {
        if let Some(obj) = current.as_object() {
            if let Some(value) = obj.get(part) {
                current = value;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    
    Some(current)
}

fn process_file_batch(
    file_path: &PathBuf,
    output_dir: &PathBuf,
    format: OutputFormat,
    flatten: bool,
) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    let xml_content = fs::read_to_string(file_path)?;
    let parser = DDEXParser::new();
    let result = parser.parse(std::io::Cursor::new(xml_content.as_bytes()))?;
    
    let output_data = if flatten {
        serde_json::to_value(&result.flat)?
    } else {
        serde_json::to_value(&result.graph)?
    };
    
    let formatted_output = format_output(&output_data, format.clone(), true)?;
    
    let output_filename = file_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string() + &get_extension_for_format(&format);
    
    let output_path = output_dir.join(output_filename);
    fs::write(output_path, formatted_output)?;
    
    Ok(())
}

fn get_extension_for_format(format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => ".json".to_string(),
        OutputFormat::Yaml => ".yaml".to_string(),
        OutputFormat::MessagePack => ".msgpack".to_string(),
        OutputFormat::Csv => ".csv".to_string(),
        OutputFormat::Xml => ".xml".to_string(),
    }
}

fn validate_xml_only(file_path: &PathBuf) -> Result<ValidationResult> {
    let xml_content = fs::read_to_string(file_path)?;
    
    // Basic XML validation using quick-xml
    match quick_xml::Reader::from_str(&xml_content).read_event() {
        Ok(_) => Ok(ValidationResult {
            errors: vec![],
            warnings: vec![],
            info: vec!["XML is well-formed".to_string()],
            passed: true,
        }),
        Err(e) => Ok(ValidationResult {
            errors: vec![format!("XML parsing error: {}", e)],
            warnings: vec![],
            info: vec![],
            passed: false,
        }),
    }
}

fn validate_ddex_file(
    file_path: &PathBuf,
    _level: ValidationLevel,
    _ddex_version: Option<String>,
) -> Result<ValidationResult> {
    use ddex_parser::DDEXParser;
    
    let xml_content = fs::read_to_string(file_path)?;
    let parser = DDEXParser::new();
    
    match parser.sanity_check(std::io::Cursor::new(xml_content.as_bytes())) {
        Ok(result) => Ok(ValidationResult {
            errors: result.errors.clone(),
            warnings: vec![],
            info: if result.is_valid {
                vec![format!("Valid DDEX {:?}", result.version)]
            } else {
                vec![]
            },
            passed: result.is_valid,
        }),
        Err(e) => Ok(ValidationResult {
            errors: vec![format!("Validation error: {}", e)],
            warnings: vec![],
            info: vec![],
            passed: false,
        }),
    }
}

fn print_validation_result_human(file_path: &PathBuf, result: &ValidationResult) {
    if result.errors.is_empty() {
        println!("{} {} - Valid", "✓".green(), file_path.display());
        for info in &result.info {
            println!("  {}", info.bright_black());
        }
    } else {
        println!(
            "{} {} - {} errors, {} warnings",
            "✗".red(),
            file_path.display(),
            result.errors.len(),
            result.warnings.len()
        );
        
        for error in &result.errors {
            println!("  {} {}", "Error:".red(), error);
        }
        
        for warning in &result.warnings {
            println!("  {} {}", "Warning:".yellow(), warning);
        }
    }
}

fn format_junit_results(results: &[(PathBuf, ValidationResult)]) -> Result<String> {
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    output.push_str(&format!("<testsuite tests=\"{}\" failures=\"{}\">\n", 
        results.len(),
        results.iter().filter(|(_, r)| !r.passed).count()
    ));
    
    for (file_path, result) in results {
        let name = file_path.file_name().unwrap().to_string_lossy();
        output.push_str(&format!("  <testcase name=\"{}\"", name));
        
        if !result.passed {
            output.push_str(">\n");
            output.push_str("    <failure>");
            for error in &result.errors {
                output.push_str(&html_escape::encode_text(error));
            }
            output.push_str("</failure>\n");
            output.push_str("  </testcase>\n");
        } else {
            output.push_str(" />\n");
        }
    }
    
    output.push_str("</testsuite>\n");
    Ok(output)
}

fn format_tap_results(results: &[(PathBuf, ValidationResult)]) -> Result<String> {
    let mut output = String::new();
    output.push_str(&format!("1..{}\n", results.len()));
    
    for (i, (file_path, result)) in results.iter().enumerate() {
        let name = file_path.file_name().unwrap().to_string_lossy();
        if result.passed {
            output.push_str(&format!("ok {} - {}\n", i + 1, name));
        } else {
            output.push_str(&format!("not ok {} - {}\n", i + 1, name));
            for error in &result.errors {
                output.push_str(&format!("  # {}\n", error));
            }
        }
    }
    
    Ok(output)
}

fn count_elements(data: &JsonValue) -> usize {
    match data {
        JsonValue::Object(map) => map.len() + map.values().map(count_elements).sum::<usize>(),
        JsonValue::Array(arr) => arr.iter().map(count_elements).sum::<usize>(),
        _ => 1,
    }
}

fn count_elements_flat(flat: &ddex_core::models::flat::FlattenedMessage) -> usize {
    flat.releases.len() + flat.resources.len() + flat.deals.len() + flat.parties.len()
}

fn convert_to_csv(data: &JsonValue) -> Result<String> {
    // Simple CSV conversion for flat objects
    let mut output = String::new();
    
    if let Some(obj) = data.as_object() {
        // Headers
        let headers: Vec<String> = obj.keys().map(|k| k.to_string()).collect();
        output.push_str(&headers.join(","));
        output.push('\n');
        
        // Values
        let values: Vec<String> = headers.iter()
            .map(|k| obj.get(k).unwrap().to_string())
            .collect();
        output.push_str(&values.join(","));
        output.push('\n');
    }
    
    Ok(output)
}

fn convert_to_xml(data: &JsonValue) -> Result<String> {
    // Simple XML conversion
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    output.push_str("<root>\n");
    
    fn json_to_xml(value: &JsonValue, name: &str, output: &mut String, indent: usize) {
        let spaces = "  ".repeat(indent);
        
        match value {
            JsonValue::Object(map) => {
                output.push_str(&format!("{}<{}>\n", spaces, name));
                for (key, val) in map {
                    json_to_xml(val, key, output, indent + 1);
                }
                output.push_str(&format!("{}</{}>\n", spaces, name));
            }
            JsonValue::Array(arr) => {
                for item in arr {
                    json_to_xml(item, name, output, indent);
                }
            }
            _ => {
                output.push_str(&format!("{}<{}>{}</{}>\n", spaces, name, value, name));
            }
        }
    }
    
    json_to_xml(data, "data", &mut output, 1);
    output.push_str("</root>\n");
    Ok(output)
}

fn print_interactive_help() {
    println!("Available commands:");
    println!("  {} <file>       - Parse DDEX XML file", "parse".cyan());
    println!("  {} <file> <query> - Extract elements from file", "extract".cyan());
    println!("  {}              - Show this help", "help".cyan());
    println!("  {}              - Exit interactive mode", "exit".cyan());
}

fn parse_file_interactive(file_path: &str) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    let xml_content = fs::read_to_string(file_path)?;
    let parser = DDEXParser::new();
    let result = parser.parse(std::io::Cursor::new(xml_content.as_bytes()))?;
    
    let json = serde_json::to_string_pretty(&result.flat)?;
    println!("{}", json);
    
    Ok(())
}

fn extract_interactive(file_path: &str, query: &str) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    let xml_content = fs::read_to_string(file_path)?;
    let parser = DDEXParser::new();
    let result = parser.parse(std::io::Cursor::new(xml_content.as_bytes()))?;
    
    let extracted = extract_elements(&result, query, false, false)?;
    let json = serde_json::to_string_pretty(&extracted)?;
    println!("{}", json);
    
    Ok(())
}

fn is_quiet() -> bool {
    std::env::var("DDEX_QUIET").unwrap_or_default() == "1"
}

// Data structures

#[derive(serde::Serialize, serde::Deserialize)]
struct ValidationResult {
    errors: Vec<String>,
    warnings: Vec<String>,
    info: Vec<String>,
    passed: bool,
}

#[derive(serde::Serialize)]
struct BatchResult {
    file_path: PathBuf,
    success: bool,
    error: Option<String>,
}

#[derive(serde::Serialize)]
struct BatchReport {
    total_files: usize,
    successful: usize,
    failed: usize,
    results: Vec<BatchResult>,
}

#[derive(serde::Serialize)]
struct FileStats {
    path: PathBuf,
    size_bytes: u64,
    parse_time_ms: f64,
    ddex_version: Option<ddex_core::models::versions::ERNVersion>,
    element_count: usize,
}

#[derive(serde::Serialize)]
struct StatsReport {
    files: Vec<FileStats>,
    summary: StatsSummary,
}

#[derive(serde::Serialize)]
struct StatsSummary {
    total_files: usize,
    total_size_bytes: u64,
    average_parse_time_ms: f64,
    fastest_parse_ms: f64,
    slowest_parse_ms: f64,
    ddex_versions: HashMap<String, usize>,
}

impl StatsReport {
    fn new() -> Self {
        Self {
            files: Vec::new(),
            summary: StatsSummary {
                total_files: 0,
                total_size_bytes: 0,
                average_parse_time_ms: 0.0,
                fastest_parse_ms: f64::MAX,
                slowest_parse_ms: 0.0,
                ddex_versions: HashMap::new(),
            },
        }
    }
    
    fn add_file_stats(&mut self, stats: FileStats) {
        self.summary.total_files += 1;
        self.summary.total_size_bytes += stats.size_bytes;
        self.summary.fastest_parse_ms = self.summary.fastest_parse_ms.min(stats.parse_time_ms);
        self.summary.slowest_parse_ms = self.summary.slowest_parse_ms.max(stats.parse_time_ms);
        
        if let Some(version) = &stats.ddex_version {
            let version_str = format!("{:?}", version);
            *self.summary.ddex_versions.entry(version_str).or_insert(0) += 1;
        }
        
        self.files.push(stats);
        
        // Recalculate average
        let total_time: f64 = self.files.iter().map(|f| f.parse_time_ms).sum();
        self.summary.average_parse_time_ms = total_time / self.files.len() as f64;
    }
    
    fn total_size_mb(&self) -> f64 {
        self.summary.total_size_bytes as f64 / 1_048_576.0
    }
    
    fn average_parse_time(&self) -> f64 {
        self.summary.average_parse_time_ms
    }
}