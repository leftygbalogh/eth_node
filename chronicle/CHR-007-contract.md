# Implementation Chronicle — Contract Module

- Chronicle ID: CHR-007
- Source task ID: T-007
- Source spec sections: §4.7 (FR-007: Contract Interaction)
- Module / component name: `eth_node::contract`
- Implementation language: Rust
- Status: Complete

---

## 1. Summary

Implemented `ContractCaller`, a runtime ABI-aware contract interaction type that:
- parses a JSON ABI (single fragment or full array) at construction time;
- resolves overloaded functions by trying each overload's `abi_encode_input` until one succeeds;
- exposes `call()` for view functions (returns `Vec<DynSolValue>`) and `send()` for state-changing calls (returns `TransactionReceipt`).

## 2. Intent to Implementation Mapping

| Spec item | Implementation |
|---|---|
| FR-007 §1 — parse ABI | `ContractCaller::new(addr, abi_json)` → `JsonAbi` via `alloy_json_abi` |
| FR-007 §2 — encode call | `abi_encode_input(args)` via `alloy_dyn_abi::DynSolValue` |
| FR-007 §3 — read call | `ContractCaller::call(fn, args, client)` → `Vec<DynSolValue>` |
| FR-007 §4 — write call | `ContractCaller::send(fn, args, signer, client, cfg)` → `TransactionReceipt` |
| FR-007 §5 — overload resolution | Try overloads in definition order; pick first successful encode |
| AC-005 — exact uint256 return | Integration test G1 asserts `balanceOf` returns exactly `U256::from(1000)` |

## 3. Implementation Decisions

**D1 — `alloy_json_abi::JsonAbi` + `alloy_dyn_abi`**: Selected as the ABI parsing and encoding stack.  `JsonAbi` handles both single-fragment and full-array JSON; `DynSolValue` provides runtime-typed encoding without code generation.

**D2 — Overload resolution by first-success encode**: `resolve_function` iterates the list of `Function` entries with matching name in definition order and picks the first where `abi_encode_input(args)` returns `Ok`.  This matches Solidity tooling convention and avoids ambiguity errors for callers who pass the correct argument count.

**D3 — `ContractCaller::send` delegates to `TxBuilder` + `EthSigner` + `Broadcaster`**: No duplicate signing or broadcasting logic lives in `contract.rs`.  The write path is: ABI encode → `TxBuilder` with `input` → `EthSigner::sign` → `Broadcaster::send`.

**D4 — Hand-assembled EVM bytecodes for integration tests**: Three minimal bytecode sequences (`STUB_TOKEN_INIT_CODE`, `NOOP_INIT_CODE`, `REVERTER_INIT_CODE`) are declared as byte-array constants.  No Solidity compiler dependency is required, which keeps the test suite fully hermetic.

## 4. Alternatives Considered

- **`ethabi` crate**: Considered instead of `alloy_json_abi`.  Rejected: `alloy_json_abi` is the alloy-ecosystem-native choice and shares types with the rest of the library.
- **Code generation via `abigen!` macro**: Rejected for Phase 1 — the spec requires dynamic (runtime) ABI handling, not compile-time-generated bindings.
- **Panic on first overload mismatch**: Rejected; silent fallthrough to the next overload is more useful and matches `cast call` behaviour.

## 5. Derived Invariants and Constraints

- `ContractError::AbiNotFound(name)` is returned when no function with the given name exists in the parsed ABI — enforced by unit test `resolve_unknown_function_returns_error`.
- The `STUB_TOKEN_INIT_CODE` runtime unconditionally returns `uint256(1000)` for any `eth_call` — the G1 integration test asserts the exact value `1000`, making this a regression anchor.

## 6. Test Results

17 tests, all passing: 11 unit + 6 integration.

| Test | What it checks |
|---|---|
| `new_accepts_single_function_fragment` | Single ABI fragment parsed |
| `new_accepts_full_abi_array` | Full ABI array parsed |
| `new_rejects_invalid_json` | Malformed JSON → `ContractError::AbiParse` |
| `resolve_known_function` | Named function found in ABI |
| `resolve_unknown_function_returns_error` | Unknown name → `AbiNotFound` |
| `resolve_overloaded_function` | Second overload selected when first arg-count mismatch |
| `encode_uint256_arg` | `uint256` argument encoded correctly |
| `encode_address_arg` | `address` argument encoded correctly |
| `encode_bool_arg` | `bool` argument encoded correctly |
| `decode_uint256_return` | `uint256` return decoded correctly |
| `decode_empty_return` | Empty return data → empty `Vec` |
| `integration_call_zero_address` | eth_call to zero address → decode error (graceful) |
| `integration_missing_fn_returns_error` | AbiNotFound on unknown fn name |
| `integration_call_balance_of_exact_value` (G1) | `balanceOf` returns exactly 1000 |
| `integration_send_write_path` (G4) | Full write path: encode → sign → broadcast → receipt.status==true |
| `integration_reverter_call_returns_err` | Reverting contract → `ContractError` |
| `integration_overload_resolution_live` | Overload resolution works against live node |

