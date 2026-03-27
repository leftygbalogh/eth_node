# Phase 2 Task List

**Project:** eth_node Phase 2 Continuation  
**Date:** 2026-03-27  
**Phase:** 2 of multi-phase roadmap  
**Owner:** Lefty  
**Stage 3 Status:** Task planning in progress; pending Stage 3 gate approval

---

## Task Overview

Phase 2 delivers 9 tasks across Track A (product continuation) and Track B (upstream contributions):
- **Track A:** 7 tasks (executor foundation, quality closure, Reth prep)
- **Track B:** 2 tasks (upstream audit, PR submission)
- **Total estimated effort:** ~40-50 hours (solo developer, learning included)

**Chronicle Entry Requirements:**  
Each task links to a chronicle document (CHR-009, CHR-010, CHR-011, CHR-012, CHR-013). Minimum sections per chronicle: Context, Implementation, Decisions, Tests, Links to Spec. Reference `templates/IMPLEMENTATION_CHRONICLE_TEMPLATE.md` for structure.

---

## Sequencing Rules

1. **Mandatory:** A-2 (T-004 + T-005) completes before A-3 (T-009) begins.
2. **Foundation first:** A-1 (T-001, T-002, T-003) completes before A-2 begins.
3. **Track B after A-2:** T-006 begins after T-004 + T-005 complete.
4. **Max 2 parallel:** 1 Track A task + 1 Track B task active simultaneously.

---

## Task List

### T-001: Setup revm Integration Boilerplate (Track A)

**Description:** Add revm dependency and create executor module scaffold.

**Scope:**
- Add `revm = "6.0"` (or latest stable) to Cargo.toml.
- Create `src/executor/lib.rs` with module exports.
- Create `src/executor/simulate.rs` with empty function stubs for `simulate_tx()`.
- Add basic error type `ExecutorError` with `InvalidInput`, `RevmFailure` variants.

**Dependencies:** None (can start immediately after Stage 3 approval).

**Deliverable:**
- `src/executor/lib.rs`
- `src/executor/simulate.rs`
- Updated `Cargo.toml`

**Acceptance Criteria:**
- Cargo build succeeds with revm dependency.
- Module compiles with stub functions.
- Error types defined with proper `thiserror` derives.

**Estimated Effort:** 2-3 hours

**Chronicle:** CHR-009-executor.md (Section: Boilerplate Setup)

**Links to Spec:** PHASE2_FORMAL_SPEC.md FR-001

---

### T-002: Implement simulate_tx() with Known-Good Inputs (Track A)

**Description:** Implement core transaction simulation logic using revm.

**Scope:**
- Initialize revm `EVM` instance with provided `BlockEnv`.
- Execute transaction via `EVM::transact()`.
- Map revm `ExecutionResult` to eth_node `ExecutionResult` struct.
- Handle expected error cases: invalid signature → `ExecutorError::InvalidInput`, wrong nonce → `ExecutorError::InvalidInput`, insufficient gas → `ExecutorError::RevmFailure`.

**Test Requirements:**
- At least 5 unit tests: simple transfer, contract deployment, contract call with return value, event emission, revert scenario.
- Integration test: compare `simulate_tx()` result to same tx executed on local Anvil (gas within 5% tolerance).

**Dependencies:** T-001 complete.

**Deliverable:**
- Implemented `simulate_tx()` in `src/executor/simulate.rs`
- Test file: `tests/executor_sim.rs` (≥5 tests)

**Acceptance Criteria:**
- AC-001: All known Anvil test scenarios match simulation results (gas, return value, logs).
- AC-002: Expected error cases return proper `ExecutorError` (no panic).

**Estimated Effort:** 8-10 hours

**Chronicle:** CHR-009-executor.md (Section: Transaction Simulation)

**Links to Spec:** PHASE2_FORMAL_SPEC.md FR-001, AC-001, AC-002

---

### T-003: Implement Contract Call Simulation and Anvil Comparison (Track A)

**Description:** Add `simulate_contract_call()` for read-only calls and `compare_to_anvil()` utility.

