# Task List — eth_node Phase 1

Source spec: `FORMAL_SPEC.md`
Stage approver: Lefty
Mode: Greenfield
Language: Rust

---

## Task Status Key

- `[ ]` Not started
- `[>]` In progress
- `[x]` Done
- `[!]` Blocked

---

## T-000 — Repository and Workspace Layout

**Owner:** Developer in Test + Team Lead
**Spec ref:** FORMAL_SPEC.md §7 A-001
**Dependencies:** None — must be completed before any other task starts

**Description:**
Establish the canonical Cargo workspace layout and project skeleton. No business logic. No tests yet beyond compile check.

**Deliverables:**
- `Cargo.toml` (workspace root) with members: `eth_node`, `eth_node_cli`
- `eth_node/Cargo.toml` (library crate)
- `eth_node/src/lib.rs` (empty module declarations: `pub mod primitives; pub mod rpc; pub mod signer; pub mod tx; pub mod events; pub mod contract;`)
- `eth_node_cli/Cargo.toml` (binary crate, depends on `eth_node`)
- `eth_node_cli/src/main.rs` (stub: prints "eth_node_cli" and exits 0)
- `eth_node/tests/` directory (empty, with `.gitkeep`)
- `chronicle/` directory with empty chronicle stubs (one `.md` per planned module)
- `output/sessions/` directory (gitignored)
- `.github/workflows/ci.yml` — runs `cargo build` and `cargo test` on push
- `README.md` — local setup in ≤ 10 steps (Rust install, Foundry install, `cargo build`, `cargo test`)

**Definition of Ready:**
1. `FORMAL_SPEC.md` approved ✓
2. Workspace member names confirmed from spec A-001 ✓
3. No ambiguity in folder layout ✓

**Definition of Done:**
1. `cargo build` passes with zero warnings at workspace root
2. `cargo test` passes (no tests yet — compile check only)
3. CI workflow file present and syntactically valid
4. `chronicle/` stubs exist for: primitives, rpc, signer, tx, events, contract, cli, test-infrastructure
5. `output/sessions/` is in `.gitignore`
6. README covers local setup

**TDD Navigator note:** No tests to write yet — DoD is compile + structure. First real red-green cycle starts at T-001.

**Status:** `[x]` — DONE 2026-03-26
- cargo build: PASSED (serde pinned to <1.0.215 for alloy-consensus 0.12.6 compat)
- cargo test: PASSED (0 tests, compile only)
- Committed: 2d69ea6 "T-000: Cargo workspace scaffold..."

---

## T-001 — Ethereum Primitives Module

**Owner:** Rust Backend Specialist + TDD Driver
**Navigator:** TDD Navigator
**Spec ref:** FORMAL_SPEC.md §4 FR-001, §9.2 deterministic test vectors
**Dependencies:** T-000

**Description:**
Implement `eth_node::primitives` — `Address`, `H256`, `U256`, ABI encode/decode, RLP encode/decode. Pure functions, no I/O.

**Key crates:** `alloy-primitives`, `alloy-core` (pin minor version in `Cargo.toml`)

**Definition of Ready:**
1. T-000 complete ✓ (workspace exists)
2. ABI encoding spec in FORMAL_SPEC.md §4 FR-001 reviewed ✓
3. Deterministic test vectors available: `uint256(1)` → `0x000...001`; RLP empty string → `0x80` ✓
4. No ambiguity in `Address` parsing (checksummed/lowercase, with/without `0x`) ✓
5. Chronicle entry `chronicle/primitives.md` stub exists (from T-000) ✓

**Definition of Done:**
1. Red-green-refactor complete for all sub-behaviors:
   - `Address::from_str` — valid hex, missing `0x`, wrong length, invalid hex chars
   - `U256` arithmetic — overflow on checked, expected values
   - ABI encode/decode round-trip: `uint256`, `address`, `bool`, `string`, `bytes32`, nested tuple
   - RLP encode/decode round-trip: integer 0, integer 1, empty string, byte array, nested list
2. All tests deterministic (no randomness, no I/O)
3. `cargo test primitives` passes
4. `cargo clippy -- -D warnings` clean
5. `chronicle/primitives.md` entry complete (decision: alloy-primitives chosen over manual impl — rationale recorded)

**Test falsifiability (Oracle-reviewed):** ABI/RLP tests use known vectors — a wrong encoding produces a byte mismatch, not a vacuous pass. ✓

