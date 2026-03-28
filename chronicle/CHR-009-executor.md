# CHR-009 Executor

## 1. Chronicle Metadata

- Chronicle ID: CHR-009
- Source task ID: T-000, T-001, T-002, T-003
- Source spec sections: PHASE2_FORMAL_SPEC.md FR-001, FR-002, FR-003
- Source requirements: AC-001 through AC-007
- Module / component name: executor
- Implementation language: Rust
- Author: GitHub Copilot
- Date: 2026-03-28
- Status: Final

## 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - FR-001 via `simulate_tx()` local EVM simulation with structured `SimulationResult`.
  - FR-002 via `simulate_contract_call()` static-call simulation with input validation.
  - FR-003 via `compare_to_anvil()` comparison report over local simulation vs Anvil RPC reference.
- What must remain functionally equivalent across languages:
  - Error semantics (`InvalidInput`, `RevmFailure`) and report semantics (`Ok(report)` unless RPC failure).
- What is intentionally language-specific in this implementation:
  - revm `CacheDB`/`EmptyDB` setup and Rust enum-based result mapping.

## 3. Implementation Decisions

- Data structures chosen and why:
  - `SimulationContext` to decouple caller context from revm `BlockEnv`.
  - `SimulationResult` for gas/return/log/success contract.
  - `ComparisonReport` for explicit delta fields + textual differences.
- Algorithms chosen and why:
  - `simulate_tx()` maps revm `ExecutionResult` variants directly.
  - `compare_to_anvil()` uses `eth_estimateGas` + `eth_call` to remain signer-agnostic while preserving deterministic comparison.
- Control-flow structure chosen and why:
  - Compare path accumulates differences instead of failing on mismatch to meet FR-003 semantics.
- Boundary and interface decisions:
  - Executor API is free functions exported via `executor/mod.rs`.
- Error-handling strategy:
  - Validate preconditions early; return `ExecutorError` without panics.
  - RPC failures are propagated as `Err`; mismatches are reported in `ComparisonReport`.
- Performance or memory trade-offs accepted:
  - In-memory `CacheDB` only; no contract/state caching in Phase 2.
- File map (concrete files changed/created):
  - `src/eth_node/src/executor/mod.rs`
  - `src/eth_node/src/executor/simulate.rs`
  - `src/eth_node/src/executor/compare.rs`
  - `src/eth_node/tests/executor_sim.rs`
  - `src/eth_node/tests/executor_anvil_compare.rs`
  - `src/eth_node/tests/executor_call.rs`
  - `src/eth_node/tests/executor_compare.rs`
- Public symbols introduced/changed:
  - `simulate_tx`, `simulate_contract_call`, `compare_to_anvil`
  - `SimulationContext`, `SimulationResult`, `ComparisonReport`, `ExecutorError`
- Signature snapshot (functions/classes/types and key fields):
  - `pub fn simulate_tx(tx: &TransactionRequest, context: &SimulationContext) -> Result<SimulationResult, ExecutorError>`
  - `pub fn simulate_contract_call(contract_address: Address, calldata: Bytes, context: &SimulationContext) -> Result<Bytes, ExecutorError>`
  - `pub async fn compare_to_anvil(tx: &TransactionRequest, anvil_rpc_url: &str, context: &SimulationContext) -> Result<ComparisonReport, ExecutorError>`

## 4. Alternatives Considered

- Alternative 1: Broadcast signed tx in `compare_to_anvil()` using `tx::send_transaction`.
- Why rejected: Requires signer/private-key plumbing not present in FR-003 function signature.
- Alternative 2: Treat local simulation failure as hard error.
- Why rejected: FR-003 requires mismatch reporting instead of failure when possible.

## 5. Derived Invariants and Constraints

- Invariant 1: `simulate_contract_call()` rejects calldata shorter than 4 bytes.
- Invariant 2: `compare_to_anvil()` returns `Ok(ComparisonReport)` for data mismatches.
- Constraints inherited from the spec:
  - AC-001 through AC-007 must be covered by tests.
- Additional implementation constraints introduced:
  - Anvil reference comparison path uses `eth_estimateGas`/`eth_call`; log parity limited without signed broadcast.
- Boundary behavior and terminal/end-state rules:
  - RPC failure is the only hard-fail condition for compare path.

