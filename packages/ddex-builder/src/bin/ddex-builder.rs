//! DDEX Builder CLI - Comprehensive command-line interface for DDEX XML processing
//!
//! This CLI builds DDEX XML from a JSON / YAML / TOML build request (`build`, which runs
//! `DDEXBuilder::build` and its preflight validation). The `convert`, `schema`, `validate`,
//! `watch` and `server` commands were removed in 0.6.0 (they gave made-up results or were not
//! implemented).

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use console::style;
use ddex_builder::*;
use serde_json::Value as JsonValue;
use std::fs;
use std::io::{self, Read};
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

    /// Build several times and check that the outputs are byte-identical
    #[arg(long)]
    check_repeatable: bool,

    /// Number of builds for --check-repeatable (default: 3)
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

    let version: Option<DdexVersion> = cmd.ddex_version.map(Into::into);

    // Validate input if requested
    if cmd.validate {
        validate_input_data(&input_data, version)?;
    }

    // Build the XML
    let xml_output = build_ddex_xml(&input_data, version)?;

    // Check that repeated builds are byte-identical, if requested
    if cmd.check_repeatable {
        check_repeatable(&input_data, version, cmd.determinism_iterations)?;
    }

    // Write output
    write_output(&xml_output, &cmd.output)?;

    if !is_quiet() {
        println!("{} DDEX XML built successfully", style("✓").green());
        if let Some(version) = cmd.ddex_version {
            println!("  Version: {:?}", version);
        }
        if cmd.check_repeatable && cmd.determinism_iterations >= 2 {
            println!(
                "  {} {} builds were byte-identical",
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
    version: Option<DdexVersion>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Preflight validation runs as part of the build.
    build_ddex_xml(data, version).map(|_| ())
}

/// Builds the request in `data`; `--ddex-version` replaces the request's `version`.
fn build_ddex_xml(
    data: &JsonValue,
    version: Option<DdexVersion>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut request: BuildRequest = serde_json::from_value(data.clone())
        .map_err(|e| format!("the input is not a build request: {e}"))?;
    if let Some(version) = version {
        request.version = version.to_string().trim_start_matches("ERN/").to_string();
    }
    Ok(DDEXBuilder::new()
        .build(request, BuildOptions::default())?
        .xml)
}

fn is_quiet() -> bool {
    std::env::var("DDEX_QUIET").unwrap_or_default() == "1"
}

/// Builds `data` `builds` times and fails at the first build whose output differs from the
/// first one.
fn check_repeatable(
    data: &JsonValue,
    version: Option<DdexVersion>,
    builds: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if builds < 2 {
        return Ok(());
    }
    let first = build_ddex_xml(data, version)?;
    for build in 2..=builds {
        let output = build_ddex_xml(data, version)?;
        if output != first {
            eprintln!(
                "{} builds differed at build {}",
                style("✗").red().bold(),
                build
            );
            if let Some(pos) = find_first_difference(&first, &output) {
                eprintln!("  First difference at byte position: {}", pos);
                let start = pos.saturating_sub(50);
                let end = std::cmp::min(pos + 100, std::cmp::min(first.len(), output.len()));
                eprintln!("  Build 1: {:?}", &first[start..end]);
                eprintln!("  Build {}: {:?}", build, &output[start..end]);
            }
            return Err(format!("builds differed at build {build}").into());
        }
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