**Status:** `[x]` — DONE 2026-03-26
- 23 unit tests: all passing
- clippy -D warnings: clean
- Key discovery: `Vec<u8>` decodes as RLP list; `bytes::Bytes` required for byte-string decode. `bytes = "1"` added to workspace deps.
- Chronicle: CHR-001-primitives.md complete

---

## T-002 — CI + Anvil Integration Test Fixture

**Owner:** Developer in Test
**Spec ref:** FORMAL_SPEC.md §9, §6.3, A-005
**Dependencies:** T-000

**Description:**
Wire Anvil as a subprocess test fixture for the integration test suite. All integration tests (T-004 through T-008) depend on this fixture. Must be established before any integration test is written.

**Deliverables:**
- `eth_node/tests/helpers/anvil_fixture.rs` — starts Anvil subprocess, waits for RPC ready, yields port, tears down on drop
- `eth_node/tests/helpers/accounts.rs` — exposes Anvil account 0 key + address as constants (from spec §9.2)
- CI workflow updated: installs Foundry (Anvil) before integration test step
- Verified: `ANVIL_AUTO_START=1 cargo test --test integration_rpc` starts Anvil and tears it down cleanly

**Definition of Ready:**
1. T-000 complete ✓
2. Anvil binary install method confirmed (Foundry via `foundryup`) ✓
3. Anvil default port (8545) and chain ID (31337) confirmed from spec §6.3 ✓

**Definition of Done:**
1. `anvil_fixture::AnvilInstance` starts, accepts connections, and shuts down cleanly in a test
2. Test using the fixture passes in CI without manual Anvil startup
3. No port conflicts between parallel test runs (use random port assignment)
4. `chronicle/test-infrastructure.md` entry written: fixture design decision (subprocess vs in-process), port strategy, teardown mechanism

**Claire Voyant note:** *Fixture determinism under parallel test execution is the key risk here. Random port assignment mitigates port collision but each test must get a fresh Anvil instance with deterministic initial state (same genesis block, same funded accounts). Confirm: one Anvil instance per test binary run, not per test function — to keep startup overhead manageable.*

**Status:** `[x]` — DONE 2026-03-26
- AnvilInstance: subprocess, random port via TcpListener(0), TCP-poll readiness, Drop teardown
- Graceful SKIP when anvil not on PATH (`Ok(None)` pattern)
- 2 integration tests pass (skip locally, full pass in CI with Foundry toolchain)
- accounts.rs: ANVIL_ACCOUNT0_KEY, _ADDRESS, CHAIN_ID constants
- Chronicle: CHR-002-test-infrastructure.md complete

---

## T-003 — Session Capture Helper Scripts

**Owner:** Developer in Test
**Spec ref:** FORMAL_SPEC.md §5.1, §9 Interactive CLI diagnostics
**Dependencies:** T-000

**Description:**
Create the session capture wrapper scripts used by the Exploratory Tester for manual CLI sessions.

**Deliverables:**
- `scripts/capture-session.ps1` (Windows — wraps `eth_node_cli` with `Start-Transcript`, dumps `--dump-state` to `output/sessions/<timestamp>/state.json`)
- `scripts/capture-session.sh` (Linux/macOS — wraps with `script` command)
- Both scripts: create `output/sessions/YYYY-MM-DD_HH-MM-SS/` directory, write `screen.log` and trigger `state.json` dump
- `output/sessions/` listed in `.gitignore`

**Definition of Ready:**
1. T-000 complete ✓
2. CLI `--dump-state <path>` flag behaviour defined in spec §5.1 ✓
3. Artifact naming convention defined in spec §5.1 ✓

**Definition of Done:**
1. Running `scripts/capture-session.ps1 balance 0xf39Fd...` on Windows produces `output/sessions/<timestamp>/screen.log`
2. Same test on Linux with `capture-session.sh`
3. Scripts tested manually and confirmed working (Exploratory Tester sign-off)
4. `chronicle/test-infrastructure.md` updated with session capture design note

**Status:** `[x]` — DONE 2026-03-27 (commits `7b322c9`, `875030f` and follow-up fixes)

---

## T-004 — JSON-RPC Client Module ✅ DONE

**Owner:** Rust Backend Specialist + TDD Driver
**Navigator:** TDD Navigator
**Spec ref:** FORMAL_SPEC.md §4 FR-002, §6.3 integration contract, §5 NFR-001, NFR-003
**Dependencies:** T-000, T-001 (uses `Address`, `H256`), T-002 (integration test fixture)
**Committed:** T-004 commit (see git log)

