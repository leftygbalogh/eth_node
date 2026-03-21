# ADR-0001: Layered Architecture for Rust Snake

## Status

Accepted - 2026-03-21

## Context

The project requires Q3-ARCH-01 layered architecture integrity and strict behavior traceability.

## Decision

Split implementation into:
- src/lib.rs: game domain and persistence rules
- src/main.rs: terminal runtime orchestration and rendering only
- run_snake.sh: launch routing across shell contexts

## Consequences

- Domain logic is testable without terminal UI.
- CLI adapter remains thin and easier to audit for scope creep.
- Launcher can evolve independently for environment detection.
