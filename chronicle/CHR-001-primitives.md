# Implementation Chronicle — Primitives Module

- Chronicle ID: CHR-001
- Source task ID: T-001
- Source spec sections: §4.1 (FR-001: Ethereum Primitives), §9.2 (deterministic test vectors)
- Module / component name: `eth_node::primitives`
- Implementation language: Rust
- Status: Final

## 2. Intent to Implementation Mapping

- Implements all FR-001 behaviors: Address parsing (hex with/without `0x`, checksummed/lowercase, typed error on wrong length/invalid hex), U256 arithmetic (checked add/sub with overflow protection), ABI encode/decode (uint256, address, bool, bytes32, string, struct tuple), RLP encode/decode (u64 integers, byte arrays, nested structs).
- All operations are pure functions — no I/O, no async.
- Spec known vectors verified: `uint256(1)` → 31 zero bytes + `0x01`; RLP of integer 0 → `0x80`; RLP of integer 1 → `0x01`.

## 3. Implementation Decisions

- **alloy-primitives chosen over manual types**: `Address`, `B256`, `U256` come directly from `alloy-primitives 0.8`. No hand-rolled type aliases needed — the alloy types carry correct semantics (checksummed display, safe arithmetic).
- **alloy-sol-types `SolValue` trait for ABI**: Encoding/decoding via `T::abi_encode()` / `T::abi_decode(data, true)`. The `sol! {}` macro generates a struct type (`AbiTuple`) for the tuple round-trip test. Strict validation (`true` flag) ensures length/type errors surface cleanly.
- **alloy-rlp for RLP**: `Encodable` / `Decodable` traits used directly. `RlpEncodable` / `RlpDecodable` derive macros used for the `RlpList` nested struct.
- **`Vec<u8>` vs `bytes::Bytes` for RLP byte strings**: `Vec<u8>` implements `Decodable` as an RLP *list*, not a byte string. Decoding raw byte arrays requires `bytes::Bytes::decode` (which decodes as a byte string). Encoding via `&[u8]::encode` is correct (encodes as byte string). Discovery: first two byte tests failed at green phase; fixed by switching decoder to `bytes::Bytes`.
- **`bytes` crate added as workspace dependency** (v1) to support above fix. `bytes` was already a transitive dependency of alloy-rlp; promoting it to explicit is correct.
- **Error type**: `PrimitiveError` with four variants (`InvalidHex`, `InvalidLength`, `AbiDecodeError`, `RlpDecodeError`) using `thiserror::Error`. Maps cleanly to FR-001 postconditions.

## 4. Alternatives Considered

- Alternative: hand-roll `Address` and `U256` types. Rejected — alloy-primitives provides production-quality, well-tested implementations compatible with the rest of the alloy stack.
- Alternative: use `ethabi` crate for ABI encode/decode. Rejected — alloy-sol-types is the alloy-native approach and aligns with T-006 (contract calls via `alloy-sol-macro`).

## 5. Derived Invariants and Constraints

- ABI encoding of fixed-width types (uint256, address, bool, bytes32) always produces exactly 32 bytes.
- ABI encoding of dynamic types (string, bytes, tuples with dynamic fields) produces variable-length output with offset headers.
- RLP of integer 0 = `0x80` (RLP empty string). RLP of integer 1 = `0x01` (single byte, value < 0x80, encoded as itself).
- `parse_address` must accept `0x`-prefixed and unprefixed input; must reject non-40-character hex.

## 6. Test Results

- 23 unit tests, all passing: `cargo test -p eth_node primitives`
- `cargo clippy -p eth_node -- -D warnings`: clean (zero warnings)
