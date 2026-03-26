# Implementation Chronicle — RPC Client Module

- Chronicle ID: CHR-003
- Source task ID: T-004
- Source spec sections: §4 FR-002 (JSON-RPC Client)
- Module / component name: `eth_node::rpc`
- Implementation language: Rust
- Status: Final

---

## What was built

`src/eth_node/src/rpc.rs` — typed JSON-RPC client wrapping `alloy_provider`.

### `RpcError` enum

Four variants matching FR-002 error-handling requirements:

| Variant | Recoverable? | When raised |
|---------|-------------|-------------|
| `Transport(String)` | Yes (retry) | Network failure: connection refused, DNS |
| `JsonRpc { code, message }` | No | Server returned JSON-RPC error object |
| `Timeout` | Yes | Detected via `"timed out"` in transport message |
| `Deserialization(String)` | No | Response could not be decoded |
| `InvalidUrl(String)` | No | `endpoint` is not a valid URL |

### `RpcClient` struct

```rust
#[derive(Debug)]
pub struct RpcClient {
    inner: RootProvider<Ethereum>,
    endpoint: String,
}
```

Constructor `RpcClient::new(endpoint: &str) -> Result<Self, RpcError>` validates the URL immediately; the network connection is lazy (first RPC call).

### Eleven typed methods (all FR-002 calls)

| Method | JSON-RPC call | Return |
|--------|--------------|--------|
| `block_number()` | `eth_blockNumber` | `u64` |
| `get_balance(addr)` | `eth_getBalance(addr, "latest")` | `U256` |
| `get_balance_at(addr, block)` | `eth_getBalance(addr, block)` | `U256` |
| `get_nonce(addr)` | `eth_getTransactionCount(addr, "latest")` | `u64` |
| `get_nonce_at(addr, block)` | `eth_getTransactionCount(addr, block)` | `u64` |
| `get_transaction_receipt(hash)` | `eth_getTransactionReceipt(hash)` | `Option<TransactionReceipt>` |
| `get_block_by_number(tag)` | `eth_getBlockByNumber(tag)` | `Option<Block>` |
| `send_raw_transaction(bytes)` | `eth_sendRawTransaction(bytes)` | `B256` (tx hash) |
| `call(tx)` | `eth_call(tx, "latest")` | `Bytes` |
| `get_logs(filter)` | `eth_getLogs(filter)` | `Vec<Log>` |
| `chain_id()` | `eth_chainId` | `u64` |
| `gas_price()` | `eth_gasPrice` | `u128` |
| `estimate_gas(tx)` | `eth_estimateGas(tx)` | `u64` |

---

## Key engineering decisions

### D1: `ProviderBuilder::default()` instead of `ProviderBuilder::new()`

`ProviderBuilder::new()` in alloy 0.12 adds recommended fillers
(`GasFiller`, `BlobGasFiller`, `NonceFiller`, `ChainIdFiller`) making its
return type `FillProvider<JoinFill<...>, RootProvider<Ethereum>, ...>` — not
nameable without importing private alloy types.

`ProviderBuilder::default()` gives a filler-free builder; `on_http(url)` then
returns plain `RootProvider<Ethereum>`, which we store directly in `RpcClient`.
No generics on the struct, no complex type alias. Since T-004 is read-only
calls only, we do not need transaction-filling fillers.

### D2: Named `map_transport_err` function instead of `From` impl

Keeps the error-mapping explicit and avoids accidental `?` coercions from
internal alloy code that surfaces `TransportError` in contexts we don't control.

### D3: `alloy_transport::TransportError` variant matching

- `ErrorResp(ErrorPayload)` → `RpcError::JsonRpc` (code + message extracted)
- `DeserError { err, .. }` → `RpcError::Deserialization`
- Other → `RpcError::Transport` (with `"timed out"` string-heuristic for `Timeout`)

### D4: `get_block_by_number` is single-arg in alloy 0.12

Unlike older alloy and Ethers.js, alloy 0.12's `Provider::get_block_by_number`
does not accept a hydration `bool`; the single-arg form returns transaction hashes.

---

## Tests added

### Unit tests (in `rpc.rs #[cfg(test)]`)

| Test | Verifies |
|------|---------|
| `new_rejects_bad_url` | `RpcError::InvalidUrl` on invalid URL |
| `new_accepts_valid_http_url` | Construction succeeds; endpoint stored correctly |

### Integration tests (`tests/integration_rpc.rs`) — 6 tests

Skip gracefully when Anvil not on PATH (same pattern as T-002 anvil-fixture tests).

| Test | Verifies |
|------|---------|
| `rpc_block_number_is_zero_on_fresh_anvil` | `eth_blockNumber` → 0 |
| `rpc_get_balance_returns_initial_anvil_balance` | Account 0 = 10,000 ETH |
| `rpc_chain_id_is_31337_for_anvil` | `eth_chainId` → 31,337 |
| `rpc_nonce_is_zero_for_fresh_account` | `eth_getTransactionCount` → 0 |
| `rpc_gas_price_is_nonzero` | `eth_gasPrice` > 0 |
| `rpc_client_rejects_invalid_url` | `RpcError::InvalidUrl` matched |

---

## Test run result

```
34 tests total (25 unit + 2 anvil-fixture + 6 rpc-integration + 1 doc-test)
all passed — Anvil tests skip gracefully locally, run for real in CI
```

---

## Dependencies added

| Crate | Reason |
|-------|--------|
| `url = "2"` | `ProviderBuilder::on_http` takes `url::Url` |