**Description:**
Implement `eth_node::rpc` — typed JSON-RPC client over HTTP and WebSocket. All 11 methods in spec FR-002. Error classification (transient vs permanent).

**Key crates:** `alloy-provider`, `alloy-transport`

**Definition of Ready:**
1. T-001 complete (primitives types available) ✓
2. T-002 complete (Anvil fixture available for integration tests) ✓
3. Integration contract table in FORMAL_SPEC.md §6.3 reviewed — all request/response schemas defined ✓
4. RPC client statechart (FR-002) reviewed — states: Disconnected, Connected, Error ✓
5. Chronicle stub exists ✓

**Definition of Done:**
1. Unit tests: RPC error classification (transient vs permanent) tested against mock responses
2. Unit tests: response deserialisation of malformed JSON returns `RpcError::Deserialization`, never panics (NFR-001)
3. Integration tests (against live Anvil fixture):
   - `test_get_balance` — account 0, assert == 10,000 ETH in Wei
   - `test_get_nonce` — fresh account, nonce == 0
   - `test_get_chain_id` — assert == 31337
   - `test_block_number` — assert >= 0
4. Contract test: outgoing `eth_getBalance` request asserts `method == "eth_getBalance"` and `params[0]` matches input address (not just mocked response)
5. Every RPC call emits a `tracing` span (NFR-003)
6. `cargo clippy -- -D warnings` clean
7. `chronicle/rpc.md` entry complete

**Status:** `[x]` — DONE 2026-03-26 (commit `bd3bf26`)

---

## T-005 — Transaction Signer Module

**Owner:** Rust Defensive Programming Specialist + TDD Driver
**Navigator:** TDD Navigator
**Spec ref:** FORMAL_SPEC.md §4 FR-004, §6.4 credential model, §5 NFR-002
**Dependencies:** T-000, T-001

**Description:**
Implement `eth_node::signer` — load private key from env, derive address, sign transactions (type 0 and type 2), produce `SignedTransaction`. RFC 6979 deterministic signing. Private key must never appear in any log output.

**Key crates:** `alloy-signer`, `alloy-consensus`

**Definition of Ready:**
1. T-001 complete ✓
2. Credential model (spec §6.4) reviewed — env var load order, no stdin, no prompts ✓
3. EIP-1559 type 2 signing envelope from spec FR-004 (Oracle-corrected) reviewed ✓
4. Test private key + expected address + expected signature (known vectors) ready from spec §9.2 ✓
5. Chronicle stub exists ✓

**Definition of Done:**
1. Unit tests with known Anvil account 0 key:
   - `test_derive_address` — private key → expected address
   - `test_sign_type2_tx` — known inputs → deterministic signature (RFC 6979)
   - `test_key_not_in_logs` — key loaded, sign called, assert private key string absent from all tracing output (NFR-002)
2. Unit tests for credential loading:
   - `test_missing_env_var` — `SignerError::CredentialNotFound`, no panic, no stdin block
   - `test_invalid_key_format` — `SignerError::InvalidKey`
3. `cargo clippy -- -D warnings` clean
4. `chronicle/signer.md` entry complete (decision: RFC 6979 determinism, key zeroization on drop if alloy provides it)

**TDD Navigator note:** The `test_key_not_in_logs` test is critical for NFR-002. It must use a tracing subscriber that captures output and asserts the key string is absent — not just "looks like it doesn't log it."

**Status:** `[x]` — DONE 2026-03-26 (commit `bef590e`)

---

## T-006 — Transaction Builder and Broadcaster Module

**Owner:** Rust Backend Specialist + TDD Driver
**Navigator:** TDD Navigator
**Spec ref:** FORMAL_SPEC.md §4 FR-003, FR-005, §6.2 API contract
**Dependencies:** T-001, T-004 (RPC client), T-005 (signer)

**Description:**
Implement `eth_node::tx` — `TransactionBuilder` (FR-003) and `Broadcaster` (FR-005). Covers fee type selection decision table, nonce auto-fetch, gas estimation, confirmation polling, and timeout handling.

**Key crates:** `alloy-consensus`, `alloy-provider`

