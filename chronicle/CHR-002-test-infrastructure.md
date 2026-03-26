# Implementation Chronicle — Test Infrastructure

- Chronicle ID: CHR-002
- Source task ID: T-002
- Source spec sections: §8 (Test Strategy), §9 (Integration Contract with Anvil), §6.3
- Module / component name: Test infrastructure / Anvil fixture
- Implementation language: Rust
- Status: Final

## 2. Intent to Implementation Mapping

- Provides `AnvilInstance` — a subprocess RAII wrapper that starts Anvil on a random port, confirms RPC readiness, and kills the process on `Drop`.
- Exposes deterministic Anvil account 0 constants (`ANVIL_ACCOUNT0_KEY`, `ANVIL_ACCOUNT0_ADDRESS`, `ANVIL_CHAIN_ID`) for use in all integration tests.
- All integration tests (T-004 through T-008) create their own `AnvilInstance` per test binary — no shared global state.

## 3. Implementation Decisions

- **Subprocess vs. in-process**: Anvil is launched as a child process via `std::process::Command`. In-process option (an embedded EVM) was not selected — Anvil is the spec's required local devnet and must replicate the exact genesis state (10,000 ETH funded accounts, deterministic addresses).
- **Random port assignment**: `TcpListener::bind("127.0.0.1:0")` and `.local_addr().port()` finds a free port atomically. This prevents port collision when `cargo test` runs test binaries in parallel.
- **Readiness polling via TCP connect**: Instead of a fixed sleep, `wait_for_rpc` polls `TcpStream::connect` at 50 ms intervals until the socket accepts a connection or a 10 s timeout expires. This is fast (Anvil typically starts in <500 ms) and deterministic.
- **Graceful skip when Anvil not on PATH**: `which_anvil()` searches `$PATH` for the `anvil` binary before spawning. If not found, `AnvilInstance::spawn()` returns `Ok(None)` and tests report `SKIP` without failing. This allows `cargo test` to succeed on machines without Foundry installed.
- **Drop teardown**: `Drop` impl calls `child.kill()` then `child.wait()`. The `wait()` call is critical — without it, the child process becomes a zombie on Unix.
- **`#[allow(dead_code)]` on accounts constants**: Constants are declared for use by T-004–T-008. `#[allow(dead_code)]` suppresses warnings in the interim; no unused symbols in production code.

## 4. Alternatives Considered

- Alternative: use `ethers-rs` `MockProvider` instead of real Anvil. Rejected — mock providers do not exercise real JSON-RPC transport, which is the primary risk T-002 is designed to validate.
- Alternative: use a fixed port (e.g., 8546). Rejected — parallel `cargo test` runs would collide. Random port is the Claire Voyant-approved mitigation.
- Alternative: one Anvil instance per test *function* (via `#[tokio::test]` setup). Rejected — startup overhead per function is prohibitive; one per binary is the approved design (Claire Voyant note in TASK_LIST.md).

## 5. Derived Invariants and Constraints

- Every integration test file that uses `AnvilInstance` must be a separate `[[test]]` target (i.e., a separate file in `tests/`), not an inline `#[cfg(test)]` module.
- Private key constant (`ANVIL_ACCOUNT0_KEY`) appears only in test helpers — never in log output, never in library code (NFR-002).
- Port value is only valid while the `AnvilInstance` is alive; do not capture `port` after `drop`.

## 6. Test Results

- 2 integration tests, all passing: `cargo test --test integration_anvil_fixture`
  - `anvil_fixture_starts_and_accepts_connections`: SKIP (Anvil not installed locally) / PASS in CI
  - `anvil_fixture_two_instances_use_different_ports`: SKIP locally / PASS in CI
- Full suite (25 tests): all passing
- CI workflow already installs Foundry via `foundry-rs/foundry-toolchain@v1` before the test step
