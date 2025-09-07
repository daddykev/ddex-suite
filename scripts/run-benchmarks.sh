#!/bin/bash
# scripts/run-benchmarks.sh

echo "Running DDEX Parser Benchmarks..."

# Build in release mode
cargo build --release

# Run criterion benchmarks
cargo bench --bench parsing

# Generate report
cargo run --bin generate-bench-report

# Copy results
cp target/criterion/parse_by_size/report/index.html benchmarks/results/
cp benchmarks/report.json benchmarks/results/

echo "Benchmark results saved to benchmarks/results/"