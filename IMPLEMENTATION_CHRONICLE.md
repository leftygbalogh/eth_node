# Implementation Chronicle

## Entry CHR-RSNK-001

- Task: T-000
- Source requirements: FR-001..FR-015
- Decision: use layered structure with reusable game core in src/lib.rs and runtime adapter in src/main.rs.
- Why: enforces Q3-ARCH-01 and keeps behavior testable independently from terminal concerns.
- Evidence: Cargo crate structure and API usage from main into lib.

## Entry CHR-RSNK-002

- Task: T-001
- Source requirements: FR-001, FR-008, FR-010
- Decision: startup size validation occurs before entering alternate screen gameplay loop.
- Why: guarantees deterministic print + log + exit even if runtime UI setup cannot proceed.
- Evidence: terminal_size_ok and startup branch in main.

## Entry CHR-RSNK-003

- Task: T-002
- Source requirements: FR-001
- Decision: head is centered and the game blocks at pre-start prompt until key press.
- Why: exact startup behavior contract.
- Evidence: board_center, start_state, and wait_for_start implementation.

## Entry CHR-RSNK-004

- Task: T-003
- Source requirements: FR-002
- Decision: fixed 120ms tick and arrow-key-only direction mapping.
- Why: user controls direction only; speed is not configurable.
- Evidence: TICK_MS, key_to_direction, and event loop logic.

## Entry CHR-RSNK-005

- Task: T-004
- Source requirements: FR-003, FR-004
- Decision: apple placement samples only empty cells from full board scan.
- Why: guarantees one-apple-on-empty-space invariant.
- Evidence: choose_apple and apple-eat branch in tick.

## Entry CHR-RSNK-006

- Task: T-005
- Source requirements: FR-004, FR-005
- Decision: enforce explicit bounds with immediate loss on out-of-range or self-hit.
- Why: no-wrap and strict wall/self-collision semantics.
- Evidence: is_inside and tick loss branches.

## Entry CHR-RSNK-007

- Task: T-006
- Source requirements: FR-007
- Decision: win is emitted only after apple consume causes snake length to equal board cell count.
- Why: matches exact final-apple full-board rule.
- Evidence: board_cells check in tick.

## Entry CHR-RSNK-008

- Task: T-007
- Source requirements: FR-009, FR-010
- Decision: runtime terminal size is checked every loop and compared against fixed startup dimensions.
- Why: fixed-board guarantee with deterministic runtime crash behavior.
- Evidence: terminal::size polling and RuntimeTooSmall branch.

## Entry CHR-RSNK-009

- Task: T-008
- Source requirements: FR-011..FR-014
- Decision: new-high-only persistence path with name length clamp to 5 chars.
- Why: exactly matches leaderboard prompt/write constraints.
- Evidence: previous_high, record_new_high, and endgame branch in main.

## Entry CHR-RSNK-010

- Task: T-009
- Source requirements: FR-015
- Decision: Bash launcher detects MSYS/WSL and uses winpty when available in Git Bash contexts.
- Why: supports mixed shell environments while keeping launcher scope minimal.
- Evidence: run_snake.sh runtime routing logic.

## Entry CHR-RSNK-011

- Task: T-010
- Source requirements: Stage 5 Verify
- Decision: verification evidence recorded from cargo test and release build; non-testable local shell gaps are documented explicitly.
- Why: maintain truthful evidence without silent assumptions.
- Evidence: cargo test -q pass, cargo build --release pass, docs/evidence/environment-matrix.md.

## Stage 4 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-21
- Notes: Build complete and chronicle links updated for all implemented tasks.
