# CHR-010 Decode Completeness

## 1. Chronicle Metadata

- Chronicle ID: CHR-010
- Source task ID: T-004
- Source spec sections: PHASE2_FORMAL_SPEC.md FR-004; AC-008 through AC-011
- Source requirements: PHASE2_TASK_LIST.md T-004
- Module / component name: NFT event decode completeness (ERC-721 / ERC-1155)
- Implementation language: Rust (+ Solidity test fixtures)
- Author: GitHub Copilot
- Date: 2026-03-28
- Status: Final

## 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - Completes decode coverage for standard ERC-721 and ERC-1155 event families, including deploy-driven live Anvil capture validation.
- What must remain functionally equivalent across languages:
  - Event signature recognition, indexed-topic extraction, data-field decoding, and error semantics for malformed payloads.
- What is intentionally language-specific in this implementation:
  - Rust decoder API/types and Alloy log handling; Solidity emitter fixtures used only for live integration verification.

## 3. Implementation Decisions

- Data structures chosen and why:
  - Strongly typed decoded-event structs with a `DecodedEvent` enum preserve explicit per-event field contracts.
- Algorithms chosen and why:
  - Topic-based signature routing for fast dispatch, then event-specific field decoding.
- Control-flow structure chosen and why:
  - Shared entry point (`decode_standard_nft_event`) for standard signatures, plus explicit function for ERC-1155 `ApprovalForAll` due signature overlap with ERC-721.
- Boundary and interface decisions:
  - Decoder accepts `alloy_rpc_types::Log` so both synthetic tests and live RPC receipts can use the same path.
- Error-handling strategy:
  - Structured `DecodeError` variants (`UnsupportedEvent`, `MissingTopic`, `InvalidData`) with detailed context.
- Performance or memory trade-offs accepted:
  - Added robust fallback decoding paths for live payload compatibility in dynamic ERC-1155 events.
- File map (concrete files changed/created):
  - `src/eth_node/src/quality/decode.rs`
  - `src/eth_node/src/quality/mod.rs`
  - `src/eth_node_cli/src/main.rs`
  - `src/eth_node_cli/tests/decode_receipt_cli.rs`
  - `src/eth_node/tests/decode_live.rs`
  - `src/eth_node/tests/decode_anvil_live.rs`
  - `src/eth_node/tests/contracts/TestERC721.sol`
  - `src/eth_node/tests/contracts/TestERC1155.sol`
  - `CLI_REFERENCE.md`
- Public symbols introduced/changed:
  - `decode_standard_nft_event(...)`
  - `decode_nft_event_lossless(...)`
  - `decode_erc721_approval_for_all(...)`
  - `decode_erc1155_approval_for_all(...)`
  - `DecodedEvent` and event payload structs
  - `eth_node_cli decode-receipt`
- Signature snapshot (functions/classes/types and key fields):
  - `pub fn decode_standard_nft_event(log: &Log) -> Result<DecodedEvent, DecodeError>`
  - `pub fn decode_nft_event_lossless(log: &Log, approval_for_all_as: Option<ApprovalForAllStandard>) -> Result<LosslessDecodedEvent, DecodeError>`
  - `pub fn decode_erc1155_approval_for_all(log: &Log) -> Result<Erc1155ApprovalForAllEvent, DecodeError>`
  - `pub enum DecodedEvent { ... }`

## 4. Alternatives Considered

- Alternative 1:
  - Keep only synthetic canonical-log tests.
- Why rejected:
  - A-2 sequencing required deploy-driven Anvil evidence before approval.
- Alternative 2:
  - Decode all events through one generic ABI decoder path.
- Why rejected:
  - Topic-index handling and shared-signature ambiguity required explicit event-specific extraction paths.

## 5. Derived Invariants and Constraints

- Invariant 1:
  - Standard signature `topic0` determines event family dispatch.
- Invariant 2:
  - Indexed fields are extracted from topics with exact position mapping.
- Constraints inherited from the spec:
  - Cover ERC-721 Transfer/Approval/ApprovalForAll and ERC-1155 TransferSingle/TransferBatch/ApprovalForAll/URI with edge-case handling.
- Additional implementation constraints introduced:
  - Fallback decoding for live ERC-1155 dynamic payload shapes (URI and TransferBatch).
- Boundary behavior and terminal/end-state rules:
  - Missing topics and malformed data never panic; they return `DecodeError`.

## 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - Shared `ApprovalForAll(address,address,bool)` signature appears in both ERC-721 and ERC-1155.
- How the ambiguity was resolved in code:
  - Main router defaults shared signature into ERC-721 path; explicit ERC-1155 helper remains available and is verified in tests.
- Additional user-facing resolution:
  - The terminal-facing `decode-receipt` command exposes shared `ApprovalForAll` as ambiguous by default and accepts an explicit override flag when the operator knows the intended standard.
- Any controlled divergence from the spec:
  - None affecting required behavior; live coverage was added as an explicit pre-A-2 closure requirement.
- Follow-up needed in the spec or task list:
  - None.

## 7. Testing Notes

