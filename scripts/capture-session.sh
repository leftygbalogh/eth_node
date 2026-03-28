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
#   ./scripts/capture-session.sh --stop-anvil
#   ./scripts/capture-session.sh balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
#   ./scripts/capture-session.sh send --private-key 0xac09... 0x7099... 1000000000000000000
#   ./scripts/capture-session.sh tx-status 0x43aa...
#   ./scripts/capture-session.sh decode-receipt 0x43aa...
#   ./scripts/capture-session.sh call --abi-file /tmp/stubtoken.abi.json 0x5FbD... balanceOf 0xf39...
#   ./scripts/capture-session.sh watch 0x5FbD...
#
# See CLI_REFERENCE.md for full examples and setup instructions.

set -euo pipefail

# ── Resolve workspace root (parent of scripts/) ───────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

MANAGED_ANVIL_DIR="$WORK_ROOT/output/anvil"
MANAGED_ANVIL_PID_FILE="$MANAGED_ANVIL_DIR/managed.pid"

mkdir -p "$MANAGED_ANVIL_DIR"

is_windows_shell() {
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*) return 0 ;;
        *) return 1 ;;
    esac
}

managed_anvil_running() {
    [[ -f "$MANAGED_ANVIL_PID_FILE" ]] || return 1

    local pid
    pid="$(<"$MANAGED_ANVIL_PID_FILE")"
    [[ -n "$pid" ]] || return 1

    if is_windows_shell; then
        tasklist.exe /FI "PID eq $pid" 2>/dev/null | grep -q "$pid"
    else
        kill -0 "$pid" 2>/dev/null
    fi
}

clear_stale_managed_pid() {
    if [[ -f "$MANAGED_ANVIL_PID_FILE" ]] && ! managed_anvil_running; then
        rm -f "$MANAGED_ANVIL_PID_FILE"
    fi
}

stop_managed_anvil() {
    clear_stale_managed_pid

    if [[ ! -f "$MANAGED_ANVIL_PID_FILE" ]]; then
        echo "No managed Anvil instance is currently tracked."
        echo "If you started Anvil manually, stop it in that terminal with Ctrl-C."
        return 0
    fi

    local pid
    pid="$(<"$MANAGED_ANVIL_PID_FILE")"

    if is_windows_shell; then
        taskkill.exe /PID "$pid" /T /F >/dev/null 2>&1 || true
    else
        kill "$pid" 2>/dev/null || true
    fi

    rm -f "$MANAGED_ANVIL_PID_FILE"
    echo "Stopped managed Anvil (pid $pid)."
}

if [[ ${1-} == "--stop-anvil" ]]; then
    stop_managed_anvil
    exit 0
fi

clear_stale_managed_pid

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

# ── Create session artifact directory (silent — before logging starts) ──────
TIMESTAMP="$(date '+%Y-%m-%d_%H-%M-%S')"
SESSION_DIR="$WORK_ROOT/output/sessions/$TIMESTAMP"
mkdir -p "$SESSION_DIR"

SCREEN_LOG="$SESSION_DIR/screen.log"
STATE_JSON="$SESSION_DIR/state.json"

# ── Redirect ALL subsequent output to both terminal and screen.log ────────────
# This captures the full session: header, Anvil status, CLI output, footer.
exec > >(tee "$SCREEN_LOG") 2>&1

echo "Session artifacts: $SESSION_DIR"
echo "Binary: $CLI_BIN"
echo ""

# ── Probe Anvil with a real JSON-RPC call (bare GET returns non-2xx)
# so curl -f wrongly reports failure on a live Anvil instance) ────────────────
anvil_ready() {
    curl -s -o /dev/null --max-time 1 \
        -X POST -H 'Content-Type: application/json' \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        http://127.0.0.1:8545
}

# ── Start Anvil if not already listening on port 8545 ───────────────────────
ANVIL_STARTED=0
if ! anvil_ready; then
    echo "Anvil not detected — starting it in the background..."
    anvil --silent &
    ANVIL_PID=$!
    ANVIL_STARTED=1
    # Wait up to 5 seconds for Anvil to be ready.
    for i in $(seq 1 10); do
        sleep 0.5
        if anvil_ready; then
            echo "Anvil ready (pid $ANVIL_PID)."
            printf '%s\n' "$ANVIL_PID" > "$MANAGED_ANVIL_PID_FILE"
            break
        fi
    done
else
    echo "Anvil already running on 127.0.0.1:8545."
fi
echo ""

# ── Provision fixture ABI files ───────────────────────────────────────────────
# Copy every config/*.abi.json into /tmp/ so that CLI_REFERENCE.md examples
# using --abi-file /tmp/<name>.abi.json work out of the box.
for abi_src in "$WORK_ROOT/config/"*.abi.json; do
    [[ -e "$abi_src" ]] || continue           # skip if glob matched nothing
    abi_name="$(basename "$abi_src")"
    abi_dst="/tmp/$abi_name"
    cp "$abi_src" "$abi_dst"
    echo "Provisioned fixture: $abi_dst"
done

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

# ── Run CLI ───────────────────────────────────────────────────────────────────
#    stdout + stderr already flow to terminal AND screen.log via the exec
#    redirect above. No extra tee needed here.
# shellcheck disable=SC2086
set +e
bash -c "$CMD"
EXIT_CODE="$?"
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

# ── Keep Anvil running for chained/manual follow-up commands ──────────────────
if [[ $ANVIL_STARTED -eq 1 ]]; then
    echo "Anvil left running (pid $ANVIL_PID)."
    echo "Stop it later with: ./scripts/capture-session.sh --stop-anvil"
fi

exit "$EXIT_CODE"