**Scope:**
- Implement `simulate_contract_call()` using revm static call (no state changes).
- Implement `compare_to_anvil()`: execute tx locally + via RPC, compute delta report.
- Return structured `ComparisonReport` with gas delta, return data match, logs match.

**Test Requirements:**
- Unit tests: ERC-20 `balanceOf()`, `totalSupply()` calls (known return values).
- Integration test: Deploy contract to Anvil, call via `simulate_contract_call()`, compare to RPC `eth_call`.
- Comparison test: Execute same tx via `simulate_tx()` and Anvil; validate `ComparisonReport` accuracy.

**Dependencies:** T-002 complete.

**Deliverable:**
- Implemented `simulate_contract_call()` and `compare_to_anvil()` in `src/executor/simulate.rs` and `src/executor/compare.rs`
- Test file: `tests/executor_call.rs`, `tests/executor_compare.rs` (≥3 tests each)

**Acceptance Criteria:**
- AC-003: `compare_to_anvil()` reports differences with exact field-level detail.
- AC-004: `simulate_contract_call()` matches Anvil `eth_call` results for known contracts.
- AC-005: Invalid calldata returns `ExecutorError::InvalidInput` (no panic).
- AC-006: `compare_to_anvil()` identifies gas mismatches exceeding 5% threshold.
- AC-007: Report includes field-level detail (which log differs, which return byte differs).

**Estimated Effort:** 6-8 hours

**Chronicle:** CHR-009-executor.md (Section: Contract Calls and Comparison)

**Links to Spec:** PHASE2_FORMAL_SPEC.md FR-002, FR-003, AC-003 through AC-007

---

### A-1 Completion Gate (T-003 → T-004/T-005)

Before starting T-004 or T-005, verify A-1 foundation is complete:
- ✅ All AC-001 through AC-007 passing
- ✅ T-003 committed to master
- ✅ No blocking executor issues in memory.md
- ✅ CHR-009-executor.md complete with traceability links

---

### T-004: Close G-001 Live Decode Completeness (Track A)

**Description:** Expand contract event decoder to handle ERC-721 and ERC-1155 edge cases.

**Scope:**
- Add ERC-721 event decoders: Transfer, Approval, ApprovalForAll.
- Add ERC-1155 event decoders: TransferSingle, TransferBatch, ApprovalForAll, URI.
- Handle edge cases: zero-value transfers, self-transfers, max uint256 token IDs, empty arrays.

**Test Requirements:**
- Deploy test ERC-721 and ERC-1155 contracts to local Anvil.
- Trigger each event variant via contract calls.
- Capture logs via RPC and decode; validate field extraction.
- At least 15 new tests (5 ERC-721, 10 ERC-1155 including edge cases).

**Dependencies:** T-001, T-002, T-003 complete (A-1 foundation done).

**Deliverable:**
- Extended `src/contract/decode.rs` (or new `src/quality/decode.rs`)
- Test file: `tests/decode_live.rs` (≥15 tests)
- Test contracts: `tests/contracts/TestERC721.sol`, `tests/contracts/TestERC1155.sol`

**Acceptance Criteria:**
- AC-008: All ERC-721 event types decode correctly with proper field extraction.
- AC-009: All ERC-1155 event types decode correctly with proper field extraction.
- AC-010: Edge cases documented in test comments with expected behavior.
- AC-011: At least 15 new decode tests added.

**Estimated Effort:** 6-8 hours

**Chronicle:** CHR-010-decode-completeness.md

**Links to Spec:** PHASE2_FORMAL_SPEC.md FR-004, AC-008 through AC-011  
**Links to Phase 1:** FORMAL_SPEC.md AC-005 (gap closure)

---

### T-005: Integrate Property-Based Fuzzing Framework (Track A)

**Description:** Close G-002 by integrating proptest; add fuzzing properties for executor, decoder, ABI encoding.

**Scope:**
- Add `proptest = "1.0"` to Cargo.toml dev-dependencies.
- Add feature flag to Cargo.toml: `[features]` section with `fuzz = []`.
- Create `src/quality/fuzz.rs` with property definitions.
- Fuzz properties:
  1. Executor never panics on random `TransactionRequest`.
  2. Decoder never panics on random log bytes.
  3. ABI encoding round-trip preserves values.
