# Phase 2 Project Brief

**Project:** eth_node Phase 2 Continuation  
**Date:** 2026-03-27  
**Mode:** Greenfield (continuation)  
**Phase:** 2 of multi-phase roadmap  
**Owner:** Lefty  
**Stage 1 Status:** Discovery complete; pending Stage 1 gate approval

---

## 1. Project Context

Phase 1 delivered a functional Ethereum execution-layer toolkit in Rust:
- **Modules completed:** primitives, RPC client, transaction builder, signer, event listener, contract caller.
- **Test coverage:** 137 tests passing (Phase 1 baseline).
- **Infrastructure:** Anvil local devnet (instant, 100% Rust).
- **Release:** Published to origin/master at Stage 6 closure (2026-03-27).

Phase 2 continues the original 12-component roadmap with:
- **Component #7:** Local EVM executor (revm integration).
- **Component #12:** Reth preparation (environment readiness for future Sepolia sync).
- **Quality debt closure:** G-001 (live contract decode completeness), G-002 (fuzzing baseline).
- **Track B introduction:** Upstream test-coverage contributions to alloy-provider and revm.

---

## 2. Structural Decision

**Conceptual Model:** Track-Based Split (Option 3)  
- **Track A:** Product continuation (eth_node feature growth).
- **Track B:** Upstream side projects (test-coverage contributions to dependencies).

**Operational Model:** Single-Repo (Option 1)  
- Both tracks implemented within the existing eth_node repository.
- Directory structure:
  ```
  src/
    executor/          [Track A] revm integration
    quality/           [Track A] deferred-gap closure
    upstream_contrib/  [Track B] contribution staging
  ```

**Future trajectory:** Can split into hub + child repos in Phase 3 if Track A/B diverge in scale or team assignment.

**Track B Extraction Triggers (R7):**  
Measurable criteria for when to split Track B into separate repository:
- **Upstream target count:** >5 repositories audited.
- **Contributor count:** >1 active contributor to Track B.
- **Commit frequency:** >50% of total commits are Track B work.
- **Integration friction:** >3 merge conflicts per month between Track A and Track B.

If any trigger is met, evaluate hub+child-repo migration in next phase planning.

**Track B Isolation Policy (R4):**  
All code in `src/upstream_contrib/` MUST NOT import from `src/executor/`, `src/quality/`, or other `eth_node` modules. This ensures Track B contributions remain portable for future repo split.

---

## 3. Design Questions (Resolved)

| Question | Decision | Date | Reasoning |
|----------|----------|------|-----------|
| Should A-2 quality closure be mandatory before A-3 (Reth prep)? | **Mandatory** | 2026-03-27 | Pay down debt before expanding scope; ensures Phase 1 foundation is solid. |
| Which network target first: Sepolia or Holesky? | **Sepolia** | 2026-03-27 | Execution-layer focus matches eth_node modules; lower hardware requirements. |
| Maximum parallelism between Track A and Track B? | **Max 2 parallel tasks (1 per track)** | 2026-03-27 | Solo developer context; minimizes context-switching overhead. |
| Minimum acceptance threshold for upstream side-project? | **Coverage ≥ 10% OR critical-path edge case** | 2026-03-27 | Quality bar without forcing artificial contributions; measurable impact. |

---

## 4. Hardware & Runtime Target

- **RAM:** 16 GB (sufficient for concurrent devnet + revm testing)
- **Disk:** 190 GB SSD (workable for Sepolia sync; monitor usage during A-3)
- **OS:** Windows (PowerShell environment)
- **Toolchain:** Rust stable (cargo, rustc)
- **Infrastructure:** Anvil (Phase 1 baseline) + Reth (Phase 2 A-3 target)

---

## 5. First Backlog Wave

Phase 2 first increment includes five items:

| ID | Description | Track | Priority | Sequencing |
|----|----|-------|----------|------------|
| P2-001 | Implement revm executor foundation (A-1): `simulate_tx()`, `simulate_contract_call()` | A | High | First |
| P2-002 | Close G-001 live decode completeness (ERC-721/ERC-1155 edge cases) | A | High | After P2-001 (parallel with P2-003) |
| P2-003 | Close G-002 fuzzing baseline (property-based testing framework) | A | High | After P2-001 (parallel with P2-002) |
| P2-005 | First upstream audit candidate selection (Track B target) | B | Medium | After P2-002/P2-003 |
| P2-004 | Reth environment checklist (A-3 prep: disk setup, env vars, first-sync script) | A | Low | After A-2 complete (mandatory sequencing) |

**Sequencing rationale:**
1. P2-001 first: revm executor is the new capability; build foundation before closing quality gaps.
2. P2-002 + P2-003 parallel: both are Phase 1 quality debt; can run simultaneously (max 2 parallel rule allows).
3. P2-005 after A-2: Track B audit begins once Phase 1 quality debt is closed and A-1 is stable.
4. P2-004 last: A-3 prep deferred until A-2 mandatory completion.

### 5.1 Scope Clarification: Attack Scenarios and Adversarial Testing

**Decision (2026-03-27):** Attack scenarios, intentional misuse testing, and fuzzing are scoped to **P2-003 (G-002 fuzzing baseline)**, not embedded in P2-001 (A-1 executor foundation).

