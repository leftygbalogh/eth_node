# Numbered Task List

## Planning Metadata

- Plan ID: PLAN-RSNK-002
- Source spec: FSP-RSNK-002
- Mode: Greenfield
- Date: 2026-03-21

## Backlog

1. T-000: Establish Rust project layout
- Source: FR-001..FR-015
- Status: Done
- DoR: Spec approved and module seams defined
- DoD evidence: Cargo project initialized with src/lib.rs, src/main.rs, run_snake.sh
- Chronicle: CHR-RSNK-001

2. T-001: Startup size check and crash logging
- Source: FR-001, FR-008, FR-010
- Status: Done
- DoR: Startup decision table approved
- DoD evidence: terminal_size_ok and log_size_crash in src/lib.rs; startup path in src/main.rs
- Chronicle: CHR-RSNK-002

3. T-002: Startup state and pre-start gate
- Source: FR-001
- Status: Done
- DoR: Center placement and random-direction contract approved
- DoD evidence: start_state and wait_for_start in src/main.rs
- Chronicle: CHR-RSNK-003

4. T-003: Movement loop and direction controls
- Source: FR-002
- Status: Done
- DoR: Tick contract approved
- DoD evidence: fixed tick interval and arrow-key mapping in src/main.rs
- Chronicle: CHR-RSNK-004

5. T-004: Apple, growth, scoring
- Source: FR-003, FR-004
- Status: Done
- DoR: Apple placement rules approved
- DoD evidence: choose_apple and tick state transition logic in src/lib.rs
- Chronicle: CHR-RSNK-005

6. T-005: Collision and no-wrap boundaries
- Source: FR-004, FR-005
- Status: Done
- DoR: Boundary contract approved
- DoD evidence: is_inside and loss branches in tick in src/lib.rs
- Chronicle: CHR-RSNK-006

7. T-006: Exact win condition
- Source: FR-007
- Status: Done
- DoR: Full-board final-apple rule approved
- DoD evidence: board_cells terminal win check in tick in src/lib.rs
- Chronicle: CHR-RSNK-007

8. T-007: Runtime resize failure handling
- Source: FR-009, FR-010
- Status: Done
- DoR: Runtime decision table approved
- DoD evidence: runtime terminal-size guard in run_game in src/main.rs
- Chronicle: CHR-RSNK-008

9. T-008: Endgame and leaderboard policy
- Source: FR-011..FR-014
- Status: Done
- DoR: Leaderboard write decision table approved
- DoD evidence: read_leaderboard, record_new_high, and endgame flow in src/main.rs/src/lib.rs
- Chronicle: CHR-RSNK-009

10. T-009: Bash launcher runtime detection
- Source: FR-015
- Status: Done
- DoR: Launcher scope fixed to Bash only
- DoD evidence: run_snake.sh with MSYS/WSL/Linux paths and winpty branch
- Chronicle: CHR-RSNK-010

11. T-010: Verification and evidence capture
- Source: Stage 5 criteria
- Status: Done
- DoR: Build complete and tests present
- DoD evidence: cargo test -q passed, cargo build --release passed, environment gap documented
- Chronicle: CHR-RSNK-011

## Stage 3 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-21
- Notes: Rust-cycle planning approved and executed.
