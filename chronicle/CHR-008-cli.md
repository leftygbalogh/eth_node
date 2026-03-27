# Implementation Chronicle — CLI Binary

- Chronicle ID: CHR-008
- Source task ID: T-008
- Source spec sections: §7.1 Q3-ARCH-01 (CLI as thin wrapper), NFR-004 (no domain logic in CLI)
- Module / component name: `eth_node_cli`
- Implementation language: Rust
- Status: Complete (Stage 5 fix applied)

---

## 1. Summary

Implemented `eth_node_cli`, a thin Clap-based binary that exposes five subcommands (`balance`, `send`, `watch`, `call`, `tx-status`) as direct wrappers over `eth_node::*` library calls.  All domain logic lives in the library; the CLI handles only argument parsing, library delegation, and output formatting.

## 2. Intent to Implementation Mapping

| Spec item | Implementation |
|---|---|
| Q3-ARCH-01 — CLI is a thin wrapper | CLI subcommand handlers delegate immediately to `eth_node::*`; no business logic |
| NFR-004 — no domain logic in CLI layer | `event_selector` moved to `eth_node::primitives` during Stage 5 (see D5) |
| §7.1 `balance` | `cmd_balance` → `RpcClient::get_balance` → print Wei + Ether |
| §7.1 `send` | `cmd_send` → `TxBuilder` + `EthSigner` + `Broadcaster` → print hash |
| §7.1 `watch` | `cmd_watch` → `Listener::subscribe` → print log events until Ctrl-C |
| §7.1 `call` | `cmd_call` → `ContractCaller::call` → print decoded values |
| §7.1 `tx-status` | `cmd_tx_status` → `RpcClient::get_transaction_receipt` → print status |
| NFR-003 — observability | `--log-level` / `--quiet` flags; `tracing-subscriber` JSON on stderr |
| §5.1 `--dump-state` | On success, serialize JSON result to the specified file path |

## 3. Implementation Decisions

**D1 — Clap `Derive` API**: `#[derive(Parser, Subcommand)]` on `Cli` / `Commands` structs.  Derive removes all manual `App::new()` boilerplate and ensures `--help` and `--version` are auto-generated.

**D2 — No business logic constraint enforced by review**: Each `cmd_*` function performs exactly: parse CLI strings → call one `eth_node::*` function → format output.  Any temptation to add logic (fee estimation, retry) is redirected to the library.

**D3 — `--dump-state <path>` flag**: On success, the JSON `serde_json::Value` returned by each `cmd_*` function is serialised and written to the specified path.  This supports automation and scripting without adding a separate `dump` subcommand.

**D4 — `--quiet` / `--log-level` for observability control**: `--quiet` overrides `--log-level` to `error`, suppressing all non-error tracing output.  `--log-level` accepts any `tracing_subscriber` level string (`trace`, `debug`, `info`, `warn`, `error`).

**D5 — `event_selector` moved to `eth_node::primitives` (Stage 5 fix)**: During the Stage 5 Traceability Mapper audit (Gap-005), the `topic0_from_str` helper — which computes `keccak256(event_sig)` — was identified as domain logic sitting in the CLI layer, violating NFR-004.  The function was renamed `event_selector`, added to `eth_node::primitives` with a doc-test, and the CLI `cmd_watch` handler was updated to call `eth_node::primitives::event_selector`.

## 4. Alternatives Considered

- **`structopt` (pre-Clap-v4 derive)**: Rejected; Clap v4 Derive is the idiomatic current choice.
- **`anyhow` for error propagation in `cmd_*`**: Rejected; `cmd_*` functions return `Result<Value, String>` — the simple string error is sufficient for a CLI that immediately prints and exits.
- **Keeping `topic0_from_str` in the CLI**: Rejected at Stage 5 review; the function is domain logic (keccak256 of an event signature) and belongs in the library.

## 5. Derived Invariants and Constraints

- No `fn` in `main.rs` may perform keccak256 hashing, RLP encoding, ABI encoding, fee estimation, or any other computation that belongs in the library.  Detected at Stage 5 via Gap-005.
- `--dump-state` must produce valid JSON that round-trips through `serde_json::from_str` — enforced by integration test `test_cli_dump_state_writes_file`.
- `eth_node::primitives::event_selector` must pass through an already-hashed 32-byte hex string unchanged — enforced by unit test `topic0_passthrough_existing_hash` (now in CLI unit tests, calling the library function).

## 6. Test Results

11 tests, all passing: 8 unit (in `main.rs #[cfg(test)]`) + 3 CLI integration (subprocess) added at Stage 5.

| Test | What it checks |
|---|---|
| `parse_address_arg` | Address string → `DynSolValue::Address` |
| `parse_bool_true_arg` | `"true"` → `DynSolValue::Bool(true)` |
| `parse_bool_false_arg` | `"false"` → `DynSolValue::Bool(false)` |
| `parse_uint256_decimal` | Decimal string → `DynSolValue::Uint` |
| `parse_bytes_hex_arg` | `0x`-prefixed hex → `DynSolValue::Bytes` |
| `parse_string_fallback_arg` | Non-numeric string → `DynSolValue::String` |
| `topic0_from_known_transfer_signature` | `event_selector("Transfer(…)")` == known keccak256 |
| `topic0_passthrough_existing_hash` | 32-byte hex string passed through unchanged |
| `test_cli_send` (integration) | `send` subcommand broadcasts ETH, prints tx hash |
| `test_cli_watch_prints_banner` (integration) | `watch` subcommand prints "Watching" banner |
| `test_cli_call_graceful` (integration) | `call` subcommand exits 0 or 1 (no crash) |

