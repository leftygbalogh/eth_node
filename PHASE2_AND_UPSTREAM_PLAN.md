# Phase 2 and Upstream Side-Project Plan

Date: 2026-03-27
Status: Draft planning baseline for subsequent phases
Scope source: This repository completed Phase 1. This document consolidates externally referenced planning intent into one working roadmap for future brief/spec/task-list cycles.

## 1. Why This Exists

This project was intentionally the first step of a larger Rust-Ethereum roadmap. The core plan was discussed across discovery prompts, memory snapshots, formal specification boundaries, and feedback artifacts. This document pulls those references into one coherent planning baseline.

## 2. Consolidated External References

The table below captures each external reference, what it says, and how it affects planning.

1. memory.md Stage 1 snapshot
   - Evidence: 12-component progression and explicit phase split.
   - Planning implication: Phase 2 is not a new direction; it is the next segment of the original roadmap.

2. prompts.md Stage 1 discussion
   - Evidence: explicit user request to contribute back to external sources by improving their test coverage where needed.
   - Planning implication: downstream work includes an upstream side-project track, not only local feature work.

3. FORMAL_SPEC.md Phase 1 out-of-scope list
   - Evidence: mempool monitor, state trie reading, and Reth integration deferred to Phase 2.
   - Planning implication: these are legitimate Phase 2 candidates with spec lineage.

4. memory.md Stage 4 completion snapshot
   - Evidence: G-001 and G-002 explicitly deferred to Phase 2.
   - Planning implication: quality debt must be planned as first-class Phase 2 backlog.

5. examples/feedback.json FB-002
   - Evidence: recommends explicit "upstream contribution intent" declaration at Stage 1.
   - Planning implication: future project cycles should declare Track B targets and contribution type at discovery time.

## 3. Confirmed Big-Plan Baseline

## 3.1 Completed Phase 1 (done)

- Local Anvil-first execution layer toolkit delivered.
- Components #1 through #6 implemented and verified.
- Component #10 (local devnet via Anvil) used as the practical foundation.

## 3.2 Original 12-Component Plan (explicit)

1. Ethereum primitives library
2. JSON-RPC client
3. Transaction builder + signer
4. Transaction broadcaster
5. Event/log listener
6. ABI-driven contract caller
7. Local EVM executor (revm)
8. Mempool monitor
9. Block + receipt indexer
10. Local devnet node (Anvil)
11. EVM state trie reader
12. Partial execution client (full Reth node)

## 3.3 Planned continuation (phase framing)

- Phase 2 scope (as recorded): start with component #7, then component #12.
- Deferred candidates from the 12-component map remain available as later slices (#8, #9, #11).
- Runtime trajectory: local-first learning path expanding toward Reth/Sepolia or Holesky once hardware allows.

## 4. Additional Carry-Over From Completed Phase 1

Quality gaps explicitly deferred to Phase 2:

1. G-001: AC-005 no live contract decode completeness.
2. G-002: NFR-001 fuzzing not implemented.

Planning rule: Phase 2 backlog should include both capability growth and deferred quality closure.

## 5. Track Model for Work After Phase 1

Use two coordinated tracks:

## 5.1 Track A - Project Continuation (eth_node)

Goal: Extend this repository with next-phase capabilities.

Suggested sequence:

1. A-1: Component #7 (Local EVM executor with revm)
   - Build deterministic simulation flows around existing tx/contract modules.
   - Add execution-equivalence tests against known Anvil scenarios.
2. A-2: Close deferred quality gaps from Phase 1
   - G-001: live contract decode completeness.
   - G-002: NFR-001 fuzzing coverage.
3. A-3: Component #12 path prep (Reth integration prerequisites)
   - environment and disk readiness checks
   - minimal sync + execute proof-of-life
4. A-4: Optional component #8 (mempool monitor)
5. A-5: Optional component #11 (state trie reader)

## 5.2 Track B - Upstream Side Projects (external dependency quality)

Goal: Contribute back to external Rust/Ethereum dependencies by improving test coverage where gaps exist.

Candidate upstream targets (based on active stack in this project):

1. alloy-related crates used by current modules
2. helper ecosystems used in testing/runtime infrastructure
3. any dependency that blocks confidence in Track A behavior due to weak tests

Contribution loop per target:

1. B-1: Baseline audit
   - Identify module/behavior with weak branch or edge-path coverage.
   - Record reproducible failing/untested behavior.
2. B-2: Minimal test-first contribution
   - Add failing test demonstrating the gap.
   - Validate local and CI behavior in upstream repo.
3. B-3: Submit contribution
   - Open PR with precise behavioral claim and evidence.
   - Link to local chronicle note for traceability.
4. B-4: Feed learnings back
   - Add feedback entry to examples/feedback.json in this governance template.

## 6. Dependency Between Tracks

- Track A remains the primary learning/product track.
- Track B runs in parallel when a dependency gap is discovered that materially affects Track A confidence.
- Track B actions remain permission-gated under governance when third-party repo operations are involved.

## 7. Phase 2 Planning Baseline for Next Spec Cycle

This section is intentionally "incomplete but detailed": enough for spec kickoffs, without pretending decisions are final.

## 7.1 Proposed first backlog wave

1. P2-001: Implement revm executor foundation (A-1)
2. P2-002: Close G-001 live decode completeness
3. P2-003: Close G-002 fuzz coverage baseline
4. P2-004: Define Reth environment readiness checklist (A-3 prep)
5. P2-005: Run first upstream audit candidate and choose first Track B target

## 7.2 Required design questions for next brief/spec

1. Should A-2 quality closure be mandatory before A-3 (Reth prep)?
2. Which network target is preferred first for Reth experiments: Sepolia or Holesky?
3. What is the maximum parallelism allowed between Track A and Track B?
4. What is the minimum acceptance threshold for an upstream side-project candidate (coverage delta, bug class, or risk class)?

## 8. Entry Criteria for Phase 2 Kickoff

1. Confirm hardware/runtime target for Reth experiments (disk and memory).
2. Approve Track A first increment (A-1) and whether A-2 is folded in immediately.
3. Declare Track B intent explicitly at Stage 1 for the next project cycle:
   - target repositories
   - contribution type (tests/docs/fixes)
   - permission acknowledgment

## 9. Definition of Done for This Planning Artifact

This document is complete when:

1. It references the recovered original intent sources.
2. It separates continuation work (Track A) from upstream side projects (Track B).
3. It defines concrete first increments, carry-over quality debt, and phase-entry criteria.
4. It is used as input to the next project brief/spec/task-list cycle.

## 10. Explicitly Not Finalized Yet

1. Final task sequencing and effort estimates.
2. Specific upstream repositories selected for first Track B contribution.
3. Detailed acceptance tests for components #8, #9, #11.
4. Phase 2 stage approvals and owners for the next project cycle.
