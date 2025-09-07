#!/bin/bash

# Function to remove profile sections from a Cargo.toml
remove_profiles() {
    local file=$1
    if [ -f "$file" ]; then
        # Create a backup
        cp "$file" "$file.bak"
        
        # Remove profile sections using sed
        sed -i '' '/^\[profile\./,/^$/d' "$file"
        sed -i '' '/^\[profile\./,/^\[/d' "$file"
        
        echo "Removed profiles from $file"
    fi
}

# Remove from parser package
remove_profiles "packages/ddex-parser/Cargo.toml"

# Remove from bindings
remove_profiles "packages/ddex-parser/bindings/node/Cargo.toml"
remove_profiles "packages/ddex-parser/bindings/python/Cargo.toml"
remove_profiles "packages/ddex-parser/bindings/wasm/Cargo.toml"

echo "Profile removal complete!"
