# Operational Notes and Rollback Plan

## Operational Notes

- Start gameplay via run_snake.sh.
- Game binary path: target/release/rust_terminal_snake_game.
- Crash records append to snake.log with timestamp_gmt, width, height, reason.
- Leaderboard persistence file is leaderboard.csv and updates only on new all-time highs.

## Rollback Triggers

- Stage-gate protocol breach.
- Release build or tests fail after accepted baseline.
- Runtime regression in startup/runtime size-failure handling.
- Leaderboard rule regression (non-record prompt/write).

## Rollback Procedure

1. Identify last good commit with passing cargo test -q and cargo build --release evidence.
2. Create a recovery branch from current HEAD.
3. Revert invalid commit set with git revert.
4. Re-run verification commands.
5. Re-check docs/evidence/traceability.md and docs/evidence/environment-matrix.md.
6. Record recovery actions in memory.md and resume gated workflow.

## Safety Rule

- Use non-destructive rollback for shared history.
