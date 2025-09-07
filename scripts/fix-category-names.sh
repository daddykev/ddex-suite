#!/bin/bash

echo "Fixing FFIErrorCategory names..."

# Replace Parsing with XmlParsing (or another appropriate variant)
sed -i '' 's/FFIErrorCategory::Parsing/FFIErrorCategory::XmlParsing/g' packages/ddex-parser/src/error/ffi.rs

echo "Category names fixed!"
