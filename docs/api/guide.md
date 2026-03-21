# API Guide

## Game Core API

Defined in src/lib.rs and consumed by src/main.rs.

- terminal_size_ok(width, height) -> bool
- start_state(width, height) -> Option<GameState>
- apply_direction(state, requested_direction)
- tick(state) -> GameOutcome
- log_size_crash(path, width, height, reason)
- read_leaderboard(path) -> Vec<LeaderboardEntry>
- previous_high(entries) -> u32
- record_new_high(path, entries, name, score) -> Vec<LeaderboardEntry>

## Layering Rule

Main runtime must call game-core API and must not duplicate gameplay rules.
