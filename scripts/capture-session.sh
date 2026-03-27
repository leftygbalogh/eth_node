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
#   ./scripts/capture-session.sh send --private-key 0xac09... 0x7099... 1000000000000000000
#   ./scripts/capture-session.sh tx-status 0x43aa...
#   ./scripts/capture-session.sh call --abi-file /tmp/stubtoken.abi.json 0x5FbD... balanceOf 0xf39...
#   ./scripts/capture-session.sh watch 0x5FbD...
#
# See CLI_REFERENCE.md for full examples and setup instructions.

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

# ── Start Anvil if not already listening on port 8545 ────────────────────────
ANVIL_STARTED=0
if ! curl -sf -o /dev/null --max-time 1 http://127.0.0.1:8545; then
    echo "Anvil not detected — starting it in the background..."
    anvil --silent &
    ANVIL_PID=$!
    ANVIL_STARTED=1
    # Wait up to 5 seconds for Anvil to be ready.
    for i in $(seq 1 10); do
        sleep 0.5
        if curl -sf -o /dev/null --max-time 1 http://127.0.0.1:8545; then
            echo "Anvil ready (pid $ANVIL_PID)."
            break
        fi
    done
else
    echo "Anvil already running on 127.0.0.1:8545."
fi
echo ""

# ── Build command string ───────────────────────────────────────────────────────
if [[ $# -eq 0 ]]; then
    # No args: capture help output so the transcript is not empty.
    CMD="\"$CLI_BIN\" --help"
else
    # --dump-state is a global flag; it must come BEFORE the subcommand.
    CMD="\"$CLI_BIN\" --dump-state $(printf '%q' "$STATE_JSON")"
    for arg in "$@"; do
        CMD="$CMD $(printf '%q' "$arg")"
    done
fi

# ── Run CLI and capture output via tee ────────────────────────────────────────
#    Works on Linux, macOS, and Git Bash (MINGW64) on Windows.
#    stdout + stderr are both written to screen.log and shown on the terminal.
# shellcheck disable=SC2086
set +e
bash -c "$CMD" 2>&1 | tee "$SCREEN_LOG"
EXIT_CODE="${PIPESTATUS[0]}"
set -e

# ── Report ─────────────────────────────────────────────────────────────────────
echo ""
echo "=== Session complete ==="
echo "Exit code   : $EXIT_CODE"
echo "Screen log  : $SCREEN_LOG"
if [[ -f "$STATE_JSON" ]]; then
    echo "State JSON  : $STATE_JSON"
else
    echo "State JSON  : (not written — command exited with error)"
fi

# ── Stop Anvil if we started it ───────────────────────────────────────────────
if [[ $ANVIL_STARTED -eq 1 ]]; then
    kill "$ANVIL_PID" 2>/dev/null || true
    echo "Anvil stopped (pid $ANVIL_PID)."
fi

exit "$EXIT_CODE"
