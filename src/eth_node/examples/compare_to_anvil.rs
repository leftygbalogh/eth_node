//! Example: Compare Local Simulation to Anvil
//!
//! This example demonstrates validating local revm simulation accuracy
//! by comparing gas usage and return data against Anvil execution.
//!
//! Prerequisites:
//! 1. Anvil running on http://127.0.0.1:8545
//! 2. Funded account available (use Anvil's default test accounts)
//!
//! Run with:
//! ```bash
//! # Terminal 1: Start Anvil
//! anvil
//!
//! # Terminal 2: Run example
//! cargo run --example compare_to_anvil
//! ```

use eth_node::executor::{compare_to_anvil, SimulationContext, ExecutorError};
use alloy_rpc_types::TransactionRequest;
use alloy_primitives::{Address, U256};

#[tokio::main]
async fn main() -> Result<(), ExecutorError> {
    println!("=== Compare Local Simulation to Anvil ===\n");

    // Use Anvil's first default test account (pre-funded with 10000 ETH)
    let anvil_account: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        .parse()
        .expect("Valid Anvil default address");

    let recipient = Address::from([0x22; 20]);
    let value = U256::from(1_000_000_000u64); // 1 gwei (reduced to avoid any gas issues)

    // Build transaction (must be executable on Anvil)
    let tx = TransactionRequest {
        from: Some(anvil_account),
        to: Some(recipient.into()),
        value: Some(value),
        gas: Some(21_000),
        gas_price: Some(20_000_000_000u128), // 20 gwei
        ..Default::default()
    };

    println!("Transaction:");
    println!("  From:  {:?}", tx.from);
    println!("  To:    {:?}", tx.to);
    println!("  Value: {} wei", value);
    println!("  Gas:   {}\n", tx.gas.unwrap());

    // Define simulation context
    let context = SimulationContext {
        block_number: 1,
        timestamp: 1710000000,
        base_fee_per_gas: Some(10),
        gas_limit: 30_000_000,
    };

    println!("Comparing local simulation to Anvil...\n");

    // Compare local simulation to Anvil execution
    match compare_to_anvil(&tx, "http://127.0.0.1:8545", &context).await {
        Ok(report) => {
            println!("Comparison Report:");
            println!("  Gas Used (Local):  {}", report.gas_used_local);
            println!("  Gas Used (Anvil):  {}", report.gas_used_anvil);
            println!("  Gas Delta:         {:+}", report.gas_delta);
            println!("  Return Data Match: {}", report.return_data_match);

            // Calculate 5% tolerance threshold
            let threshold = (report.gas_used_anvil as f64 * 0.05).ceil() as u64;
            let within_tolerance = report.gas_delta.unsigned_abs() <= threshold;

            println!("\nGas Tolerance Check (±5%):");
            println!("  Threshold:    {} gas", threshold);
            println!("  Delta:        {} gas", report.gas_delta.unsigned_abs());
            println!("  Status:       {}", if within_tolerance { "✓ PASS" } else { "✗ FAIL" });

            if !report.differences.is_empty() {
                println!("\nDifferences Detected:");
                for diff in &report.differences {
                    println!("  • {}", diff);
                }
            } else {
                println!("\n✓ Perfect match! Local simulation matches Anvil exactly.");
            }

            println!("\nKey Points:");
            println!("  • compare_to_anvil() always returns Ok(report) unless RPC fails");
            println!("  • Does NOT fail on mismatches—reports them in differences field");
            println!("  • Caller interprets thresholds (5% gas tolerance recommended)");
            println!("  • Uses eth_estimateGas + eth_call (no transaction signing required)");
        }
        Err(e) => {
            eprintln!("✗ Comparison failed: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("  1. Is Anvil running? Check http://127.0.0.1:8545");
            eprintln!("  2. Is the sender account funded? Use Anvil's default accounts");
            eprintln!("  3. Check network connectivity");
            return Err(e);
        }
    };

    Ok(())
}