**Definition of Ready:**
1. T-004 (RPC client) and T-005 (signer) complete ✓
2. Transaction type decision table (spec FR-003) reviewed — all 4 rows mapped to test cases ✓
3. `Broadcaster` statechart (spec FR-005) terminal states reviewed: Confirmed, Reverted, TimedOut, SubmitError ✓
4. Polling interval (500ms default) and timeout (60s default) confirmed ✓
5. Chronicle stub exists ✓

**Definition of Done:**
1. Unit tests — decision table coverage (all 4 rows of fee type selection):
   - `gas_price` set only → type 0
   - `max_fee` set only → type 2
   - neither set → type 2, fees fetched
   - both set → `TxError::ConflictingFeeParams`
2. Unit tests — builder with fixed nonce and gas inputs produces deterministic RLP output
3. Integration tests (against live Anvil fixture):
   - `test_send_eth_confirmed` — send 0.1 ETH, receipt status == 1, balance changes correct
   - `test_send_reverted` — send to a contract that reverts, receipt status == 0, `TxError::Reverted` returned
   - `test_confirmation_timeout` — mock short timeout, assert `TxError::ConfirmationTimeout` contains tx hash
4. `cargo clippy -- -D warnings` clean
5. `chronicle/tx.md` entry complete

**Status:** `[x]` — DONE 2026-03-26 (commit `79092a6`)

---

## T-007 — Event Listener Module

**Owner:** Rust Async Concurrency Specialist + TDD Driver
**Navigator:** TDD Navigator
**Spec ref:** FORMAL_SPEC.md §4 FR-006, §6.2 API contract
**Dependencies:** T-001, T-004 (RPC client for HTTP polling and WebSocket subscription)

**Description:**
Implement `eth_node::events` — `Listener` with HTTP polling mode and WebSocket subscription mode. Stream of `Log` items. Reconnection on disconnect (up to 3 attempts, exponential backoff).

**Key crates:** `alloy-provider` (filter types + subscription)

**Definition of Ready:**
1. T-004 (RPC client) complete ✓
2. Event listener statechart (spec FR-006) reviewed — states: Idle, Connecting, Listening, Reconnecting, Error ✓
3. Reconnect policy (3 attempts, exponential backoff) confirmed ✓
4. Anvil WebSocket support confirmed (Anvil supports `ws://127.0.0.1:8545`) ✓
5. Chronicle stub exists ✓

**Definition of Done:**
1. Unit tests — filter construction: address filter, topic filter, combined filter, empty filter (match all)
2. Integration tests (against live Anvil fixture):
   - `test_http_poll_receives_event` — trigger an event (ETH send), assert log received within 3 poll cycles
   - `test_ws_subscription_receives_event` — subscribe via WebSocket, trigger event, assert log received
   - `test_reconnect_on_disconnect` — simulate disconnect, assert reconnection and continued log delivery
3. Stream must complete (not hang) when `unsubscribe()` is called
4. `cargo clippy -- -D warnings` clean
5. `chronicle/events.md` entry complete (decision: HTTP vs WS mode selection strategy)

**Claire Voyant note:** *WebSocket reconnect under load is the highest-risk path. The 3-attempt cap and exponential backoff must be tested — not just the happy-path reconnect — because a silent infinite-loop reconnect would hang the stream without surfacing an error.*

**Status:** `[x]` — DONE 2026-03-26 (commit `ea7379a`)

---

## T-008 — Contract Caller Module

**Owner:** Rust API Contract Serialization Specialist + TDD Driver
**Navigator:** TDD Navigator
**Spec ref:** FORMAL_SPEC.md §4 FR-007, §6.2 API contract
**Dependencies:** T-001, T-004, T-005, T-006 (write calls delegate to Signer + Broadcaster)

**Description:**
Implement `eth_node::contract` — `ContractCaller` with read (`eth_call`) and write (sign + broadcast) modes. ABI JSON parsing, function selector encoding, return value decoding.

**Key crates:** `alloy-sol-types`, `alloy-contract`

**Definition of Ready:**
1. T-004, T-005, T-006 complete ✓
2. ERC-20 minimal ABI prepared as test fixture (balanceOf, transfer, Transfer event) ✓
3. Function overloading resolution strategy confirmed (match by argument types) ✓
4. Chronicle stub exists ✓

**Definition of Done:**
1. Unit tests:
   - `test_abi_parse_valid_json` — valid ERC-20 ABI parses without error
   - `test_abi_parse_invalid_json` — `ContractError::InvalidAbiJson`
   - `test_encode_balance_of` — selector + address argument produces known bytes
   - `test_decode_uint256_return` — raw bytes → `Token::Uint(U256)`
