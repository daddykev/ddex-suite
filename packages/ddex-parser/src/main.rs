//! DDEX Parser CLI entry point

mod cli;
mod error;
mod parser;
mod transform;

// Re-export for CLI use
pub use ddex_parser::DDEXParser;

fn main() {
    if let Err(e) = cli::main() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
