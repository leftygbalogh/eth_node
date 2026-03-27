#!/usr/bin/env bash
# capture-multi.sh — run multiple eth_node_cli commands in one recorded session.
#
# Reads a plain-text command file where each non-blank, non-comment line is one
# subcommand with its arguments (same syntax as capture-session.sh).
# All steps share one session artifact directory and one screen.log.
#
# Behaviour:
#   - Aborts after the first failed step (exit code ≠ 0).
#   - Reports which step number failed and its exit code in the session footer.
#   - Writes state.json only for steps that succeed (named state-<N>.json).
#
# Usage:
#   ./scripts/capture-multi.sh <commands-file>
#
# Commands file format (save as e.g. scripts/my-scenario.txt):
#   # comment lines and blank lines are ignored
#   balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
#   send --private-key 0xac09... 0x7099... 1000000000000000000
#   balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
#   balance 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
#
# See CLI_REFERENCE.md §8 for full worked example.

set -euo pipefail

# ── Args ──────────────────────────────────────────────────────────────────────
if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <commands-file>" >&2
    exit 1
fi

COMMANDS_FILE="$1"

if [[ ! -f "$COMMANDS_FILE" ]]; then
    echo "ERROR: commands file not found: $COMMANDS_FILE" >&2
    exit 1
fi

# ── Resolve workspace root ────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── Locate binary ─────────────────────────────────────────────────────────────
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

# ── Session directory ─────────────────────────────────────────────────────────
TIMESTAMP="$(date '+%Y-%m-%d_%H-%M-%S')"
SESSION_DIR="$WORK_ROOT/output/sessions/$TIMESTAMP"
mkdir -p "$SESSION_DIR"

SCREEN_LOG="$SESSION_DIR/screen.log"

# Redirect all output to terminal + screen.log from here on.
exec > >(tee "$SCREEN_LOG") 2>&1

echo "=== Multi-step session ==="
echo "Session artifacts: $SESSION_DIR"
echo "Binary:            $CLI_BIN"
echo "Commands file:     $COMMANDS_FILE"
echo ""

# ── Probe / start Anvil ───────────────────────────────────────────────────────
anvil_ready() {
    curl -s -o /dev/null --max-time 1 \
        -X POST -H 'Content-Type: application/json' \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        http://127.0.0.1:8545
}

ANVIL_STARTED=0
if ! anvil_ready; then
    echo "Anvil not detected — starting it in the background..."
    anvil --silent &
    ANVIL_PID=$!
    ANVIL_STARTED=1
    for i in $(seq 1 10); do
        sleep 0.5
        if anvil_ready; then
            echo "Anvil ready (pid $ANVIL_PID)."
            break
        fi
    done
else
    echo "Anvil already running on 127.0.0.1:8545."
fi
echo ""

# ── Provision fixture ABI files ───────────────────────────────────────────────
for abi_src in "$WORK_ROOT/config/"*.abi.json; do
    [[ -e "$abi_src" ]] || continue
    abi_name="$(basename "$abi_src")"
    cp "$abi_src" "/tmp/$abi_name"
    echo "Provisioned fixture: /tmp/$abi_name"
done

# ── Parse command file and run each step ─────────────────────────────────────
STEP=0
TOTAL=0

# Count non-blank non-comment lines
while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%$'\r'}"          # strip Windows CR
    [[ -z "$line" || "$line" == \#* ]] && continue
    (( TOTAL++ )) || true
done < "$COMMANDS_FILE"

echo ""
echo "--- Running $TOTAL step(s) ---"
echo ""

set +e  # manage exit codes manually from here

while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%$'\r'}"          # strip Windows CR
    # Skip blank lines and comments.
    [[ -z "$line" || "$line" == \#* ]] && continue

    (( STEP++ )) || true
    STATE_JSON="$SESSION_DIR/state-$STEP.json"

    echo "--- Step $STEP/$TOTAL: $line ---"

    # shellcheck disable=SC2086
    bash -c "\"$CLI_BIN\" --dump-state $(printf '%q' "$STATE_JSON") $line"
    EXIT_CODE=$?

    if [[ $EXIT_CODE -ne 0 ]]; then
        echo ""
        echo "=== ABORTED at step $STEP/$TOTAL ==="
        echo "Command   : $line"
        echo "Exit code : $EXIT_CODE"
        echo "Screen log: $SCREEN_LOG"
        if [[ $ANVIL_STARTED -eq 1 ]]; then
            kill "$ANVIL_PID" 2>/dev/null || true
        fi
        exit "$EXIT_CODE"
    fi

    if [[ -f "$STATE_JSON" ]]; then
        echo "(state saved: state-$STEP.json)"
    fi
    echo ""
done < "$COMMANDS_FILE"

set -e

# ── Footer ────────────────────────────────────────────────────────────────────
echo "=== All $TOTAL step(s) completed successfully ==="
echo "Screen log: $SCREEN_LOG"
echo "State JSONs:"
for f in "$SESSION_DIR"/state-*.json; do
    [[ -e "$f" ]] && echo "  $f"
done

if [[ $ANVIL_STARTED -eq 1 ]]; then
    kill "$ANVIL_PID" 2>/dev/null || true
    echo "Anvil stopped (pid $ANVIL_PID)."
fi
