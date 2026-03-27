# Phase 2 Formal Specification

**Project:** eth_node Phase 2 Continuation  
**Date:** 2026-03-27  
**Mode:** Greenfield (continuation)  
**Phase:** 2 of multi-phase roadmap  
**Owner:** Lefty  
**Stage 2 Status:** Approved 2026-03-27

---

## 1. Overview

Phase 2 extends eth_node with:
- **Component #7:** Local EVM executor (revm integration) for transaction simulation.
- **Quality debt closure:** G-001 (live contract decode completeness), G-002 (fuzzing baseline).
- **Component #12 prep:** Reth environment readiness (no actual sync; preparation only).
- **Track B introduction:** Upstream test-coverage contributions to alloy-provider or revm.

This specification builds on Phase 1 deliverables (137 tests, 6 modules: primitives, rpc, tx, signer, events, contract).

---

## 2. Functional Requirements

### FR-001: Transaction Simulation (Component #7, Track A)

**Description:** Execute transactions locally using revm without broadcasting to network.

**API Surface:**
```rust
pub struct Executor {
    // Internal revm instance
}

/// Simulation context (decouples from revm::primitives::BlockEnv)
pub struct SimulationContext {
    pub block_number: u64,
    pub timestamp: u64,
    pub base_fee_per_gas: Option<u64>,
    pub gas_limit: u64,
}

pub fn simulate_tx(
    tx: TransactionRequest,
    context: SimulationContext,
) -> Result<ExecutionResult, ExecutorError>;
```

**Inputs:**
- `tx`: TransactionRequest with from, to, value, data, gas, nonce.
- `context`: SimulationContext with block number, timestamp, base fee, gas limit (converted to revm BlockEnv internally).

**Outputs:**
- `ExecutionResult`: Contains gas used, return data, logs, state changes.
- `ExecutorError`: Invalid input, revm failure, or comparison mismatch.

**Behavior:**
1. Initialize revm with provided block environment.
2. Execute transaction in isolated context.
3. Return execution result with gas usage and logs.
4. On invalid input (bad signature, wrong nonce, zero gas) → `ExecutorError::InvalidInput`.
5. On revm internal failure → `ExecutorError::RevmFailure`.

**Error Cases (Expected Failures):**
- Invalid signature → `ExecutorError::InvalidInput`
- Wrong nonce → `ExecutorError::InvalidInput`
- Insufficient gas → `ExecutorError::RevmFailure`

**Out of Scope for Phase 2:**

*Deferred to P2-003 (G-002 Fuzzing):*
- Attack scenarios (malicious calldata, reentrancy patterns)
- Adversarial inputs (random byte arrays, extreme gas limits)
- Intentional misuse testing (10k+ fuzzing iterations)

*Deferred to Phase 3 (Decision Q1):*
- **Executor contract caching:** Caching compiled contracts between simulations deferred to Phase 3 Component #8 (mempool). Phase 2 operates in one-shot simulation mode. Caching justified only when batched simulations appear.

*Phase 3 Extension Point (R6):*
- **StateProvider trait:** For state-fork scenarios (Reth historical state, Anvil custom state, custom backends), executor will accept `StateProvider` trait instead of hard-coding revm's default state. Phase 2 uses revm's default in-memory state.

**Phase 2 Implementation Note:**  
Phase 2 initializes revm with default `CacheDB`. In Phase 3, refactor to accept `impl StateProvider` via constructor. Current API remains unchanged; Phase 3 change is additive (new constructor variant).

**Acceptance Criteria:**
- **AC-001:** `simulate_tx()` executes known-good Anvil scenarios and matches gas usage within 5% tolerance.
- **AC-002:** Expected error cases return proper `ExecutorError` variants (no panic).
- **AC-003:** At least 10 simulation test cases covering: simple transfer, contract deployment, contract call, event emission, revert scenarios.

---

### FR-002: Contract Call Simulation (Component #7, Track A)

**Description:** Simulate read-only contract calls (like `eth_call`) using revm.

**API Surface:**
```rust
pub fn simulate_contract_call(
    contract_address: Address,
    calldata: Bytes,
    context: SimulationContext,
) -> Result<Bytes, ExecutorError>;
```