**Rationale:**
- **Separation of concerns:** P2-001 proves basic executor correctness with known-good inputs and expected error cases (invalid signature, wrong nonce, zero gas → proper `ExecutorError`). P2-003 proves resilience under adversarial/malformed inputs (random byte arrays, extreme gas limits, malicious calldata, reentrancy patterns → no panic).
- **Avoid scope dilution:** Attack-scenario testing is open-ended. Embedding it in A-1 risks delaying the core executor foundation indefinitely.
- **Sequential benefit:** Complete A-1 with clean API surface first, then run P2-003 fuzzing to discover which attack vectors need hardening. Findings from fuzzing feed back into A-1 error-handling refinements if needed.

**Practical split:**
- **P2-001 error cases (A-1):** Invalid signature, wrong nonce, insufficient gas → proper `ExecutorError::InvalidInput` or `ExecutorError::RevmFailure`.
- **P2-003 fuzzing (G-002):** Property-based testing with 10k+ iterations; random inputs; edge states; malicious patterns → no panic, graceful degradation.

This design preserves focus on core capability delivery while ensuring adversarial resilience is systematically addressed in the quality-closure phase.

---

## 6. Track B Intent Declaration

**Target repositories:** alloy-provider, revm  
**Contribution type:** Unit tests or property tests demonstrating missing edge cases  
**Permission acknowledgment:** Track B submissions require explicit owner approval before upstream PR creation  
**Acceptance threshold:** Test coverage delta ≥ 10% OR critical-path edge case (panic, wrong-value, security-relevant)  
**Traceability requirement:** Each Track B PR must link back to eth_node chronicle entry for audit trail

---

## 7. Success Criteria for Phase 2

**Functional:**
- ✅ revm executor integrated; `simulate_tx()` and `simulate_contract_call()` work with known Anvil scenarios.
- ✅ G-001 closed: ERC-721/ERC-1155 live-decode edge cases covered.
- ✅ G-002 closed: Fuzzing framework integrated; 10k+ iterations per property without panic.
- ✅ A-3 prep complete: Reth environment checklist documented; first-sync instructions available.

**Track B:**
- ✅ At least one upstream audit candidate identified.
- ✅ Failing test case for upstream gap produced (if candidate meets threshold).
- ✅ Upstream PR submitted (if gap meets acceptance threshold and owner approves).

**Quality:**
- ✅ Test suite grows to 170+ tests passing (Phase 1 baseline: 137; Phase 2 adds ~33).
- ✅ All Phase 1 tests still pass (regression check).
- ✅ Traceability: each task linked to spec, each test linked to AC.

**Governance:**
- ✅ All Stage 1-6 gates executed with explicit approvals.
- ✅ Joint post-mortem produced (agent + owner).
- ✅ Feedback entries added to examples/feedback.json (Track B experience, single-repo dual-track lessons).

---

## 8. Entry Criteria (Phase 2 Stage 1 Approval)

Before proceeding to Stage 2 (formal specification):

1. ✅ All design questions resolved (Section 3).
2. ✅ Hardware/runtime target confirmed (Section 4).
3. ✅ First backlog wave approved (Section 5).
4. ✅ Track B intent declared explicitly (Section 6).
5. ⏳ **Project brief approved by product owner** (this gate).

---

## 9. Key Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| 190 GB disk insufficient for Reth sync | A-3 blocks | Monitor disk usage; defer A-3 if <50 GB free; consider pruning or external drive. |
| Upstream PR rejected or ignored | Track B goal unmet | Select high-visibility repo with active maintainers; follow contribution guidelines; link to reproducible test. |
| Context-switching between Track A/B reduces velocity | Slower delivery | Enforce 1-task-per-track rule; complete A-2 fully before opening Track B work. |
| Fuzzing framework integration breaks existing tests | Regression | Run full test suite after G-002; isolate fuzz tests in separate module. |

---

## 10. Out of Scope for Phase 2

The following remain deferred to Phase 3 or later:

- Component #8: Mempool monitor
- Component #9: Block + receipt indexer
- Component #11: EVM state trie reader
- Full Reth integration (actual Sepolia sync + execution)
- Multi-contributor team expansion
- Separate repository split for Track B

---

## 11. References

1. **PHASE2_AND_UPSTREAM_PLAN.md** — Consolidated planning baseline derived from Phase 1 artifacts.
2. **PHASE2_TRANSITION_TASKLIST.md** — 30-step execution plan (Stage 1-6 gates, Track A/B separation).
3. **memory.md** — Historical Stage 1 discovery snapshot with 12-component roadmap.
4. **FORMAL_SPEC.md** — Phase 1 out-of-scope items deferred to Phase 2.
5. **examples/feedback.json** — FB-002 governance formalization of Track B contribution intent.

---

## 12. Stage 1 Gate Approval

**Approver:** Product Owner (Lefty)  
**Approval Date:** 2026-03-27  
**Approval Notes:** Stage 1 gate approved. All discovery questions resolved; design decisions recorded; Track A/B intent declared. Proceed to Stage 2 (formal specification).

---

## Revision History

- 2026-03-27: Created from Phase 2 transition planning and discovery Q&A.
