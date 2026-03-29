# Validate documentation: rustdoc compilation, doc tests, and markdown integrity.
# Exit 0 if all checks pass, exit 1 on any failure.

Write-Host "=== Documentation Validation ===" -ForegroundColor Cyan
Write-Host ""

# Check 1: Verify rustdoc compiles without errors (warnings are OK)
Write-Host "--- Check 1: Rustdoc compilation ---" -ForegroundColor Yellow
cargo doc --no-deps
if ($LASTEXITCODE -ne 0) {
    Write-Host "[FAIL] Rustdoc compilation failed" -ForegroundColor Red
    exit 1
}
Write-Host "[PASS] Rustdoc compiles cleanly" -ForegroundColor Green
Write-Host ""

# Check 2: Run doc tests
Write-Host "--- Check 2: Doc tests ---" -ForegroundColor Yellow
cargo test --doc
if ($LASTEXITCODE -ne 0) {
    Write-Host "[FAIL] Doc tests failed" -ForegroundColor Red
    exit 1
}

Write-Host "[PASS] Doc tests pass" -ForegroundColor Green
Write-Host ""

# Check 3: Verify critical markdown files exist and are non-empty
Write-Host "--- Check 3: Markdown file integrity ---" -ForegroundColor Yellow
$docs = @(
    "CLI_REFERENCE.md",
    "docs\LIBRARY_API_GUIDE.md",
    "README.md",
    "FORMAL_SPEC.md"
)

foreach ($doc in $docs) {
    if (-not (Test-Path $doc)) {
        Write-Host "[FAIL] Missing required doc: $doc" -ForegroundColor Red
        exit 1
    }
    
    if ((Get-Item $doc).Length -eq 0) {
        Write-Host "[FAIL] Empty doc file: $doc" -ForegroundColor Red
        exit 1
    }
    
    # Basic markdown syntax check (detect unclosed code fences)
    $content = Get-Content $doc -Raw
    $fenceCount = ([regex]::Matches($content, '^```', [Text.RegularExpressions.RegexOptions]::Multiline)).Count
    
    if ($fenceCount % 2 -ne 0) {
        Write-Host "[FAIL] Unclosed code fence in $doc (found $fenceCount backtick lines)" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "  [OK] $doc" -ForegroundColor Gray
}

Write-Host "[PASS] All markdown files present and valid" -ForegroundColor Green
Write-Host ""

# Check 4: Verify LIBRARY_API_GUIDE.md contains required sections
Write-Host "--- Check 4: Library API Guide completeness ---" -ForegroundColor Yellow
$guide = "docs\LIBRARY_API_GUIDE.md"
$guideContent = Get-Content $guide -Raw

$requiredSections = @(
    "Executor Module",
    "Transaction Simulation",
    "Contract Calls",
    "Anvil Comparison",
    "Decoder Module",
    "Standard Event Decoding",
    "Lossless Decoding",
    "ApprovalForAll Ambiguity",
    "Integration Examples"
)

foreach ($section in $requiredSections) {
    if ($guideContent -notmatch [regex]::Escape($section)) {
        Write-Host "[FAIL] Missing required section in $guide : $section" -ForegroundColor Red
        exit 1
    }
}

Write-Host "[PASS] Library API Guide complete" -ForegroundColor Green
Write-Host ""

# Check 5: Verify CLI_REFERENCE.md contains complex scenarios
Write-Host "--- Check 5: CLI Reference scenarios ---" -ForegroundColor Yellow
$cli = "CLI_REFERENCE.md"
$cliContent = Get-Content $cli -Raw

if ($cliContent -notmatch "Complex Scenarios") {
    Write-Host "[FAIL] Missing 'Complex Scenarios' section in $cli" -ForegroundColor Red
    exit 1
}

$requiredScenarios = @(
    "Scenario 1: Executor Pipeline",
    "Scenario 2: NFT Lifecycle",
    "Scenario 3: Multi-Contract"
)

foreach ($scenario in $requiredScenarios) {
    if ($cliContent -notmatch [regex]::Escape($scenario)) {
        Write-Host "[FAIL] Missing scenario in $cli : $scenario" -ForegroundColor Red
        exit 1
    }
}

Write-Host "[PASS] CLI Reference scenarios present" -ForegroundColor Green
Write-Host ""

Write-Host "=== All validation checks passed ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "To validate executable examples, run:" -ForegroundColor Yellow
Write-Host '  bash ./scripts/test-complete-docs.sh' -ForegroundColor Gray
exit 0
