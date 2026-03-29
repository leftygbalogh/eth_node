# Reth Environment Dry-Run Validation Script
# Purpose: Validate Reth prerequisites without downloading chain data
# Usage: .\scripts\reth_dryrun.ps1
# Task: T-009 (AC-017)

param(
    [switch]$Verbose
)

Write-Host ""
Write-Host "=== Reth Environment Readiness Dry-Run ===" -ForegroundColor Cyan
Write-Host "Validating prerequisites for Reth Sepolia sync..." -ForegroundColor Cyan
Write-Host ""

$allChecksPassed = $true

# =====================================================================
# CHECK 1: Rust Toolchain Version
# =====================================================================
Write-Host "[1/5] Checking Rust toolchain..." -ForegroundColor Yellow

try {
    $rustVersion = rustc --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  [OK] Rust installed: $rustVersion" -ForegroundColor Green
        
        # Extract version number (e.g., "rustc 1.75.0" -> 1.75)
        if ($rustVersion -match "rustc (\d+)\.(\d+)") {
            $majorVersion = [int]$matches[1]
            $minorVersion = [int]$matches[2]
            
            if ($majorVersion -gt 1 -or ($majorVersion -eq 1 -and $minorVersion -ge 70)) {
                Write-Host "  [OK] Version 1.70+ requirement satisfied" -ForegroundColor Green
            } else {
                Write-Host "  [FAIL] Rust version too old (1.70+ required)" -ForegroundColor Red
                Write-Host "     Update with: rustup update stable" -ForegroundColor Yellow
                $allChecksPassed = $false
            }
        }
    }
} catch {
    Write-Host "  [FAIL] Rust not found" -ForegroundColor Red
    Write-Host "     Install from: https://rustup.rs/" -ForegroundColor Yellow
    $allChecksPassed = $false
}

# =====================================================================
# CHECK 2: Disk Space
# =====================================================================
Write-Host ""
Write-Host "[2/5] Checking available disk space..." -ForegroundColor Yellow

try {
    $drive = Get-PSDrive -Name C
    $freeSpaceGB = [math]::Round($drive.Free / 1GB, 2)
    $totalSpaceGB = [math]::Round(($drive.Free + $drive.Used) / 1GB, 2)
    
    Write-Host "  Drive C: $freeSpaceGB GB free / $totalSpaceGB GB total" -ForegroundColor Cyan
    
    if ($freeSpaceGB -lt 190) {
        Write-Host "  [FAIL] CRITICAL: Free space below 190 GB minimum" -ForegroundColor Red
        Write-Host "     Reth sync will fail. Free up disk space before proceeding." -ForegroundColor Yellow
        $allChecksPassed = $false
    } elseif ($freeSpaceGB -lt 500) {
        Write-Host "  [WARN] Free space below 500 GB recommended" -ForegroundColor Yellow
        Write-Host "     Sync may succeed but monitor disk usage closely" -ForegroundColor Yellow
    } else {
        Write-Host "  [OK] Disk space adequate (500+ GB available)" -ForegroundColor Green
    }
} catch {
    Write-Host "  [FAIL] Unable to check disk space" -ForegroundColor Red
    $allChecksPassed = $false
}

# =====================================================================
# CHECK 3: Reth Installation
# =====================================================================
Write-Host ""
Write-Host "[3/5] Checking Reth installation..." -ForegroundColor Yellow

try {
    $rethVersion = reth --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  [OK] Reth installed: $rethVersion" -ForegroundColor Green
    } else {
        throw "Reth command failed"
    }
} catch {
    Write-Host "  [FAIL] Reth not found in PATH" -ForegroundColor Red
    Write-Host "     Installation options:" -ForegroundColor Yellow
    Write-Host "       1. Download binary: https://github.com/paradigmxyz/reth/releases" -ForegroundColor Yellow
    Write-Host "       2. Build from source: git clone https://github.com/paradigmxyz/reth.git && cd reth && cargo build --release" -ForegroundColor Yellow
    $allChecksPassed = $false
}

# =====================================================================
# CHECK 4: Network Connectivity
# =====================================================================
Write-Host ""
Write-Host "[4/5] Checking network connectivity..." -ForegroundColor Yellow

# Test 1: Basic internet connectivity
try {
    $pingResult = Test-Connection -ComputerName 8.8.8.8 -Count 2 -Quiet
    if ($pingResult) {
        Write-Host "  [OK] Internet connectivity confirmed (ping 8.8.8.8)" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] No internet connectivity" -ForegroundColor Red
        $allChecksPassed = $false
    }
} catch {
    Write-Host "  [FAIL] Unable to test connectivity" -ForegroundColor Red
    $allChecksPassed = $false
}

