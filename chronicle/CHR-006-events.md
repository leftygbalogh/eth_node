# Implementation Chronicle — Events Module

- Chronicle ID: CHR-006
- Source task ID: T-006
- Source spec sections: §4.6 (FR-006: Event Listener)
- Module / component name: `eth_node::events`
- Implementation language: Rust
- Status: Complete

---

## 1. Summary

Implemented `Listener`, a type that returns a `Stream<Item = Result<Log, EventError>>` of Ethereum logs, supporting both HTTP-poll and WebSocket transport modes.  The stream reconnects automatically on transient failures up to a configurable limit.

## 2. Intent to Implementation Mapping

| Spec item | Implementation |
|---|---|
| FR-006 §1 — subscribe to logs | `Listener::new(endpoint).subscribe(filter)` → `Pin<Box<dyn Stream>>` |
| FR-006 §2 — HTTP poll fallback | `Listener` detects non-WS endpoint → uses HTTP poll loop |
| FR-006 §3 — reconnect on error | `max_reconnect: Option<u32>` controls retry budget |
| AC-006 — stream item type | `Result<Log, EventError>` per item |

## 3. Implementation Decisions

**D1 — `async_stream::stream!` macro**: Used for both HTTP and WS stream bodies.  The macro enables `yield` semantics inside an `async` block without requiring manual `Stream` impl boilerplate.

**D2 — WS stream is lazy**: The WebSocket connection is opened only on the first `poll_next()` call, not at `subscribe()` time.  This ensures no network activity occurs until the caller actually drives the stream — consistent with the broader library's contract.

**D3 — `max_reconnect: Option<u32>`**: `None` means infinite reconnects; `Some(n)` caps consecutive error reconnects at `n`.  Default `Some(3)` balances reliability against masking permanent failures.  This is A1 Option D from the design decision session.

**D4 — HTTP poll uses `consecutive_errors` counter**: The HTTP poll loop tracks consecutive non-transient errors and emits `EventError::ReconnectExhausted` once the budget is exceeded, mirroring the WS reconnect behaviour for transport-neutral error handling.

**D5 — Inline tokio TCP proxy for reconnect test (G3)**: Rather than requiring an external tool, the reconnect integration test wires a `tokio::net::TcpListener` as a transparent proxy that drops the connection mid-stream.  This keeps the test hermetic and fast.

**D6 — Transport-neutral error display**: `EventError::ReconnectExhausted` displays as `"reconnect exhausted after N consecutive errors"` without naming HTTP or WS, so callers do not need to branch on transport type.

## 4. Alternatives Considered

- **`tokio-stream`'s `unfold`**: Considered for the HTTP poll loop.  Rejected in favour of `async_stream::stream!` for readability — `stream!` reads like sequential imperative code and is easier to extend.
- **A1 Option A — fail-fast (no reconnect)**: Rejected; the spec requires survivability through transient node restarts.
- **A1 Option B — infinite reconnect by default**: Rejected; infinite retry without a budget can mask persistent failures and make tests hang.

## 5. Derived Invariants and Constraints

- A stream with `max_reconnect = Some(0)` must emit at most one item before yielding `ReconnectExhausted` on the first error — verified by unit test.
- WS stream must not open a TCP connection until the first `poll_next()` — verified by spy test that asserts no connection during `subscribe()`.

## 6. Test Results

14 tests, all passing: 9 unit + 5 integration.

| Test | What it checks |
|---|---|
| `listener_new_http` | HTTP endpoint accepted |
| `listener_new_ws` | WS endpoint accepted |
| `stream_item_type_is_result` | Stream yields `Result<Log, EventError>` |
| `reconnect_exhausted_after_budget` | `max_reconnect=Some(1)` → `ReconnectExhausted` on 2nd error |
| `infinite_reconnect_none` | `max_reconnect=None` does not emit `ReconnectExhausted` |
| `error_display_transport_neutral` | Display does not mention HTTP or WS |
| `ws_lazy_no_connect_at_subscribe` | No TCP open until first poll |
| `http_consecutive_errors_tracked` | Counter increments per error |
| `filter_passed_to_get_logs` | Filter address/topics forwarded to RPC |
| `integration_http_*` | ×5 against Anvil: subscribe, emit event, receive log |

