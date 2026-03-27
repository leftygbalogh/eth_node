# Implementation Chronicle — Signer Module

- Chronicle ID: CHR-004
- Source task ID: T-004
- Source spec sections: §4.4 (FR-004: Transaction Signing)
- Module / component name: `eth_node::signer`
- Implementation language: Rust
- Status: Complete

---

## 1. Summary

Implemented `EthSigner`, a thin wrapper around `alloy_signer_local::PrivateKeySigner` that:
- loads a secp256k1 private key from a hex string or from `ETH_PRIVATE_KEY` env var;
- signs both EIP-1559 and legacy transactions via a single `sign(UnsignedTx)` method;
- never exposes the raw key in `Debug` output, logs, or error messages.

## 2. Intent to Implementation Mapping

| Spec item | Implementation |
|---|---|
| FR-004 §1 — sign EIP-1559 tx | `EthSigner::sign(UnsignedTx::Eip1559(tx))` → `SignedBytes` |
| FR-004 §2 — sign legacy tx | `EthSigner::sign(UnsignedTx::Legacy(tx))` → `SignedBytes` |
| FR-004 §3 — load from env | `EthSigner::from_env()` reads `ETH_PRIVATE_KEY` |
| NFR-003 — key must not appear in logs | Custom `Debug` impl + `key_not_in_log_output` test |

## 3. Implementation Decisions

**D1 — `alloy_signer_local::PrivateKeySigner`**: Selected as the signing primitive because it is the canonical alloy v0.x secp256k1 signer and handles both EIP-1559 and legacy encoding internally.

**D2 — `UnsignedTx` enum**: A two-variant enum (`Eip1559` / `Legacy`) was introduced so the single `sign()` method can accept either transaction type without overloading.  The enum lives in `signer.rs`, not in `tx.rs`, to keep signing self-contained.

**D3 — Custom `Debug` impl**: `EthSigner` implements `Debug` manually, printing only `EthSigner { address: 0x… }`.  The derived `Debug` would have exposed the raw key bytes through `PrivateKeySigner`'s own `Debug`.

**D4 — `from_key` vs `from_env`**: Two constructors keep the public API simple.  `from_key` is the building block; `from_env` is a convenience wrapper that reads the env var and delegates.

## 4. Alternatives Considered

- **`eth-keystore` file-based loading**: Rejected for Phase 1 — the spec requires private-key-string input; keystore support is deferred.
- **Two separate `sign_eip1559` / `sign_legacy` methods**: Rejected; a single `sign(UnsignedTx)` is cleaner and matches the spec's single FR-004 signing method requirement.

## 5. Derived Invariants and Constraints

- `key_not_in_log_output`: a tracing subscriber captures log output during a `sign()` call; the test asserts the raw key bytes do not appear in any log line.  This constraint must be preserved when adding new log statements to `signer.rs`.
- `EthSigner::address()` always returns the Ethereum address derived from the loaded key — verified by round-trip tests in `from_key_*` suite.

## 6. Test Results

9 unit tests, all passing:

| Test | What it checks |
|---|---|
| `from_key_valid_hex_with_prefix` | Constructor accepts `0x`-prefixed key |
| `from_key_valid_hex_no_prefix` | Constructor accepts bare hex |
| `from_key_invalid_hex_returns_err` | Non-hex input → `SignerError::InvalidKey` |
| `from_key_wrong_length_returns_err` | 31-byte key → `SignerError::InvalidKey` |
| `from_env_missing_returns_err` | Missing env var → `SignerError::MissingEnvVar` |
| `from_env_set_reads_key` | Env var set → signer constructed, address matches |
| `sign_eip1559_produces_rlp` | Signed EIP-1559 tx → non-empty RLP bytes |
| `sign_legacy_produces_rlp` | Signed legacy tx → non-empty RLP bytes |
| `key_not_in_log_output` | Raw key bytes absent from all tracing output |

