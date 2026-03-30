//! Example: Transaction Simulation
//!
//! This example demonstrates local EVM transaction simulation using revm.
//!
//! Run with:
//! ```bash
//! cargo run --example simulate_transaction
//! ```

use eth_node::executor::{simulate_tx, SimulationContext};
use alloy_rpc_types::TransactionRequest;
use alloy_primitives::{Address, U256};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Transaction Simulation Example ===\n");

    // 1. Build a simple ETH transfer transaction
    let sender = Address::from([0x11; 20]);
    let recipient = Address::from([0x22; 20]);
    let value = U256::from(1_000_000_000u64); // 1 gwei

    let tx = TransactionRequest {
        from: Some(sender),
        to: Some(recipient.into()),
        value: Some(value),
        gas: Some(21_000),
        gas_price: Some(20),
        ..Default::default()
    };

    println!("Transaction:");
    println!("  From:  {:?}", tx.from);
    println!("  To:    {:?}", tx.to);
    println!("  Value: {} wei", value);
    println!("  Gas:   {}\n", tx.gas.unwrap());

    // 2. Define simulation context (block environment)
    let context = SimulationContext {
        block_number: 1,
        timestamp: 1710000000,
        base_fee_per_gas: Some(10),
        gas_limit: 30_000_000,
    };

    println!("Simulation Context:");
    println!("  Block: {}", context.block_number);
    println!("  Timestamp: {}", context.timestamp);
    println!("  Base Fee: {} wei\n", context.base_fee_per_gas.unwrap());

    // 3. Execute simulation
    println!("Executing simulation...\n");
    let result = simulate_tx(&tx, &context)?;

    // 4. Inspect results
    println!("Simulation Results:");
    println!("  Success:     {}", result.success);
    println!("  Gas Used:    {}", result.gas_used);
    println!("  Return Data: {} bytes", result.return_data.len());
    println!("  Logs:        {} events emitted", result.logs.len());

    if !result.success {
        println!("\n⚠️  Transaction would revert!");
    } else {
        println!("\n✓ Transaction would succeed");
    }

    println!("\nKey Points:");
    println!("  • Simulation completes even if tx reverts (check result.success)");
    println!("  • No state is persisted (ephemeral in-memory execution)");
    println!("  • Use this for pre-flight validation before broadcasting");

    Ok(())
}
