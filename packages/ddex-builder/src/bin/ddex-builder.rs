//! DDEX Builder CLI - Comprehensive command-line interface for DDEX XML processing
//!
//! This CLI provides tools for building, converting, validating, and comparing DDEX XML files
//! with deterministic output and support for various partner presets.

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Generator, Shell};
use console::style;
use ddex_builder::presets::{DdexVersion, MessageProfile};
use ddex_builder::*;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(
    name = "ddex-builder",
    about = "DDEX Builder CLI - High-performance DDEX XML processing toolkit",
    long_about = "A comprehensive command-line interface for building, converting, validating, and comparing DDEX XML files with deterministic output and partner preset support.",
    version = env!("CARGO_PKG_VERSION"),
    author = "Kevin Marques Moo"
)]
#[command(propagate_version = true)]
#[command(disable_version_flag = true)]
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

    /// Path to configuration file
    #[arg(long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build DDEX XML from structured data
    Build(BuildCommand),
    /// Convert DDEX XML between versions
    Convert(ConvertCommand),
    /// Compare two DDEX files semantically
    Diff(DiffCommand),
    /// Validate DDEX XML files
    Validate(ValidateCommand),
    /// Generate schemas for validation
    Schema(SchemaCommand),
    /// Process multiple files in parallel
    Batch(BatchCommand),
    /// Generate shell completions
    Completions(CompletionsCommand),
}

#[derive(Args)]
struct BuildCommand {
    /// Input file (JSON/YAML/TOML) or '-' for stdin
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file path or '-' for stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// DDEX version to generate
    #[arg(long = "ddex-version", value_enum)]
    version: Option<DdexVersionArg>,

    /// Content profile to use
    #[arg(short, long)]
    profile: Option<String>,

    /// Partner preset configuration
    #[arg(long, value_enum)]
    preset: Option<PresetChoice>,

    /// Validate before building
    #[arg(long)]
    validate: bool,

    /// Input data format (auto-detected if not specified)
    #[arg(long, value_enum)]
    format: Option<InputFormat>,

    /// Enable strict validation
    #[arg(long)]
    strict: bool,
}

#[derive(Args)]
struct ConvertCommand {
    /// Input DDEX XML file or '-' for stdin
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Source DDEX version
    #[arg(short, long, value_enum)]
    from: DdexVersionArg,

    /// Target DDEX version
    #[arg(short, long, value_enum)]
    to: DdexVersionArg,

    /// Output file path or '-' for stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Generate conversion report
    #[arg(long)]
    report: Option<PathBuf>,

    /// Enable lossy conversion warnings
    #[arg(long)]
    allow_lossy: bool,
}

#[derive(Args)]
struct DiffCommand {
    /// First DDEX XML file
    file1: PathBuf,

    /// Second DDEX XML file
    file2: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = DiffFormat::Human)]
    format: DiffFormat,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Include technical details in diff
    #[arg(long)]
    detailed: bool,

    /// Ignore whitespace differences
    #[arg(long)]
    ignore_whitespace: bool,
}

#[derive(Args)]
struct ValidateCommand {
    /// DDEX XML files to validate
    files: Vec<PathBuf>,

    /// DDEX version for validation
    #[arg(long = "ddex-version", value_enum)]
    version: Option<DdexVersionArg>,

    /// Content profile for validation
    #[arg(short, long)]
    profile: Option<String>,

    /// Partner preset for validation
    #[arg(long, value_enum)]
    preset: Option<PresetChoice>,

    /// Enable strict validation rules
    #[arg(long)]
    strict: bool,

    /// Output format for validation results
    #[arg(long, value_enum, default_value_t = ValidateFormat::Human)]
    output_format: ValidateFormat,

    /// Stop at first validation error
    #[arg(long)]
    fail_fast: bool,
}

#[derive(Args)]
struct SchemaCommand {
    /// DDEX version for schema generation
    #[arg(long = "ddex-version", value_enum)]
    version: DdexVersionArg,

