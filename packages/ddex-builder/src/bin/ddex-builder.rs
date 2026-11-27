//! DDEX Builder CLI - Comprehensive command-line interface for DDEX XML processing
//!
//! This CLI builds DDEX XML from a JSON / YAML / TOML build request (`build`, which runs
//! `DDEXBuilder::build` and its preflight validation). `validate`, `watch` and `server` are
//! not implemented: they exit with an error (before 0.5.1 they printed made-up results).
//! The `convert` and `schema` commands were removed in 0.6.0 (they gave made-up results).

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use console::style;
use ddex_builder::*;
use serde_json::Value as JsonValue;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(
    name = "ddex-builder",
    about = "DDEX Builder CLI - High-performance DDEX XML processing toolkit",
    long_about = "A comprehensive command-line interface for building DDEX XML files with DB-C14N/1.0 canonical output.",
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
    /// Build DDEX XML from structured data
    Build(BuildCommand),
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
    ddex_version: Option<DdexVersionArg>,

    /// Validate before building
    #[arg(long)]
    validate: bool,

    /// Input data format (auto-detected if not specified)
    #[arg(long, value_enum)]
    format: Option<InputFormat>,

    /// Verify determinism by building multiple times
    #[arg(long)]
    verify_determinism: bool,

    /// Number of iterations for determinism verification (default: 3)
    #[arg(long, default_value_t = 3)]
    determinism_iterations: usize,
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

#[derive(ValueEnum, Clone, Debug)]
enum InputFormat {
    Json,
    Yaml,
    Toml,
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

    let result = match cli.command {
        Commands::Build(cmd) => handle_build_command(cmd),
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

fn handle_build_command(cmd: BuildCommand) -> Result<(), Box<dyn std::error::Error>> {
    let input_data = read_input_data(&cmd.input, cmd.format)?;

    let mut builder = Builder::new();

    if let Some(version) = cmd.ddex_version {
        builder.with_version(version.into());
    }

    // Validate input if requested
    if cmd.validate {
        validate_input_data(&input_data, &builder)?;
    }

    // Build the XML
    let xml_output = build_ddex_xml(&input_data, &builder)?;

    // Verify determinism if requested
    if cmd.verify_determinism {
        verify_build_determinism(&input_data, &builder, cmd.determinism_iterations)?;
    }

    // Write output
    write_output(&xml_output, &cmd.output)?;

    if !is_quiet() {
        println!("{} DDEX XML built successfully", style("✓").green());
        if let Some(version) = cmd.ddex_version {
            println!("  Version: {:?}", version);
        }
        if cmd.verify_determinism {
            println!(
                "  {} Determinism verified with {} iterations",
                style("✓").green(),
                cmd.determinism_iterations
            );
        }
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

fn read_input_data(
    input: &Option<PathBuf>,
    format: Option<InputFormat>,
) -> Result<JsonValue, Box<dyn std::error::Error>> {
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

fn validate_input_data(
    data: &JsonValue,
    builder: &Builder,
) -> Result<(), Box<dyn std::error::Error>> {
    // Preflight validation runs as part of the build.
    build_ddex_xml(data, builder).map(|_| ())
}

fn build_ddex_xml(
    data: &JsonValue,
    builder: &Builder,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut request: BuildRequest = serde_json::from_value(data.clone())
        .map_err(|e| format!("the input is not a build request: {e}"))?;
    if let Some(version) = builder.target_version() {
        request.version = version.to_string().trim_start_matches("ERN/").to_string();
    }
    Ok(DDEXBuilder::new()
        .build(request, BuildOptions::default())?
        .xml)
}

fn is_quiet() -> bool {
    std::env::var("DDEX_QUIET").unwrap_or_default() == "1"
}

fn verify_build_determinism(
    data: &JsonValue,
    builder: &Builder,
    iterations: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};

    if iterations < 2 {
        return Ok(());
    }

    if !is_quiet() {
        println!(
            "  {} Verifying determinism with {} iterations...",
            style("→").blue(),
            iterations
        );
    }

    let mut outputs = Vec::with_capacity(iterations);
    let mut hashes = Vec::with_capacity(iterations);

    // Build XML multiple times
    for _i in 0..iterations {
        let xml = build_ddex_xml(data, builder)?;

        // Calculate SHA-256 hash
        let mut hasher = Sha256::new();
        hasher.update(xml.as_bytes());
        let hash = hasher.finalize();
        let hash_hex = format!("{:x}", hash);

        outputs.push(xml);
        hashes.push(hash_hex);

        if !is_quiet() && iterations > 3 {
            print!(".");
            io::stdout().flush().unwrap_or_default();
        }
    }

    if !is_quiet() && iterations > 3 {
        println!(); // New line after dots
    }

    // Compare all outputs byte-for-byte
    let first_output = &outputs[0];
    let first_hash = &hashes[0];

    for (i, (output, hash)) in outputs[1..].iter().zip(hashes[1..].iter()).enumerate() {
        if output != first_output || hash != first_hash {
            eprintln!(
                "{} Determinism verification failed!",
                style("✗").red().bold()
            );
            eprintln!("  Output from iteration 1 differs from iteration {}", i + 2);
            eprintln!("  Hash 1: {}", first_hash);
            eprintln!("  Hash {}: {}", i + 2, hash);

            // Show byte-level differences for first 1000 characters
            let diff_start = find_first_difference(first_output, output);
            if let Some(pos) = diff_start {
                eprintln!("  First difference at byte position: {}", pos);
                let start = pos.saturating_sub(50);
                let end = std::cmp::min(pos + 100, std::cmp::min(first_output.len(), output.len()));
                eprintln!("  Context around difference:");
                eprintln!("  Output 1: {:?}", &first_output[start..end]);
                eprintln!("  Output {}: {:?}", i + 2, &output[start..end]);
            }

            return Err(
                "Determinism verification failed - outputs differ between iterations".into(),
            );
        }
    }

    if !is_quiet() {
        println!(
            "  {} All {} iterations produced identical output",
            style("✓").green(),
            iterations
        );
        println!("  SHA-256: {}", first_hash);
    }

    Ok(())
}

fn find_first_difference(a: &str, b: &str) -> Option<usize> {
    a.bytes()
        .zip(b.bytes())
        .position(|(x, y)| x != y)
        .or_else(|| {
            if a.len() != b.len() {
                Some(std::cmp::min(a.len(), b.len()))
            } else {
                None
            }
        })
}
