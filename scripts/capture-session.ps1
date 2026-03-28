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
    .\scripts\capture-session.ps1 --stop-anvil

.EXAMPLE
    .\scripts\capture-session.ps1 balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

.EXAMPLE
    .\scripts\capture-session.ps1 send --to 0xAbC... --value 1000000000000000000

.EXAMPLE
    .\scripts\capture-session.ps1 decode-receipt 0x43aa...
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
$ManagedAnvilDir = Join-Path $WorkRoot 'output\anvil'
$ManagedAnvilPidFile = Join-Path $ManagedAnvilDir 'managed.pid'

New-Item -ItemType Directory -Path $ManagedAnvilDir -Force | Out-Null

function Test-AnvilReady {
    try {
        $body = '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'
        Invoke-WebRequest -Uri 'http://127.0.0.1:8545' -Method Post -ContentType 'application/json' -Body $body -UseBasicParsing -TimeoutSec 1 | Out-Null
        return $true
    } catch {
        return $false
    }
}

function Clear-StaleManagedAnvilPid {
    if (-not (Test-Path $ManagedAnvilPidFile)) {
        return
    }

    $pidText = (Get-Content -Path $ManagedAnvilPidFile -Raw).Trim()
    if ([string]::IsNullOrWhiteSpace($pidText)) {
        Remove-Item -Path $ManagedAnvilPidFile -Force -ErrorAction SilentlyContinue
        return
    }

    $managedPid = 0
    if (-not [int]::TryParse($pidText, [ref]$managedPid)) {
        Remove-Item -Path $ManagedAnvilPidFile -Force -ErrorAction SilentlyContinue
        return
    }

    if (-not (Get-Process -Id $managedPid -ErrorAction SilentlyContinue)) {
        Remove-Item -Path $ManagedAnvilPidFile -Force -ErrorAction SilentlyContinue
    }
}

function Stop-ManagedAnvil {
    Clear-StaleManagedAnvilPid

    if (-not (Test-Path $ManagedAnvilPidFile)) {
        Write-Host 'No managed Anvil instance is currently tracked.'
        Write-Host 'If you started Anvil manually, stop it in that terminal with Ctrl-C.'
        return
    }

    $managedPid = (Get-Content -Path $ManagedAnvilPidFile -Raw).Trim()
    Stop-Process -Id ([int]$managedPid) -Force -ErrorAction SilentlyContinue
    Remove-Item -Path $ManagedAnvilPidFile -Force -ErrorAction SilentlyContinue
    Write-Host "Stopped managed Anvil (pid $managedPid)."
}

if ($CliArgs.Count -gt 0 -and $CliArgs[0] -eq '--stop-anvil') {
    Stop-ManagedAnvil
    exit 0
}

Clear-StaleManagedAnvilPid

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

# ── Write all subsequent output to both terminal and screen.log ───────────────
New-Item -ItemType File -Path $ScreenLog -Force | Out-Null

function Write-SessionLine {
    param(
        [AllowEmptyString()]
        [string]$Text
    )

    $Text | Tee-Object -FilePath $ScreenLog -Append
}

Write-SessionLine "Session artifacts: $SessionDir"
Write-SessionLine "Binary: $CliBin"
Write-SessionLine ""

# ── Start Anvil if not already listening on port 8545 ─────────────────────────
$AnvilStarted = $false
$AnvilPid = $null
if (-not (Test-AnvilReady)) {
    Write-SessionLine 'Anvil not detected - starting it in the background...'
    $anvilProcess = Start-Process -FilePath 'anvil' -ArgumentList '--silent' -PassThru -WindowStyle Hidden
    $AnvilPid = $anvilProcess.Id
    $AnvilStarted = $true

    for ($i = 0; $i -lt 10; $i++) {
        Start-Sleep -Milliseconds 500
        if (Test-AnvilReady) {
            Write-SessionLine "Anvil ready (pid $AnvilPid)."
            Set-Content -Path $ManagedAnvilPidFile -Value $AnvilPid
            break
        }
    }
} else {
    Write-SessionLine 'Anvil already running on 127.0.0.1:8545.'
}

Write-SessionLine ''

if ($CliArgs.Count -eq 0) {
    Write-SessionLine "Command: $CliBin --help"
    & $CliBin --help 2>&1 | ForEach-Object { Write-SessionLine $_.ToString() }
} else {
    Write-SessionLine "Command: $CliBin --dump-state $StateJson $($CliArgs -join ' ')"
    & $CliBin --dump-state $StateJson @CliArgs 2>&1 | ForEach-Object { Write-SessionLine $_.ToString() }
}

$ExitCode = if ($null -ne $LASTEXITCODE) { $LASTEXITCODE } else { 0 }

# ── Report ─────────────────────────────────────────────────────────────────────
Write-SessionLine ""
Write-SessionLine "Exit code   : $ExitCode"
Write-SessionLine "screen.log  : $ScreenLog"
if (Test-Path $StateJson) {
    Write-SessionLine "state.json  : $StateJson"
} else {
    Write-SessionLine "state.json  : (not written - eth_node_cli did not produce one)"
}

if ($AnvilStarted -and $null -ne $AnvilPid) {
    Write-SessionLine "Anvil left running (pid $AnvilPid)."
    Write-SessionLine 'Stop it later with: .\scripts\capture-session.ps1 --stop-anvil'
}

exit $ExitCode
