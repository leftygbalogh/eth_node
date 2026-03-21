# Release Checklist

## Stage 5 Verify Results

- [x] Rust build and test pass: cargo test -q
- [x] Release build passes: cargo build --release
- [x] Startup size constraint behavior implemented and logged path present
- [x] Startup pre-start gate implemented: Press any key to start.
- [x] Continuous movement and arrow-key direction control implemented
- [x] Apple, growth, score, and one-apple invariant implemented
- [x] Wall/self-collision and no-wrap behavior implemented
- [x] Exact win condition implemented
- [x] Runtime resize-too-small print+log+exit path implemented
- [x] Leaderboard new-high-only write policy implemented
- [x] No extra gameplay features or configuration added
- [x] Environment validation matrix recorded in docs/evidence/environment-matrix.md
- [x] Git Bash smoke run reaches pre-start screen via run_snake.sh

## Stage 6 Release

- [x] Deliverables present: src/lib.rs, src/main.rs, run_snake.sh
- [x] Crash log contract implemented via snake.log append format
- [x] Leaderboard contract implemented via leaderboard.csv policy
- [x] Stage artifacts updated for Rust cycle
- [x] Operational and monitoring docs updated
- [x] Known environment gaps documented in runbook and evidence matrix

## Stage 5 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-21
- Notes: Verification complete; Git Bash smoke run succeeded, Linux/WSL parity remains a tracked follow-up.

## Stage 6 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-21
- Notes: Release artifacts complete with explicit environment-gap disclosure.