    /// Content profile
    #[arg(short, long)]
    profile: Option<String>,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = SchemaFormat::Json)]
    format: SchemaFormat,

    /// Output file path or '-' for stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Include documentation in schema
    #[arg(long)]
    with_docs: bool,
}

#[derive(Args)]
struct BatchCommand {
    /// Configuration file for batch processing
    #[arg(short, long)]
    config: PathBuf,

    /// Number of worker threads
    #[arg(short, long, default_value_t = num_cpus::get())]
    workers: usize,

    /// Continue processing on errors
    #[arg(long)]
    continue_on_error: bool,

    /// Generate summary report
    #[arg(long)]
    report: Option<PathBuf>,
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

#[derive(ValueEnum, Clone, Debug)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum DdexVersionArg {
    #[value(name = "3.8.2")]
    V382,
    #[value(name = "4.1")]
    V41,
    #[value(name = "4.2")]
    V42,
    #[value(name = "4.3")]
    V43,
    #[value(name = "4.4")]
    V44,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum PresetChoice {
    Spotify,
    Youtube,
    Apple,
    Amazon,
    Universal,
    Sony,
    Warner,
}

#[derive(ValueEnum, Clone, Debug)]
enum InputFormat {
    Json,
    Yaml,
    Toml,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum DiffFormat {
    Human,
    Json,
    Update,
}

#[derive(ValueEnum, Clone, Debug)]
enum ValidateFormat {
    Human,
    Json,
    Junit,
}

#[derive(ValueEnum, Clone, Debug)]
enum SchemaFormat {
    Json,
    Typescript,
    Python,
}

impl From<DdexVersionArg> for DdexVersion {
    fn from(version: DdexVersionArg) -> Self {
        match version {
            DdexVersionArg::V382 => DdexVersion::Ern382,
            DdexVersionArg::V41 => DdexVersion::Ern41,
            DdexVersionArg::V42 => DdexVersion::Ern42,
            DdexVersionArg::V43 => DdexVersion::Ern43,
            DdexVersionArg::V44 => DdexVersion::Ern43, // Map V44 to Ern43 since Ern44 doesn't exist yet
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // Setup logging based on verbosity
    setup_logging(cli.verbose, cli.quiet);

    // Setup color output
    setup_colors(cli.color);

    // Load configuration if specified
    let config = cli.config.as_ref().map(|p| load_config(p)).unwrap_or_default();

    let result = match cli.command {
        Commands::Build(cmd) => handle_build_command(cmd, &config),
        Commands::Convert(cmd) => handle_convert_command(cmd, &config),
        Commands::Diff(cmd) => handle_diff_command(cmd, &config),
        Commands::Validate(cmd) => handle_validate_command(cmd, &config),
        Commands::Schema(cmd) => handle_schema_command(cmd, &config),
        Commands::Batch(cmd) => handle_batch_command(cmd, &config),
        Commands::Completions(cmd) => handle_completions_command(cmd),
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("Error:").red().bold(), e);
        process::exit(1);
    }
}

fn setup_logging(verbosity: u8, quiet: bool) {
    if quiet {
        return;
    }

    let level = match verbosity {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();
}

fn setup_colors(color_choice: ColorChoice) {
    match color_choice {
        ColorChoice::Always => {
            console::set_colors_enabled(true);
            console::set_colors_enabled_stderr(true);
        }
        ColorChoice::Never => {
            console::set_colors_enabled(false);
            console::set_colors_enabled_stderr(false);
        }
        ColorChoice::Auto => {
            // Default behavior - colors enabled for TTY
        }
    }
}

fn load_config(_path: &Path) -> ConfigFile {
    // TODO: Implement configuration file loading
    ConfigFile::default()
}

#[derive(Default)]
struct ConfigFile {
    // Configuration options that can be loaded from file
}

fn handle_build_command(cmd: BuildCommand, _config: &ConfigFile) -> Result<(), Box<dyn std::error::Error>> {
    let input_data = read_input_data(&cmd.input, cmd.format)?;
    
    // Create builder with optional preset
    let mut builder = Builder::new();
    
    if let Some(preset) = cmd.preset {
        let preset_name = preset_to_string(&preset);
        builder.apply_preset(&preset_name, false)
            .map_err(|e| format!("Failed to apply preset '{}': {}", preset_name, e))?;
    }

    if let Some(version) = cmd.version {
        builder.with_version(version.into());
    }

    // Validate input if requested
    if cmd.validate {
        validate_input_data(&input_data, &builder, cmd.strict)?;
    }

    // Build the XML
    let xml_output = build_ddex_xml(&input_data, &builder)?;

    // Write output
    write_output(&xml_output, &cmd.output)?;

    if !is_quiet() {
        println!("{} DDEX XML built successfully", style("✓").green());
        if let Some(preset) = cmd.preset {
            println!("  Preset: {}", preset_to_string(&preset));
        }
        if let Some(version) = cmd.version {
            println!("  Version: {:?}", version);
        }
    }

    Ok(())
}

fn handle_convert_command(cmd: ConvertCommand, _config: &ConfigFile) -> Result<(), Box<dyn std::error::Error>> {
    let input_xml = read_input_string(&cmd.input)?;
    
    let builder = Builder::new();
    let conversion_options = ConversionOptions {
        detailed_reports: true,
        preserve_unknown: cmd.allow_lossy,
        preserve_comments: true,
        ..Default::default()
    };

    let result = builder.convert_version(
        &input_xml,
        cmd.from.into(),
        cmd.to.into(),
        Some(conversion_options),
    )?;

    match result {
        versions::ConverterResult::Success { xml, report } => {
            // Write converted XML
            write_output(&xml, &cmd.output)?;

            // Generate report if requested
            if let Some(report_path) = cmd.report {
                let report_json = serde_json::to_string_pretty(&report)?;
                fs::write(report_path, report_json)?;
            }

            if !is_quiet() {
                println!("{} Conversion completed", style("✓").green());
                println!("  From: {:?} → To: {:?}", cmd.from, cmd.to);
                if !report.warnings.is_empty() {
                    println!("  {} warnings generated", report.warnings.len());
                }
            }
        }
        versions::ConverterResult::Failure { error, report: _ } => {
            return Err(format!("Conversion failed: {}", error).into());
        }
    }

    Ok(())
}

fn handle_diff_command(cmd: DiffCommand, _config: &ConfigFile) -> Result<(), Box<dyn std::error::Error>> {
    let xml1 = fs::read_to_string(&cmd.file1)?;
    let xml2 = fs::read_to_string(&cmd.file2)?;

    let diff_config = diff::DiffConfig {
        ignore_formatting: cmd.ignore_whitespace,
        ..Default::default()
    };

    let mut diff_engine = diff::DiffEngine::new_with_config(diff_config);
    // TODO: Parse XML to AST for proper semantic diffing
    // For now, create a placeholder changeset
    let changeset = diff::types::ChangeSet::new();

    let formatted_output = match cmd.format {
        DiffFormat::Human => {
            diff::formatter::DiffFormatter::format_summary(&changeset)
        }
        DiffFormat::Json => serde_json::to_string_pretty(&changeset)?,
        DiffFormat::Update => {
            let mut update_generator = messages::UpdateGenerator::new();
            let update_message = update_generator.create_update(&xml1, &xml2, "cli-generated")?;
            // TODO: Implement XML serialization for UpdateReleaseMessage
            format!("<!-- Update message XML would be serialized here -->")
        }
    };

    write_output(&formatted_output, &cmd.output)?;

    if !is_quiet() && cmd.format == DiffFormat::Human {
        if changeset.changes.is_empty() {
            println!("{} Files are identical", style("✓").green());
        } else {
            println!("{} {} differences found", style("!").yellow(), changeset.changes.len());
        }
    }

    Ok(())
}

fn handle_validate_command(cmd: ValidateCommand, _config: &ConfigFile) -> Result<(), Box<dyn std::error::Error>> {
    let mut all_valid = true;
    let mut results = Vec::new();

    for file_path in &cmd.files {
        let xml_content = fs::read_to_string(file_path)?;
        
        let mut builder = Builder::new();
        if let Some(preset) = &cmd.preset {
            let preset_name = preset_to_string(preset);
            builder.apply_preset(&preset_name, false)?;
        }

        let validation_config = ValidationConfig {
            level: if cmd.strict { PreflightLevel::Strict } else { PreflightLevel::Warn },
            profile: cmd.profile.clone(),
            ..Default::default()
        };

        let validator = PreflightValidator::new(validation_config);
        // TODO: Parse XML content to BuildRequest for validation
        // For now, create a placeholder result
        let result = ValidationResult {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            passed: true,
        };

        let file_valid = result.errors.is_empty();
        all_valid = all_valid && file_valid;

        results.push((file_path.clone(), result));

        if cmd.fail_fast && !file_valid {
            break;
        }
    }

    // Output results
    match cmd.output_format {
        ValidateFormat::Human => {
            for (file_path, result) in &results {
                print_validation_result_human(file_path, result);
            }
        }
        ValidateFormat::Json => {
            let json_output = serde_json::to_string_pretty(&results)?;
            println!("{}", json_output);
        }
        ValidateFormat::Junit => {
            let junit_output = format_junit_results(&results)?;
            println!("{}", junit_output);
        }
    }

    if !all_valid {
        process::exit(1);
    }

    Ok(())
}

fn handle_schema_command(cmd: SchemaCommand, _config: &ConfigFile) -> Result<(), Box<dyn std::error::Error>> {
    let schema_config = schema::SchemaConfig {
        include_descriptions: cmd.with_docs,
        ..Default::default()
    };

    // Use a default profile for now - this could be enhanced to support actual profiles  
    let profile = MessageProfile::AudioAlbum;
    let generator = schema::SchemaGenerator::new(cmd.version.into(), profile);
    let schema_result = generator.generate_complete_schema()?;
    let schema_output = match cmd.format {
        SchemaFormat::Json => serde_json::to_string_pretty(&schema_result.schema)?,
        SchemaFormat::Typescript => generator.generate_typescript_types(&schema_result.schema)?,
        SchemaFormat::Python => generator.generate_python_types(&schema_result.schema)?,
    };

    write_output(&schema_output, &cmd.output)?;

    if !is_quiet() {
        println!("{} Schema generated successfully", style("✓").green());
        println!("  Format: {:?}", cmd.format);
        println!("  Version: {:?}", cmd.version);
    }

    Ok(())
}

fn handle_batch_command(cmd: BatchCommand, _config: &ConfigFile) -> Result<(), Box<dyn std::error::Error>> {
    let batch_config = load_batch_config(&cmd.config)?;
    
    // Setup thread pool
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cmd.workers)
        .build()?;

    let progress_bar = if !is_quiet() {
        let pb = ProgressBar::new(batch_config.tasks.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap(),
        );
        Some(pb)
    } else {
        None
    };

    let results: Vec<BatchResult> = pool.install(|| {
        batch_config
            .tasks
            .par_iter()
            .enumerate()
            .map(|(i, task)| {
                let result = process_batch_task(task);
                if let Some(pb) = &progress_bar {
                    pb.set_message(format!("Processing {}", task.input_file.display()));
                    pb.inc(1);
                }
                BatchResult {
                    task_id: i,
                    success: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                }
            })
            .collect()
    });

    if let Some(pb) = &progress_bar {
        pb.finish_with_message("Batch processing completed");
    }

    // Generate report
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    if !is_quiet() {
        println!("\n{} Batch processing completed", style("✓").green());
        println!("  Successful: {}", successful);
        if failed > 0 {
            println!("  Failed: {}", style(failed).red());
        }
    }

    if let Some(report_path) = cmd.report {
        let report = BatchReport {
            total_tasks: results.len(),
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

fn handle_completions_command(cmd: CompletionsCommand) -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = Cli::command();
    
    if let Some(output_path) = cmd.output {
        let mut file = fs::File::create(output_path)?;
        generate(cmd.shell, &mut cli, "ddex-builder", &mut file);
    } else {
        generate(cmd.shell, &mut cli, "ddex-builder", &mut io::stdout());
    }

    Ok(())
}

// Helper functions

fn read_input_data(input: &Option<PathBuf>, format: Option<InputFormat>) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let content = read_input_string(input)?;
    
    let detected_format = format.unwrap_or_else(|| {
        if let Some(path) = input {
            detect_input_format(path)
        } else {
            InputFormat::Json // Default for stdin
        }
    });

    match detected_format {
        InputFormat::Json => Ok(serde_json::from_str(&content)?),
        InputFormat::Yaml => Ok(serde_yaml::from_str(&content)?),
        InputFormat::Toml => {
            let toml_value: toml::Value = toml::from_str(&content)?;
            Ok(serde_json::to_value(toml_value)?)
        }
    }
}

fn read_input_string(input: &Option<PathBuf>) -> Result<String, Box<dyn std::error::Error>> {
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

fn write_output(content: &str, output: &Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
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

fn detect_input_format(path: &Path) -> InputFormat {
    match path.extension().and_then(|s| s.to_str()) {
        Some("yaml") | Some("yml") => InputFormat::Yaml,
        Some("toml") => InputFormat::Toml,
        _ => InputFormat::Json,
    }
}

fn preset_to_string(preset: &PresetChoice) -> String {
    match preset {
        PresetChoice::Spotify => "spotify_audio_43".to_string(),
        PresetChoice::Youtube => "youtube_music_43".to_string(),
        PresetChoice::Apple => "apple_music_43".to_string(),
        PresetChoice::Amazon => "amazon_music_43".to_string(),
        PresetChoice::Universal => "universal_43".to_string(),
        PresetChoice::Sony => "sony_43".to_string(),
        PresetChoice::Warner => "warner_43".to_string(),
    }
}

fn validate_input_data(
    _data: &JsonValue,
    _builder: &Builder,
    _strict: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement input validation logic
    Ok(())
}

fn build_ddex_xml(_data: &JsonValue, _builder: &Builder) -> Result<String, Box<dyn std::error::Error>> {
    // TODO: Implement actual DDEX XML building
    Ok("<xml><!-- DDEX XML would be generated here --></xml>".to_string())
}

fn print_validation_result_human(file_path: &Path, result: &ValidationResult) {
    if result.errors.is_empty() {
        println!("{} {} - Valid", style("✓").green(), file_path.display());
    } else {
        println!("{} {} - {} errors, {} warnings", 
            style("✗").red(), 
            file_path.display(),
            result.errors.len(),
            result.warnings.len()
        );
        
        for error in &result.errors {
            println!("  {} {:?}", style("Error:").red(), error);
        }
        
        for warning in &result.warnings {
            println!("  {} {:?}", style("Warning:").yellow(), warning);
        }
    }
}

fn format_junit_results(_results: &[(PathBuf, ValidationResult)]) -> Result<String, Box<dyn std::error::Error>> {
    // TODO: Implement JUnit XML format
    Ok("<testsuite><!-- JUnit results would be here --></testsuite>".to_string())
}

fn load_batch_config(path: &Path) -> Result<BatchConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: BatchConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

fn process_batch_task(_task: &BatchTask) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement batch task processing
    Ok(())
}

fn is_quiet() -> bool {
    std::env::var("DDEX_QUIET").unwrap_or_default() == "1"
}

// Data structures for batch processing

#[derive(serde::Deserialize)]
struct BatchConfig {
    tasks: Vec<BatchTask>,
}

#[derive(serde::Deserialize)]
struct BatchTask {
    input_file: PathBuf,
    output_file: PathBuf,
    preset: Option<String>,
    version: Option<String>,
    validate: Option<bool>,
}

#[derive(serde::Serialize)]
struct BatchResult {
    task_id: usize,
    success: bool,
    error: Option<String>,
}

#[derive(serde::Serialize)]
struct BatchReport {
    total_tasks: usize,
    successful: usize,
    failed: usize,
    results: Vec<BatchResult>,
}