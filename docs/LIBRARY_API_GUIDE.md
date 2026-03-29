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
