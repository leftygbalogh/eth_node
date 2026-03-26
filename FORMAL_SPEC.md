# Formal Specification — eth_node (Phase 1)

## 1. Specification Metadata

Layer metadata: Layer 2 of the three-layer documentation stack (Commander's Intent → Behavioral Specification → Implementation Chronicle).
Required linkage: sourced from `memory.md` Stage 1 decisions (PROJECT_BRIEF artifact promoted to memory pending brief file creation); implementation plans map to `IMPLEMENTATION_CHRONICLE.md`.

- Spec ID: SPEC-001
- Version: 1.0
- Project mode: Greenfield
- Declared implementation language: Rust (primary)
- Language-specific constraints captured: Rust ownership model, trait-based API abstraction, `async`/`await` with Tokio runtime, Cargo workspace structure
- Source brief: memory.md Stage 1 decisions (2026-03-26)
- Approval authority: Lefty (owner-only for Stage 2)
- Status: Draft — awaiting Stage 2 approval
- Author: Greenfield Formal Specification Author
- Reviewers: Oracle Agent, Claire Voyant Agent
- Active Q3 modules: Q3-ARCH-01 (Layered Architecture — active)

---

## 1.1 Mode Constraints

- Greenfield constraints: no legacy behavior to preserve; architecture decisions are forward-only; all design choices must justify their evolution path.
- Evolution assumption: the library is designed so Phase 2 components (#7–#12) can be added as new crate modules without changing Phase 1 API surface.

---

## 1.2 Behavioral Specification Approach

- Statechart coverage: RPC client connection lifecycle; Transaction lifecycle (build → sign → broadcast → pending → confirmed/failed); Event listener lifecycle (connect → subscribe → listening → event received → error/disconnect)
- Contract coverage: `rpc::Client::call`, `signer::Signer::sign_transaction`, `tx::Broadcaster::send`, `events::Listener::subscribe`, `contract::ContractCaller::call`
- Decision table coverage: transaction fee selection (legacy vs EIP-1559); event filter construction; error classification (transient vs permanent)
- Escalation to mathematical verification: No — Phase 1 scope and risk level do not require it

---

## 2. Scope

**In scope (Phase 1):**
1. Ethereum primitives (addresses, hashes, U256, ABI encoding/decoding, RLP serialization)
2. JSON-RPC client connecting to any Ethereum-compatible endpoint (Anvil, hosted RPC)
3. Transaction builder (legacy and EIP-1559 types)
4. Local transaction signer using a private key
5. Transaction broadcaster with confirmation tracking
6. Event/log listener (polling and subscription modes)
7. ABI-driven contract caller (read/write)
8. CLI binary providing interactive access to capabilities #1–#7
9. Anvil local devnet as the primary test and development target

**Out of scope (Phase 1):**
- Peer-to-peer networking or devp2p protocol
- Block or chain state synchronization
- Consensus layer (PoS, attestations, beacon chain)
- Validator key management or slashing protection
- Mempool monitoring (deferred to Phase 2, component #8)
- State trie reading (deferred to Phase 2, component #11)
- Reth node integration (deferred to Phase 2 pending disk hardware)
- Wallet UI or GUI of any kind
- Solidity/Vyper compilation or deployment tooling
- Production deployment or public-facing exposure
- Smart contract authoring

---

## 3. Domain Model

**Ubiquitous language:**

| Term | Definition |
|------|-----------|
| Account | An Ethereum address (20 bytes) with an optional private key. Two types: externally owned (EOA) and contract. |
| Transaction | A signed instruction from an EOA causing state change on-chain. |
| Receipt | The post-execution record of a transaction: status, gas used, logs emitted. |
| Log / Event | A structured record emitted by contract execution, indexed by topic hash. |
| ABI | Application Binary Interface — the JSON schema defining a contract's callable functions and events. |
| RPC Endpoint | A URL accepting JSON-RPC 2.0 calls on the Ethereum API surface. |
| Nonce | A monotonically increasing counter per account preventing transaction replay. |
| Gas | Computational cost unit; transactions specify a gas limit and fee. |
| EIP-1559 | The fee market reform specifying `max_fee_per_gas` and `max_priority_fee_per_gas`. |
| Block hash | A 32-byte Keccak-256 hash uniquely identifying a block. |
| Chain ID | An integer identifying the Ethereum network (1 = mainnet, 31337 = Anvil default). |
| Anvil | A local Ethereum devnet (Foundry project, Rust) — zero disk, instant startup. |

**Core entities:**
- `Address` (20-byte value object)
- `H256` / `B256` (32-byte hash value object)
- `U256` (256-bit unsigned integer)
- `Transaction` (unsigned transaction ready for signing)
- `SignedTransaction` (transaction + signature: v, r, s)
- `TransactionReceipt` (post-execution record)
- `Log` (single emitted event with topics and data)
- `Block` (header + transactions + receipts)
- `Contract` (ABI + address pair)

**Value objects:** `Address`, `H256`, `U256`, `ChainId`, `Nonce`, `GasLimit`, `Wei`

**Domain services:**
- `RpcClient` — queries chain state
- `Signer` — signs transactions locally
- `Broadcaster` — submits signed transactions and tracks confirmation
- `EventListener` — subscribes to and emits logs
- `ContractCaller` — encodes ABI calls and decodes responses

---

## 4. Functional Behavior

---

### FR-001: Ethereum Primitives

**Example:** A developer imports `eth_node::primitives` and creates an `Address` from a hex string `"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"`. They encode a `uint256` value and a `bytes32` hash into ABI format for later use in a contract call.

- Preconditions: none (pure computation)
- Trigger: developer calls primitive constructors or encoding functions
- Expected behavior:
  - Parse hex addresses (with or without `0x` prefix, checksummed or lower)
  - Reject malformed addresses with typed error (wrong length, invalid hex)
  - Parse hex hashes (H256/B256)
  - Perform U256 arithmetic: add, sub, mul, div with overflow protection (panic or checked)
  - ABI-encode and decode: `uint256`, `address`, `bytes32`, `bool`, `string`, `bytes`, `tuple`, dynamic arrays
  - RLP-encode and decode: unsigned integers, byte arrays, lists, nested lists
- Postconditions: correct typed values returned; encoding is deterministic and reversible
- Error handling: `PrimitiveError::InvalidHex`, `PrimitiveError::InvalidLength`, `PrimitiveError::AbiDecodeError`

---

### FR-002: JSON-RPC Client

**Example:** A developer creates a client pointed at `http://127.0.0.1:8545` (Anvil default) and calls `get_balance("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", "latest")`. The balance is returned as a `U256` in Wei.

- Preconditions: an Ethereum-compatible RPC endpoint is reachable
- Trigger: developer calls a typed method on `RpcClient`
- Expected behavior — supported calls:
  - `eth_blockNumber` → `u64`
  - `eth_getBalance(address, block_tag)` → `U256` (Wei)
  - `eth_getTransactionCount(address, block_tag)` → `u64` (nonce)
  - `eth_getTransactionReceipt(tx_hash)` → `Option<TransactionReceipt>`
  - `eth_getBlockByNumber(block_tag, full_txs)` → `Option<Block>`
  - `eth_sendRawTransaction(signed_rlp_bytes)` → `H256` (tx hash)
  - `eth_call(call_object, block_tag)` → `Bytes` (raw return data)
  - `eth_getLogs(filter)` → `Vec<Log>`
  - `eth_chainId` → `u64`
  - `eth_gasPrice` → `U256`
  - `eth_estimateGas(call_object)` → `U256`
- Connection types: HTTP (synchronous/async); WebSocket (async, required for subscriptions)
- Preconditions: valid endpoint URL; network reachable
- Postconditions: deserialized typed response or structured error
- Error handling: `RpcError::Transport(msg)`, `RpcError::JsonRpc{code, message}`, `RpcError::Timeout`, `RpcError::Deserialization(msg)`. Transient errors (connection refused, timeout) are distinguished from permanent errors (invalid response, unknown method).

**RPC Client state machine:**
```
Disconnected
  --[connect(url)]--> Connected
Connected
  --[call(method)]--> Connected (on success)
  --[call(method)]--> Error (on transport failure)
  --[disconnect()]--> Disconnected
Error
  --[retry()]--> Connected (if transient)
  --[fail()]--> Disconnected (if permanent)
```

---

### FR-003: Transaction Builder

**Example:** A developer wants to send 0.1 ETH from account A to account B on Anvil (chain ID 31337). They call `TransactionBuilder::transfer(from, to, value).with_chain_id(31337).estimate_gas(&client).await`. The builder returns an unsigned `Transaction` ready for signing.

- Preconditions: chain ID known; sender address known; nonce retrievable from RPC or provided manually
- Trigger: developer calls builder methods
- Expected behavior:
  - Support EIP-1559 transactions (type 2): `chain_id`, `nonce`, `max_fee_per_gas`, `max_priority_fee_per_gas`, `gas_limit`, `to`, `value`, `data`
  - Support legacy transactions (type 0): `chain_id`, `nonce`, `gas_price`, `gas_limit`, `to`, `value`, `data`
  - Auto-fetch nonce from RPC if not manually provided
  - Auto-estimate gas via `eth_estimateGas` if not manually provided
  - Accept `data` field as raw bytes (for contract calls) or empty (for ETH transfers)
  - Enforce: `max_fee_per_gas >= max_priority_fee_per_gas` for EIP-1559
- Postconditions: a valid unsigned `Transaction` struct ready for RLP encoding and signing
- Error handling: `TxError::NonceUnavailable`, `TxError::GasEstimationFailed`, `TxError::InvalidFeeParams`

**Decision table — transaction type selection:**

| User sets gas_price | User sets max_fee | Result |
|--------------------|--------------------|--------|
| Yes | No | Legacy (type 0) |
| No | Yes | EIP-1559 (type 2) |
| No | No | EIP-1559 (type 2), fees fetched from RPC |
| Yes | Yes | Error: `TxError::ConflictingFeeParams` |

---

### FR-004: Transaction Signer

**Example:** A developer holds a private key in the env var `ETH_PRIVATE_KEY`. They call `Signer::from_env()` to load it, then `signer.sign(tx)` to produce a `SignedTransaction` with valid ECDSA signature (v, r, s components) and correct EIP-155 replay protection.

- Preconditions: valid 32-byte private key available; unsigned `Transaction` provided
- Trigger: `Signer::sign(tx)` called
- Expected behavior:
  - Derive `Address` from private key (secp256k1 public key → Keccak-256 → last 20 bytes)
  - RLP-encode unsigned transaction per its type (type 0 or type 2)
  - Keccak-256 hash the encoded transaction
  - ECDSA sign the hash using secp256k1
  - Produce `SignedTransaction` with fields: `raw_bytes` (fully encoded, broadcast-ready), `hash` (H256), `from` (Address)
  - EIP-155 replay protection applied for legacy transactions (type 0)
  - EIP-1559 signing envelope applied for type 2: `rlp([chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, to, value, data, access_list, signature_y_parity, signature_r, signature_s])` (per EIP-1559 spec; EIP-2930 type 1 is out of scope for Phase 1)
- Postconditions: `SignedTransaction.raw_bytes` is valid RLP, accepted by `eth_sendRawTransaction`
- Error handling: `SignerError::InvalidKey`, `SignerError::SigningFailed`
- **Security constraint:** private key must never be logged, printed, or included in any error message or debug output

**Credential model:**
- Load order: `ETH_PRIVATE_KEY` env var first → keystore file (path via `ETH_KEYSTORE_PATH`) → error
- In test contexts: test fixtures use hardcoded Anvil-generated keys (not production secrets)
- Interactive prompts: none — tests must not block on stdin; absence of credential is a typed error
- Keys must not appear in tracing output at any log level

---

### FR-005: Transaction Broadcaster

**Example:** A developer calls `Broadcaster::send(&signed_tx, &client).await`. The broadcaster submits via `eth_sendRawTransaction`, receives the tx hash, then polls `eth_getTransactionReceipt` until either a receipt is returned (confirmed) or the timeout expires (unconfirmed).

- Preconditions: `SignedTransaction` available; `RpcClient` connected
- Trigger: `Broadcaster::send(signed_tx, client)` called
- Expected behavior:
  - Submit via `eth_sendRawTransaction`; receive tx hash `H256`
  - Poll for receipt at configurable interval (default: 500ms) up to configurable timeout (default: 60s)
  - Return `TransactionReceipt` on confirmation
  - Distinguish: status 0x1 (success) vs 0x0 (reverted)
  - On timeout: return `TxError::ConfirmationTimeout(tx_hash)` — tx hash preserved so caller can retry
- Postconditions: receipt returned or timeout error with tx hash
- Error handling: `TxError::Reverted{hash, reason}`, `TxError::ConfirmationTimeout(hash)`, `TxError::SubmitFailed(rpc_error)`

**Transaction lifecycle statechart:**
```
Unsigned
  --[sign(key)]--> Signed
Signed
  --[send(client)]--> Pending (tx hash received)
  --[send fails]--> SubmitError
Pending
  --[receipt received, status=1]--> Confirmed
  --[receipt received, status=0]--> Reverted
  --[timeout]--> TimedOut
SubmitError   (terminal — caller retries from Signed)
Confirmed     (terminal)
Reverted      (terminal — receipt available for inspection)
TimedOut      (terminal — tx hash available for external query)
```

---

### FR-006: Event / Log Listener

**Example:** A developer deploys an ERC-20 contract on Anvil and wants to watch for `Transfer(address,address,uint256)` events. They call `Listener::subscribe(filter).await` where the filter specifies the contract address and the Transfer event topic. Each time a matching log appears in a new block, the listener emits a typed `Log`.

- Preconditions: RPC endpoint supports `eth_getLogs` (HTTP mode) or `eth_subscribe("logs", filter)` (WebSocket mode)
- Trigger: `Listener::subscribe(filter)` called
- Expected behavior:
  - Accept filter: `address` (optional), `topics` (optional, up to 4), `from_block`, `to_block`
  - HTTP mode: poll `eth_getLogs` at configurable interval (default: 1s); emit logs as `Stream<Item=Log>`
  - WebSocket mode: subscribe via `eth_subscribe("logs", filter)`; emit logs as `Stream<Item=Log>`
  - Filter mode selected automatically based on client transport type
  - Gracefully reconnect on WebSocket disconnect (up to 3 attempts, exponential backoff)
- Postconditions: a `Stream` that yields `Log` items matching the filter
- Error handling: `ListenerError::SubscribeFailed`, `ListenerError::ReconnectExhausted`, `ListenerError::FilterInvalid`

**Event listener statechart:**
```
Idle
  --[subscribe(filter)]--> Connecting
Connecting
  --[connected]--> Listening
  --[connection failed]--> Error
Listening
  --[log received]--> Listening (emit log, stay listening)
  --[disconnect/error]--> Reconnecting
  --[unsubscribe()]--> Idle
Reconnecting
  --[reconnect success]--> Listening
  --[max retries exceeded]--> Error
Error (terminal — emit ListenerError, stream ends)
```

---

### FR-007: ABI-Driven Contract Caller

**Example:** A developer has the ABI JSON for an ERC-20 contract and its deployed address on Anvil. They call `ContractCaller::new(address, abi)`, then `caller.call("balanceOf", &[Token::Address(user)])`, which ABI-encodes the call data, sends `eth_call`, and decodes the raw `bytes` response back into `Token::Uint(U256)`.

- Preconditions: valid ABI JSON; deployed contract address; `RpcClient` connected
- Trigger: `ContractCaller::call(fn_name, args)` (read) or `ContractCaller::send(fn_name, args, signer)` (write) called
- Expected behavior:
  - Parse ABI JSON into callable function and event descriptors
  - For read calls (`eth_call`):
    - ABI-encode function selector + arguments
    - Execute `eth_call` against the contract at `latest` block (configurable)
    - ABI-decode response bytes into typed `Vec<Token>`
    - Return typed tokens
  - For write calls (state-changing):
    - ABI-encode call data
    - Build transaction with encoded data as `tx.data`
    - Delegate to `Signer` and `Broadcaster` (FR-004, FR-005)
  - Support function overloading resolution by argument types
- Postconditions: decoded return tokens or typed error
- Error handling: `ContractError::AbiNotFound(fn_name)`, `ContractError::AbiDecodeError`, `ContractError::CallReverted(reason)`, `ContractError::InvalidAbiJson`

---

## 5. Non-Functional Requirements Mapping

| NFR-ID | Dimension | Metric | Target | Validation method |
|--------|-----------|--------|--------|-------------------|
| NFR-001 | Reliability & Resilience | No panics on bad RPC responses | Zero panics in test suite on malformed/empty RPC responses | Fuzz test RPC response parser with random payloads |
| NFR-002 | Security | No secrets in logs or errors | Private key string never appears in any log output at any level | Static grep in CI; tracing integration test with key in scope |
| NFR-003 | Observability | Structured log output per operation | Each FR-002 through FR-007 operation emits at least one `tracing` span | Integration tests assert on span presence |
| NFR-004 | Maintainability | Library/CLI separation | Zero domain logic in CLI layer found at code review | Architecture review at Stage 5 |
| NFR-005 | Reliability & Resilience | Graceful credential absence | Missing `ETH_PRIVATE_KEY` returns typed error, not panic, and does not hang on stdin | Unit test: call `Signer::from_env()` with env var unset |
| NFR-006 | Testing | Integration test coverage | At least one integration test per FR (FR-001 through FR-007) passing against live Anvil instance | CI test run with Anvil started as test fixture |

---

## 5.1 Observability and CLI UX

- Events logged to stderr during a run: client connect, each RPC method call (method name, block tag), tx build (type, gas, nonce), tx sign (from address only — never the key), tx submit (hash), confirmation poll (attempt count), event received (address, topic[0]), contract call (fn name, args summary)
- Log format: structured JSON via `tracing-subscriber` with `fmt::json()` layer
- Suppression mechanism: `--quiet` flag sets log level to `error` only; `--log-level <level>` accepts `trace|debug|info|warn|error`
- Log level default: `info`
- Private key: must not appear at any log level

---

## 6. Data and Interface Contracts

### 6.1 Exact Data Formats

- RPC transport: JSON-RPC 2.0 over HTTP or WebSocket. Request: `{"jsonrpc":"2.0","method":"...","params":[...],"id":1}`. Response: `{"jsonrpc":"2.0","result":...,"id":1}` or `{"jsonrpc":"2.0","error":{"code":...,"message":"..."},"id":1}`.
- Hex encoding: all byte values in JSON-RPC are `0x`-prefixed lowercase hex strings.
- Log artifacts from CLI session capture: stored as `.log` files in `output/sessions/YYYY-MM-DD_HH-MM-SS/` with `screen.log` (terminal output) and `state.json` (application-state snapshot).

### 6.2 API Signature and Side-Effect Contract

| Operation | Inputs | Output | Side effects | Pure? |
|-----------|--------|--------|-------------|-------|
| `primitives::Address::from_str(s)` | hex string | `Result<Address>` | none | Yes |
| `primitives::abi_encode(tokens)` | `&[Token]` | `Bytes` | none | Yes |
| `rpc::Client::new(url)` | URL string | `Result<Client>` | opens HTTP/WS connection | No |
| `rpc::Client::get_balance(addr, tag)` | `Address`, `BlockTag` | `Result<U256>` | outbound RPC call | No |
| `rpc::Client::get_nonce(addr)` | `Address` | `Result<u64>` | outbound RPC call | No |
| `signer::Signer::from_env()` | — | `Result<Signer>` | reads env var | No |
| `signer::Signer::sign(tx)` | `Transaction` | `Result<SignedTransaction>` | none | Yes (deterministic — RFC 6979) |
| `tx::Broadcaster::send(signed, client)` | `SignedTransaction`, `&Client` | `Result<TransactionReceipt>` | outbound RPC call, polls | No |
| `events::Listener::subscribe(filter)` | `LogFilter` | `Stream<Item=Result<Log>>` | outbound RPC subscription | No |
| `contract::ContractCaller::call(fn, args)` | fn name, `&[Token]` | `Result<Vec<Token>>` | outbound RPC call | No |
| `contract::ContractCaller::send(fn, args, signer)` | fn name, `&[Token]`, `&Signer` | `Result<TransactionReceipt>` | sign + broadcast | No |

### 6.3 External Integration Contracts

**Integration point: Anvil / Ethereum JSON-RPC endpoint**

| Field | Value |
|-------|-------|
| Protocol | HTTP (default port 8545) / WebSocket (default port 8545, `ws://`) |
| Authentication | None for Anvil; Bearer token via `Authorization` header for hosted providers (optional, config-driven) |
| `eth_getBalance` | Params: `[address_hex, block_tag]`. Response: `{result: hex_wei_string}` |
| `eth_getTransactionCount` | Params: `[address_hex, block_tag]`. Response: `{result: hex_nonce}` |
| `eth_sendRawTransaction` | Params: `[signed_rlp_hex]`. Response: `{result: tx_hash_hex}` or `{error: {code, message}}` |
| `eth_getTransactionReceipt` | Params: `[tx_hash_hex]`. Response: `{result: null}` (pending) or receipt object |
| `eth_call` | Params: `[{to, data}, block_tag]`. Response: `{result: hex_return_data}` |
| `eth_getLogs` | Params: `[{address?, topics?, fromBlock, toBlock}]`. Response: `{result: [log...]}` |
| `eth_subscribe` | WebSocket only. Params: `["logs", filter]`. Response: subscription ID; events pushed as notifications |
| `eth_chainId` | Params: `[]`. Response: `{result: hex_chain_id}` (Anvil: `0x7a69` = 31337) |
| `eth_estimateGas` | Params: `[call_object]`. Response: `{result: hex_gas}` |
| Spike required? | No — Anvil is stable, spec is public (Ethereum JSON-RPC spec + EIP-1474) |

### 6.4 Credential Model

- Credential acquisition order: `ETH_PRIVATE_KEY` env var → `ETH_KEYSTORE_PATH` env var pointing to keystore JSON + `ETH_KEYSTORE_PASSWORD` → `SignerError::CredentialNotFound`
- Interactive prompt suppression: no interactive prompt exists; absence of credentials is a typed error
- Acceptance criterion: `cargo test` must complete without blocking on stdin; all signer tests use Anvil-generated test keys set in env
- Secret storage: env vars only (local dev); test fixtures use Anvil pre-funded accounts (keys are public and safe for test use)
- Key rotation: not applicable in Phase 1 (local devnet only)

---

## 7. Architecture and Design Decisions

### Decision A-001: Cargo Workspace Structure

- Decision: Cargo workspace with two members: `eth_node` (library crate) and `eth_node_cli` (binary crate)
- Rationale: enforces library/CLI separation (Q3-ARCH-01); library is testable in isolation; CLI tests can be integration-style without tangling with library internals
- Alternatives considered: single crate with `lib.rs` + `main.rs` — rejected because it allows business logic to leak into `main.rs` without architectural enforcement
- Consequences: two `Cargo.toml` files; shared deps in workspace root `Cargo.toml`

### Decision A-002: Async Runtime

- Decision: Tokio as the async runtime
- Rationale: required by `alloy` and `revm` crates; de facto standard in the Rust ecosystem
- Alternatives considered: `async-std` — rejected due to weaker ecosystem alignment for Ethereum crates
- Consequences: all async functions use `#[tokio::main]` in binary; `#[tokio::test]` in tests

### Decision A-003: Primary Ethereum Crate

- Decision: `alloy` ecosystem (`alloy-core`, `alloy-provider`, `alloy-signer`, `alloy-sol-types`, `alloy-contract`, `alloy-transport`, `alloy-consensus`)
- Rationale: actively maintained by Paradigm; modern replacement for `ethers-rs`; designed for Reth compatibility (Phase 2 alignment); 100% Rust
- Alternatives considered: `ethers-rs` — rejected (maintenance-mode, recommends migration to alloy)
- Consequences: alloy API drives all type decisions in this spec

### Decision A-004: Logging

- Decision: `tracing` + `tracing-subscriber` for structured logging
- Rationale: standard for async Rust; integrates with Tokio instrumentation; JSON output supported natively
- Consequences: all library functions must instrument with `#[tracing::instrument]` or manual spans

### Decision A-005: Test Infrastructure

- Decision: Anvil started as a test fixture (via `anvil` binary or `anvil` as library via `foundry-anvil` crate) in integration tests
- Rationale: zero external dependency for integration tests; deterministic chain state; instant startup
- Consequences: Foundry must be installed on dev machine and CI; or `foundry-anvil` crate used as embedded fixture

---

### 7.1 Layered Architecture (Q3-ARCH-01)

**Module interfaces — `eth_node` library crate:**

| Module | Public trait / type | Responsibility boundary |
|--------|--------------------|-----------------------|
| `eth_node::primitives` | `Address`, `H256`, `U256`, `Token`, `abi_encode`, `abi_decode`, `rlp_encode`, `rlp_decode` | Pure types and encoding — no I/O |
| `eth_node::rpc` | `RpcClient`, `BlockTag`, `LogFilter` | All outbound JSON-RPC — no business logic |
| `eth_node::signer` | `Signer`, `SignedTransaction` | Key management and transaction signing — no I/O beyond env read |
| `eth_node::tx` | `TransactionBuilder`, `Broadcaster`, `TransactionReceipt` | Transaction construction and lifecycle — delegates to rpc and signer |
| `eth_node::events` | `Listener`, `Log`, `LogFilter` | Log subscription and streaming — delegates to rpc |
| `eth_node::contract` | `ContractCaller`, `Abi` | ABI-encoding and contract interaction — delegates to rpc, signer, tx |

**API surface:**
- Internal (same crate): all public items above are callable across modules
- External library consumer: all items in `eth_node::*` via `use eth_node::...`
- CLI layer: imports `eth_node::*` for all operations; adds only argument parsing and output formatting

**CLI-to-API mapping:**

| CLI command | API call |
|-------------|----------|
| `eth_node_cli balance <address>` | `rpc::Client::get_balance(address, BlockTag::Latest)` |
| `eth_node_cli send <to> <value_eth>` | `tx::TransactionBuilder::transfer` → `signer::Signer::sign` → `tx::Broadcaster::send` |
| `eth_node_cli watch <contract> <event>` | `events::Listener::subscribe(filter)` |
| `eth_node_cli call <contract> <fn> [args...]` | `contract::ContractCaller::call(fn, args)` |
| `eth_node_cli tx-status <hash>` | `rpc::Client::get_transaction_receipt(hash)` |

**Business logic placement constraint:** No business logic (fee calculation, nonce management, ABI encoding, signature computation) may reside in `eth_node_cli`. The CLI may only: parse arguments, call `eth_node::*` functions, and format/print results. Any business logic found in the CLI layer during Stage 5 review is an architecture violation and a build blocker.

**GUI-to-API mapping:** Not applicable — no GUI in Phase 1.

---

## 8. Quality Dimension Targets (Q2 Pack)

| Dimension | Target | N/A rationale |
|-----------|--------|---------------|
| Performance & Efficiency | RPC round-trip < 100ms on localhost (Anvil); startup time < 500ms for CLI | — |
| Reliability & Resilience | Zero panics on malformed RPC responses; `Broadcaster` timeout does not lose tx hash; `Listener` reconnects up to 3 times before error | — |
| Maintainability Over Time | Phase 2 modules (#7–#12) can be added as new modules without changing Phase 1 API signatures; all public types are `pub` and documented | — |
| Behavioral Specification Rigor | All FR operations have pre/post conditions in this spec; statecharts defined for RPC client, tx lifecycle, event listener | — |
| Developer Experience | `cargo test` starts Anvil fixture automatically; `README.md` covers local setup in ≤ 10 steps | — |
| Security | Private key never in logs; credentials via env vars only; no `unwrap()` on external input paths | — |
| Scalability | Not a target in Phase 1 — local devnet only | N/A: local devnet, no concurrent users |
| Compliance / Regulatory | Not applicable — Phase 1 is a learning project with no user data | N/A: no regulated scope |

---

## 9. Test Strategy (TDD-aligned)

**Unit test approach:**
- Each module in `eth_node` has a `#[cfg(test)]` module
- Pure functions (primitives, ABI encoding, RLP) are fully unit-tested with known vectors
- Signer tested with known private key → known address → known signature vectors
- TransactionBuilder tested with fixed gas/nonce inputs → deterministic RLP output
- All error paths tested: invalid hex, missing credential, conflicting fee params, ABI decode failure

**Integration test approach:**
- Integration tests live in `eth_node/tests/` and `eth_node_cli/tests/`
- Anvil is started as a subprocess fixture before test suite; torn down after
- One integration test per FR (FR-001 through FR-007) against live Anvil
- Broadcaster test: deploy test contract → send ETH → assert receipt status = 1
- Event listener test: trigger event on Anvil → assert log received by listener within 3 poll cycles
- Contract caller test: call `balanceOf` on deployed ERC-20 → assert matches expected value

**Acceptance test approach:**
- AC-001 through AC-006 are validated by the integration test suite
- Each AC maps to at least one named integration test

**Exploratory test focus areas:**
- Malformed RPC responses (fuzz the JSON deserializer)
- WebSocket disconnection mid-subscription
- Nonce race conditions (two send calls in rapid succession)
- Contract call with wrong argument types

**Interactive CLI diagnostics:**
- Screen-state capture: `script` (Linux) / PowerShell `Start-Transcript` (Windows) records full terminal session
- Application-state capture: CLI `--dump-state <path>` flag writes a JSON snapshot of the last operation result
- Manual session execution: developer runs `eth_node_cli` through the capture wrapper; session artifacts stored and linked to bug reports
- Artifact storage path: `output/sessions/YYYY-MM-DD_HH-MM-SS/screen.log` + `output/sessions/YYYY-MM-DD_HH-MM-SS/state.json`
- Naming convention: timestamp-prefixed directory per session

---

### 9.1 Downstream Implementation Chronicle Expectations

- Chronicle entry required per module: `primitives`, `rpc`, `signer`, `tx`, `events`, `contract`, `cli`
- Each entry must record: key design decisions, alternatives rejected, tradeoffs made, test vectors chosen and why
- Areas where alternatives are expected: async stream implementation for `events` (polling vs subscription mode); gas estimation strategy in `TransactionBuilder`
- Reconstruction-critical details: ABI encoding format for tuple types; EIP-1559 signing envelope format; Anvil test fixture startup sequence

### 9.2 Deterministic Test Vector Appendix

- Known private key for tests: Anvil account 0 (`0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`) — address `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`, balance 10,000 ETH on fresh Anvil
- Known ABI encoding vector: `uint256(1)` → `0x0000000000000000000000000000000000000000000000000000000000000001`
- Known RLP vector: empty string `""` → `0x80`; integer 0 → `0x80`; integer 1 → `0x01`
- Replay procedure: run `cargo test` with `ANVIL_AUTO_START=1`; all vectors are compiled into test constants

---

## 10. Traceability Matrix

| ID | Spec section | Planned tests | Chronicle entry |
|----|-------------|---------------|----------------|
| FR-001 | §4 FR-001 | `primitives::tests::*`, `tests/integration_primitives.rs` | `chronicle/primitives.md` |
| FR-002 | §4 FR-002 | `rpc::tests::*`, `tests/integration_rpc.rs` | `chronicle/rpc.md` |
| FR-003 | §4 FR-003 | `tx::tests::builder_*`, `tests/integration_tx_build.rs` | `chronicle/tx.md` |
| FR-004 | §4 FR-004 | `signer::tests::*`, `tests/integration_signer.rs` | `chronicle/signer.md` |
| FR-005 | §4 FR-005 | `tx::tests::broadcaster_*`, `tests/integration_broadcast.rs` | `chronicle/tx.md` |
| FR-006 | §4 FR-006 | `events::tests::*`, `tests/integration_events.rs` | `chronicle/events.md` |
| FR-007 | §4 FR-007 | `contract::tests::*`, `tests/integration_contract.rs` | `chronicle/contract.md` |
| AC-001 | §2 AC-001 | `tests/integration_rpc.rs::test_anvil_responds` | `chronicle/rpc.md` |
| AC-002 | §2 AC-002 | `tests/integration_rpc.rs::test_balance_query` | `chronicle/rpc.md` |
| AC-003 | §2 AC-003 | `tests/integration_broadcast.rs::test_send_confirmed` | `chronicle/tx.md` |
| AC-004 | §2 AC-004 | `tests/integration_events.rs::test_event_received` | `chronicle/events.md` |
| AC-005 | §2 AC-005 | `tests/integration_contract.rs::test_contract_call` | `chronicle/contract.md` |
| AC-006 | §2 AC-006 | All integration tests passing in CI | All chronicle entries |
| NFR-001 | §5 NFR-001 | `tests/fuzz_rpc_response.rs` | `chronicle/rpc.md` |
| NFR-002 | §5 NFR-002 | `signer::tests::test_key_not_in_logs` | `chronicle/signer.md` |
| Q3-ARCH-01 | §7.1 | Architecture review at Stage 5 | `chronicle/architecture.md` |

---

## 11. Stage Approval

- Approved by: Lefty
- Approval date: 2026-03-26
- Notes: Oracle objection FR-004 resolved. Developer-in-test and Exploratory Tester agents added. Poke & probe Q&A window completed.
