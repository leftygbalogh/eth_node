# Implementation Chronicle — Transaction Module

- Chronicle ID: CHR-005
- Source task ID: T-005
- Source spec sections: §4.3 (FR-003: Transaction Builder), §4.5 (FR-005: Broadcaster)
- Module / component name: `eth_node::tx`
- Implementation language: Rust
- Status: Complete

---

## 1. Summary

Implemented two collaborating types in `eth_node::tx`:
- `TxBuilder` — constructs and optionally signs EIP-1559 or legacy transactions, with fee auto-selection;
- `Broadcaster` — submits a signed transaction and polls for confirmation with configurable timeout.

## 2. Intent to Implementation Mapping

| Spec item | Implementation |
|---|---|
| FR-003 §1 — build EIP-1559 tx | `TxBuilder::build()` with `FeeConfig::Eip1559` |
| FR-003 §2 — build legacy tx | `TxBuilder::build()` with `FeeConfig::Legacy` |
| FR-003 §3 — auto fee selection | `FeeConfig::Auto` → prefer EIP-1559 if `eth_maxFeePerGas` available |
| FR-003 §4 — conflicting fee params | Two bool flags; `ConflictingFeeParams` error if both set |
| FR-005 §1 — broadcast & wait | `Broadcaster::send()` polls receipt until confirmed or timeout |
| FR-005 §2 — timeout with hash | `TxError::ConfirmationTimeout { hash }` preserves hash for retry |

## 3. Implementation Decisions

**D1 — `FeeConfig` enum (`Auto` / `Eip1559` / `Legacy`)**: Drives the fee-selection decision table from the spec.  `Auto` does a capability probe at build-time; the other two are explicit.

**D2 — `DEFAULT_AUTO_TIP_DIVISOR = 10`**: Named constant (extracted during a Lefty review of magic numbers).  The divisor sets the priority fee to `gas_price / 10` in Auto mode.

**D3 — Two bool flags for conflict detection**: `has_gas_price` and `has_max_fee` flags track which fee fields the caller set.  A `ConflictingFeeParams` error is returned if both are set, rather than silently preferring one.

**D4 — `BroadcastConfig` struct**: Holds `poll_interval` (default 500 ms) and `timeout` (default 60 s) as `Duration` fields.  Passing `None` to `Broadcaster::send()` uses defaults.  This avoids bloating the `send` signature with six positional parameters.

**D5 — `TxError::ConfirmationTimeout { hash }`**: Preserves the transaction hash in the timeout error so callers can resume polling without resubmitting.

## 4. Alternatives Considered

- **`fee_config: Option<FeeConfig>`** with `None = Auto`: Rejected; an explicit `Auto` variant is more readable at call sites and avoids `unwrap_or_default` chains.
- **Inline poll loop in `send()`**: Accepted; the poll logic is 20 lines and does not warrant a separate type.

## 5. Derived Invariants and Constraints

- A `TxBuilder` with both `gas_price` and `max_fee_per_gas` set must return `ConflictingFeeParams` — enforced by unit test `builder_conflicting_fee_params_returns_err`.
- `ConfirmationTimeout` must carry the transaction hash — enforced by matching on the error variant in the broadcaster integration test.

## 6. Test Results

13 tests, all passing: 7 `TxBuilder` unit + 1 `BroadcastConfig` unit + 5 integration.

| Test | What it checks |
|---|---|
| `builder_sets_recipient` | `to` field round-trips through build |
| `builder_sets_value` | `value` field round-trips |
| `builder_sets_data` | `input` bytes round-trip |
| `builder_auto_fee_uses_rpc_gas_price` | Auto mode fetches gas price from RPC |
| `builder_eip1559_fee_config` | Explicit EIP-1559 fees respected |
| `builder_legacy_fee_config` | Explicit legacy gas price respected |
| `builder_conflicting_fee_params_returns_err` | Both fee fields → `ConflictingFeeParams` |
| `broadcast_config_defaults` | Default poll interval = 500 ms, timeout = 60 s |
| `integration_send_and_confirm_*` | ×5 against Anvil: build → sign → broadcast → receipt |

