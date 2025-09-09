//! DDEX Parser CLI implementation (simplified version for now)

use anyhow::{Context, Result};
use std::fs;

pub fn main() -> Result<()> {
    // For now, let's create a simple working CLI
    // We'll expand this later
    
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_help();
        return Ok(());
    }
    
    match args[1].as_str() {
        "parse" => {
            if args.len() < 3 {
                eprintln!("Error: parse requires a file argument");
                std::process::exit(1);
            }
            parse_file(&args[2])
        }
        "version" | "detect-version" => {
            if args.len() < 3 {
                eprintln!("Error: detect-version requires a file argument");
                std::process::exit(1);
            }
            detect_version(&args[2])
        }
        "check" | "sanity-check" => {
            if args.len() < 3 {
                eprintln!("Error: sanity-check requires a file argument");
                std::process::exit(1);
            }
            sanity_check(&args[2])
        }
        "--version" | "-V" => {
            println!("ddex-parser {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "--help" | "-h" => {
            print_help();
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("DDEX Parser v{}", env!("CARGO_PKG_VERSION"));
    println!("\nUSAGE:");
    println!("    ddex-parser <COMMAND> [OPTIONS]");
    println!("\nCOMMANDS:");
    println!("    parse <FILE>           Parse DDEX XML file");
    println!("    detect-version <FILE>  Detect DDEX version");
    println!("    sanity-check <FILE>    Check file validity");
    println!("\nOPTIONS:");
    println!("    -h, --help     Print help");
    println!("    -V, --version  Print version");
}

fn parse_file(path: &str) -> Result<()> {
    use ddex_parser::DDEXParser;
    
    let xml = fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path))?;
    
    let parser = DDEXParser::new();
    let result = parser.parse(std::io::Cursor::new(xml.as_bytes()))?;
    
    // Output as JSON
    let json = serde_json::to_string_pretty(&result.flat)?;
    println!("{}", json);
    
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