- Unit tests added:
  - None.
- Integration tests added:
  - `src/eth_node/tests/decode_live.rs` (16 canonical/edge tests)
  - `src/eth_node/tests/decode_anvil_live.rs` (7 deploy-driven live capture tests)
  - `src/eth_node_cli/tests/decode_receipt_cli.rs` (4 CLI receipt-decode tests)
- Property-based tests added:
  - Decoder no-panic property is covered in T-005 fuzz suite.
- Edge cases covered:
  - Self-transfer, zero-value transfer, max uint256 ids, empty TransferBatch arrays, unsupported signatures, missing topics.
- Failure modes exercised:
  - Malformed/missing topic and invalid ABI payload decode paths.
- Runtime/orchestration branch evidence covered:
  - Real Anvil deployment, event emission via tx, receipt-log capture, and decode assertions for both standards.
- Golden input/output examples captured:
  - Canonical synthetic logs and live emitted logs from Solidity fixtures.

## 7.1 Verification Artifact (mandatory — entry is incomplete without this field)

Verification artifact:
- `cargo test --package eth_node --test decode_live`
  - Result: 16 passed; 0 failed.
- `cargo test --package eth_node --test decode_anvil_live`
  - Result: 7 passed; 0 failed.
- `cargo test --package eth_node_cli`
  - Result: 32 passed; 0 failed.
- `cargo test --package eth_node --all-features`
  - Result: full package green, including both decode suites.
- `scripts/capture-session.ps1 decode-receipt 0xa81c52b998ee2b353815d1c7790eb1b67f0840b4b86d0f2e9e97445c1e4bb983`
  - Result: live session artifact captured at `output/sessions/2026-03-28_19-38-53/` showing ERC-721 `Transfer` decode from a mined Anvil receipt with command body and output present in `screen.log`.

## 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
  - Rebuild typed decoders first, then synthetic decode tests, then live Solidity emitters + Anvil receipt-capture tests, then the CLI `decode-receipt` surface and guide examples.
- Order of implementation steps that mattered:
  - Functional decode baseline first, then live deploy/capture closure before gate review.
- Non-obvious pitfalls discovered during implementation:
  - Live ERC-1155 dynamic event data can require fallback decode paths beyond synthetic struct ABI assumptions.
- What not to change without updating the behavioral spec:
  - Event signature routing and required event-family coverage matrix.
- Side effects and external touchpoints (files/network/stdout/runtime context):
  - Live tests require Anvil and Forge in PATH.
  - Manual CLI evidence also uses `cast send` and a current release build when routed through `capture-session.ps1`; the PowerShell capture helper was updated to tee native command output into `screen.log` reliably.

## 9. Known Limitations

- Limitation 1:
  - Shared `ApprovalForAll` signature remains ambiguous in generic routing without contract-context hints.
- Reason accepted:
  - Explicit per-standard helper APIs and tests provide deterministic decode paths where caller context exists.
- Revisit trigger:
  - If a future API revision requires automatic ERC-721 vs ERC-1155 disambiguation from generic logs.

## 10. Approval / Review

- Reviewed by:
  - Pending Team Lead delegated gate review (A-2 window)
- Review date:
  - Pending
- Notes:
  - T-004 evidence package now includes both canonical and deploy-driven live decode validation.
- Pair-programming references (if applicable):
  - Session log path:
    - Not separately recorded
  - Driver role(s):
    - GitHub Copilot implementation
  - Navigator/reviewer role(s):
    - Team governance constraints and stage checks
  - Key disagreement and resolution summary:
    - Live tests exposed dynamic-payload decode mismatch; resolved via robust fallback decoding.

## 11. Reconstruction Bundle (required at Stage 4 close for major modules)

- Source tree manifest relevant to this module:
  - `src/eth_node/src/quality/decode.rs`
  - `src/eth_node/src/quality/mod.rs`
  - `src/eth_node_cli/src/main.rs`
  - `src/eth_node_cli/tests/decode_receipt_cli.rs`
  - `src/eth_node/tests/decode_live.rs`
  - `src/eth_node/tests/decode_anvil_live.rs`
  - `src/eth_node/tests/contracts/TestERC721.sol`
  - `src/eth_node/tests/contracts/TestERC1155.sol`
  - `CLI_REFERENCE.md`
- Import/dependency graph expectations:
  - Uses `alloy-rpc-types` logs, `alloy-sol-types` ABI decode, and shared Anvil/RPC test helpers.
- Recommended reconstruction order:
  - Decoder types and dispatch -> canonical tests -> live emitter contracts -> deploy/capture tests -> CLI decode-receipt command -> guide examples -> release/manual validation.
- Post-rebuild validation commands:
  - `cargo test --package eth_node --test decode_live`
  - `cargo test --package eth_node --test decode_anvil_live`
  - `cargo test --package eth_node_cli`
  - `cargo test --package eth_node --all-features`
- Expected validation outputs/pass criteria:
  - 23 library decode tests plus 4 CLI receipt-decode tests pass, with no decode panics, full CLI package green, and one successful capture-session decode artifact.
