# Reth Environment Readiness Checklist

## Purpose

This checklist documents prerequisites for future Reth Sepolia sync experiments. **Note:** Phase 2 does NOT include actual Reth integration — this is preparation only.

## Hardware Requirements

### Minimum Disk Space

- **Sepolia full sync**: ~100-150 GB (as of March 2026)
- **Recommended free space**: 500 GB to allow for growth and pruning overhead
- **Critical threshold**: 190 GB minimum for initial feasibility testing on constrained systems

### Disk Space Validation Command (PowerShell)

```powershell
# Check available space on C: drive (adjust drive letter as needed)
$drive = Get-PSDrive -Name C
$freeSpaceGB = [math]::Round($drive.Free / 1GB, 2)
$usedSpaceGB = [math]::Round($drive.Used / 1GB, 2)
$totalSpaceGB = [math]::Round(($drive.Free + $drive.Used) / 1GB, 2)

Write-Host "Drive C: Status:"
Write-Host "  Total: $totalSpaceGB GB"
Write-Host "  Used:  $usedSpaceGB GB"
Write-Host "  Free:  $freeSpaceGB GB"

if ($freeSpaceGB -lt 190) {
    Write-Host "❌ WARNING: Free space below 190 GB minimum threshold" -ForegroundColor Red
    Write-Host "   Reth sync will likely fail or require frequent pruning" -ForegroundColor Red
} elseif ($freeSpaceGB -lt 500) {
    Write-Host "⚠️  CAUTION: Free space below 500 GB recommended" -ForegroundColor Yellow
    Write-Host "   Monitor disk usage during sync" -ForegroundColor Yellow
} else {
    Write-Host "✅ Disk space adequate for Reth Sepolia sync" -ForegroundColor Green
}
```

### System Requirements

- **CPU**: 4+ cores recommended (2 cores minimum)
- **RAM**: 8 GB minimum, 16 GB recommended
- **Network**: Stable broadband connection (sync bandwidth ~1-5 Mbps sustained)
- **OS**: Windows 10/11, Linux (Ubuntu 20.04+), or macOS 12+

---

## Environment Configuration

### Required Environment Variables

Create a `.env` file or set system environment variables:

```bash
# Reth data directory (adjust path for your system)
RETH_DATA_DIR=C:\Users\<YourUsername>\AppData\Local\reth\sepolia

# Chain selection (sepolia for testnet)
RETH_CHAIN=sepolia

# RPC endpoint URL (default listens on localhost)
RETH_RPC_URL=http://127.0.0.1:8545

# Optional: Custom P2P port (default 30303)
# RETH_P2P_PORT=30303

# Optional: Log level (error, warn, info, debug, trace)
# RUST_LOG=info
```

### PowerShell Configuration Example

```powershell
# Set environment variables for current session
$env:RETH_DATA_DIR = "C:\Users\$env:USERNAME\AppData\Local\reth\sepolia"
$env:RETH_CHAIN = "sepolia"
$env:RETH_RPC_URL = "http://127.0.0.1:8545"
$env:RUST_LOG = "info"

# Persist for future sessions (optional)
[System.Environment]::SetEnvironmentVariable('RETH_DATA_DIR', $env:RETH_DATA_DIR, 'User')
[System.Environment]::SetEnvironmentVariable('RETH_CHAIN', $env:RETH_CHAIN, 'User')
```

---

## Reth Installation Guide

### Prerequisites

1. **Rust toolchain** (stable channel, 1.70+ required)
   ```powershell
   # Check Rust version
   rustc --version
   
   # If not installed, download from https://rustup.rs/
   # Or install via:
   # Invoke-WebRequest -Uri https://win.rustup.rs/ -OutFile rustup-init.exe
   # .\rustup-init.exe
   ```

2. **Git** (for building from source)
   ```powershell
   git --version
   ```

3. **Build tools** (Windows: Visual Studio Build Tools, Linux: build-essential)

### Installation Option 1: Pre-built Binary (Recommended)

```powershell
# Download latest Reth release from GitHub
# https://github.com/paradigmxyz/reth/releases

# Example: Download Windows binary (adjust version as needed)
$version = "v0.2.0-beta.5"  # Check latest release
$url = "https://github.com/paradigmxyz/reth/releases/download/$version/reth-$version-x86_64-pc-windows-msvc.zip"

# Create installation directory
New-Item -ItemType Directory -Force -Path "C:\Program Files\reth"

# Download and extract (manual step: extract zip to C:\Program Files\reth)
Start-Process $url

# Add to PATH (run as Administrator)
$existingPath = [System.Environment]::GetEnvironmentVariable('Path', 'Machine')
$newPath = "C:\Program Files\reth"
if ($existingPath -notlike "*$newPath*") {
    [System.Environment]::SetEnvironmentVariable('Path', "$existingPath;$newPath", 'Machine')
    Write-Host "Added $newPath to system PATH. Restart terminal to apply." -ForegroundColor Green
}

# Verify installation
reth --version
```

### Installation Option 2: Build from Source

