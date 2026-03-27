#!/usr/bin/env bash
# test-capture-session-balance.sh
#
# Automated check for: capture-session.sh balance produces a clean session
# with no spurious errors when Anvil is already running.
#
# Expected (after fix):
#   - screen.log contains no "Error:" lines
#   - screen.log contains "Balance:"
#   - state.json exists and contains "balance_wei"
#   - exit code from capture-session.sh is 0
#
# TDD workflow:
#   Run this script BEFORE fixing capture-session.sh to confirm it reproduces
#   the problem. Run again after the fix to confirm all assertions pass.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK_ROOT="$(dirname "$SCRIPT_DIR")"

CAPTURE="$SCRIPT_DIR/capture-session.sh"
ADDR="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

PASS=0
FAIL=0

ok()   { echo "  PASS: $*"; ((PASS++)) || true; }
fail() { echo "  FAIL: $*"; ((FAIL++)) || true; }

echo "=== test-capture-session-balance ==="
echo ""

# ── Ensure Anvil is running before the test ───────────────────────────────────
ANVIL_PID=""
anvil_ready() {
    # Use a real JSON-RPC probe — bare GET returns non-2xx and fools curl -f.
    curl -s -o /dev/null --max-time 1 \
        -X POST -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        http://127.0.0.1:8545
}

if anvil_ready; then
    echo "Anvil already running on 127.0.0.1:8545."
else
    echo "Starting Anvil for this test..."
    anvil --silent &
    ANVIL_PID="$!"
    for i in $(seq 1 10); do
        sleep 0.5
        if anvil_ready; then
            echo "Anvil ready (pid $ANVIL_PID)."
            break
        fi
    done
    if ! anvil_ready; then
        echo "ERROR: Anvil failed to start." >&2
        exit 1
    fi
fi
echo ""

# ── Run capture-session.sh and capture its output ─────────────────────────────
set +e
OUTPUT="$("$CAPTURE" balance "$ADDR" 2>&1)"
CAPTURE_EXIT="$?"
set -e

echo "--- capture-session.sh output ---"
echo "$OUTPUT"
echo "---------------------------------"
echo ""

# ── Find the session directory from output ────────────────────────────────────
SESSION_DIR="$(echo "$OUTPUT" | grep '^Session artifacts:' | sed 's/Session artifacts: //')"
SCREEN_LOG="$SESSION_DIR/screen.log"
STATE_JSON="$SESSION_DIR/state.json"

# ── Assertions ────────────────────────────────────────────────────────────────

# 1. Exit code must be 0
if [[ "$CAPTURE_EXIT" -eq 0 ]]; then
    ok "exit code is 0"
else
    fail "exit code is $CAPTURE_EXIT (expected 0)"
fi

# 2. screen.log must exist
if [[ -f "$SCREEN_LOG" ]]; then
    ok "screen.log exists: $SCREEN_LOG"
else
    fail "screen.log not found at: $SCREEN_LOG"
fi

# 3. screen.log must contain "Balance:"
if [[ -f "$SCREEN_LOG" ]] && grep -q "Balance:" "$SCREEN_LOG"; then
    ok "screen.log contains 'Balance:'"
else
    fail "screen.log missing 'Balance:' line"
fi

# 4. Full session output must NOT contain "Error:" (spurious Anvil conflict error)
if echo "$OUTPUT" | grep -q "^Error:"; then
    fail "session output contains spurious 'Error:' line(s):"
    echo "$OUTPUT" | grep "^Error:" | sed 's/^/       /'
else
    ok "session output has no spurious 'Error:' lines"
fi

# 5b. screen.log must also contain no "Error:" — it should capture the full session
if [[ -f "$SCREEN_LOG" ]] && grep -q "^Error:" "$SCREEN_LOG"; then
    fail "screen.log contains spurious 'Error:' line(s)"
else
    ok "screen.log has no spurious 'Error:' lines"
fi

# 5c. screen.log must contain the session header (proves full output is recorded)
if [[ -f "$SCREEN_LOG" ]] && grep -q "Session artifacts:" "$SCREEN_LOG"; then
    ok "screen.log contains session header"
else
    fail "screen.log missing session header — wrapper output not captured"
fi

# 5. state.json must exist
if [[ -f "$STATE_JSON" ]]; then
    ok "state.json exists: $STATE_JSON"
else
    fail "state.json not found at: $STATE_JSON"
fi

# 6. state.json must contain balance_wei
if [[ -f "$STATE_JSON" ]] && grep -q "balance_wei" "$STATE_JSON"; then
    ok "state.json contains 'balance_wei'"
else
    fail "state.json missing 'balance_wei'"
fi

# ── Cleanup ───────────────────────────────────────────────────────────────────
if [[ -n "$ANVIL_PID" ]]; then
    kill "$ANVIL_PID" 2>/dev/null || true
    echo ""
    echo "Anvil stopped (pid $ANVIL_PID)."
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
if [[ "$FAIL" -gt 0 ]]; then
    exit 1
fi
