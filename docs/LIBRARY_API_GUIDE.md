# Library API Usage Guide

This guide demonstrates how to use the `eth_node` library crate for transaction simulation, contract calls, event decoding, RPC operations, signing, transaction broadcasting, event streaming, and contract interaction.

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
4. [RPC Module](#rpc-module)
   - [Creating a Client](#creating-a-client)
   - [Query Operations](#query-operations)
   - [Transaction Operations](#transaction-operations)
   - [Log Queries](#log-queries)
5. [Signer Module](#signer-module)
   - [Loading Private Keys](#loading-private-keys)
   - [Signing Transactions](#signing-transactions)
6. [Transaction Module](#transaction-module)
   - [Building Transactions](#building-transactions)
   - [Broadcasting with Confirmation](#broadcasting-with-confirmation)
7. [Events Module](#events-module)
   - [HTTP Polling Mode](#http-polling-mode)
   - [WebSocket Subscription Mode](#websocket-subscription-mode)
   - [Filter by Event Signature](#filter-by-event-signature)
8. [Contract Module](#contract-module)
   - [Creating a Contract Caller](#creating-a-contract-caller)
   - [Read Calls (view functions)](#read-calls-view-functions)
   - [Write Transactions (state-changing)](#write-transactions-state-changing)
9. [Primitives Module](#primitives-module)
   - [Address Parsing](#address-parsing)
   - [ABI Encoding/Decoding](#abi-encodingdecoding)
   - [RLP Encoding/Decoding](#rlp-encodingdecoding)
10. [Real-World Use Cases](#real-world-use-cases)
    - [Use Case 1: Freelancer Payment Processor](#use-case-1-freelancer-payment-processor)
    - [Use Case 2: DAO Voting with Gas Sponsorship](#use-case-2-dao-voting-with-gas-sponsorship)
    - [Use Case 3: Token Vesting Schedule Checker](#use-case-3-token-vesting-schedule-checker)
    - [Use Case 4: NFT Batch Minter with Gas Optimization](#use-case-4-nft-batch-minter-with-gas-optimization)
    - [Use Case 5: Multi-Sig Wallet Transaction Proposer](#use-case-5-multi-sig-wallet-transaction-proposer)
11. [Rust Patterns & Best Practices](#rust-patterns--best-practices)
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

## RPC Module

The `rpc` module provides an async JSON-RPC client for Ethereum-compatible endpoints.

### Creating a Client

```rust
use eth_node::rpc::RpcClient;

let client = RpcClient::new("http://127.0.0.1:8545")?;
println!("Connected to: {}", client.endpoint());
```

### Query Operations

**Get block number:**
```rust
let block_num = client.block_number().await?;
println!("Latest block: {}", block_num);
```

**Get account balance:**
```rust
use alloy_primitives::Address;

let address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse()?;
let balance = client.get_balance(address).await?;
println!("Balance: {} wei", balance);
```

**Get balance at specific block:**
```rust
use alloy_rpc_types::BlockId;

let balance = client.get_balance_at(address, BlockId::number(100)).await?;
```

**Get transaction count (nonce):**
```rust
let nonce = client.get_transaction_count(address).await?;
println!("Next nonce: {}", nonce);
```

**Get gas price:**
```rust
let gas_price = client.gas_price().await?;
println!("Gas price: {} wei", gas_price);
```

### Transaction Operations

**Estimate gas:**
```rust
use alloy_rpc_types::TransactionRequest;

let tx = TransactionRequest {
    from: Some(sender),
    to: Some(recipient.into()),
    value: Some(U256::from(1_000_000)),
    ..Default::default()
};

let gas_estimate = client.estimate_gas(tx).await?;
println!("Estimated gas: {}", gas_estimate);
```

**Send raw transaction:**
```rust
use alloy_primitives::Bytes;

let raw_tx = Bytes::from(signed_transaction.raw_bytes);
let tx_hash = client.send_raw_transaction(raw_tx).await?;
println!("Submitted: {:?}", tx_hash);
```

**Get transaction receipt:**
```rust
let receipt = client.get_transaction_receipt(tx_hash).await?;
if let Some(receipt) = receipt {
    println!("Status: {:?}", receipt.status());
    println!("Gas used: {}", receipt.gas_used);
}
```

**Call contract (read-only):**
```rust
let result = client.call(tx_request).await?;
println!("Return data: 0x{}", hex::encode(&result));
```

### Log Queries

**Get logs with filter:**
```rust
use alloy_rpc_types::Filter;

let filter = Filter::new()
    .address(contract_address)
    .from_block(1000)
    .to_block(2000);

let logs = client.get_logs(filter).await?;
println!("Found {} logs", logs.len());
```

---

## Signer Module

The `signer` module handles private key management and transaction signing.

### Loading Private Keys

**From environment variable:**
```rust
use eth_node::signer::EthSigner;

// Expects ETH_PRIVATE_KEY=0x...
let signer = EthSigner::from_env()?;
println!("Signer address: {}", signer.address());
```

**From hex string:**
```rust
let key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
let signer = EthSigner::from_key(key)?;
```

### Signing Transactions

**EIP-1559 transaction:**
```rust
use eth_node::signer::UnsignedTx;
use alloy_consensus::TxEip1559;
use alloy_primitives::{TxKind, U256};

let unsigned = UnsignedTx::Eip1559(TxEip1559 {
    chain_id: 1,
    nonce: 0,
    gas_limit: 21_000,
    max_fee_per_gas: 50_000_000_000,
    max_priority_fee_per_gas: 2_000_000_000,
    to: TxKind::Call(recipient),
    value: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
    input: Default::default(),
    access_list: Default::default(),
});

let signed = signer.sign(unsigned)?;
println!("Transaction hash: {:?}", signed.hash);
println!("From: {}", signed.from);
// signed.raw_bytes ready for eth_sendRawTransaction
```

**Legacy transaction:**
```rust
use alloy_consensus::TxLegacy;

let unsigned = UnsignedTx::Legacy(TxLegacy {
    chain_id: Some(1),
    nonce: 0,
    gas_price: 20_000_000_000,
    gas_limit: 21_000,
    to: TxKind::Call(recipient),
    value: U256::from(1_000_000_000_000_000_000u64),
    input: Default::default(),
});

let signed = signer.sign(unsigned)?;
```

---

## Transaction Module

The `tx` module provides a fluent builder for transactions and a broadcaster with confirmation polling.

### Building Transactions

**Auto-fetch fees (EIP-1559):**
```rust
use eth_node::tx::TxBuilder;

let builder = TxBuilder::new()
    .to(recipient)
    .value(U256::from(1_000_000_000_000_000_000u64)) // 1 ETH
    .build_unsigned(&client, &signer).await?;
// Nonce and fees automatically fetched from RPC
```

**Custom EIP-1559 fees:**
```rust
let builder = TxBuilder::new()
    .to(recipient)
    .value(U256::from(500_000_000_000_000_000u64)) // 0.5 ETH
    .max_fee(50_000_000_000, 2_000_000_000) // max_fee, priority_fee
    .build_unsigned(&client, &signer).await?;
```

**Legacy gas price:**
```rust
let builder = TxBuilder::new()
    .to(recipient)
    .value(U256::from(1_000_000))
    .gas_price(20_000_000_000)
    .build_unsigned(&client, &signer).await?;
```

**Contract interaction with data:**
```rust
use alloy_primitives::Bytes;

let calldata = Bytes::from(hex::decode("a9059cbb...")?); // transfer(address,uint256)
let builder = TxBuilder::new()
    .to(token_contract)
    .data(calldata)
    .gas(100_000)
    .build_unsigned(&client, &signer).await?;
```

### Broadcasting with Confirmation

**Send and wait for confirmation:**
```rust
use eth_node::tx::{Broadcaster, BroadcastConfig};

let config = BroadcastConfig {
    poll_interval_ms: 1000,
    timeout_secs: 60,
};

let broadcaster = Broadcaster::new(client, signer);
let receipt = broadcaster.send(tx_builder, config).await?;

println!("Mined in block: {:?}", receipt.block_number);
println!("Gas used: {}", receipt.gas_used);
```

**Fire-and-forget (no wait):**
```rust
let config = BroadcastConfig {
    poll_interval_ms: 0, // Skip polling
    timeout_secs: 0,
};

// Returns as soon as transaction is accepted by mempool
broadcaster.send(tx_builder, config).await?;
```

**Handle reverted transactions:**
```rust
use eth_node::tx::TxError;

match broadcaster.send(tx_builder, config).await {
    Ok(receipt) => println!("Success!"),
    Err(TxError::Reverted { hash, receipt }) => {
        eprintln!("Transaction reverted: {:?}", hash);
        eprintln!("Gas used: {}", receipt.gas_used);
    }
    Err(e) => return Err(e.into()),
}
```

---

## Events Module

The `events` module streams contract events via HTTP polling or WebSocket subscriptions.

### HTTP Polling Mode

```rust
use eth_node::events::Listener;
use alloy_rpc_types::Filter;
use futures::StreamExt;

let filter = Filter::new()
    .address(contract_address)
    .from_block(BlockNumberOrTag::Latest);

let mut stream = Listener::new("http://127.0.0.1:8545")
    .with_poll_interval(Duration::from_secs(2))
    .subscribe(filter);

while let Some(result) = stream.next().await {
    match result {
        Ok(log) => println!("Log: {:?}", log),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### WebSocket Subscription Mode

```rust
let mut stream = Listener::new("ws://127.0.0.1:8545")
    .with_max_reconnect(Some(5)) // Retry 5 times on disconnect
    .subscribe(filter);

while let Some(result) = stream.next().await {
    match result {
        Ok(log) => {
            // Process log
            println!("Block: {:?}, Tx: {:?}", log.block_number, log.transaction_hash);
        }
        Err(ListenerError::ReconnectExhausted(n)) => {
            eprintln!("Gave up after {} reconnect attempts", n);
            break;
        }
        Err(e) => eprintln!("Stream error: {}", e),
    }
}
```

### Filter by Event Signature

```rust
use alloy_primitives::B256;

// Transfer(address,address,uint256) topic0
let transfer_topic = B256::from([
    0xdd, 0xf2, 0x52, 0xad, 0x1b, 0xe2, 0xc8, 0x9b,
    0x69, 0xc2, 0xb0, 0x68, 0xfc, 0x37, 0x8d, 0xaa,
    0x95, 0x2b, 0xa7, 0xf1, 0x63, 0xc4, 0xa1, 0x16,
    0x28, 0xf5, 0x5a, 0x4d, 0xf5, 0x23, 0xb3, 0xef,
]);

let filter = Filter::new()
    .address(contract_address)
    .event_signature(transfer_topic);

let mut stream = Listener::new("ws://127.0.0.1:8545").subscribe(filter);
```

---

## Contract Module

The `contract` module provides ABI-driven contract interaction (read calls and write transactions).

### Creating a Contract Caller

```rust
use eth_node::contract::ContractCaller;

let abi_json = r#"[
  {
    "name": "balanceOf",
    "type": "function",
    "inputs": [{"name": "account", "type": "address"}],
    "outputs": [{"name": "", "type": "uint256"}],
    "stateMutability": "view"
  },
  {
    "name": "transfer",
    "type": "function",
    "inputs": [
      {"name": "recipient", "type": "address"},
      {"name": "amount", "type": "uint256"}
    ],
    "outputs": [{"name": "", "type": "bool"}],
    "stateMutability": "nonpayable"
  }
]"#;

let contract = ContractCaller::new(token_address, abi_json)?;
```

### Read Calls (view functions)

```rust
use alloy_dyn_abi::DynSolValue;

let user = Address::from([0x11; 20]);
let args = vec![DynSolValue::Address(user)];

let result = contract.call("balanceOf", &args, &client).await?;

// result is Vec<DynSolValue>
if let Some(DynSolValue::Uint(balance, _)) = result.first() {
    println!("Balance: {}", balance);
}
```

### Write Transactions (state-changing)

```rust
let recipient = Address::from([0x22; 20]);
let amount = DynSolValue::Uint(U256::from(1_000_000_000_000_000_000u64), 256);

let args = vec![
    DynSolValue::Address(recipient),
    amount,
];

let receipt = contract.send("transfer", &args, &signer, &client).await?;
println!("Transfer mined in block: {:?}", receipt.block_number);
```

---

## Primitives Module

The `primitives` module provides low-level encoding/decoding utilities for ABI and RLP formats.

### Address Parsing

```rust
use eth_node::primitives::parse_address;

let addr = parse_address("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")?;
let addr_no_prefix = parse_address("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266")?; // Also works
```

### ABI Encoding/Decoding

**uint256:**
```rust
use eth_node::primitives::{abi_encode_uint256, abi_decode_uint256};
use alloy_primitives::U256;

let value = U256::from(12345);
let encoded = abi_encode_uint256(value);
let decoded = abi_decode_uint256(&encoded)?;
assert_eq!(value, decoded);
```

**address:**
```rust
use eth_node::primitives::{abi_encode_address, abi_decode_address};

let addr = Address::from([0xAA; 20]);
let encoded = abi_encode_address(addr);
let decoded = abi_decode_address(&encoded)?;
```

**bool:**
```rust
use eth_node::primitives::{abi_encode_bool, abi_decode_bool};

let encoded = abi_encode_bool(true);
let decoded = abi_decode_bool(&encoded)?;
```

**string:**
```rust
use eth_node::primitives::{abi_encode_string, abi_decode_string};

let encoded = abi_encode_string("Hello, Ethereum!");
let decoded = abi_decode_string(&encoded)?;
```

### RLP Encoding/Decoding

**u64:**
```rust
use eth_node::primitives::{rlp_encode_u64, rlp_decode_u64};

let value = 42u64;
let encoded = rlp_encode_u64(value);
let decoded = rlp_decode_u64(&encoded)?;
```

**bytes:**
```rust
use eth_node::primitives::{rlp_encode_bytes, rlp_decode_bytes};

let data = b"arbitrary bytes";
let encoded = rlp_encode_bytes(data);
let decoded = rlp_decode_bytes(&encoded)?;
```

---

## Real-World Use Cases

### Use Case 1: Freelancer Payment Processor

**Scenario:** Alice hires Bob to design a logo. She escrows 12,345 wei in a smart contract. When Bob delivers, Alice approves payment, and Bob receives the funds.

```rust
use eth_node::{rpc::RpcClient, signer::EthSigner, tx::{TxBuilder, Broadcaster, BroadcastConfig}, contract::ContractCaller};
use alloy_dyn_abi::DynSolValue;
use alloy_primitives::{Address, U256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup
    let client = RpcClient::new("http://127.0.0.1:8545")?;
    let alice = EthSigner::from_key("0xac09...")?; // Alice's key
    let bob_address: Address = "0x7099...".parse()?;
    
    let escrow_abi = r#"[
      {"name":"deposit","type":"function","inputs":[],"outputs":[],"stateMutability":"payable"},
      {"name":"release","type":"function","inputs":[{"name":"recipient","type":"address"}],"outputs":[],"stateMutability":"nonpayable"},
      {"name":"balance","type":"function","inputs":[],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}
    ]"#;
    
    let escrow_contract = ContractCaller::new(escrow_address, escrow_abi)?;
    
    // 2. Alice deposits 12,345 wei
    println!("Alice deposits payment...");
    let deposit_builder = TxBuilder::new()
        .to(escrow_address)
        .value(U256::from(12_345))
        .gas(100_000);
    
    let broadcaster = Broadcaster::new(&client, &alice);
    let config = BroadcastConfig { poll_interval_ms: 1000, timeout_secs: 30 };
    
    let receipt = broadcaster.send(deposit_builder, config).await?;
    println!("✓ Deposited in block {}", receipt.block_number.unwrap());
    
    // 3. Check escrow balance
    let balance = escrow_contract.call("balance", &[], &client).await?;
    if let Some(DynSolValue::Uint(bal, _)) = balance.first() {
        println!("Escrow balance: {} wei", bal);
        assert_eq!(*bal, U256::from(12_345));
    }
    
    // 4. Bob delivers logo → Alice approves release
    println!("\nBob delivered logo. Alice approves payment...");
    let args = vec![DynSolValue::Address(bob_address)];
    let release_receipt = escrow_contract.send("release", &args, &alice, &client).await?;
    println!("✓ Payment released in block {}", release_receipt.block_number.unwrap());
    
    // 5. Verify Bob received funds
    let bob_balance = client.get_balance(bob_address).await?;
    println!("Bob's new balance: {} wei", bob_balance);
    
    Ok(())
}
```

**Key Points:**
- Escrow ensures Alice doesn't pay until delivery
- Bob guaranteed payment once approved (on-chain enforcement)
- Contract holds funds trustlessly (no intermediary needed)

---

### Use Case 2: DAO Voting with Gas Sponsorship

**Scenario:** A DAO member wants to vote on proposal #42. Gas fees are sponsored by the DAO treasury (relay pattern).

```rust
use eth_node::{rpc::RpcClient, signer::EthSigner, contract::ContractCaller, events::Listener};
use alloy_dyn_abi::DynSolValue;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::new("http://127.0.0.1:8545")?;
    let dao_treasury = EthSigner::from_env()?; // DAO pays gas
    
    let dao_abi = r#"[
      {"name":"vote","type":"function","inputs":[
        {"name":"proposalId","type":"uint256"},
        {"name":"support","type":"bool"}
      ],"outputs":[],"stateMutability":"nonpayable"},
      {"name":"VoteCast","type":"event","inputs":[
        {"name":"voter","indexed":true,"type":"address"},
        {"name":"proposalId","indexed":true,"type":"uint256"},
        {"name":"support","indexed":false,"type":"bool"}
      ]}
    ]"#;
    
    let dao_contract = ContractCaller::new(dao_address, dao_abi)?;
    
    // 1. Member submits vote (DAO treasury pays gas)
    let proposal_id = DynSolValue::Uint(U256::from(42), 256);
    let support = DynSolValue::Bool(true); // Vote YES
    
    println!("Casting vote on proposal 42...");
    let vote_receipt = dao_contract.send(
        "vote", 
        &[proposal_id, support], 
        &dao_treasury, 
        &client
    ).await?;
    
    println!("✓ Vote cast in tx: {:?}", vote_receipt.transaction_hash);
    println!("Gas paid by DAO treasury: {} wei", 
             vote_receipt.gas_used * vote_receipt.effective_gas_price.unwrap_or(U256::from(0)));
    
    // 2. Listen for VoteCast events (real-time)
    let filter = Filter::new()
        .address(dao_address)
        .from_block(BlockNumberOrTag::Latest)
        .event("VoteCast(address,uint256,bool)");
    
    let mut stream = Listener::new("ws://127.0.0.1:8545").subscribe(filter);
    
    println!("\nWatching for votes...");
    while let Some(Ok(log)) = stream.next().await {
        // Decode VoteCast event
        if let Ok(DecodedEvent::Custom(event)) = decode_custom_event(&log) {
            println!("Vote detected:");
            println!("  Voter: {}", event.voter);
            println!("  Proposal: {}", event.proposal_id);
            println!("  Support: {}", event.support);
        }
    }
    
    Ok(())
}
```

**Key Points:**
- Gas sponsorship enables fee-less voting for members
- Real-time event streaming confirms votes as they happen
- Treasury address can be a multisig or gnosis-safe

---

### Use Case 3: Token Vesting Schedule Checker

**Scenario:** An employee wants to check how many tokens are unlocked from their vesting contract at any given timestamp.

```rust
use eth_node::{rpc::RpcClient, contract::ContractCaller};
use alloy_dyn_abi::DynSolValue;
use alloy_primitives::U256;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::new("http://127.0.0.1:8545")?;
    
    let vesting_abi = r#"[
      {"name":"getUnlockedAmount","type":"function","inputs":[
        {"name":"timestamp","type":"uint256"}
      ],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"},
      {"name":"totalVested","type":"function","inputs":[],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"},
      {"name":"claimed","type":"function","inputs":[],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}
    ]"#;
    
    let vesting_contract = ContractCaller::new(vesting_address, vesting_abi)?;
    
    // 1. Check total vested amount
    let total = vesting_contract.call("totalVested", &[], &client).await?;
    let total_tokens = if let Some(DynSolValue::Uint(val, _)) = total.first() {
        *val
    } else {
        U256::ZERO
    };
    
    println!("Total vested: {} tokens", total_tokens);
    
    // 2. Check already claimed
    let claimed = vesting_contract.call("claimed", &[], &client).await?;
    let claimed_tokens = if let Some(DynSolValue::Uint(val, _)) = claimed.first() {
        *val
    } else {
        U256::ZERO
    };
    
    println!("Already claimed: {} tokens", claimed_tokens);
    
    // 3. Check unlocked amount at different timestamps
    let timestamps = vec![
        1704067200, // 2024-01-01
        1719792000, // 2024-07-01
        1735689600, // 2025-01-01
    ];
    
    for ts in timestamps {
        let args = vec![DynSolValue::Uint(U256::from(ts), 256)];
        let result = vesting_contract.call("getUnlockedAmount", &args, &client).await?;
        
        if let Some(DynSolValue::Uint(unlocked, _)) = result.first() {
            let claimable = if *unlocked > claimed_tokens {
                *unlocked - claimed_tokens
            } else {
                U256::ZERO
            };
            
            let date = chrono::NaiveDateTime::from_timestamp_opt(ts as i64, 0)
                .unwrap()
                .format("%Y-%m-%d");
            
            println!("\n{}: {} tokens unlocked", date, unlocked);
            println!("  → {} claimable now", claimable);
        }
    }
    
    Ok(())
}
```

**Key Points:**
- Read-only calls cost zero gas
- Can simulate future unlock amounts without waiting
- Useful for financial planning

---

### Use Case 4: NFT Batch Minter with Gas Optimization

**Scenario:** Mint 100 NFTs to different recipients, monitoring gas usage per transaction to optimize batch size.

```rust
use eth_node::{rpc::RpcClient, signer::EthSigner, tx::{TxBuilder, Broadcaster, BroadcastConfig}, executor::{simulate_tx, SimulationContext}};
use alloy_rpc_types::TransactionRequest;
use alloy_primitives::{Address, Bytes, U256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::new("http://127.0.0.1:8545")?;
    let minter = EthSigner::from_env()?;
    
    let nft_address: Address = "0x5FbD...".parse()?;
    let recipients: Vec<Address> = generate_recipients(100); // 100 distinct addresses
    
    // 1. Simulate single mint to estimate gas
    let single_mint_data = encode_mint_call(recipients[0], 1);
    let sim_tx = TransactionRequest {
        from: Some(minter.address()),
        to: Some(nft_address.into()),
        data: Some(single_mint_data.clone()),
        gas: Some(500_000),
        ..Default::default()
    };
    
    let context = SimulationContext {
        block_number: 1,
        timestamp: 1710000000,
        base_fee_per_gas: Some(10),
        gas_limit: 30_000_000,
    };
    
    let sim_result = simulate_tx(&sim_tx, &context)?;
    println!("Single mint gas: {}", sim_result.gas_used);
    
    // 2. Determine optimal batch size (target: 80% of block gas limit)
    let block_gas_limit = 30_000_000u64;
    let target_gas = (block_gas_limit as f64 * 0.8) as u64;
    let single_gas = sim_result.gas_used;
    let batch_size = std::cmp::min(target_gas / single_gas, 100);
    
    println!("Optimal batch size: {} mints per tx", batch_size);
    
    // 3. Batch mint with gas tracking
    let config = BroadcastConfig { poll_interval_ms: 500, timeout_secs: 60 };
    let broadcaster = Broadcaster::new(&client, &minter);
    
    let mut total_gas = 0u64;
    let mut token_id = 1u64;
    
    for batch in recipients.chunks(batch_size as usize) {
        let batch_data = encode_batch_mint_call(batch, token_id);
        
        let tx_builder = TxBuilder::new()
            .to(nft_address)
            .data(batch_data)
            .gas((single_gas * batch.len() as u64) + 50_000); // Add safety margin
        
        println!("\nMinting tokens {} to {}...", token_id, token_id + batch.len() as u64 - 1);
        let receipt = broadcaster.send(tx_builder, config).await?;
        
        let gas_used = receipt.gas_used;
        total_gas += gas_used;
        
        println!("✓ Batch mined in block {}", receipt.block_number.unwrap());
        println!("  Gas used: {}", gas_used);
        println!("  Avg per mint: {}", gas_used / batch.len() as u64);
        
        token_id += batch.len() as u64;
    }
    
    println!("\n=== Summary ===");
    println!("Total tokens minted: {}", token_id - 1);
    println!("Total gas used: {}", total_gas);
    println!("Average gas per token: {}", total_gas / (token_id - 1));
    
    Ok(())
}

fn encode_mint_call(recipient: Address, token_id: u64) -> Bytes {
    // Encode mint(address,uint256) function call
    use alloy_sol_types::{sol, SolCall};
    sol! {
        function mint(address to, uint256 tokenId);
    }
    Bytes::from(mint::mintCall { to: recipient, tokenId: U256::from(token_id) }.abi_encode())
}

fn encode_batch_mint_call(recipients: &[Address], start_token_id: u64) -> Bytes {
    // Encode batch mint logic (contract-specific)
    todo!("Implement based on your NFT contract's batch mint function")
}

fn generate_recipients(count: usize) -> Vec<Address> {
    (0..count)
        .map(|i| Address::from_slice(&[i as u8; 20]))
        .collect()
}
```

**Key Points:**
- Simulation predicts gas cost without spending gas
- Batch size optimization prevents block gas limit hits
- Gas tracking enables cost analysis

---

### Use Case 5: Multi-Sig Wallet Transaction Proposer

**Scenario:** Propose a transaction to a multi-sig wallet that requires 3 of 5 signatures.

```rust
use eth_node::{rpc::RpcClient, signer::EthSigner, contract::ContractCaller};
use alloy_dyn_abi::DynSolValue;
use alloy_primitives::{Address, Bytes, U256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::new("http://127.0.0.1:8545")?;
    let proposer = EthSigner::from_env()?; // One of the 5 owners
    
    let multisig_abi = r#"[
      {"name":"submitTransaction","type":"function","inputs":[
        {"name":"destination","type":"address"},
        {"name":"value","type":"uint256"},
        {"name":"data","type":"bytes"}
      ],"outputs":[{"name":"transactionId","type":"uint256"}],"stateMutability":"nonpayable"},
      {"name":"confirmTransaction","type":"function","inputs":[
        {"name":"transactionId","type":"uint256"}
      ],"outputs":[],"stateMutability":"nonpayable"},
      {"name":"getTransactionCount","type":"function","inputs":[
        {"name":"pending","type":"bool"},
        {"name":"executed","type":"bool"}
      ],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}
    ]"#;
    
    let multisig = ContractCaller::new(multisig_address, multisig_abi)?;
    
    // 1. Propose transaction: Send 5 ETH to charity
    let charity: Address = "0xCafe...".parse()?;
    let amount = U256::from(5_000_000_000_000_000_000u64); // 5 ETH
    let empty_data = Bytes::default();
    
    println!("Proposing transaction: Send 5 ETH to charity...");
    let submit_args = vec![
        DynSolValue::Address(charity),
        DynSolValue::Uint(amount, 256),
        DynSolValue::Bytes(empty_data.to_vec()),
    ];
    
    let submit_result = multisig.send("submitTransaction", &submit_args, &proposer, &client).await?;
    
    // Extract transaction ID from event logs
    let tx_id = extract_transaction_id_from_receipt(&submit_result)?;
    println!("✓ Transaction #{} proposed", tx_id);
    
    // 2. Auto-confirm as proposer (1 of 3 required signatures)
    println!("\nConfirming as proposer...");
    let confirm_args = vec![DynSolValue::Uint(U256::from(tx_id), 256)];
    multisig.send("confirmTransaction", &confirm_args, &proposer, &client).await?;
    println!("✓ Confirmation 1/3 submitted");
    
    // 3. Check pending transaction count
    let pending_args = vec![
        DynSolValue::Bool(true),  // Include pending
        DynSolValue::Bool(false), // Exclude executed
    ];
    let count = multisig.call("getTransactionCount", &pending_args, &client).await?;
    
    if let Some(DynSolValue::Uint(pending_count, _)) = count.first() {
        println!("\nPending transactions: {}", pending_count);
    }
    
    println!("\nWaiting for 2 more owners to confirm transaction #{}...", tx_id);
    println!("Once reached, transaction will auto-execute.");
    
    Ok(())
}

fn extract_transaction_id_from_receipt(receipt: &TransactionReceipt) -> Result<u64, Box<dyn std::error::Error>> {
    // Parse Submission(uint256 indexed transactionId) event
    for log in &receipt.inner.logs() {
        if log.topics().len() >= 2 {
            let tx_id_bytes = log.topics()[1];
            return Ok(U256::from_be_bytes(tx_id_bytes.0).to::<u64>());
        }
    }
    Err("Transaction ID not found in logs".into())
}
```

**Key Points:**
- Multi-sig enforces consensus for high-value operations
- Transaction proposed but not executed until threshold reached
- On-chain state machine tracks confirmations

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