**Inputs:**
- `contract_address`: Target contract address.
- `calldata`: ABI-encoded function call.
- `context`: SimulationContext for simulation environment.

**Outputs:**
- `Bytes`: ABI-encoded return data.
- `ExecutorError`: Invalid input or revm failure.

**Behavior:**
1. Initialize revm with block environment.
2. Execute static call to contract (no state changes).
3. Return ABI-encoded result.
4. On invalid calldata → `ExecutorError::InvalidInput`.

**Acceptance Criteria:**
- **AC-004:** `simulate_contract_call()` matches Anvil `eth_call` results for known contracts (ERC-20 balanceOf, totalSupply).
- **AC-005:** Invalid calldata returns `ExecutorError::InvalidInput` (no panic).

---

### FR-003: Anvil Comparison Utility (Component #7, Track A)

**Description:** Compare local simulation results against Anvil reference node.

**API Surface:**
```rust
pub fn compare_to_anvil(
    tx: TransactionRequest,
    anvil_rpc_url: &str,
) -> Result<ComparisonReport, ExecutorError>;

pub struct ComparisonReport {
    pub gas_used_local: u64,
    pub gas_used_anvil: u64,
    pub gas_delta: i64,
    pub return_data_match: bool,
    pub logs_match: bool,
}
```

**Behavior:**
1. Execute transaction locally via `simulate_tx()`.
2. Send same transaction to Anvil via RPC.
3. Compare: gas used, return data, emitted logs.
4. Return structured comparison report.

**Return Semantics:**  
`compare_to_anvil()` always returns `Ok(ComparisonReport)` unless RPC failure occurs. Report contains `gas_delta` field; caller interprets threshold. Does not fail on mismatch—reporting only.

**Acceptance Criteria:**
- **AC-006:** `compare_to_anvil()` identifies gas mismatches exceeding 5% threshold.
- **AC-007:** Report includes field-level detail (which log differs, which return byte differs).

---

### FR-004: Live Contract Decode Completeness (G-001, Track A)

**Description:** Close Phase 1 acceptance criterion gap AC-005 by adding ERC-721 and ERC-1155 live-decode scenarios.

**Scope (Explicit):**

**IN SCOPE (Standard Events Only):**
- ERC-721: Transfer, Approval, ApprovalForAll
- ERC-1155: TransferSingle, TransferBatch, ApprovalForAll, URI
- Edge cases: zero-value transfers, self-transfers, max uint256 token IDs, empty arrays in TransferBatch

**OUT OF SCOPE (Defer to Phase 3):**
- Extension events: Paused, Unpaused, Burned, Minted, ConsecutiveTransfer (ERC-2309)
- Enumerable extensions (ERC-721Enumerable): event types beyond standard
- Supply tracking extensions (ERC-1155Supply): not required for Phase 2

**Test Contract Selection:** Use minimal test contracts (SimpleERC721, SimpleERC1155) OR filter events from full OpenZeppelin contracts to standard events only. If extension event encountered during testing, document in test comments: "Extension event observed but decoding deferred to Phase 3 per FR-004 scope."

**Acceptance Criteria:
- **AC-008:** All ERC-721 event types decode correctly from live Sepolia contracts.
- **AC-009:** All ERC-1155 event types decode correctly from live Sepolia contracts.
- **AC-010:** Edge cases documented in test comments with expected behavior.
- **AC-011:** At least 15 new decode tests added (5 per standard variant).

---

### FR-005: Property-Based Fuzzing Baseline (G-002, Track A)

**Description:** Close Phase 1 non-functional requirement NFR-001 by integrating property-based testing framework.

**Framework:** proptest (recommended for Rust ecosystem maturity).

**Feature Flag (R5):**  
Fuzzing tests gated behind `fuzz` feature:
- `Cargo.toml`: `[features] fuzz = []`
- **Local execution:** `cargo test --features fuzz` (opt-in; prevents 60s slowdown on every test run).
- **CI execution:** GitHub Actions workflow runs `cargo test --all-features` (includes fuzzing in CI pipeline automatically).

