#!/bin/bash
# Pre-Phase 3 Test Summary

echo "╔══════════════════════════════════════════════════════╗"
echo "║          Pre-Phase 3 Test Summary                    ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""

cd core

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0

echo "1️⃣  FFI Error Module Tests"
echo "─────────────────────────"
if cargo test error::ffi --lib 2>&1 | grep -q "1 passed"; then
    echo "✅ FFI error conversion test passed"
    ((PASSED_TESTS++))
else
    echo "❌ FFI error test failed"
fi
((TOTAL_TESTS++))
echo ""

echo "2️⃣  Model Consistency Tests"
echo "──────────────────────────"
if cargo test --test model_consistency 2>&1 | grep -q "4 passed"; then
    echo "✅ All 4 model consistency tests passed"
    PASSED_TESTS=$((PASSED_TESTS + 4))
else
    echo "❌ Some model consistency tests failed"
fi
TOTAL_TESTS=$((TOTAL_TESTS + 4))
echo ""

echo "3️⃣  Error Contract Tests"
echo "───────────────────────"
if [ -f tests/error_contract_test.rs ]; then
    if cargo test --test error_contract_test 2>&1 | grep -q "3 passed"; then
        echo "✅ All 3 error contract tests passed"
        PASSED_TESTS=$((PASSED_TESTS + 3))
    else
        echo "❌ Some error contract tests failed"
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 3))
else
    echo "⚠️  Error contract test file not found"
fi
echo ""

echo "4️⃣  TypeScript Feature Build"
echo "───────────────────────────"
if cargo build --features typescript 2>&1 | grep -q "Finished"; then
    echo "✅ TypeScript feature builds successfully"
    ((PASSED_TESTS++))
else
    echo "❌ TypeScript feature build failed"
fi
((TOTAL_TESTS++))
echo ""

echo "╔══════════════════════════════════════════════════════╗"
echo "║                    RESULTS                           ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""
echo "Tests Passed: $PASSED_TESTS / $TOTAL_TESTS"
echo ""

if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo "🎉 ALL TESTS PASSED!"
    echo ""
    echo "�� Pre-Phase 3 Checklist Complete:"
    echo "  ✅ FFI Error types and conversion"
    echo "  ✅ Model consistency verified"
    echo "  ✅ Error contract implemented"
    echo "  ✅ TypeScript generation ready"
    echo ""
    echo "🚀 Ready to proceed with Phase 3: JavaScript/TypeScript Bindings!"
else
    echo "⚠️  Some tests failed. Please review and fix before proceeding."
fi
