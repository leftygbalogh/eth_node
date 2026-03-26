#!/usr/bin/env bash
# capture-session.sh — wraps eth_node_cli in a recorded terminal session.
#
# Creates output/sessions/<timestamp>/ and records:
#   screen.log  : full terminal transcript (via 'script' command)
#   state.json  : JSON snapshot written by eth_node_cli --dump-state
#
# Spec ref: FORMAL_SPEC.md §5.1, §9 Interactive CLI diagnostics
# Task ref: T-003
#
# Usage:
#   ./scripts/capture-session.sh balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
#   ./scripts/capture-session.sh send --to 0xAbC... --value 1000000000000000000

set -euo pipefail

# ── Resolve workspace root (parent of scripts/) ───────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── Locate eth_node_cli binary ─────────────────────────────────────────────────
RELEASE_BIN="$WORK_ROOT/target/release/eth_node_cli"
DEBUG_BIN="$WORK_ROOT/target/debug/eth_node_cli"

if [[ -x "$RELEASE_BIN" ]]; then
    CLI_BIN="$RELEASE_BIN"
elif [[ -x "$DEBUG_BIN" ]]; then
    CLI_BIN="$DEBUG_BIN"
else
    echo "ERROR: eth_node_cli binary not found. Run 'cargo build' first." >&2
    exit 1
fi

# ── Create session artifact directory ─────────────────────────────────────────
TIMESTAMP="$(date '+%Y-%m-%d_%H-%M-%S')"
SESSION_DIR="$WORK_ROOT/output/sessions/$TIMESTAMP"
mkdir -p "$SESSION_DIR"

SCREEN_LOG="$SESSION_DIR/screen.log"
STATE_JSON="$SESSION_DIR/state.json"

echo "Session artifacts: $SESSION_DIR"
echo "Binary: $CLI_BIN"
echo ""

# ── Build command string ───────────────────────────────────────────────────────
if [[ $# -eq 0 ]]; then
    # No args: capture help output so the transcript is not empty.
    CMD="\"$CLI_BIN\" --help"
else
    # Shell-quote each argument to handle spaces/special chars safely.
    CMD="\"$CLI_BIN\""
    for arg in "$@"; do
        CMD="$CMD $(printf '%q' "$arg")"
    done
    CMD="$CMD --dump-state $(printf '%q' "$STATE_JSON")"
fi

# ── Run CLI and capture output via tee ────────────────────────────────────────
#    Works on Linux, macOS, and Git Bash (MINGW64) on Windows.
#    stdout + stderr are both written to screen.log and shown on the terminal.
# shellcheck disable=SC2086
bash -c "$CMD" 2>&1 | tee "$SCREEN_LOG" || true

# ── Report ─────────────────────────────────────────────────────────────────────
echo ""
echo "screen.log  : $SCREEN_LOG"
if [[ -f "$STATE_JSON" ]]; then
    echo "state.json  : $STATE_JSON"
else
    echo "state.json  : (not written — eth_node_cli did not produce one)"
fi
