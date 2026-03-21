# Changelog

## 2.0.0 - 2026-03-21

- Started new approved Greenfield Rust cycle for terminal Snake.
- Replaced prior Python-stage artifacts with Rust-aligned brief, spec, plan, and chronicle.
- Implemented Rust game core in src/lib.rs and terminal runtime in src/main.rs.
- Added Bash launcher run_snake.sh with MSYS/WSL/Linux routing and winpty support path.
- Added crash logging contract for startup/runtime terminal size failures.
- Added leaderboard new-high-only persistence logic and endgame flow.
- Verified with cargo test -q and cargo build --release.
- Verified launcher smoke execution in Git Bash and documented remaining Linux/WSL parity checks.

## 1.1.0 - 2026-03-20

- Completed Stage 5 Verify with explicit evidence checks.
- Added Stage 6 release readiness artifacts:
  - `RELEASE_CHECKLIST.md`
  - `OPERATIONS_AND_ROLLBACK.md`
  - `POST_RELEASE_MONITORING.md`
  - `RUNBOOK_KNOWN_FAILURES.md`
  - `GETTING_STARTED.md`
- Recorded Stage 5 approval and transition to Stage 6.

## 1.0.0 - 2026-03-19

- Established baseline AI governance template.
- Completed and approved Stage 1 through Stage 4 artifacts.
- Added mode-first startup flow, stage-gate denial logging, task status discipline, cross-agent routing, Brownfield confidence gate, and Verify-stage auditability requirements.
