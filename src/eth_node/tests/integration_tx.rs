//! Integration tests for `eth_node::tx` (FR-003 + FR-005).
//!
//! These tests require a live Anvil process and skip gracefully if Anvil is not
//! on PATH. Each test gets its own Anvil instance for complete isolation.

mod helpers;

use alloy_primitives::U256;
use eth_node::{
    primitives::parse_address,
    rpc::RpcClient,
    signer::EthSigner,
    tx::{BroadcastConfig, TxBuilder, TxError, send_transaction},
};
use tokio::time::Duration;

const SKIP: &str = "SKIP — Anvil not on PATH";

// Well-known Anvil account 1 (receiver for transfers).
const ANVIL_ACCOUNT1_ADDRESS: &str = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";

// ── ETH transfer — confirmed ──────────────────────────────────────────────────

#[tokio::test]
async fn tx_send_eth_confirmed_eip1559() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("{SKIP}");
        return;
    };
    let client = RpcClient::new(&anvil.endpoint).unwrap();
    let signer = EthSigner::from_key(helpers::accounts::ANVIL_ACCOUNT0_KEY).unwrap();
    let from = signer.address();
    let to = parse_address(ANVIL_ACCOUNT1_ADDRESS).unwrap();

    let before = client.get_balance(to).await.unwrap();

    let value = U256::from(1_000_000_000_000_000_u64); // 0.001 ETH
    let chain_id = client.chain_id().await.unwrap();
    let nonce = client.get_nonce(from).await.unwrap();
    let gas_price = client.gas_price().await.unwrap();

    let builder = TxBuilder::transfer(chain_id, from, to, value)
        .nonce(nonce)
        .gas_limit(21_000)
        .max_fee(gas_price * 2, gas_price);

    let config = BroadcastConfig {
        poll_interval: Duration::from_millis(100),
        timeout: Duration::from_secs(10),
    };

    let receipt = send_transaction(builder, &signer, &client, Some(config))
        .await
        .expect("transaction should confirm");

    assert!(receipt.status(), "receipt status should be 1 (success)");

    let after = client.get_balance(to).await.unwrap();
    assert_eq!(after - before, value, "receiver balance should increase by value");
}

#[tokio::test]
async fn tx_send_eth_confirmed_legacy() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("{SKIP}");
        return;
    };
    let client = RpcClient::new(&anvil.endpoint).unwrap();
    let signer = EthSigner::from_key(helpers::accounts::ANVIL_ACCOUNT0_KEY).unwrap();
    let from = signer.address();
    let to = parse_address(ANVIL_ACCOUNT1_ADDRESS).unwrap();

    let value = U256::from(500_000_000_000_000_u64); // 0.0005 ETH
    let chain_id = client.chain_id().await.unwrap();
    let nonce = client.get_nonce(from).await.unwrap();
    let gas_price = client.gas_price().await.unwrap();

    let builder = TxBuilder::transfer(chain_id, from, to, value)
        .nonce(nonce)
        .gas_limit(21_000)
        .gas_price(gas_price);

    let config = BroadcastConfig {
        poll_interval: Duration::from_millis(100),
        timeout: Duration::from_secs(10),
    };

    let receipt = send_transaction(builder, &signer, &client, Some(config))
        .await
        .expect("legacy tx should confirm");

    assert!(receipt.status(), "receipt status should be 1 (success)");
}

// ── Confirmation timeout ───────────────────────────────────────────────────────

#[tokio::test]
async fn tx_confirmation_timeout_returns_hash() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("{SKIP}");
        return;
    };
    let client = RpcClient::new(&anvil.endpoint).unwrap();
    let signer = EthSigner::from_key(helpers::accounts::ANVIL_ACCOUNT0_KEY).unwrap();
    let from = signer.address();
    let to = parse_address(ANVIL_ACCOUNT1_ADDRESS).unwrap();

    let chain_id = client.chain_id().await.unwrap();
    let nonce = client.get_nonce(from).await.unwrap();
    let gas_price = client.gas_price().await.unwrap();

    // Build a valid transaction but use a 1ms timeout — too short to mine.
    let builder = TxBuilder::transfer(chain_id, from, to, U256::from(1u64))
        .nonce(nonce)
        .gas_limit(21_000)
        .max_fee(gas_price * 2, gas_price);

    let unsigned = builder.build(&client).await.unwrap();
    let signed = signer.sign(unsigned).unwrap();

    let config = BroadcastConfig {
        poll_interval: Duration::from_millis(1),
        timeout: Duration::from_millis(1), // immediate timeout
    };
    let broadcaster = eth_node::tx::Broadcaster::with_config(config);
    let err = broadcaster.send(&signed, &client).await.unwrap_err();

    assert!(
        matches!(err, TxError::ConfirmationTimeout { .. }),
        "expected ConfirmationTimeout, got {err:?}"
    );

    // The tx hash must be preserved in the error.
    if let TxError::ConfirmationTimeout { hash, .. } = err {
        assert_ne!(hash, alloy_primitives::B256::ZERO, "hash must not be zero");
    }
}

// ── Block number increments after confirmed tx ────────────────────────────────

#[tokio::test]
async fn tx_send_increments_block_number() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("{SKIP}");
        return;
    };
    let client = RpcClient::new(&anvil.endpoint).unwrap();
    let signer = EthSigner::from_key(helpers::accounts::ANVIL_ACCOUNT0_KEY).unwrap();
    let from = signer.address();
    let to = parse_address(ANVIL_ACCOUNT1_ADDRESS).unwrap();

    let block_before = client.block_number().await.unwrap();
    let chain_id = client.chain_id().await.unwrap();
    let nonce = client.get_nonce(from).await.unwrap();
    let gas_price = client.gas_price().await.unwrap();

    let builder = TxBuilder::transfer(chain_id, from, to, U256::from(1u64))
        .nonce(nonce)
        .gas_limit(21_000)
        .max_fee(gas_price * 2, gas_price);

    let config = BroadcastConfig {
        poll_interval: Duration::from_millis(100),
        timeout: Duration::from_secs(10),
    };
    send_transaction(builder, &signer, &client, Some(config)).await.unwrap();

    let block_after = client.block_number().await.unwrap();
    assert!(block_after > block_before, "block number must advance after mining a tx");
}
