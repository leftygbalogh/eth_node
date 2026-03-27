# Phase 2 Transition Task List

**Date:** 2026-03-27  
**Status:** Planning baseline for Phase 2 project cycle  
**Structural Decision:** Track-Based Conceptual Model (Option 3) + Single-Repo Operational (Option 1)  
**Source:** Derived from PHASE2_AND_UPSTREAM_PLAN.md and approved at Phase 1 closure

---

## Stage 1: Discover (Phase 2 Project Brief)

1. **Resolve design questions** from PHASE2_AND_UPSTREAM_PLAN.md Section 8.2:
   - A-2 mandatory before A-3, or parallel? → **DECIDED: Mandatory A-2 before A-3**
   - Network target: Sepolia or Holesky first? → **DECIDED: Sepolia**
   - Track A/B max parallelism (e.g., 1 contributor per track, or full team split)? → **DECIDED: Max 2 parallel (1 per track)**
   - Minimum upstream acceptance threshold (test coverage delta %, bug severity, performance win %)? → **DECIDED: Coverage ≥ 10% OR critical-path edge case**

2. **Define hardware/runtime target** (Section 9 entry criterion #1):
   - Confirm disk space for Reth sync experiments. → **DECIDED: 190 GB SSD (tight but workable; monitor usage)**
   - Confirm RAM for concurrent devnet + revm execution tests. → **DECIDED: 16 GB RAM (sufficient)**
   - Document in project brief.

3. **Approve first backlog wave** (Section 8.1):
   - P2-001: revm executor foundation (A-1). → **APPROVED**
   - P2-002: live decode completeness (G-001). → **APPROVED**
   - P2-003: fuzzing baseline (G-002). → **APPROVED**
   - P2-004: Reth environment checklist (A-3 prep). → **APPROVED**
   - P2-005: first upstream audit candidate (Track B candidate selection). → **APPROVED**
   - **Sequencing:** P2-001 → P2-002 + P2-003 (parallel) → P2-005 (Track B) → P2-004 (after A-2 complete)

4. **Declare Track B intent explicitly** (Section 9 entry criterion #3):
   - Target repositories (e.g., alloy-core, alloy-provider, revm test suites). → **DECIDED: alloy-provider, revm**
   - Contribution type (unit tests, property tests, integration tests, or docs). → **DECIDED: Unit tests or property tests**
   - Record permission acknowledgment (owner agrees to Track B permission-gating). → **ACKNOWLEDGED: Track B submissions require explicit owner approval before PR**

5. **Approve project brief** — Stage 1 gate sign-off (product owner). → **APPROVED 2026-03-27**

---

## Stage 2: Specify (Phase 2 Formal Spec)

6. **Decompose A-1** (Component #7: revm executor):
   - API surface: `simulate_tx()`, `simulate_contract_call()`, `compare_to_anvil()`.
   - Error cases and fallback behavior.
   - Acceptance criteria: all known Anvil scenarios match; no panic on malformed input.

7. **Decompose A-2** (Quality closure):
   - G-001 AC: all ERC-721/ERC-1155 live-decode scenarios covered; document edge cases.
   - G-002 AC: fuzzing framework integrated; 10k+ iterations per property without panic.

8. **Decompose A-3 prep** (Reth readiness):
   - Checklist: disk setup, environment variables, minimal-sync script, first-block decode.
   - No implementation; just a verification plan for later Phase 2 steps.

9. **Select first Track B target** from approved candidates:
   - Audit candidate (e.g., alloy-provider branch coverage, revm edge case test gaps).
   - Define failing test case + expected fix.
   - Plan upstream PR scope.

10. **Approve formal spec** — Stage 2 gate sign-off (product owner). → **APPROVED 2026-03-27**

---

## Stage 3: Plan (Phase 2 Task List)

11. **Create numbered task list** clearly labeled with track and module:
    - T-001 (Track A): Setup revm integration, boilerplate, basic `simulate_tx()`.
    - T-002 (Track A): Deploy `simulate_contract_call()` with ABI+calldata support.
    - T-003 (Track A): Build comparison matrix (eth_node sim vs Anvil result).
    - T-004 (Track A): Close G-001 (live decode completeness).
    - T-005 (Track A): Close G-002 (fuzzing baseline).
    - T-006 (Track B): Audit upstream target, produce failing test.
    - T-007 (Track B): Write + validate fix in upstream fork.
    - T-008 (Track B): Submit upstream PR with traceability link to T-006.
    - T-009 (Track A): Prepare A-3 Reth environment readiness checklist (dry-run).

12. **Set up repo structure** (single-repo layout):
    ```
    src/
      executor/          [NEW] revm integration module
        lib.rs
        simulate.rs
      quality/           [NEW] deferred-gap closure
        decode.rs        [G-001]
        fuzz.rs          [G-002]
      upstream_contrib/  [NEW] Track B contribution staging
        audits/
        fork_tests/
    ```
    → **DECIDED (2026-03-27): Defer directory creation until Stage 4. Create directories on-demand per task (T-001 creates executor/, T-004 creates quality/, T-006 creates upstream_contrib/).**

13. **Approve task list with role assignment** — Stage 3 gate sign-off (product owner + team lead).

---

## Stage 4: Build (Phase 2 Implementation)

14. **Track A: Implement A-1** (revm executor, T-001 through T-003):
    - Add revm dependency to Cargo.toml.
    - Implement `simulate_tx()` with fallback error handling (expected error cases: invalid signature, wrong nonce, insufficient gas).
    - Add Anvil reference tests (compare results).
    - **Scope boundary:** Attack scenarios and adversarial testing deferred to P2-003 (G-002 fuzzing). A-1 focuses on known-good inputs and expected error cases only.
    - Commit: "feat(executor): add revm simulation foundation".

15. **Track A: Close G-001 + G-002** (T-004, T-005):
    - Expand contract decoder with missing ERC token standards.
    - Integrate property-based fuzzing framework (proptest or quickcheck).
    - **Scope expansion:** G-002 includes attack scenarios, intentional misuse, and adversarial patterns (random byte arrays, extreme gas, malicious calldata, reentrancy) → no panic under 10k+ iterations.
    - Commit: "feat(quality): close decode completeness and fuzz baseline".

16. **Track B (in parallel): Audit and prepare upstream PR** (T-006, T-007):
    - Fork target repo locally.
    - Write failing test demonstrating gap.
    - Implement fix in fork; validate CI passes.
    - Document upstream PR scope.
    - Commit to feature branch locally: "chore(track-b): upstream audit + test contribution ready".

17. **Track B: Submit upstream PR** (T-008):
    - Create PR on target repo with test case and fix.
    - Link to eth_node chronicle entry for traceability.
    - Add feedback entry to examples/feedback.json (contributor experience, response time expectations).

18. **Track A: Reth environment readiness** (T-009):
    - Create dry-run checklist; document first-sync instructions.
    - No actual Reth integration yet; just preparation for Phase 2.2 or Phase 3.
    - Commit: "docs(reth): environment readiness checklist and first-sync guide".

---

## Stage 5: Verify (Phase 2 Testing & Coverage)

19. **Run full cargo test**:
    - Verify all Phase 1 tests still pass (94 existing).
    - Verify all new A-1, A-2 tests pass (~30 new estimated).
    - Total target: 130+ tests passing.

20. **Coverage validation**:
    - Confirm G-001 decode tests cover new ERC branches.
    - Confirm G-002 fuzzing runs without panic.
    - Measure branch coverage delta vs Phase 1 baseline.

21. **Traceability audit**:
    - Link each task to formal spec section.
    - Link each test to acceptance criterion.
    - Link Track B work to upstream PR (comment/reference).

22. **Approve verification checklist** — Stage 5 gate sign-off (QA lead / owner).

---

## Stage 6: Release (Phase 2 Publication)

23. **Create joint post-mortem** (agent + owner):
    - Agent: summarize improvements, learnings from hybrid Track A/B model, any process friction.
    - Owner: explicit pass/notes on governance execution, team effectiveness, Track B contribution results.

24. **Add feedback entries** to examples/feedback.json:
    - Record lessons from Track B upstream submission (PR response time, merge outcome, collaboration experience).
    - Record lessons from single-repo dual-track structure (team coordination, CI/CD overhead, directory clarity).

25. **Update PHASE2_AND_UPSTREAM_PLAN.md** for Phase 3 reuse:
    - Mark A-1, A-2, A-3-prep tasks as complete.
    - Note upstream PR outcome (merged / waiting / rejected / refined).
    - Document any pivots to original plan.

26. **Final cargo build + test**:
    - Ensure clean build from HEAD.
    - Confirm 130+ test suite passes.

27. **Commit release artifacts**:
    - Commit: `docs(stage6): Phase 2 release — A-1, G-001/G-002, Track B audit + PR`.

28. **Approve Stage 6 release** — Stage 6 gate sign-off (owner explicit pass).

29. **Push to origin/master**:
    - Confirm remote target = origin/master.
    - Push final Phase 2 build + post-mortem artifacts.

---

## Post-Release Checkpoint

30. **Decide Phase 3 structure** (next session):
    - Evaluate: Did single-repo dual-track work well, or split repos needed?
    - Input: feedback from examples/feedback.json, team observation during implementation.
    - Options: Continue single-repo, or migrate Track B to separate repo for organizational clarity.

---

## Gate Summary

- **Stage 1 Gate:** Brief approved with design questions resolved + Track B intent declared.
- **Stage 2 Gate:** Formal spec for A-1, A-2, A-3-prep, and Track B target approved.
- **Stage 3 Gate:** Task list approved with Track A/B labels and role assignment.
- **Stage 5 Gate:** 130+ tests passing, coverage validated, traceability complete.
- **Stage 6 Gate:** Owner explicit pass + joint post-mortem + release artifacts pushed.

---

## Track A/B Operational Separation (Single-Repo Mode)

**Directory Structure:**
```
src/
  executor/          [Track A] revm integration module
  quality/           [Track A] deferred-gap closure (G-001, G-002)
  upstream_contrib/  [Track B] contribution staging (audits, fork tests)
```

**Commit Labeling:**
- Track A commits: `feat(executor):`, `feat(quality):`, etc.
- Track B commits: `chore(track-b):` or `docs(upstream):` as appropriate.

**Permission-Gating:**
- Track A: standard workflow (no special gates beyond stage approvals).
- Track B: explicit owner permission before upstream PR submission (Step 17).

---

## Revision History

- 2026-03-27: Created from Phase 1 closure and post-release planning baseline (PHASE2_AND_UPSTREAM_PLAN.md).
