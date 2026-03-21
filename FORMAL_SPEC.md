
# Formal Specification

## 1. Metadata

This section must be filled in based on the current project brief and stage instructions. Specify the spec ID, version, mode, source brief, and status as appropriate for the project.

## 2. Behavioral Contracts

All behavioral contracts must be defined according to the requirements and constraints in the approved project brief. Remove any example content and replace with project-specific contracts for:


## 3. Decision Tables

Define all decision tables required for the project, based on the formal requirements and edge cases in the project brief. Do not use example values; all logic must be derived from the current project’s needs.

## 4. Traceability Seed

Map all functional requirements (FR-XXX) to their corresponding tasks (T-XXX) as defined in the project’s task list. This section must be updated for each new project.
- FR-005 -> T-006
- FR-006 -> T-002, T-003
- FR-007 -> T-007

## 4.1 Quality Dimension Targets (Q2 Pack)

- Performance and efficiency targets:
  - Tick processing remains bounded to a single movement/update/render cycle per tick.
  - Leaderboard read/write operations complete without blocking gameplay loop (startup/endgame only).
- Reliability and resilience targets:
  - Terminal-too-small at startup and runtime must deterministically emit message + structured crash log + controlled exit.
  - Missing leaderboard file is handled as empty baseline (no crash).
- Maintainability over time targets:
  - Game loop, collision logic, and persistence logic remain separated as independently testable seams.
  - Leaderboard policy remains encapsulated to prevent gameplay-rule coupling.
- Not-applicable declarations:
  - Throughput/SLA metrics for multi-user load: N/A (single-user local terminal game).
  - Network resilience targets: N/A (no network calls).

## 5. Architecture and Design Decisions

- Decision: enforce layered structure with gameplay API separated from terminal input/output handling.
- Rationale: keeps domain behavior testable and preserves CLI independence for future interface extensions.

### 5.1 Layered Architecture (Q3-ARCH-01)

- Module interface definitions:
  - `game_state`: state transitions, movement, collision, win/loss resolution
  - `leaderboard_store`: read/write and ranking constraints for `leaderboard.csv`
  - `runtime_loop`: tick orchestration and input dispatch
- API surface:
  - Internal operations for move tick, apple consumption, collision evaluation, and leaderboard update decision
  - CLI layer calls API operations but does not host business rules
- CLI-to-API mapping:
  - arrow key event -> direction update API
  - tick event -> move/apply-rules API
  - game-end event -> leaderboard decision/write API
- GUI-to-API mapping:
  - N/A (no GUI in scope)

## 6. Stage 2 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-20
- Notes: Specification frozen for planning and build.