**Rationale:** Fuzzing adds 10k+ iterations per property (~60s); feature flag enables developer productivity without skipping quality checks in CI.

**Target Modules:**
1. **Executor module** (FR-001, FR-002 output): Never panics on random TransactionRequest inputs.
2. **Contract decoder** (FR-004 output): Never panics on random log data.
3. **ABI encoding** (Phase 1 primitives): Round-trip preserves values.

**Properties to Test:**
1. **Executor resilience:** `simulate_tx()` called with random tx params → returns `Ok` or `Err`, never panics.
2. **Decoder resilience:** Contract event decoder called with random bytes → returns `Ok` or `Err`, never panics.
3. **ABI round-trip:** `encode(decode(x)) == x` for all supported ABI types.

**Adversarial Inputs (Included in Fuzzing):**
- Random byte arrays (0-10k bytes).
- Extreme gas limits (0, u64::MAX).
- Malicious calldata patterns (buffer overruns, format string attacks).
- Reentrancy simulation attempts.

**Acceptance Criteria:**
- **AC-012:** Fuzzing framework integrated; runs via `cargo test --features fuzz`.
- **AC-013:** 10k+ iterations per property without panic.
- **AC-014:** Fuzzing success criteria (3-tier threshold):
  - **Pass definition:** Property passes if returns `Ok` or `Err` without panic. Panic = fail.
  - **≥95% pass rate:** A-2 closure approved automatically.
  - **90-94% pass rate:** Owner approval required; document gap assessment (security impact? user-facing?).
  - **<90% pass rate:** A-2 BLOCKED; extend T-005 to 25hr cap or triage panics (fix critical, document non-critical).
  - **Phase 1 panic policy:** If fuzzing discovers panic in Phase 1 module (primitives, RPC, signer): critical-path panics (executor, signer, RPC core) MUST be fixed before A-2 closure; non-critical panics (debug utils, pretty-printing, CLI helpers) MAY be documented as known limitations. Time-box Phase 1 fixes: 8 hours maximum; remaining panics documented and deferred to Phase 3.
- **AC-015:** Fuzzing tests run in <60 seconds on CI (timeout protection).

---

### FR-006: Reth Environment Readiness Checklist (A-3 Prep, Track A)

**Description:** Prepare environment for future Reth Sepolia sync experiments (no actual integration in Phase 2).

**Deliverable:** `docs/reth_readiness_checklist.md`

**Content:**
1. **Disk space validation:**
   - Command to check free space on target drive.
   - Requirement: ≥50 GB free after Phase 2 artifacts.
2. **Environment configuration template:**
   - `RETH_DATA_DIR`: Path to chain data directory.
   - `RETH_CHAIN`: Sepolia network identifier.
   - `RETH_RPC_URL`: Optional bootstrap node (Infura or Alchemy).
3. **Reth installation guide:**
   - Build from source (Git clone + cargo build --release).
   - OR binary download from GitHub releases.
4. **Dry-run script:**
   - PowerShell script that validates prerequisites without downloading chain data.
   - Checks: disk space, Rust toolchain, network connectivity to bootstrap node.
5. **Rollback procedure:**
   - If sync fails, how to clean data directory and restart.

**Acceptance Criteria:**
- **AC-016:** Checklist is runnable on Windows with 190 GB SSD.
- **AC-017:** Dry-run script validates environment without downloading chain data.
- **AC-018:** Documentation includes rollback steps if sync fails or disk fills.

---

### FR-007: Upstream Test-Coverage Audit (Track B)

**Description:** Identify test coverage gaps in alloy-provider or revm; produce failing test case demonstrating gap.

**Audit Sequence:**
1. Audit alloy-provider first (dependency for A-1).
   - Focus: `Provider::call()` and `Provider::send_raw_transaction()` error paths.
2. If threshold met (≥10% coverage delta OR critical edge case): proceed to test contribution.
3. If threshold not met: audit revm next.
   - Focus: `EVM::transact()` with malformed bytecode or invalid precompile calls.

