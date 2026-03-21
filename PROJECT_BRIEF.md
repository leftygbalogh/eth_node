# Project Brief

## 1. Project Overview

- Project name: Rust Terminal Snake Game
- Project mode: Greenfield
- Primary implementation language: Rust
- Secondary language: Bash (launcher/runtime detection only)
- Problem statement:
  - Build a terminal-shell Snake game in Rust with ASCII rendering only.
  - Launch through a Bash script that detects terminal environment and starts the game correctly.
  - Delivery must follow strict no-scope-creep constraints from sponsor brief.
- Desired outcome:
  - A simple, fully functional Rust terminal Snake game launched by Bash.
- In-scope goals:
  - Exactly two entities: snake and apple.
  - Snake visual representation: .oOOO(:)=
  - Apple visual representation: *
  - Arrow-key direction input only; steady continuous movement speed.
  - Startup prompt: Press any key to start, then game begins.
  - Board size fixed from startup terminal dimensions.
  - Minimum startup terminal size: width >= 11 and height >= 11.
  - Startup/runtime size failure handling with error message, crash log entry, and immediate exit.
  - Crash entries written to snake.log with GMT timestamp, width, height, and reason.
  - Explicit bounded coordinates with no wraparound.
  - Loss on wall collision or self-collision.
  - Win only on exact final-apple full-board condition.
  - Score increments by exactly 1 per apple and snake grows by exactly 1 per apple.
  - Score hidden before game start and during gameplay.
  - Leaderboard persisted in leaderboard.csv with top-10 cap and new-all-time-high-only write rule.
- Out-of-scope:
  - Menus, settings, game modes, visual effects, gameplay variants, quality-of-life additions.
  - Any feature or configuration not explicitly defined in approved requirements.

## 1.2 Quality Module Declarations

- Data Quality module active? Yes
  - Rationale: leaderboard persistence in leaderboard.csv.
- Compliance and Auditability module active? Yes
  - Rationale: explicitly declared active for this cycle by project owner.
- Interactive CLI diagnostics required? Yes
  - Rationale: interactive terminal gameplay and runtime resize behavior verification.
- Security and production-readiness loop required? Yes
  - Rationale: inherited by active governance quality and verification expectations for this cycle.
- Layered architecture constraint active? Yes
  - Rationale: Rust project with first-class module, type, and interface boundaries.

## 2. Stakeholders and Authority

- Sponsor: Lefty
- Product owner: Lefty
- Stage-gate approvals delegated to: Team Lead Agent (Stage 2 through Stage 6)
- Delegation notes:
  - Within approved stage scope, implementation details may be resolved autonomously.
  - Stage transitions still require explicit delegated approver approval.

## 3. Functional Requirements

1. FR-001 Startup flow places head at center, sets random initial direction, shows Press any key to start, and waits for any key.
2. FR-002 Snake movement is continuous at steady speed with four-direction arrow-key control only.
3. FR-003 Only one apple exists at a time and apples spawn randomly on empty spaces only.
4. FR-004 Eating apple increases score by exactly 1 and grows snake by exactly 1 segment.
5. FR-005 Board coordinates are explicitly bounded; any move outside bounds is wall collision; no wraparound.
6. FR-006 Loss occurs on wall collision or self-collision.
7. FR-007 Win occurs only when snake fills board except final apple space and then consumes that final apple.
8. FR-008 Startup terminal size check enforces minimum 11x11 with error, log, and immediate exit on failure.
9. FR-009 Runtime terminal resize that breaks fixed playfield triggers error, log, and immediate exit.
10. FR-010 Size-related crashes are logged to snake.log with GMT timestamp, width, height, and reason.
11. FR-011 End-of-game shows framed Congratulations! sign.
12. FR-012 Name entry is requested only for a new all-time high score; max length 5.
13. FR-013 Leaderboard updates only on new all-time high, inserts at top, preserves max 10 entries, and keeps chronology for ties.
14. FR-014 After recording new high score, leaderboard is shown with new entry at top.
15. FR-015 Running Bash launcher starts the game correctly across target shell contexts.

## 4. Non-Functional Requirements

- NFR-001 ASCII-only output.
- NFR-002 Terminal-shell runtime only.
- NFR-003 Rust-only game implementation code.
- NFR-004 Bash-only launcher and terminal-context detection script.
- NFR-005 No extra features or configuration beyond approved scope.
- NFR-006 Deterministic scope control: delegated details must not override explicit requirements.

## 5. Risks

- Terminal compatibility differences across Linux shell, WSL, Git Bash, and PowerShell hosting contexts.
- Arrow-key input behavior differences across terminal/host combinations.
- Runtime resize detection semantics differ by terminal implementation.
- Cross-platform launcher behavior may diverge without explicit detection matrix and tests.

## 6. Stage 1 Status

- Stage 1 approval: Approved
- Approved by: Lefty (project owner)
- Approval date: 2026-03-21
- Notes: Discovery inputs accepted; transition to Stage 2 requires explicit start authorization.