# Test 2: Sepolia RPC endpoint availability (public endpoint)
try {
    $sepoliaRpcUrl = "https://rpc.sepolia.org"
    $body = @{
        jsonrpc = "2.0"
        method = "eth_blockNumber"
        params = @()
        id = 1
    } | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri $sepoliaRpcUrl -Method Post -Body $body -ContentType "application/json" -TimeoutSec 5 -ErrorAction Stop
    
    if ($response.result) {
        $blockNumber = [Convert]::ToInt64($response.result, 16)
        Write-Host "  [OK] Sepolia RPC endpoint reachable (block: $blockNumber)" -ForegroundColor Green
    } else {
        Write-Host "  [WARN] Sepolia RPC returned unexpected response" -ForegroundColor Yellow
    }
} catch {
    Write-Host "  [WARN] Unable to reach Sepolia RPC endpoint" -ForegroundColor Yellow
    Write-Host "     This is not critical for local sync, but may indicate network issues" -ForegroundColor Gray
    if ($Verbose) {
        Write-Host "     Error: $($_.Exception.Message)" -ForegroundColor Gray
    }
}

# =====================================================================
# CHECK 5: Environment Variables
# =====================================================================
Write-Host ""
Write-Host "[5/5] Checking environment configuration..." -ForegroundColor Yellow

$envVarsConfigured = 0
$envVarsTotal = 3

if ($env:RETH_DATA_DIR) {
    Write-Host "  [OK] RETH_DATA_DIR set: $env:RETH_DATA_DIR" -ForegroundColor Green
    $envVarsConfigured++
} else {
    Write-Host "  [WARN] RETH_DATA_DIR not set (optional, but recommended)" -ForegroundColor Yellow
}

if ($env:RETH_CHAIN) {
    Write-Host "  [OK] RETH_CHAIN set: $env:RETH_CHAIN" -ForegroundColor Green
    $envVarsConfigured++
} else {
    Write-Host "  [WARN] RETH_CHAIN not set (optional, defaults to mainnet)" -ForegroundColor Yellow
}

if ($env:RUST_LOG) {
    Write-Host "  [OK] RUST_LOG set: $env:RUST_LOG" -ForegroundColor Green
    $envVarsConfigured++
} else {
    Write-Host "  [WARN] RUST_LOG not set (optional, defaults to error level)" -ForegroundColor Yellow
}

if ($envVarsConfigured -eq 0) {
    Write-Host "  [INFO] No Reth environment variables configured" -ForegroundColor Cyan
    Write-Host "     See docs/reth_readiness_checklist.md for configuration examples" -ForegroundColor Gray
}

# =====================================================================
# SUMMARY
# =====================================================================
Write-Host ""
Write-Host "=== Validation Summary ===" -ForegroundColor Cyan

if ($allChecksPassed) {
    Write-Host "[PASS] All critical checks passed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Your system meets the minimum requirements for Reth Sepolia sync." -ForegroundColor Green
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. Review configuration: docs/reth_readiness_checklist.md" -ForegroundColor White
    Write-Host "  2. Set environment variables (RETH_DATA_DIR, RETH_CHAIN)" -ForegroundColor White
    Write-Host "  3. Start sync: reth node --chain sepolia --http" -ForegroundColor White
    exit 0
} else {
    Write-Host "[FAIL] One or more checks failed!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Resolve the issues above before attempting Reth sync." -ForegroundColor Yellow
    Write-Host "See docs/reth_readiness_checklist.md for detailed troubleshooting." -ForegroundColor Yellow
    exit 1
}


# =====================================================================
# SUMMARY
# =====================================================================
Write-Host ""
Write-Host "=== Validation Summary ===" -ForegroundColor Cyan

if ($allChecksPassed) {
    Write-Host "[PASS] All critical checks passed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Your system meets the minimum requirements for Reth Sepolia sync." -ForegroundColor Green
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. Review configuration: docs/reth_readiness_checklist.md" -ForegroundColor White
    Write-Host "  2. Set environment variables (RETH_DATA_DIR, RETH_CHAIN)" -ForegroundColor White
    Write-Host "  3. Start sync: reth node --chain sepolia --http" -ForegroundColor White
    exit 0
} else {
    Write-Host "[FAIL] One or more checks failed!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Resolve the issues above before attempting Reth sync." -ForegroundColor Yellow
    Write-Host "See docs/reth_readiness_checklist.md for detailed troubleshooting." -ForegroundColor Yellow
    exit 1
}