**Coverage Methodology:**
- Clone target repository.
- Run `cargo tarpaulin --out Html` or `cargo llvm-cov`.
- Identify modules with <90% branch coverage.
- Analyze uncovered branches for critical-path impact.

**Deliverable:** `src/upstream_contrib/audits/AUDIT_REPORT_001.md`

**Report Content:**
1. Repository audited (name, commit hash, date).
2. Module/function analyzed.
3. Coverage percentage (before contribution).
4. Identified gap (missing test case description).
5. Failing test case code (demonstrating gap).
6. Decision: pursue contribution (yes/no) with threshold justification.

**Acceptance Criteria:**
- **AC-019:** At least one candidate audited with coverage data.
- **AC-020:** Gap meets threshold (≥10% delta OR critical-path edge case: panic, wrong-value, or security-relevant).
- **AC-021:** Failing test case written and validated locally in upstream fork.

---

## 3. Non-Functional Requirements

### NFR-001: Performance

- **Executor simulation:** Latency baseline established during T-003 implementation by measuring 100 varied transactions (simple transfer, ERC-20 call, complex contract interaction). Target: ≤2x Anvil RPC round-trip time for equivalent transactions. If empirical baseline exceeds 200ms, justify with revm version and hardware specs.
- **Fuzzing throughput:** 10k+ iterations complete in <60 seconds.
- **Anvil comparison:** <200ms round-trip (local simulation + RPC call).

### NFR-002: Error Handling

- **No panics:** All public APIs return `Result<T, ExecutorError>` on invalid input.
- **Defensive design:** Validate inputs before passing to revm or RPC layer.
- **Error detail:** `ExecutorError` variants include context (which validation failed, which field).

### NFR-003: Testability

- **Unit tests:** Each function has at least 3 test cases (happy path, expected error, edge case).
- **Integration tests:** Executor modules tested against live Anvil instance.
- **Fuzzing coverage:** All public APIs fuzzed with proptest.

### NFR-004: Documentation

- **API docs:** All public functions have rustdoc with examples.
- **Chronicle entries:** Each task links to chronicle document (CHR-009, CHR-010, etc.).
- **Audit traceability:** Track B work links upstream PR to local audit report.

---

## 4. Success Criteria

**Phase 2 complete when:**

1. ✅ All FR-001 through FR-007 implemented and tested.
2. ✅ All AC-001 through AC-021 verified.
3. ✅ Test suite grows to 170+ tests passing (Phase 1 baseline: 137, Phase 2 new: ~33).
4. ✅ All Phase 1 tests still pass (regression check).
5. ✅ G-001 and G-002 explicitly closed (no remaining Phase 1 quality debt).
6. ✅ At least one Track B contribution submitted (if threshold met) or documented (if threshold not met). **Stage 6 success = PR submitted with traceability link.** Merge outcome (accepted/rejected/pending) tracked in feedback.json but does NOT block Stage 6 closure.
7. ✅ Reth readiness checklist validated on target hardware.

---

## 5. Dependencies

**External:**
- **revm:** Core dependency for FR-001, FR-002.
- **proptest:** Fuzzing framework for FR-005.
- **cargo-tarpaulin or cargo-llvm-cov:** Coverage tooling for FR-007 audit.

**Internal (Phase 1):**
- **primitives module:** Address parsing, ABI/RLP codecs used by executor.
- **rpc module:** Provider client used in `compare_to_anvil()`.
- **contract module:** ABI encoding/decoding used in contract call simulation.

---

## 6. Out of Scope for Phase 2

**Deferred to Phase 3 or later:**
- Component #8: Mempool monitor.
- Component #9: Block + receipt indexer.
- Component #11: EVM state trie reader.
- Actual Reth Sepolia sync and execution (A-3 prep only in Phase 2).
- Multi-contributor team expansion.
- Separate repository split for Track B.
- Executor contract caching (Q1: deferred until batched simulations needed).
- StateProvider abstraction (R6: extension point for state-fork scenarios).

---

## 7. Detailed Requirements by Component

### 7.1 Component #7: EVM Executor (FR-001, FR-002, FR-003)

**Module:** `src/executor/`