```powershell
# Clone Reth repository
git clone https://github.com/paradigmxyz/reth.git
cd reth

# Checkout latest stable release (optional, or use main branch)
git checkout v0.2.0-beta.5

# Build release binary (takes 10-30 minutes depending on system)
cargo build --release

# Binary location: target/release/reth.exe
.\target\release\reth.exe --version

# Copy to PATH location (optional)
Copy-Item .\target\release\reth.exe -Destination "C:\Program Files\reth\reth.exe" -Force
```

---

## Dry-Run Validation Script

**Location**: `scripts/reth_dryrun.ps1`

**Purpose**: Validates environment prerequisites without downloading chain data.

**Usage**:
```powershell
cd C:\Users\<YourUsername>\Documents\VScode\ethereum_node_rust
.\scripts\reth_dryrun.ps1
```

**Checks performed**:
1. Rust toolchain version (1.70+)
2. Available disk space (190 GB minimum, 500 GB recommended)
3. Reth binary installation (`reth --version`)
4. Network connectivity (ping public DNS, check Sepolia RPC endpoint availability)
5. Environment variable configuration

---

## Running Reth Sepolia Sync

### Start Sync Process

```powershell
# Ensure environment variables are set (see Environment Configuration above)
$env:RETH_DATA_DIR = "C:\Users\$env:USERNAME\AppData\Local\reth\sepolia"
$env:RETH_CHAIN = "sepolia"
$env:RUST_LOG = "info"

# Start Reth node with Sepolia testnet
reth node `
    --chain sepolia `
    --datadir $env:RETH_DATA_DIR `
    --http `
    --http.addr 127.0.0.1 `
    --http.port 8545 `
    --http.api "eth,net,web3"

# Expected output:
# - P2P connection establishment
# - Block sync progress (syncing from genesis to tip)
# - RPC server listening on http://127.0.0.1:8545
```

### Monitor Sync Progress

```powershell
# In a separate terminal, query sync status via RPC
$body = @{
    jsonrpc = "2.0"
    method = "eth_syncing"
    params = @()
    id = 1
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://127.0.0.1:8545" -Method Post -Body $body -ContentType "application/json"

# Response:
# - `false` if fully synced
# - Object with `currentBlock`, `highestBlock`, `startingBlock` if syncing
```

### Stop Sync Process

```powershell
# Graceful shutdown: Press Ctrl+C in Reth terminal
# Or kill process by PID:
Get-Process reth | Stop-Process
```

---

## Rollback Procedure

### When to Roll Back

- Disk space exhausted (< 10 GB free)
- Sync stalled for >24 hours without progress
- Corrupted database errors
- Need to restart with different configuration

### Rollback Steps

1. **Stop Reth process**
   ```powershell
   Get-Process reth | Stop-Process
   ```

2. **Delete data directory**
   ```powershell
   # WARNING: This deletes all synced data (100+ GB). Confirm before running.
   $dataDir = $env:RETH_DATA_DIR
   if (Test-Path $dataDir) {
       Remove-Item -Path $dataDir -Recurse -Force
       Write-Host "Deleted Reth data directory: $dataDir" -ForegroundColor Yellow
   } else {
       Write-Host "Data directory not found: $dataDir" -ForegroundColor Green
   }
   ```

3. **Verify deletion**
   ```powershell
   Test-Path $env:RETH_DATA_DIR
   # Should return False
   ```

4. **Restart sync** (if desired)
   ```powershell
   # Re-run sync command from "Running Reth Sepolia Sync" section
   reth node --chain sepolia --datadir $env:RETH_DATA_DIR --http
   ```

---

## Troubleshooting

### Common Issues

1. **"reth: command not found"**
   - Solution: Add Reth binary location to PATH environment variable
   - Verify: `reth --version`

2. **"Disk space exhausted" during sync**
   - Solution: Follow rollback procedure, free up disk space, restart
   - Prevention: Monitor disk usage with `Get-PSDrive`

3. **Sync stalled / no progress**
   - Check network connectivity: `ping 8.8.8.8`
   - Check peer count: Query `net_peerCount` via RPC
   - Restart Reth process if peer count is 0

4. **"Database corruption" errors**
   - Solution: Follow rollback procedure to delete and re-sync

5. **High CPU/memory usage**
   - Expected during sync (4-8 GB RAM typical)
   - Limit parallelism with `--max-parallel-tx-downloads` flag if needed

---

## Phase 2 Status

**Current state**: Documentation only — no Reth integration in Phase 2

**Reason for deferral**: Disk space and sync time requirements exceed Phase 2 learning objectives. Phase 2 focuses on local EVM simulation (revm) and upstream alloy contribution.

**Future work**: Reth integration deferred to Phase 3 or standalone learning experiment when:
- 500+ GB free disk space available
- Time available for multi-hour sync (Sepolia: 4-8 hours typical)
- Learning objectives shift to full-node operations and state management

---

## References

- **Reth documentation**: https://paradigmxyz.github.io/reth/
- **Reth repository**: https://github.com/paradigmxyz/reth
- **Sepolia testnet info**: https://sepolia.dev/
- **Ethereum JSON-RPC API**: https://ethereum.org/en/developers/docs/apis/json-rpc/

---

**Last updated**: 2026-03-29  
**Task**: T-009 (Phase 2 Track A)  
**Acceptance criteria**: AC-016 (runnable checklist), AC-017 (dry-run validation), AC-018 (rollback procedure)
