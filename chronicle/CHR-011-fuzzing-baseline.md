# CHR-011 Fuzzing Baseline

## 1. Chronicle Metadata

- Chronicle ID: CHR-011
- Source task ID: T-005
- Source spec sections: PHASE2_FORMAL_SPEC.md FR-005; AC-012 through AC-015
- Source requirements: PHASE2_TASK_LIST.md T-005
- Module / component name: Fuzzing baseline for executor, decoder, and ABI helpers
- Implementation language: Rust
- Author: GitHub Copilot
- Date: 2026-03-28
- Status: Final

## 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - Adds an opt-in property-based fuzzing baseline that exercises executor simulation, NFT log decoding, and ABI round-trip behavior.
- What must remain functionally equivalent across languages:
  - No-panic guarantees at module boundaries and value-preserving ABI encode/decode behavior.
- What is intentionally language-specific in this implementation:
  - Rust `proptest` configuration, feature gating, and `catch_unwind`-based panic detection.

## 3. Implementation Decisions

- Data structures chosen and why:
  - Shared fuzz input builders in `src/eth_node/src/quality/fuzz.rs` normalize construction of `TransactionRequest`, `SimulationContext`, and `Log` values so the properties stay focused on behavior rather than setup.
- Algorithms chosen and why:
  - Three broad properties were chosen instead of many narrow ones to cover the required surfaces while keeping total runtime under 60 seconds.
- Control-flow structure chosen and why:
  - Each property uses `catch_unwind` where the acceptance criterion is specifically "never panics"; this makes a panic a test failure rather than process abort.
- Boundary and interface decisions:
  - Fuzzing is gated behind `feature = "fuzz"` locally and included automatically in CI through `--all-features`.
- Error-handling strategy:
  - The executor and decoder properties accept returned errors; they only fail on panic.
- Performance or memory trade-offs accepted:
  - Random byte payloads up to 10 KB are included to satisfy adversarial input coverage while still completing in roughly 20 seconds for the fuzz suite.
- File map (concrete files changed/created):
  - `src/eth_node/Cargo.toml`
  - `.github/workflows/ci.yml`
  - `src/eth_node/src/quality/mod.rs`
  - `src/eth_node/src/quality/fuzz.rs`
  - `src/eth_node/tests/fuzz_properties.rs`
- Public symbols introduced/changed:
  - `eth_node::quality::fuzz`
  - `make_tx_request(...)`
  - `make_context(...)`
  - `make_log(...)`
- Signature snapshot (functions/classes/types and key fields):
  - `pub fn make_tx_request(...) -> TransactionRequest`
  - `pub fn make_context(...) -> SimulationContext`
  - `pub fn make_log(...) -> Log`

## 4. Alternatives Considered

- Alternative 1:
  - Put all property setup directly in the test file.
- Why rejected:
  - It duplicated verbose construction logic and made the properties harder to read.
- Alternative 2:
  - Run fuzzing unconditionally in the standard test path.
- Why rejected:
  - The task explicitly called for local opt-in to avoid slowing ordinary feedback loops.

## 5. Derived Invariants and Constraints

- Invariant 1:
  - `simulate_tx()` must not panic for arbitrary `TransactionRequest` inputs and arbitrary block-context values.
- Invariant 2:
  - `decode_standard_nft_event()` must not panic for arbitrary topic/data combinations.
- Constraints inherited from the spec:
  - Minimum 3 properties, 10k+ cases each, adversarial byte coverage up to 10 KB, feature-gated locally, under 60 seconds total.
- Additional implementation constraints introduced:
  - `max_shrink_iters` is disabled in the proptest config to keep the runtime predictable for the baseline suite.
- Boundary behavior and terminal/end-state rules:
  - Returned errors are acceptable; panic is not.

## 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - The task required `src/quality/fuzz.rs` with property definitions while also calling for a separate integration test file.
- How the ambiguity was resolved in code:
  - `src/quality/fuzz.rs` contains shared fuzz builders, while `tests/fuzz_properties.rs` owns the executable properties.