**Files:**
- `lib.rs`: Public API surface, error types, type exports.
- `simulate.rs`: Core simulation logic using revm.
- `compare.rs`: Anvil comparison utility.

**Error Type Hierarchy:**
```rust
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Invalid transaction input: {0}")]
    InvalidInput(String),
    
    #[error("Revm execution failed: {0}")]
    RevmFailure(String),
    
    #[error("Anvil comparison mismatch: {0}")]
    AnvilMismatch(String),
    
    #[error("RPC error: {0}")]
    RpcError(#[from] alloy_provider::Error),
    
    #[error("Execution error: {message}")]
    Context {
        message: String,
        details: HashMap<String, String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

// Example usage:
// ExecutorError::Context {
//     message: "Simulation failed during execution".into(),
//     details: [
//         ("tx_hash", "0x123..."),
//         ("block_number", "12345"),
//         ("revm_error", "OutOfGas"),
//     ].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
//     source: Some(Box::new(revm_error)),
// }
```

**Error Classification Rules:**

| Error Scenario | Classification | Rationale |
|---------------|----------------|-----------||
| Invalid signature | `InvalidInput` | Caller provided bad data; fix before calling |
| Wrong nonce (too high/low) | `InvalidInput` | Caller's nonce management error |
| Zero gas limit | `InvalidInput` | Caller must provide non-zero gas |
| Gas limit exceeds block gas limit | `InvalidInput` | Caller's responsibility to check block limit |
| Insufficient sender balance | `RevmFailure` | EVM rejected (valid tx, insufficient funds) |
| Contract revert with reason | `RevmFailure` | EVM executed and reverted (valid behavior) |
| Invalid opcode in bytecode | `RevmFailure` | EVM rejected during execution |
| Stack overflow during execution | `RevmFailure` | EVM internal limit reached |
| Out of gas during execution | `RevmFailure` | EVM ran out of gas (valid exhaustion) |
| Precompile call failed | `RevmFailure` | EVM precompile rejected input |
| Precompile input validation failure | `InvalidInput` | Caller provided malformed precompile args (e.g., ecrecover with invalid signature bytes); discovered during execution but represents caller data error |
| Invalid contract address (0x0) | `InvalidInput` | Caller provided invalid target |
| Malformed calldata (wrong ABI) | `InvalidInput` | Caller's encoding error |

**Classification Principle:** `InvalidInput` = caller can fix without execution; `RevmFailure` = EVM attempted execution and rejected.

**Test Strategy:**
- **Unit tests:** Each function with known inputs/outputs (at least 3 per function).
- **Integration tests:** Execute against live Anvil instance; compare gas and logs.
- **Regression tests:** Ensure Phase 1 RPC/tx modules still work after executor integration.

**Traceability:**
- Chronicle entry: CHR-009-executor.md
- Links to: PHASE2_PROJECT_BRIEF.md Section 5 (P2-001)

---

### 7.2 G-001 Implementation Notes (FR-004)

#### 7.2.1 Event Signature Reference

**Standard Event Signatures (EIP-721, EIP-1155):**

| Standard | Event | Signature | Topic[0] Hash |
|----------|-------|-----------|---------------|
| ERC-721 | Transfer | `Transfer(address indexed from, address indexed to, uint256 indexed tokenId)` | `0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef` |
| ERC-721 | Approval | `Approval(address indexed owner, address indexed approved, uint256 indexed tokenId)` | `0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925` |
| ERC-721 | ApprovalForAll | `ApprovalForAll(address indexed owner, address indexed operator, bool approved)` | `0x17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31` |
| ERC-1155 | TransferSingle | `TransferSingle(address indexed operator, address indexed from, address indexed to, uint256 id, uint256 value)` | `0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62` |
| ERC-1155 | TransferBatch | `TransferBatch(address indexed operator, address indexed from, address indexed to, uint256[] ids, uint256[] values)` | `0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb` |
| ERC-1155 | ApprovalForAll | `ApprovalForAll(address indexed account, address indexed operator, bool approved)` | `0x17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31` |
| ERC-1155 | URI | `URI(string value, uint256 indexed id)` | `0x6bb7ff708619ba0610cba295a58592e0451dee2622938c8755667688daf3529b` |

