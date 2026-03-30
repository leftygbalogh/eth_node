# Library API Usage Guide

This guide demonstrates how to use the `eth_node` library crate for transaction simulation, contract calls, and event decoding.

## Table of Contents

1. [Executor Module](#executor-module)
   - [Transaction Simulation](#transaction-simulation)
   - [Contract Calls](#contract-calls)
   - [Anvil Comparison](#anvil-comparison)
2. [Decoder Module](#decoder-module)
   - [Standard Event Decoding](#standard-event-decoding)
   - [Lossless Decoding](#lossless-decoding)
   - [ApprovalForAll Ambiguity](#approvalforall-ambiguity)
3. [Integration Examples](#integration-examples)
   - [Simulate and Decode](#simulate-and-decode)
   - [Compare and Validate](#compare-and-validate)
4. [Rust Patterns & Best Practices](#rust-patterns--best-practices)
   - [Error Handling](#error-handling)
   - [Ownership & Borrowing](#ownership--borrowing)
   - [Module Design](#module-design)

---

## Executor Module

The `executor` module provides local EVM transaction simulation using `revm`.

### Transaction Simulation

Simulate transaction execution without broadcasting to a live network:

```rust
use eth_node::executor::{simulate_tx, SimulationContext};
use alloy_rpc_types::TransactionRequest;
use alloy_primitives::{Address, U256};

// Build transaction
let tx = TransactionRequest {
    from: Some(Address::from([0x11; 20])),
    to: Some(Address::from([0x22; 20]).into()),
    value: Some(U256::from(1_000_000_000u64)), // 1 gwei
    gas: Some(21_000),
    gas_price: Some(U256::from(10)),
    ..Default::default()
};

// Define simulation context
let context = SimulationContext {
    block_number: 1,
    timestamp: 1710000000,
    base_fee_per_gas: Some(10),
    gas_limit: 30_000_000,
};

// Execute simulation
let result = simulate_tx(&tx, &context)?;

// Inspect results
println!("Success: {}", result.success);
println!("Gas used: {}", result.gas_used);
println!("Return data: {:?}", result.return_data);
println!("Logs: {} events emitted", result.logs.len());
```

**Key Points:**
- Simulation always completes even if the transaction reverts (`result.success == false`)
- No state is persisted (ephemeral in-memory execution)
- Use this for pre-flight validation before broadcasting

### Contract Calls

Execute static contract calls (similar to `eth_call`):

```rust
use eth_node::executor::simulate_contract_call;
use alloy_primitives::{Address, Bytes};
use alloy_sol_types::{sol, SolCall};

// Define contract interface
sol! {
    function balanceOf(address owner) external view returns (uint256);
}

// Encode call
let call = balanceOf {
    owner: Address::from([0x11; 20]),
};
let calldata = Bytes::from(call.abi_encode());

// Contract address
let contract_address = Address::from([0xAA; 20]);

// Execute static call
let result = simulate_contract_call(contract_address, calldata, &context)?;

// Decode return value
let balance = U256::abi_decode(&result, true)?;
println!("Balance: {}", balance);
```

**Key Points:**
- Static calls have zero value and do not modify state
- Requires at least 4 bytes of calldata (function selector)
- Phase 2 note: calls to empty contracts return empty data (contract code loading deferred to Phase 3)

### Anvil Comparison

Validate simulation accuracy by comparing to Anvil execution:

```rust
use eth_node::executor::{compare_to_anvil, SimulationContext};
use alloy_rpc_types::TransactionRequest;

// Build transaction (must be executable on Anvil)
let tx = TransactionRequest {
    from: Some(anvil_funded_account),
    to: Some(target_address.into()),
    value: Some(U256::from(1000)),
    gas: Some(21_000),
    ..Default::default()
};

let context = SimulationContext {
    block_number: 1,
    timestamp: 1710000000,
    base_fee_per_gas: Some(10),
    gas_limit: 30_000_000,
};

// Compare local simulation to Anvil
let report = compare_to_anvil(&tx, "http://127.0.0.1:8545", &context).await?;

// Inspect differences
println!("Gas used (local): {}", report.gas_used_local);
println!("Gas used (Anvil): {}", report.gas_used_anvil);
println!("Gas delta: {}", report.gas_delta);
println!("Return data match: {}", report.return_data_match);

// Enforce 5% gas tolerance
let threshold = (report.gas_used_anvil as f64 * 0.05).ceil() as u64;
if (report.gas_delta).unsigned_abs() > threshold {
    eprintln!("Gas mismatch exceeds 5% threshold!");
    for diff in &report.differences {
        eprintln!("  - {}", diff);
    }
}
```

**Key Points:**
- Always returns `Ok(ComparisonReport)` unless RPC fails
- Does NOT fail on mismatches—reports them in `differences` field
- Caller interprets thresholds (5% gas tolerance recommended)
- Uses `eth_estimateGas` and `eth_call` for Anvil reference (no transaction signing required)

---

## Decoder Module

The `quality::decode` module provides event decoding for standard NFT events (ERC-721 and ERC-1155).

### Standard Event Decoding

Decode NFT events from transaction logs:

```rust
use eth_node::quality::{decode_standard_nft_event, DecodedEvent};
use alloy_rpc_types::Log;

// Log from a transaction receipt
let log: Log = ...; // obtained from receipt.logs

match decode_standard_nft_event(&log)? {
    DecodedEvent::Erc721Transfer(evt) => {
        println!("ERC-721 Transfer: token {} from {} to {}", 
                 evt.token_id, evt.from, evt.to);
    }
    DecodedEvent::Erc721Approval(evt) => {
        println!("ERC-721 Approval: token {} approved to {}", 
                 evt.token_id, evt.approved);
    }
    DecodedEvent::Erc721ApprovalForAll(evt) => {
        println!("ERC-721 ApprovalForAll: {} operator {} approved={}", 
                 evt.owner, evt.operator, evt.approved);
    }
    DecodedEvent::Erc1155TransferSingle(evt) => {
        println!("ERC-1155 TransferSingle: id={} value={} from {} to {}", 
                 evt.id, evt.value, evt.from, evt.to);
    }
    DecodedEvent::Erc1155TransferBatch(evt) => {
        println!("ERC-1155 TransferBatch: {} items from {} to {}", 
                 evt.ids.len(), evt.from, evt.to);
    }
    DecodedEvent::Erc1155ApprovalForAll(evt) => {
        println!("ERC-1155 ApprovalForAll: {} operator {} approved={}", 
                 evt.account, evt.operator, evt.approved);
    }
    DecodedEvent::Erc1155Uri(evt) => {
        println!("ERC-1155 URI: id={} uri={}", evt.id, evt.value);
    }
}
```

**Supported Events:**
- ERC-721: `Transfer`, `Approval`, `ApprovalForAll`
- ERC-1155: `TransferSingle`, `TransferBatch`, `ApprovalForAll`, `URI`

### Lossless Decoding

Preserve ApprovalForAll ambiguity when the standard is unknown:

```rust
use eth_node::quality::{decode_nft_event_lossless, LosslessDecodedEvent, ApprovalForAllStandard};
use alloy_rpc_types::Log;

let log: Log = ...; // ApprovalForAll event from unknown contract

// Option 1: Explicitly classify as ERC-721
let decoded = decode_nft_event_lossless(&log, Some(ApprovalForAllStandard::Erc721))?;

// Option 2: Explicitly classify as ERC-1155
let decoded = decode_nft_event_lossless(&log, Some(ApprovalForAllStandard::Erc1155))?;

// Option 3: Preserve ambiguity for downstream resolution
let decoded = decode_nft_event_lossless(&log, None)?;
match decoded {
    LosslessDecodedEvent::Decoded(event) => {
        println!("Unambiguous event: {:?}", event);
    }
    LosslessDecodedEvent::AmbiguousApprovalForAll(evt) => {
        println!("Ambiguous ApprovalForAll event:");
        println!("  Subject: {}", evt.subject);
        println!("  Operator: {}", evt.operator);
        println!("  Approved: {}", evt.approved);
        
        // Downstream: query contract via ERC-165 or heuristic
        let standard = detect_nft_standard(&evt.subject).await?;
        // Re-decode with explicit standard...
    }
}
```

**Key Points:**
- Use when working with multi-standard indexers or unknown contracts
- `None` preserves ambiguity without data loss
- Downstream can resolve via ERC-165 supportsInterface queries

### ApprovalForAll Ambiguity

The `ApprovalForAll(address,address,bool)` event signature is **identical** in ERC-721 and ERC-1155:

```solidity
// ERC-721
event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

// ERC-1155
event ApprovalForAll(address indexed account, address indexed operator, bool approved);
```

**Only the semantic meaning of topic[1] differs** (`owner` vs `account`):
- Use `decode_standard_nft_event` if you assume ERC-721 (historical default)
- Use `decode_nft_event_lossless` with explicit standard if you know the contract type
- Use `decode_nft_event_lossless` with `None` if the standard is unknown and you need to resolve it downstream

---

## Integration Examples

### Simulate and Decode

Deploy a contract, simulate an NFT mint, and decode the emitted event:

```rust
use eth_node::executor::{simulate_tx, SimulationContext};
use eth_node::quality::{decode_standard_nft_event, DecodedEvent};
use alloy_rpc_types::TransactionRequest;
use alloy_primitives::{Address, Bytes, U256};

// 1. Build mint transaction calldata
let mint_call = mintTo {
    to: recipient_address,
    tokenId: U256::from(1),
};
let calldata = Bytes::from(mint_call.abi_encode());

// 2. Simulate transaction
let tx = TransactionRequest {
    from: Some(minter_address),
    to: Some(nft_contract_address.into()),
    data: Some(calldata),
    gas: Some(100_000),
    ..Default::default()
};

let context = SimulationContext {
    block_number: 1,
    timestamp: 1710000000,
    base_fee_per_gas: Some(10),
    gas_limit: 30_000_000,
};

let result = simulate_tx(&tx, &context)?;

// 3. Decode emitted events
for log in &result.logs {
    if let Ok(event) = decode_standard_nft_event(log) {
        match event {
            DecodedEvent::Erc721Transfer(evt) => {
                println!("✅ Minted token {} to {}", evt.token_id, evt.to);
            }
            _ => {}
        }
    }
}
```

### Compare and Validate

Deploy to Anvil, simulate locally, compare results, and validate events:

```rust
use eth_node::executor::{simulate_tx, compare_to_anvil, SimulationContext};
use eth_node::quality::{decode_standard_nft_event, DecodedEvent};
use eth_node::rpc::RpcClient;

// 1. Deploy contract to Anvil
let anvil = AnvilInstance::spawn();
let client = RpcClient::new(&anvil.endpoint)?;
let contract_address = deploy_erc721_to_anvil(&client).await?;

// 2. Build transfer transaction
let tx = TransactionRequest {
    from: Some(owner_address),
    to: Some(contract_address.into()),
    data: Some(transfer_calldata),
    gas: Some(100_000),
    ..Default::default()
};

let context = SimulationContext {
    block_number: 1,
    timestamp: 1710000000,
    base_fee_per_gas: Some(10),
    gas_limit: 30_000_000,
};

// 3. Compare local simulation to Anvil execution
let report = compare_to_anvil(&tx, &anvil.endpoint, &context).await?;

println!("Gas comparison:");
println!("  Local: {}", report.gas_used_local);
println!("  Anvil: {}", report.gas_used_anvil);
println!("  Delta: {}", report.gas_delta);

// 4. Decode events from local simulation
let local_result = simulate_tx(&tx, &context)?;
for log in &local_result.logs {
    if let Ok(DecodedEvent::Erc721Transfer(evt)) = decode_standard_nft_event(log) {
        println!("✅ Local simulation emitted Transfer: token {} from {} to {}", 
                 evt.token_id, evt.from, evt.to);
    }
}

// 5. Validate consistency
if report.logs_match {
    println!("✅ Log events match between local and Anvil");
} else {
    eprintln!("⚠️ Log mismatch detected!");
}
```

---

## Rust Patterns & Best Practices

This section explains Rust-specific patterns used throughout the library to help you write idiomatic code when using `eth_node`.

### Error Handling

The library uses `Result<T, E>` for all fallible operations. Understanding when and how to propagate errors is critical for robust code.

#### When to Use `?` vs `match`

Use the `?` operator for straightforward error propagation when you don't need to inspect or transform the error:

```rust
use eth_node::executor::simulate_tx;

fn run_simulation() -> Result<(), Box<dyn std::error::Error>> {
    let tx = build_transaction();
    let context = SimulationContext::default();
    
    // ✓ Use ? when propagating errors up the call stack
    let result = simulate_tx(&tx, &context)?;
    
    println!("Gas used: {}", result.gas_used);
    Ok(())
}
```

Use `match` when you need to handle specific error variants differently:

```rust
use eth_node::quality::{decode_standard_nft_event, DecodeError};

match decode_standard_nft_event(&log) {
    Ok(event) => process_event(event),
    Err(DecodeError::UnsupportedEvent) => {
        // Log and skip unsupported events gracefully
        eprintln!("Skipping unsupported event");
    }
    Err(DecodeError::InvalidData(msg)) => {
        // Critical error - abort processing
        panic!("Invalid event data: {}", msg);
    }
    Err(e) => return Err(e.into()), // Propagate other errors
}
```

#### Error Type Design with `thiserror`

The library modules define their own error types using `thiserror`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("simulation failed: {0}")]
    SimulationFailed(String),
    
    #[error("RPC communication error: {0}")]
    RpcError(#[from] alloy_transport::TransportError),
    
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),
}
```

**Key patterns:**
- `#[error("...")]` defines display format (shown in error messages)
- `#[from]` enables automatic conversion via `?` operator (e.g., `TransportError` → `ExecutorError::RpcError`)
- Use descriptive error messages that aid debugging

#### Explicit Error Propagation Discipline

When calling library functions, always handle or propagate errors—never unwrap in production code:

```rust
// ✗ BAD: Panic on error (only acceptable in examples/tests)
let result = simulate_tx(&tx, &context).unwrap();

// ✓ GOOD: Propagate error to caller
let result = simulate_tx(&tx, &context)?;

// ✓ ALSO GOOD: Provide context when propagating
let result = simulate_tx(&tx, &context)
    .map_err(|e| format!("Failed to simulate transaction: {}", e))?;
```

### Ownership & Borrowing

Rust's ownership system prevents data races and use-after-free bugs. Understanding when to use references vs owned values is essential.

#### When to Use `&T` vs `&mut T` vs Owned Values

Use **`&T` (shared reference)** when you only need to read data:

```rust
use eth_node::executor::{simulate_tx, SimulationContext};

// SimulationContext is borrowed immutably (can't be modified)
fn run_simulation(tx: &TransactionRequest, context: &SimulationContext) -> Result<SimulationResult, ExecutorError> {
    simulate_tx(tx, context) // Both parameters borrowed, original values unchanged
}
```

Use **`&mut T` (mutable reference)** when you need to modify data in-place:

```rust
fn update_context(context: &mut SimulationContext, new_timestamp: u64) {
    context.timestamp = new_timestamp; // Modify in-place via mutable borrow
}
```

Use **owned values (`T`)** when transferring ownership or storing long-term:

```rust
use std::collections::HashMap;

struct SimulationCache {
    results: HashMap<String, SimulationResult>, // Owned values stored in cache
}

impl SimulationCache {
    fn insert(&mut self, key: String, result: SimulationResult) {
        // Takes ownership of both key and result
        self.results.insert(key, result);
    }
}
```

#### Lifetime Requirements in Function Signatures

Most library functions accept references with implicit lifetimes:

```rust
// Implicit lifetime: returned Log references live as long as the input result
fn extract_logs(result: &SimulationResult) -> &[Log] {
    &result.logs
}
```

When returning references derived from multiple inputs, lifetimes must be explicit:

```rust
// Explicit lifetime: returned reference tied to 'a (the input parameter)
fn find_event<'a>(logs: &'a [Log], topic: &B256) -> Option<&'a Log> {
    logs.iter().find(|log| log.topics().first() == Some(topic))
}
```

#### Clone vs Move Tradeoffs

Moving transfers ownership (zero cost), cloning duplicates data (may be expensive):

```rust
use alloy_rpc_types::TransactionRequest;

// Move (no clone): tx ownership transferred to simulate_tx
let tx = TransactionRequest::default();
let result = simulate_tx(&tx, &context)?; // Borrow (no move)

// Clone when you need to reuse the original:
let tx = TransactionRequest::default();
let result1 = simulate_tx(&tx, &context)?;    // First simulation
let result2 = simulate_tx(&tx, &context)?;    // Reuse tx (borrowed, no clone needed)

// Clone when crossing thread boundaries:
let tx = TransactionRequest::default();
let tx_clone = tx.clone(); // Explicit clone for thread
std::thread::spawn(move || {
    simulate_tx(&tx_clone, &context) // tx_clone moved into thread
});
// Original tx still usable here
```

**Guidelines:**
- Prefer borrowing (`&T`) when possible—zero-cost and most flexible
- Clone only when necessary (thread boundaries, persistent storage, or modification without affecting original)
- Types like `Address`, `U256` are `Copy` (implicit clone)—cheap to pass by value

### Module Design

The library follows modular design principles to maintain clear boundaries and composability.

#### Trait Boundaries

The executor module works with any type implementing `revm::StateProvider`:

```rust
use revm::db::{StateProvider, InMemoryDB};
use eth_node::executor::simulate_tx;

// Works with in-memory database
let db = InMemoryDB::default();
let result = simulate_tx(&tx, &context)?;

// Also works with custom state providers (e.g., RPC-backed state)
struct RpcStateProvider { /* ... */ }
impl StateProvider for RpcStateProvider { /* ... */ }

let rpc_db = RpcStateProvider::new("http://localhost:8545");
// Same function, different state provider
```

**Design principle:** Accept trait objects or generic bounds (`impl Trait`, `T: Trait`) rather than concrete types to maximize composability.

#### Public API Decisions

Each module exposes a minimal public API:

```rust
// Public API (module root):
pub use executor::{simulate_tx, simulate_contract_call, compare_to_anvil};
pub use quality::{decode_standard_nft_event, decode_nft_event_lossless};

// Internal functions (not re-exported):
mod executor {
    pub fn simulate_tx(...) -> Result<SimulationResult, ExecutorError> { /* ... */ }
    
    // Internal helper (not in public API)
    fn build_evm_env(...) -> Env { /* ... */ }
}
```

**Guidelines:**
- Minimize public surface area (easier to maintain backward compatibility)
- Re-export commonly used types from module roots
- Keep implementation details private (users depend on behavior, not internals)

#### Internal vs Public Functions

Use `pub(crate)` for functions needed across modules but not exposed to library users:

```rust
// Public API (exposed to library users)
pub fn simulate_tx(...) -> Result<SimulationResult, ExecutorError> { /* ... */ }

// Internal API (visible within crate only)
pub(crate) fn validate_transaction(tx: &TransactionRequest) -> Result<(), ExecutorError> {
    // Shared validation logic used by multiple public functions
    // Not exposed to end users
}

// Private (visible within module only)
fn build_evm_env(...) -> Env {
    // Implementation detail, not shared
}
```

#### Extensibility Points

The library provides hooks for customization via trait implementations:

```rust
// Example: Custom event decoder
pub trait EventDecoder {
    fn decode(&self, log: &Log) -> Result<DecodedEvent, DecodeError>;
}

// Users can implement custom decoders
struct MyCustomDecoder;
impl EventDecoder for MyCustomDecoder {
    fn decode(&self, log: &Log) -> Result<DecodedEvent, DecodeError> {
        // Custom decoding logic
    }
}
```

**Design principle:** Provide trait-based extension points for common customization needs rather than hard-coding behavior.

---

## API Reference

For detailed function signatures and error handling, refer to the rustdoc:

```bash
cargo doc --no-deps --open
```

Module paths:
- **Executor:** `eth_node::executor::{simulate_tx, simulate_contract_call, compare_to_anvil}`
- **Decoder:** `eth_node::quality::{decode_standard_nft_event, decode_nft_event_lossless}`

See also:
- [`CLI_REFERENCE.md`](../CLI_REFERENCE.md) for command-line usage examples
- [`FORMAL_SPEC.md`](../FORMAL_SPEC.md) for functional requirements and behavioral contracts
- [`PHASE2_FORMAL_SPEC.md`](../PHASE2_FORMAL_SPEC.md) for executor and decoder specifications
