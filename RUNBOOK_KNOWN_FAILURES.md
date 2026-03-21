# Runbook for Known Failure Scenarios

## Scenario 1: Startup terminal too small

- Symptom: game exits immediately at startup.
- Expected behavior: print size error, append snake.log entry, exit code non-zero.
- Action:
  1. Check current terminal dimensions.
  2. Confirm width >= 11 and height >= 11.
  3. Re-run via run_snake.sh.

## Scenario 2: Runtime resize crash

- Symptom: game exits during session after terminal resize.
- Expected behavior: print size error, append snake.log entry with runtime reason.
- Action:
  1. Inspect latest snake.log line.
  2. Validate that terminal was reduced below fixed startup board size.
  3. Re-open larger terminal and restart game.

## Scenario 3: Launcher does not run in current shell

- Symptom: run_snake.sh cannot execute.
- Action:
  1. Confirm Bash runtime is available.
  2. For Git Bash on Windows, ensure winpty exists in PATH if required.
  3. Validate script path and executable permissions in Linux/WSL hosts.
  4. Check docs/evidence/environment-matrix.md for known gaps.

## Scenario 4: Leaderboard unexpected behavior

- Symptom: name prompt appears without new high score, or leaderboard writes non-record score.
- Action:
  1. Reproduce with controlled score values.
  2. Verify previous_high and record_new_high call conditions.
  3. Run cargo test -q and inspect src/main.rs endgame branch.

## Scenario 5: Governance breach or stage drift

- Symptom: stage moved forward without proper approval evidence.
- Action:
  1. Stop further changes.
  2. Log breach in memory.md.
  3. Revert unauthorized commits with git revert.
  4. Replay affected stage with correct approvals.