**Sources:** [EIP-721](https://eips.ethereum.org/EIPS/eip-721), [EIP-1155](https://eips.ethereum.org/EIPS/eip-1155)

#### 7.2.2 Implementation Details

**Critical edge cases:**
- Zero-value transfers, self-transfers, max uint256 token IDs, empty arrays in TransferBatch, non-ASCII URI strings

**Test approach:** Deploy test contracts to Anvil, trigger events, capture via RPC, decode and validate.

---

### 7.3 G-002 Implementation Notes (FR-005)

**Proptest code examples:**

```rust
// Executor resilience
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]
    #[cfg(feature = "fuzz")]
    fn executor_never_panics(tx in arb_transaction_request()) {
        let result = simulate_tx(tx, default_context());
        // Assert: result is Ok or Err, never panics
    }
}

// Decoder resilience
proptest! {
    #[cfg(feature = "fuzz")]
    fn decoder_never_panics(log_data in prop::collection::vec(any::<u8>(), 0..1024)) {
        let result = decode_event_log(&log_data);
        // Assert: result is Ok or Err, never panics
    }
}

// ABI round-trip
proptest! {
    #[cfg(feature = "fuzz")]
    fn abi_roundtrip_preserves_value(value in arb_abi_token()) {
        assert_eq!(decode(&encode(&value)), value);
    }
}
```

---

### 7.4 A-3 Prep Implementation Notes (FR-006)

**PowerShell dry-run script example:**

```powershell
# scripts/reth_dryrun.ps1
$freespace = (Get-PSDrive C).Free / 1GB
if ($freespace -lt 50) {
    Write-Error "Insufficient disk: $freespace GB free (need 50 GB)"
    exit 1
}
Write-Host "Disk check: OK ($freespace GB free)"

# Check Rust toolchain
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Rust toolchain not found"
    exit 1
}
Write-Host "Rust toolchain: OK"
```

**Environment template:**
```
RETH_DATA_DIR=C:\Users\geb\Documents\reth_data\sepolia
RETH_CHAIN=sepolia
RETH_RPC_URL=https://sepolia.infura.io/v3/YOUR_PROJECT_ID
```

---

### 7.5 TrackB Audit Implementation Notes (FR-007)

**Coverage analysis commands:**
```bash
git clone https://github.com/alloy-rs/alloy.git
cd alloy/crates/provider
cargo tarpaulin --out Html
```

**Failing test example:**
```rust
#[tokio::test]
async fn test_send_tx_nonce_collision() {
    // Demonstrates gap: two txs with same nonce submitted rapidly
    // Expected: second tx fails with proper error, no panic
}
```

**Decision criteria:** Coverage delta ≥10% OR critical-path edge case (panic, security issue, wrong-value error).

---

## 8. Test Plan

### 8.1 Test Categories

| Category | Count Target | Examples |
|----------|--------------|----------|
| Executor unit tests | 10+ | `simulate_tx()` with known inputs, error cases |
| Executor integration tests | 5+ | Compare to Anvil, gas matching |
| Decode completeness tests | 15+ | ERC-721/ERC-1155 all event types + edge cases |
| Fuzzing properties | 3+ | Executor resilience, decoder resilience, ABI round-trip |
| Phase 1 regression tests | 137 | All existing tests still pass |
| **Total Phase 2 target** | **170+** | Phase 1 baseline (137) + Phase 2 new (~33) |

### 8.2 Coverage Targets

- **Line coverage:** ≥85% for new modules (executor, quality).
- **Branch coverage:** ≥80% for error-handling paths.
- **Fuzzing iteration count:** 10k+ per property.

### 8.3 Test Execution

- **Local:** `cargo test` (all tests run on developer machine).
- **CI:** GitHub Actions workflow (existing ci.yml; no changes needed).
- **Anvil dependency:** Integration tests start local Anvil instance before execution.

### 8.4 Anvil Test Lifecycle

Integration tests requiring Anvil use `std::process::Command` to spawn `anvil` in test setup and terminate in teardown. Each test spawns on random port to avoid conflicts.

**Helper function** in `tests/common/mod.rs`:
```rust
pub fn spawn_anvil() -> (AnvilHandle, RpcUrl) {
    // Spawns anvil on random port, returns handle and URL
}
```

**Rationale:** Removes manual Anvil startup step; ensures test isolation; prevents "works on my machine" issues when different tests share state.

---

## 9. Implementation Constraints

### 9.1 Sequencing (Mandatory)

1. **A-2 before A-3:** P2-002 + P2-003 must complete before P2-004 begins.
2. **A-1 before A-2:** P2-001 completes before P2-002/P2-003 start (foundation before quality).
3. **Track B after A-2:** P2-005 begins after P2-002/P2-003 complete.

### 9.2 Parallelism Limit

- **Max 2 parallel tasks:** 1 from Track A, 1 from Track B.
- **Solo developer context:** Minimize context-switching; complete one increment fully before opening next.

### 9.3 Permission Gates

- **Track A:** Standard governance gates (Stage 1-6 approvals).
- **Track B:** Additional permission gate before upstream PR submission (Step 17 in PHASE2_TRANSITION_TASKLIST.md).

### 9.4 Module Dependency Rules

**One-way dependency from executor to contract (R3):**
- `src/executor/` MAY import from `src/contract/` (use event decoding, ABI utilities).
- `src/contract/` MUST NOT import from `src/executor/` (prevents circular dependencies in Phase 3).

**Track B isolation policy (R4):**
- `src/upstream_contrib/` code MUST NOT import from `src/executor/`, `src/quality/`, or any other `eth_node` module.
- Purpose: Prepares Track B code for future extraction into separate repository; eliminates coupling risk.
- Enforcement: Task acceptance criteria include "no eth_node imports" check.

**Rationale:** Prevents circular dependency cycles (Phase 3 feature risk) and ensures Track B contributions remain portable (future hub-model migration).

---

## 10. Traceability Matrix

| Requirement | Chronicle | Task IDs | Acceptance Criteria | Test Files |
|-------------|-----------|----------|---------------------|------------|
| FR-001 (simulate_tx) | CHR-009 | T-001 | AC-001, AC-002, AC-003 | tests/executor_sim.rs |
| FR-002 (contract call) | CHR-009 | T-002 | AC-004, AC-005 | tests/executor_call.rs |
| FR-003 (Anvil compare) | CHR-009 | T-003 | AC-006, AC-007 | tests/executor_compare.rs |
| FR-004 (G-001 decode) | CHR-010 | T-004 | AC-008, AC-009, AC-010, AC-011 | tests/decode_live.rs |
| FR-005 (G-002 fuzz) | CHR-011 | T-005 | AC-012, AC-013, AC-014, AC-015 | tests/fuzz_properties.rs |
| FR-006 (A-3 prep) | CHR-012 | T-009 | AC-016, AC-017, AC-018 | docs/reth_readiness_checklist.md |
| FR-007 (Track B audit) | CHR-013 | T-006, T-007, T-008 | AC-019, AC-020, AC-021 | src/upstream_contrib/audits/AUDIT_REPORT_001.md |

---

## 11. Stage 2 Gate Approval

**Approver:** Product Owner (Lefty)  
**Approval Date:** 2026-03-27  
**Approval Notes:** Stage 2 gate approved. All functional requirements (FR-001 through FR-007), non-functional requirements (NFR-001 through NFR-004), and acceptance criteria clearly defined. Attack scenario scope explicitly separated (P2-001 vs P2-003). Proceed to Stage 3 (task planning).

---

## Revision History

- 2026-03-27: Created from Stage 2 specification decomposition (A-1, A-2, A-3 prep, Track B audit).
- 2026-03-27: Applied architect recommendations R1-R7 (SimulationContext wrapper, ExecutorError::Context, module dependencies, Track B isolation, fuzzing feature flag, StateProvider extension point, extraction triggers).
- 2026-03-27: Consolidated Section 7 to remove duplication; condensed from 544 to 453 lines (~17% reduction). Fixed header status (approved) and test count (170+).