- Any controlled divergence from the spec:
  - None affecting behavior or acceptance criteria.
- Follow-up needed in the spec or task list:
  - None.

## 7. Testing Notes

- Unit tests added:
  - None.
- Integration tests added:
  - `tests/fuzz_properties.rs`
- Property-based tests added:
  - `simulate_tx_never_panics_on_random_inputs`
  - `nft_decoder_never_panics_on_random_logs`
  - `abi_roundtrip_preserves_values`
- Edge cases covered:
  - Empty calldata, 10 KB calldata/data blobs, `u64::MAX` gas, zero gas, zero gas price, create-vs-call routing, arbitrary topics.
- Failure modes exercised:
  - Executor input validation failures, decoder unsupported-signature failures, ABI decode rejection paths.
- Runtime/orchestration branch evidence covered:
  - CI path now executes the fuzz feature through `cargo test --all --all-features`.
- Golden input/output examples captured:
  - Not applicable for randomized properties.

## 7.1 Verification Artifact (mandatory — entry is incomplete without this field)

Verification artifact:
- `cargo test --package eth_node --features fuzz --test fuzz_properties`
  - Result: 3 passed; 0 failed; finished in 18.82s
- `cargo test --package eth_node --all-features`
  - Result: full `eth_node` package green; `fuzz_properties` passed 3/3 and finished in 19.69s

## 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
  - Keep fuzzing local-opt-in via Cargo feature, push execution into CI with `--all-features`, and preserve the three acceptance surfaces: executor, decoder, ABI round-trip.
- Order of implementation steps that mattered:
  - Feature flag first, helper module second, property tests third, CI wiring last.
- Non-obvious pitfalls discovered during implementation:
  - The generated `AbiTuple` type does not implement `Debug` or `PartialEq`, so tuple round-trip checks must compare fields directly.
- What not to change without updating the behavioral spec:
  - The 10k-case threshold and local-vs-CI execution model.
- Side effects and external touchpoints (files/network/stdout/runtime context):
  - CI runtime increases slightly because fuzzing now runs under `--all-features`.

## 9. Known Limitations

- Limitation 1:
  - This baseline focuses on no-panic and round-trip guarantees rather than semantic equivalence for every random executor input.
- Reason accepted:
  - That matches the current T-005 acceptance criteria and keeps runtime bounded.
- Revisit trigger:
  - If Phase 3 requires semantic fuzz or stateful differential fuzzing.

## 10. Approval / Review

- Reviewed by:
  - Pending owner review as part of the A-2 workstream
- Review date:
  - Pending
- Notes:
  - T-005 is implementation-complete; A-2 remains blocked on T-004 live Anvil capture evidence.
- Pair-programming references (if applicable):
  - Session log path:
    - Not separately recorded
  - Driver role(s):
    - GitHub Copilot implementing T-005
  - Navigator/reviewer role(s):
    - User-approved XP/team workflow constraints
  - Key disagreement and resolution summary:
    - Local fuzzing remained feature-gated while CI runs all features automatically.

## 11. Reconstruction Bundle (required at Stage 4 close for major modules)

- Source tree manifest relevant to this module:
  - `src/eth_node/src/quality/mod.rs`
  - `src/eth_node/src/quality/fuzz.rs`
  - `src/eth_node/tests/fuzz_properties.rs`
  - `src/eth_node/Cargo.toml`
  - `.github/workflows/ci.yml`
- Import/dependency graph expectations:
  - Uses existing `proptest`, `alloy-primitives`, `alloy-rpc-types`, `eth_node::executor`, and `eth_node::primitives` APIs.
- Recommended reconstruction order:
  - Add Cargo feature, expose module, add helper builders, add property tests, update CI, run feature-gated tests, run all-features package tests.
- Post-rebuild validation commands:
  - `cargo test --package eth_node --features fuzz --test fuzz_properties`
  - `cargo test --package eth_node --all-features`
- Expected validation outputs/pass criteria:
  - 3 fuzz properties pass with 10k cases each and total runtime under 60 seconds.
