//! DDEX Parser CLI entry point
//!
//! The CLI uses the library through the `ddex_parser` crate; it must not
//! re-declare library modules with `mod`, which would compile a second,
//! unused copy of them into the binary.

mod cli;

fn main() {
    if let Err(e) = cli::main() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