## 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - FR-003 says "send same transaction to Anvil" but signature does not include signer.
- How the ambiguity was resolved in code:
  - Implemented signer-agnostic reference path (`eth_estimateGas`, `eth_call`) and explicit diff reporting.
- Any controlled divergence from the spec:
  - Log parity in compare path is reported as unavailable when local logs exist.
- Follow-up needed in the spec or task list:
  - If strict log-by-log parity is required, extend FR-003 signature with signer input or signed payload.

## 7. Testing Notes

- Unit tests added:
  - `executor_sim.rs` expanded to 11 tests for simulate_tx behavior and error cases.
  - `executor_call.rs` 4 tests (including invalid calldata and Anvil parity on empty account).
- Integration tests added:
  - `executor_anvil_compare.rs` simple-transfer gas parity.
  - `executor_compare.rs` 3 tests for report shape, gas-threshold detection, field-level diff detail.
- Property-based tests added:
  - None in T-003 (scheduled in T-005).
- Edge cases covered:
  - Missing fields, zero gas, short calldata, empty calldata, self-transfer, value+data transfer.
- Failure modes exercised:
  - Revert, halt/error mapping, fee-floor conflicts, RPC mismatch reporting.
- Runtime/orchestration branch evidence covered:
  - Anvil-backed tests for executor call/comparison paths.
- Golden input/output examples captured:
  - 21k gas transfer parity and eth_call parity on empty target.

## 7.1 Verification Artifact (mandatory — entry is incomplete without this field)

Verification artifact:
- `cargo test --package eth_node` passed with all suites green:
  - unit: 79 passed
  - executor integration: `executor_anvil_compare` 1 passed, `executor_call` 4 passed, `executor_compare` 3 passed, `executor_sim` 10 passed / 1 ignored
  - other integration: `integration_anvil_fixture` 2 passed, `integration_contract` 6 passed, `integration_events` 5 passed, `integration_rpc` 6 passed, `integration_tx` 5 passed
  - doc-tests: 6 passed / 3 ignored

## 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
  - Build around revm `Evm` and map all result variants explicitly.
  - Keep compare utility mismatch-reporting behavior non-failing for non-RPC mismatches.
- Order of implementation steps that mattered:
  - T-000 structure -> T-001 scaffold -> T-002 transaction simulation -> T-003 call/comparison.
- Non-obvious pitfalls discovered during implementation:
  - `gas_price` must meet basefee constraints to avoid `GasPriceLessThanBasefee`.
- What not to change without updating the behavioral spec:
  - compare return semantics (`Ok(report)` on mismatches).
- Side effects and external touchpoints (files/network/stdout/runtime context):
  - Anvil process spawn in integration tests, RPC calls to local endpoint.

## 9. Known Limitations

- Limitation 1: compare log parity cannot be fully validated without signed tx broadcast path.
- Reason accepted:
  - FR-003 signature currently lacks signer credentials.
- Revisit trigger:
  - Phase 3 or FR-003 revision adding signer input.

## 10. Approval / Review

- Reviewed by: Pending user review
- Review date: 2026-03-28
- Notes: Ready for A-1 gate probe and approval decision.
- Pair-programming references (if applicable):
  - Session log path: N/A (single-agent implementation with XP workflow constraints)
  - Driver role(s): TDD Driver workflow simulated in sequence
  - Navigator/reviewer role(s): TDD Navigator checks simulated through test-first progression
  - Key disagreement and resolution summary: Anvil reference path selected over signer-dependent broadcast due signature constraints

## 11. Reconstruction Bundle (required at Stage 4 close for major modules)

- Source tree manifest relevant to this module:
  - `src/eth_node/src/executor/*`
  - `src/eth_node/tests/executor_*`
- Import/dependency graph expectations:
  - revm + alloy primitives/rpc types + internal rpc client.
- Recommended reconstruction order:
  1. error/context/result types
  2. simulate_tx
  3. simulate_contract_call
  4. compare_to_anvil
  5. unit + integration tests
- Post-rebuild validation commands:
  - `cargo build --package eth_node`
  - `cargo test --package eth_node`
- Expected validation outputs/pass criteria:
  - full package passes with executor suites green and no compile errors.
