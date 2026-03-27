#Requires -Version 5.1
<#
.SYNOPSIS
    Wraps an eth_node_cli command in a recorded PowerShell transcript session.

.DESCRIPTION
    Creates output/sessions/<timestamp>/ and records:
      - screen.log   : full terminal transcript (Start-Transcript)
      - state.json   : JSON snapshot written by eth_node_cli --dump-state

    Spec ref: FORMAL_SPEC.md §5.1, §9 Interactive CLI diagnostics
    Task ref: T-003

.PARAMETER CliArgs
    All arguments to pass through to eth_node_cli (subcommand + flags).

.EXAMPLE
    .\scripts\capture-session.ps1 balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

.EXAMPLE
    .\scripts\capture-session.ps1 send --to 0xAbC... --value 1000000000000000000
#>

[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CliArgs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ── Resolve workspace root (parent of scripts/) ───────────────────────────────
$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Path
$WorkRoot   = Split-Path -Parent $ScriptDir

# ── Locate eth_node_cli binary ─────────────────────────────────────────────────
# Prefer release build; fall back to debug.
$ReleaseBin = Join-Path $WorkRoot 'target\release\eth_node_cli.exe'
$DebugBin   = Join-Path $WorkRoot 'target\debug\eth_node_cli.exe'

if (Test-Path $ReleaseBin) {
    $CliBin = $ReleaseBin
} elseif (Test-Path $DebugBin) {
    $CliBin = $DebugBin
} else {
    Write-Error "eth_node_cli binary not found. Run 'cargo build' first."
    exit 1
}

# ── Create session artifact directory ─────────────────────────────────────────
$Timestamp  = Get-Date -Format 'yyyy-MM-dd_HH-mm-ss'
$SessionDir = Join-Path $WorkRoot "output\sessions\$Timestamp"
New-Item -ItemType Directory -Path $SessionDir -Force | Out-Null

$ScreenLog  = Join-Path $SessionDir 'screen.log'
$StateJson  = Join-Path $SessionDir 'state.json'

Write-Host "Session artifacts: $SessionDir"
Write-Host "Binary: $CliBin"
Write-Host ""

# ── Start transcript ───────────────────────────────────────────────────────────
Start-Transcript -Path $ScreenLog -Append | Out-Null

try {
    if ($CliArgs.Count -eq 0) {
        # No args: print help so the transcript is not empty.
        & $CliBin --help
    } else {
        & $CliBin --dump-state $StateJson @CliArgs
    }
} finally {
    Stop-Transcript | Out-Null
}

# ── Report ─────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "screen.log  : $ScreenLog"
if (Test-Path $StateJson) {
    Write-Host "state.json  : $StateJson"
} else {
    Write-Host "state.json  : (not written - eth_node_cli did not produce one)"
}
