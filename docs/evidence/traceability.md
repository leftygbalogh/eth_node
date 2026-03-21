# Traceability Report

## FR to Task to Implementation

- FR-001 -> T-001/T-002 -> src/lib.rs (terminal_size_ok, start_state), src/main.rs (wait_for_start)
- FR-002 -> T-003 -> src/main.rs (tick loop, arrow key mapping)
- FR-003 -> T-004 -> src/lib.rs (choose_apple, tick)
- FR-004 -> T-005 -> src/lib.rs (is_inside, tick loss branches)
- FR-005 -> T-005 -> src/lib.rs (bounded coordinates)
- FR-006 -> T-005 -> src/lib.rs (self-collision branch)
- FR-007 -> T-006 -> src/lib.rs (full-board win check)
- FR-008 -> T-001 -> src/main.rs (startup size failure branch)
- FR-009 -> T-007 -> src/main.rs (runtime resize failure branch)
- FR-010 -> T-001/T-007 -> src/lib.rs (log_size_crash), src/main.rs (startup/runtime reason tags)
- FR-011 -> T-008 -> src/main.rs (framed Congratulations sign)
- FR-012 -> T-008 -> src/main.rs (conditional name prompt, max 5 char trim)
- FR-013 -> T-008 -> src/lib.rs (record_new_high top insert and cap 10)
- FR-014 -> T-008 -> src/main.rs (display leaderboard after write)
- FR-015 -> T-009 -> run_snake.sh (MSYS/WSL/Linux launcher routing)

## Verification Evidence

- Build and test: cargo test -q passed.
- Release build: cargo build --release passed.
- Shell-specific launcher execution: environment-dependent; current host documented in environment matrix.
