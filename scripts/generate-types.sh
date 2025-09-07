#!/bin/bash
# scripts/generate-types.sh

echo "Generating TypeScript type definitions..."

cd core
cargo build --features typescript

# The ts-rs generated files will be in target/ts
mkdir -p ../bindings/node/generated
cp -r target/ts/* ../bindings/node/generated/

echo "TypeScript definitions generated in bindings/node/generated/"