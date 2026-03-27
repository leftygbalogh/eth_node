//! T-002 integration test: compare simulate_tx to Anvil execution.
//!
//! AC-001: All known Anvil scenarios match simulation results (gas within 5% tolerance).

mod helpers;

use alloy_primitives::{Address, TxKind, U256};
use alloy_rpc_types::TransactionRequest;
use eth_node::{
    executor::{simulate_tx, SimulationContext},
    rpc::RpcClient,
    signer::EthSigner,
    tx::{send_transaction, BroadcastConfig, TxBuilder},
};
use tokio::time::Duration;

const SKIP: &str = "SKIP — Anvil not on PATH";

/// Compare simulate_tx gas usage to Anvil execution (AC-001).
#[tokio::test]
async fn integration_anvil_comparison_simple_transfer() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("{SKIP}");
        return;
    };

    let client = RpcClient::new(&anvil.endpoint).expect("valid Anvil URL");
    let signer = EthSigner::from_key(helpers::accounts::ANVIL_ACCOUNT0_KEY).unwrap();
    let from = signer.address();
    let to = Address::repeat_byte(0x42); // Arbitrary recipient

    // Get chain state
    let chain_id = client.chain_id().await.unwrap();
    let nonce = client.get_nonce(from).await.unwrap();
    let gas_price = client.gas_price().await.unwrap();
    let block_number = client.block_number().await.unwrap();

    // Build transaction for local simulation
    let tx_request = TransactionRequest {
        from: Some(from),
        to: Some(TxKind::Call(to)),
        value: Some(U256::from(1_000_000)), // 0.000001 ETH
        gas: Some(21_000),
        gas_price: Some(gas_price),
        nonce: Some(nonce),
        chain_id: Some(chain_id),
        ..Default::default()
    };

    // Simulate locally
    let context = SimulationContext {
        block_number,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        base_fee_per_gas: Some(gas_price.try_into().expect("gas price fits u64")),
        gas_limit: 30_000_000,
    };

    let local_result = simulate_tx(&tx_request, &context).expect("simulation should succeed");

    // Execute on Anvil using Phase 1 tx infrastructure
    let builder = TxBuilder::transfer(chain_id, from, to, U256::from(1_000_000))
        .nonce(nonce)
        .gas_limit(21_000)
        .gas_price(gas_price);

    let config = BroadcastConfig {
        poll_interval: Duration::from_millis(100),
        timeout: Duration::from_secs(10),
    };

    let receipt = send_transaction(builder, &signer, &client, Some(config))
        .await
        .expect("transaction should confirm");

    let anvil_gas = receipt.gas_used;

    // Compare gas usage (AC-001: within 5% tolerance)
    let gas_delta = (local_result.gas_used as i64 - anvil_gas as i64).abs() as u64;
    let tolerance = (anvil_gas * 5) / 100; // 5%

    assert!(
        gas_delta <= tolerance,
        "gas mismatch exceeds 5% tolerance: local={}, anvil={}, delta={}, tolerance={}",
        local_result.gas_used,
        anvil_gas,
        gas_delta,
        tolerance
    );

    // For simple transfers, gas should be exactly 21,000
    assert_eq!(
        anvil_gas, 21_000,
        "simple transfer on Anvil should use 21k gas"
    );
    assert_eq!(
        local_result.gas_used, 21_000,
        "simple transfer simulation should use 21k gas"
    );

    assert!(local_result.success, "transfer should succeed");
    assert!(receipt.status(), "anvil tx should succeed");
}