- Include adversarial inputs: random bytes (0-10k), extreme values (u64::MAX), malicious calldata patterns.
- Gate fuzzing tests behind `fuzz` feature: `#[cfg(feature = "fuzz")]` attribute on test modules.

**Feature Flag Execution (R5):**
- **Local:** `cargo test --features fuzz` (opt-in; prevents 60s slowdown on standard test runs).
- **CI:** Update `.github/workflows/ci.yml` to run `cargo test --all-features` (includes fuzzing automatically).

**Test Requirements:**
- At least 3 proptest properties with 10k+ iterations each.
- Tests complete in <60 seconds total (timeout protection).
- Any discovered panics fixed or documented as known limitations.

**Dependencies:** T-004 complete (decoder extended; A-2 in progress).

**Deliverable:**
- `src/quality/fuzz.rs`
- Test file: `tests/fuzz_properties.rs` (≥3 properties, gated behind `#[cfg(feature = "fuzz")]`)
- Cargo.toml updated with `fuzz = []` feature.

**Acceptance Criteria:**
- AC-012: Fuzzing framework integrated; `cargo test --features fuzz` runs fuzz tests.
- AC-013: 10k+ iterations per property without panic.
- AC-014: ≥95% of properties pass 10k iterations without panic. If <95%, document unfixed edges as known limitations with security impact assessment. Any critical-path panic (executor, signer, RPC) MUST be fixed before A-2 closure; non-critical panics (debug utils, pretty-printing) can be documented.
- AC-015: Fuzzing tests complete in <60 seconds on CI.

**Estimated Effort:** 8-10 hours (includes fixing any discovered panics)

**Chronicle:** CHR-011-fuzzing-baseline.md

**Links to Spec:** PHASE2_FORMAL_SPEC.md FR-005, AC-012 through AC-015  
**Links to Phase 1:** FORMAL_SPEC.md NFR-001 (gap closure)

---

### T-006: Audit Upstream Test Coverage (Track B)

