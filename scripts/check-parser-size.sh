# scripts/check-parser-size.sh
#!/bin/bash
# Check the size of the ddex-parser npm package

set -e

echo "📊 Checking ddex-parser bundle sizes"
echo "===================================="

cd packages/ddex-parser/bindings/node

# JavaScript bundle
if [ -d "dist" ]; then
  printf "\n📦 JavaScript bundle:\n"
  for file in dist/*.js dist/*.d.ts; do
    if [ -f "$file" ]; then
      SIZE=$(ls -lh "$file" | awk '{print $5}')
      printf "  %s: %s\n" "$(basename $file)" "$SIZE"
    fi
  done
  TOTAL_JS=$(du -sh dist | cut -f1)
  printf "  Total: %s\n" "$TOTAL_JS"
fi

# WASM module
WASM_PATH="../wasm/pkg/ddex_parser_bg.wasm"
if [ -f "$WASM_PATH" ]; then
  printf "\n🎯 WASM module:\n"
  WASM_SIZE=$(stat -f%z "$WASM_PATH" 2>/dev/null || stat -c%s "$WASM_PATH" 2>/dev/null || echo "0")
  WASM_KB=$((WASM_SIZE / 1024))
  printf "  Size: %dKB\n" "$WASM_KB"
  
  if [ $WASM_KB -gt 500 ]; then
    printf "  ⚠️ WARNING: Exceeds 500KB target!\n"
  elif [ $WASM_KB -gt 0 ]; then
    printf "  ✓ Under 500KB target\n"
  fi
else
  printf "\n⚠️ WASM not built yet\n"
fi

# Total package size
printf "\n📦 Total package size:\n"
npm pack --dry-run 2>&1 | grep "package size:" | sed 's/npm notice /  /'

cd - > /dev/null