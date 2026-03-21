# Post-Release Monitoring Plan

## Monitoring Window

- Immediate: first end-to-end runtime session in supported Bash host.
- Short-term: first 5 gameplay sessions across target terminal environments.
- Ongoing: each release candidate cycle.

## Signals to Monitor

- snake.log entries for startup/runtime size failures.
- Launcher execution success across Bash environments.
- Leaderboard correctness for new-high-only write behavior.
- Runtime stability during repeated direction changes and long sessions.

## Alert Thresholds

- Critical: crash loop on startup in valid terminal size.
- High: leaderboard rule violation (prompt/write when not new high).
- Medium: launcher fails in expected target environment.
- Low: cosmetic rendering variance without behavioral regression.

## Alerting Route

1. Record issue details and reproduction steps in memory.md.
2. Add evidence reference under docs/evidence/.
3. Open fix task in TASK_LIST.md with source requirement link.
4. Re-run cargo test -q and cargo build --release before closure.