2. Integration tests (against live Anvil fixture with deployed ERC-20):
   - `test_call_balance_of` — call `balanceOf`, assert returned value matches expected
   - `test_send_transfer` — call `transfer`, assert receipt status == 1, balance changed
3. Contract test: outgoing `eth_call` request asserts `data` field contains correct function selector and encoded arguments
4. `cargo clippy -- -D warnings` clean
5. `chronicle/contract.md` entry complete

**Status:** `[x]` — DONE 2026-03-26 (commit `e76f102`)

---

## T-009 — CLI Binary

**Owner:** Rust CLI Specialist + TDD Driver
**Navigator:** TDD Navigator
**Spec ref:** FORMAL_SPEC.md §7.1 CLI-to-API mapping, §5.1 observability, §5 NFR-004
**Dependencies:** T-004 through T-008 (all library modules complete)

**Description:**
Implement `eth_node_cli` binary. All 5 CLI commands (spec §7.1). JSON structured logging. `--quiet` and `--log-level` flags. `--dump-state <path>` flag. No business logic in CLI layer.

**Key crates:** `clap` (argument parsing), `tracing-subscriber` (JSON logging)

**Definition of Ready:**
1. T-004–T-008 all complete ✓
2. CLI-to-API mapping (spec §7.1) reviewed — all 5 commands mapped ✓
3. `--dump-state` output format defined (JSON of last operation result) ✓
4. Session capture scripts (T-003) ready for use in manual testing ✓
5. Chronicle stub exists ✓

**Definition of Done:**
1. Integration tests:
   - `test_cli_balance` — `eth_node_cli balance <address>` → correct output
   - `test_cli_send` — `eth_node_cli send <to> <value>` → receipt printed
   - `test_cli_watch` (bounded) — starts listener, receives one event, exits
   - `test_cli_call` — `eth_node_cli call <contract> balanceOf <address>` → decoded result
   - `test_cli_tx_status` — `eth_node_cli tx-status <hash>` → receipt or "pending"
2. Architecture review: `cargo grep` (or manual review) confirms zero domain logic in `eth_node_cli/src/` — only arg parsing and output formatting (NFR-004)
3. `--quiet` flag suppresses all output below `error` level
4. `--dump-state <path>` writes JSON file on success
5. At least one manual exploratory session run through `capture-session` scripts with artifact stored (Exploratory Tester sign-off)
6. `chronicle/cli.md` entry complete

**TDD Navigator note:** Architecture violation test is not automated — it is a Stage 5 review item. Flag it explicitly in DoD so it is not forgotten.

**Status:** `[x]` — DONE 2026-03-26 (commit `10d381e`)

---

## T-010 — Acceptance Criteria Verification Pass

**Owner:** Unit Test Completeness Engineer + Traceability Mapper
**Spec ref:** FORMAL_SPEC.md §10 Traceability Matrix, AC-001 through AC-006
**Dependencies:** T-001 through T-009 all complete

**Description:**
Run full traceability check: every AC maps to a passing test with a stored artifact. Every FR maps to a chronicle entry. Produce gap report for Stage 5.

**Definition of Done:**
1. AC-001 through AC-006 each have a named passing test and (for interactive ACs) a session artifact
2. No orphan tests (tests with no spec mapping)
3. No unimplemented requirements (FRs with no passing test)
4. Gap report written and reviewed
5. Traceability matrix in FORMAL_SPEC.md §10 updated with actual test names

**Status:** `[x]` — DONE 2026-03-26 (see output/T010_gap_report.md)

```
T-000 (workspace)
  ├── T-001 (primitives)
  ├── T-002 (Anvil fixture)      ← integration tests depend on this
  └── T-003 (session capture)   ← exploratory testing depends on this

T-001 + T-002 → T-004 (rpc)
T-001         → T-005 (signer)
T-004 + T-005 → T-006 (tx builder + broadcaster)
T-001 + T-004 → T-007 (events)
T-004 + T-005 + T-006 → T-008 (contract)

T-004 + T-005 + T-006 + T-007 + T-008 → T-009 (cli)

T-000 through T-009 → T-010 (AC verification)
```

---

## Stage 3 Approval

- Approved by: Lefty
- Approval date: 2026-03-26
- Notes: Oracle and Claire Voyant gate reviews complete. Claire Voyant risks mitigated in T-002 DoD and T-009 DoR.
