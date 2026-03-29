#!/usr/bin/env bash
# Validate documentation: rustdoc compilation, doc tests, and markdown integrity.
# Exit 0 if all checks pass, exit 1 on any failure.

set -e  # Exit on first error

echo "=== Documentation Validation ==="
echo ""

# Check 1: Verify rustdoc compiles without errors
echo "--- Check 1: Rustdoc compilation ---"
cargo doc --no-deps 2>&1 | tee /tmp/cargo-doc.log

if grep -q "error:" /tmp/cargo-doc.log; then
    echo "❌ FAIL: Rustdoc compilation errors detected"
    exit 1
fi

echo "✅ PASS: Rustdoc compiles cleanly"
echo ""

# Check 2: Run doc tests
echo "--- Check 2: Doc tests ---"
cargo test --doc 2>&1 | tee /tmp/cargo-test-doc.log

if grep -qE "(test result: .* FAILED|error:)" /tmp/cargo-test-doc.log; then
    echo "❌ FAIL: Doc tests failed"
    exit 1
fi

echo "✅ PASS: Doc tests pass"
echo ""

# Check 3: Verify critical markdown files exist and are non-empty
echo "--- Check 3: Markdown file integrity ---"
DOCS=(
    "CLI_REFERENCE.md"
    "docs/LIBRARY_API_GUIDE.md"
    "README.md"
    "FORMAL_SPEC.md"
)

for doc in "${DOCS[@]}"; do
    if [[ ! -f "$doc" ]]; then
        echo "❌ FAIL: Missing required doc: $doc"
        exit 1
    fi
    
    if [[ ! -s "$doc" ]]; then
        echo "❌ FAIL: Empty doc file: $doc"
        exit 1
    fi
    
    # Basic markdown syntax check (detect unclosed code fences)
    FENCE_OPEN=$(grep -c '^```' "$doc" || true)
    if (( FENCE_OPEN % 2 != 0 )); then
        echo "❌ FAIL: Unclosed code fence in $doc (found $FENCE_OPEN backtick lines)"
        exit 1
    fi
    
    echo "  ✓ $doc"
done

echo "✅ PASS: All markdown files present and valid"
echo ""

# Check 4: Verify LIBRARY_API_GUIDE.md contains required sections
echo "--- Check 4: Library API Guide completeness ---"
GUIDE="docs/LIBRARY_API_GUIDE.md"

REQUIRED_SECTIONS=(
    "Executor Module"
    "Transaction Simulation"
    "Contract Calls"
    "Anvil Comparison"
    "Decoder Module"
    "Standard Event Decoding"
    "Lossless Decoding"
    "ApprovalForAll Ambiguity"
    "Integration Examples"
)

for section in "${REQUIRED_SECTIONS[@]}"; do
    if ! grep -q "$section" "$GUIDE"; then
        echo "❌ FAIL: Missing required section in $GUIDE: $section"
        exit 1
    fi
done

echo "✅ PASS: Library API Guide complete"
echo ""

# Check 5: Verify CLI_REFERENCE.md contains complex scenarios
echo "--- Check 5: CLI Reference scenarios ---"
CLI="CLI_REFERENCE.md"

if ! grep -q "Complex Scenarios" "$CLI"; then
    echo "❌ FAIL: Missing 'Complex Scenarios' section in $CLI"
    exit 1
fi

REQUIRED_SCENARIOS=(
    "Scenario 1: Executor Pipeline"
    "Scenario 2: NFT Lifecycle"
    "Scenario 3: Multi-Contract"
)

for scenario in "${REQUIRED_SCENARIOS[@]}"; do
    if ! grep -q "$scenario" "$CLI"; then
        echo "❌ FAIL: Missing scenario in $CLI: $scenario"
        exit 1
    fi
done

echo "✅ PASS: CLI Reference scenarios present"
echo ""

echo "=== All validation checks passed ✅ ==="
exit 0