**Description:** Audit alloy-provider (or revm if alloy-provider doesn't meet threshold) for test coverage gaps.

**Scope:**
- Clone alloy repository: `git clone https://github.com/alloy-rs/alloy.git`
- Run coverage analysis on provider crate: `cargo tarpaulin --out Html` or `cargo llvm-cov`
- Identify modules with <90% branch coverage.
- Analyze uncovered branches: error-handling paths, edge cases, timeout scenarios.
- Select one gap meeting threshold (≥10% coverage delta OR critical-path edge case).
- Write failing test case demonstrating gap.

**Dependencies:** T-004 + T-005 complete (A-2 done; Track B begins after A-2 per sequencing rule).

**Deliverable:**
- `src/upstream_contrib/audits/AUDIT_REPORT_001.md` with:
  - Repository audited (commit hash, date)
  - Module analyzed
  - Coverage % before
  - Identified gap description
  - Failing test case code
  - Threshold justification
  - Decision: pursue contribution or skip

**Acceptance Criteria:**
- AC-019: At least one candidate audited with coverage report.
- AC-020: Gap meets threshold (≥10% delta OR critical-path edge case).
- AC-021: Failing test case written and validated locally in upstream fork.
- **Track B isolation check (R4):** Audit code in `src/upstream_contrib/` contains NO imports from `src/executor/`, `src/quality/`, or other `eth_node` modules (ensures portability for future repo split).

**Estimated Effort:** 10-15 hours (includes learning upstream codebase). **Time-box rule:** If alloy-provider audit exceeds 10 hours without identifying threshold-meeting gap, pivot to revm immediately (do not exceed 15 total hours on T-006 audit phase).

**Chronicle:** CHR-013-upstream-audit.md (Section: Coverage Analysis)

**Links to Spec:** PHASE2_FORMAL_SPEC.md FR-007, AC-019 through AC-021

---

### T-007: Implement Upstream Test Contribution in Fork (Track B)

**Description:** Fix identified gap in upstream fork; validate test passes in upstream CI environment.

**Scope:**
- Fork target repository on GitHub.
- Clone fork locally.
- Add test case from T-006 to appropriate test file.
- If gap requires fix (not just test), implement minimal fix.
- Run upstream test suite: `cargo test` (ensure all tests pass).
- Run upstream CI locally if possible (GitHub Actions workflow).

**Dependencies:** T-006 complete (gap identified; threshold met).

**Deliverable:**
- Feature branch in upstream fork with test contribution.
- Local validation evidence (test output, CI logs).
- Commit message: `test(module): add missing edge-case test for [behavior]`

**Acceptance Criteria:**
- All upstream tests pass locally (original + new).
- New test demonstrates gap (fails before fix, passes after).
- Commit follows upstream contribution guidelines (if available).

**Estimated Effort:** 4-6 hours

**Chronicle:** CHR-013-upstream-audit.md (Section: Test Implementation)

**Links to Spec:** PHASE2_FORMAL_SPEC.md FR-007

---

### T-008: Submit Upstream PR with Traceability (Track B)

**Description:** Open pull request on upstream repository; link to eth_node audit trail.

**Scope:**
- Create PR from fork branch to upstream main/master.
- PR title: `test(module): add missing edge-case test for [behavior]`
- PR description:
  - What gap was identified (coverage delta or edge case).
  - Why it matters (critical path, security-relevant, or correctness).
  - Link to eth_node audit report: `https://github.com/leftygbalogh/eth_node/blob/master/src/upstream_contrib/audits/AUDIT_REPORT_001.md`
  - Reproducible test case and evidence.
- Add feedback entry to eth_node `examples/feedback.json`: Track B contribution experience.

**Permission Gate:** Explicit owner approval required before PR submission. **Escalation rule:** If owner unavailable >48 hours, team lead escalates to next session for expedited decision OR defers Track B to Phase 3 (all Track A work proceeds to Stage 6 normally).

**Dependencies:** T-007 complete (test validated in fork).

**Deliverable:**
- Upstream PR URL.
- Feedback entry in `examples/feedback.json` (FB-010 or next ID).
- Chronicle entry linking PR to audit report.

**Acceptance Criteria:**
- PR submitted with traceability link.
- Feedback entry records: submission date, response time, merge outcome (if available by Stage 6).

**Estimated Effort:** 2-3 hours (PR creation, description, feedback log)

**Chronicle:** CHR-013-upstream-audit.md (Section: PR Submission)

**Links to Spec:** PHASE2_FORMAL_SPEC.md FR-007

---

### T-009: Prepare Reth Environment Readiness Checklist (Track A)

**Description:** Document prerequisites for future Reth Sepolia sync experiments; no actual integration in Phase 2.

**Scope:**
- Create `docs/reth_readiness_checklist.md` with:
  1. Disk space validation command (PowerShell).
  2. Environment configuration template (RETH_DATA_DIR, RETH_CHAIN, RETH_RPC_URL).
  3. Reth installation guide (build from source OR binary download).
  4. Dry-run script: `scripts/reth_dryrun.ps1` (validates prereqs without downloading chain data).
  5. Rollback procedure (stop process, delete data dir, restart).

**Test Requirements:**
- Run dry-run script on Windows with 190 GB SSD; verify it passes.
- Validate disk-check command correctly identifies free space.

**Dependencies:** T-004 + T-005 complete (A-2 mandatory before A-3). **Early-start option:** T-009 can begin in parallel with T-005 if developer has low-complexity documentation cycles available (no dependency on A-2 code).

**Deliverable:****
- `docs/reth_readiness_checklist.md`
- `scripts/reth_dryrun.ps1`

**Acceptance Criteria:**
- AC-016: Checklist is runnable on Windows with 190 GB SSD.
- AC-017: Dry-run script validates environment without downloading chain data.
- AC-018: Documentation includes rollback steps if sync fails or disk fills.

**Estimated Effort:** 3-4 hours

**Chronicle:** CHR-012-reth-prep.md

**Links to Spec:** PHASE2_FORMAL_SPEC.md FR-006, AC-016 through AC-018

---

## Task Dependency Graph

```
T-001 (A-1 boilerplate)
  ↓
T-002 (A-1 simulate_tx)
  ↓
T-003 (A-1 contract call + comparison)
  ↓
[A-1 complete; A-2 begins]
  ↓
T-004 (G-001 decode) ← parallel → T-005 (G-002 fuzzing)
  ↓
[A-2 complete; Track B + A-3 can begin]
  ↓
T-006 (Track B audit) ← parallel → T-009 (A-3 prep)  
*Note: After A-2 completes, T-006 (Track B) and T-009 (A-3 prep) MAY run in parallel per max-2-parallel rule.*
  ↓
T-007 (Track B test in fork)
  ↓
T-008 (Track B PR submission) [permission gate]
```

**Critical Path:** T-001 → T-002 → T-003 → T-004/T-005 → T-006 → T-007 → T-008  
**Duration (estimated):** 40-50 hours solo developer time

---

## Role Assignments

**Solo Developer Mode (Lefty):**
- **Track A implementation:** All T-001 through T-005, T-009.
- **Track B implementation:** T-006, T-007, T-008.
- **Stage gate approvals:** Product owner (Lefty) approves all gates.
- **Permission gates:** Product owner (Lefty) approves Track B PR submission at T-008.

**Agent Role:**
- Execute tasks per approved spec.
- Maintain chronicle entries per task.
- Update prompts.md and memory.md throughout.
- Flag blockers immediately; do not proceed past gates without explicit approval.

---

## Test Plan by Task

| Task | New Tests | Test Type | Coverage Target |
|------|-----------|-----------|-----------------|
| T-001 | 0 | Compilation check | N/A (boilerplate) |
| T-002 | 5+ | Unit + integration | AC-001, AC-002 |
| T-003 | 6+ | Unit + integration | AC-003 through AC-007 |
| T-004 | 15+ | Integration (live decode) | AC-008 through AC-011 |
| T-005 | 3+ | Property-based fuzzing | AC-012 through AC-015 |
| T-006 | 1+ | Upstream fork validation | AC-019 through AC-021 |
| T-007 | 0 | Upstream CI validation | (covered in T-006 AC) |
| T-008 | 0 | PR submission | (governance/feedback) |
| T-009 | 0 | Documentation + script | AC-016 through AC-018 |
| **Total** | **30+** | Phase 2 additions | Phase 1 (137) + Phase 2 (30+) = **170+ total** |

---

## Commit Strategy

**Commit Labeling:**
- **Track A (product):** `feat(executor):`, `feat(quality):`, `docs(reth):`
- **Track B (upstream):** `chore(track-b):`, `docs(upstream):`

**Commit Frequency:**
- After each task completion (T-001 through T-009).
- After each chronicle entry creation/update.
- After Stage gate approvals.

**Example Commit Messages:**
- T-001: `feat(executor): add revm integration boilerplate`
- T-002: `feat(executor): implement simulate_tx with error handling`
- T-003: `feat(executor): add contract call simulation and Anvil comparison`
- T-004: `feat(quality): close G-001 live decode completeness (ERC-721/1155)`
- T-005: `feat(quality): integrate proptest fuzzing framework (G-002)`
- T-006: `chore(track-b): audit alloy-provider coverage and document gap`
- T-007: `chore(track-b): implement upstream test contribution in fork`
- T-008: `docs(upstream): submit PR and add feedback entry`
- T-009: `docs(reth): add environment readiness checklist and dry-run script`

---

## Stage 3 Gate Approval

**Approver:** Product Owner (Lefty) + Team Lead (if delegation active)  
**Approval Date:** _Pending_  
**Approval Notes:** _To be filled after owner review_

**Gate Question:** Do you approve this Phase 2 task list with role assignments? If yes, we proceed to Stage 4 (implementation). If no, specify which task needs revision.

---

## Revision History

- 2026-03-27: Created from Stage 3 task planning; derived from PHASE2_FORMAL_SPEC.md.
