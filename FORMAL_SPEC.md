# Formal Specification

## 1. Metadata

- Spec ID: FSP-SNK-001
- Version: 1.0
- Mode: Greenfield
- Source brief: PROJECT_BRIEF.md
- Status: Approved

## 2. Behavioral Contracts

### 2.1 Board Contract

- Valid coordinates: `0 <= x <= maxX`, `0 <= y <= maxY`
- Any coordinate outside valid bounds is wall collision.
- Board dimensions are fixed at startup terminal size.

### 2.2 Startup Contract

- If startup terminal width < 11 or height < 11:
  - print error message
  - append crash record to `snake.log`
  - exit immediately
- Otherwise:
  - place head `:` at board center
  - choose random start direction
  - place one apple on empty cell
  - show `Press any key to start.`
  - wait for any key

### 2.3 Tick Contract

- Snake moves exactly one cell per tick.
- Direction can be changed only by arrow keys.
- Reverse direction is permitted; if head enters body, self-collision loss applies.
- If head enters apple cell:
  - score += 1 exactly
  - snake length += 1 exactly behind head
  - respawn one apple on empty cell

### 2.4 Endgame Contract

- Loss on wall collision or self-collision.
- Win when snake fills board except final apple cell and then consumes it.
- Display framed `Congratulations!` after game ends.

### 2.5 Leaderboard Contract

- File: `leaderboard.csv`
- If file missing, previous high score = 0.
- Ask player name only if current score > previous high.
- Name max length = 5.
- Record only new all-time highs.
- Insert new entry at top; keep max 10 entries.
- Show leaderboard only after recording new all-time high.

## 3. Decision Tables

### Startup size decision

| width>=11 | height>=11 | action |
| --- | --- | --- |
| yes | yes | continue |
| no | any | print+log+exit |
| any | no | print+log+exit |

### Leaderboard write decision

| score > previous_high | action |
| --- | --- |
| yes | prompt, write, display |
| no | no prompt, no write |

## 4. Traceability Seed

- FR-001 -> T-001, T-002
- FR-002 -> T-003
- FR-003 -> T-004
- FR-004 -> T-005
- FR-005 -> T-006
- FR-006 -> T-002, T-003
- FR-007 -> T-007

## 5. Stage 2 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-20
- Notes: Specification frozen for planning and build.
