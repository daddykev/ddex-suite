#!/bin/bash
# scripts/generate-completions.sh

echo "Generating shell completions..."

COMPLETION_DIR="packages/ddex-parser/completions"
mkdir -p "$COMPLETION_DIR"

cd packages/ddex-parser

# Build the CLI first
cargo build --release --bin ddex-parser --features cli

# Generate completions for each shell
echo "Generating Bash completion..."
./target/release/ddex-parser completions bash > "$COMPLETION_DIR/ddex-parser.bash"

echo "Generating Zsh completion..."
./target/release/ddex-parser completions zsh > "$COMPLETION_DIR/_ddex-parser"

echo "Generating Fish completion..."
./target/release/ddex-parser completions fish > "$COMPLETION_DIR/ddex-parser.fish"

echo "Generating PowerShell completion..."
./target/release/ddex-parser completions powershell > "$COMPLETION_DIR/_ddex-parser.ps1"

echo "Shell completions generated in $COMPLETION_DIR"

# Installation instructions
cat << 'EOF' > "$COMPLETION_DIR/README.md"
# DDEX Parser Shell Completions

## Installation

### Bash
```bash
# Add to ~/.bashrc or ~/.bash_profile
source /path/to/ddex-parser.bash