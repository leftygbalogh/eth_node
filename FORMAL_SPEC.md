# Formal Specification

## 1. Metadata

- Spec ID: FSP-RSNK-002
- Version: 2.0
- Mode: Greenfield
- Source brief: PROJECT_BRIEF.md
- Primary language: Rust
- Secondary language: Bash (launcher only)
- Status: Approved

## 2. Behavioral Contracts

### 2.1 Board Contract

- Startup terminal size determines fixed board size for the full session.
- Valid positions satisfy 0 <= x <= maxX and 0 <= y <= maxY.
- Any out-of-range position is wall collision.
- No wraparound behavior exists.

### 2.2 Startup Contract

- If width < 11 or height < 11:
  - print size error message
  - append crash entry to snake.log
  - exit immediately
- Otherwise:
  - place head at center cell
  - initialize a 9-segment snake
  - pick random initial direction
  - place one apple on empty space
  - display Press any key to start.
  - wait for key press before first tick

### 2.3 Tick Contract

- Tick interval is fixed and steady.
- Direction input accepts arrow keys only.
- Each tick moves head exactly one cell in active direction.
- On apple collision:
  - score increments by exactly 1
  - snake length increases by exactly 1
  - one new apple spawns on empty space
- If board is fully occupied after final apple consumption, game outcome is Win.

### 2.4 Runtime Size Contract

- During gameplay, current terminal size is polled against fixed startup board size.
- If runtime terminal size becomes smaller than fixed board:
  - print size error message
  - append crash entry to snake.log
  - exit immediately

### 2.5 Endgame and Leaderboard Contract

- Terminal outcomes:
  - Win on exact full-board final-apple condition
  - Loss on wall collision or self-collision
- End-of-game always prints framed Congratulations! sign.
- Leaderboard file is leaderboard.csv.
- If leaderboard file is absent, previous high score is 0.
- Name prompt occurs only when score > previous high.
- Name is truncated to max length 5.
- Only new all-time highs are written.
- New entry is inserted at top and file is capped to 10 entries.
- Leaderboard is displayed only after recording a new all-time high.

## 3. Statechart (Text Form)

States:
- S0 Boot
- S1 PreStart
- S2 Running
- S3 Win
- S4 Loss
- S5 CrashExit

Transitions:
- S0 -> S5 when startup size invalid
- S0 -> S1 when startup size valid and initial state built
- S1 -> S2 on any key press
- S2 -> S5 when runtime size drops below fixed board size
- S2 -> S4 on wall collision or self collision
- S2 -> S3 on exact final-apple full-board condition
- S3 -> end after congratulations and conditional leaderboard flow
- S4 -> end after congratulations and conditional leaderboard flow

## 4. Decision Tables

### Startup size

| width >= 11 | height >= 11 | action |
| --- | --- | --- |
| yes | yes | continue to pre-start |
| no | any | print + log + exit |
| any | no | print + log + exit |

### Runtime size

| current width >= fixed width | current height >= fixed height | action |
| --- | --- | --- |
| yes | yes | continue |
| no | any | print + log + exit |
| any | no | print + log + exit |

### Leaderboard write rule

| score > previous high | prompt name | write leaderboard | display leaderboard |
| --- | --- | --- | --- |
| yes | yes | yes | yes |
| no | no | no | no |

## 5. Traceability Matrix

- FR-001 -> T-001, T-002
- FR-002 -> T-003
- FR-003 -> T-004
- FR-004 -> T-005
- FR-005 -> T-006
- FR-006 -> T-007
- FR-007 -> T-008
- FR-008 -> T-001
- FR-009 -> T-007
- FR-010 -> T-001, T-007
- FR-011 -> T-008
- FR-012 -> T-008
- FR-013 -> T-008
- FR-014 -> T-008
- FR-015 -> T-009

## 6. Quality Dimension Targets (Q2/Q3)

- Q2 Performance: one deterministic move/update/render loop per tick with no speed controls exposed.
- Q2 Reliability: startup/runtime size failures always print + log + terminate.
- Q2 Maintainability: gameplay logic, terminal runtime, and persistence logic remain separated in distinct modules.
- Q3 Data Quality: leaderboard.csv structure remains consistent name,score per line.
- Q3 Compliance/Auditability: crash logging records timestamp_gmt, width, height, reason for each size-related crash.
- Q3-ARCH-01 Layered architecture:
  - game core API in src/lib.rs
  - terminal/CLI adapter in src/main.rs
  - launcher shim in run_snake.sh

## 7. Stage 2 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-21
- Notes: Rust-cycle specification approved for planning and build.
