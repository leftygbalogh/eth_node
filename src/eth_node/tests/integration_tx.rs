//! Integration tests for `eth_node::tx` (FR-003 + FR-005).
//!
//! These tests require a live Anvil process and skip gracefully if Anvil is not
//! on PATH. Each test gets its own Anvil instance for complete isolation.

mod helpers;

use alloy_consensus::TxEip1559;
use alloy_primitives::{TxKind, U256};
use eth_node::{
    primitives::parse_address,
    rpc::RpcClient,
    signer::{EthSigner, UnsignedTx},
    tx::{BroadcastConfig, Broadcaster, TxBuilder, TxError, send_transaction},
};
use tokio::time::Duration;

// ── Reverter contract bytecode ────────────────────────────────────────────────
//
// A 17-byte init code that deploys a 5-byte runtime which unconditionally
// REVERTs any incoming call.
//
// Init code disassembly (12 bytes of preamble):
//   PUSH1 0x05   — runtime code length = 5
//   PUSH1 0x0c   — runtime code offset in init bytecode = 12
//   PUSH1 0x00   — memory destination offset
//   CODECOPY     — copy runtime[0..5] → mem[0..5]
//   PUSH1 0x05   — return 5 bytes
//   PUSH1 0x00   — from memory offset 0
//   RETURN       — deploy 5-byte runtime
//
// Runtime code disassembly (5 bytes, starting at init offset 12):
//   PUSH1 0x00   — revert data size = 0
//   PUSH1 0x00   — revert data offset = 0
//   REVERT       — always reverts with empty revert data
const REVERTER_INIT_CODE: &[u8] = &[
    // Init preamble (12 bytes):
    0x60, 0x05, // PUSH1 5    — runtime length
    0x60, 0x0c, // PUSH1 12   — runtime offset in this bytecode
    0x60, 0x00, // PUSH1 0    — memory dest
    0x39,       // CODECOPY
    0x60, 0x05, // PUSH1 5    — return size
    0x60, 0x00, // PUSH1 0    — return offset
    0xf3,       // RETURN
    // Runtime code (5 bytes):
    0x60, 0x00, // PUSH1 0    — revert offset
    0x60, 0x00, // PUSH1 0    — revert size
    0xfd,       // REVERT
];

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

// ── Reverted tx — receipt status == 0 → TxError::Reverted ────────────────────
//
// Spec ref: FR-005 statechart `Reverted` terminal state; T-006 DoD item 3b
//
// Deploys a contract whose runtime code unconditionally REVERTs on any call,
// then sends a tx to it and asserts the Broadcaster returns TxError::Reverted
// (not a panic and not a success receipt).

async fn deploy_reverter(client: &RpcClient, signer: &EthSigner) -> alloy_primitives::Address {
    let from = signer.address();
    let nonce = client.get_nonce(from).await.expect("get nonce");
    let gas_price = client.gas_price().await.expect("gas price");
    let chain_id = client.chain_id().await.expect("chain id");

    let tx = TxEip1559 {
        chain_id,
        nonce,
        max_fee_per_gas: gas_price * 2,
        max_priority_fee_per_gas: gas_price,
        gas_limit: 100_000,
        to: TxKind::Create,
        value: U256::ZERO,
        input: REVERTER_INIT_CODE.to_vec().into(),
        ..Default::default()
    };
    let signed = signer.sign(UnsignedTx::Eip1559(tx)).expect("sign deploy");
    let receipt = Broadcaster::new().send(&signed, client).await.expect("deploy reverter");
    assert!(receipt.status(), "reverter deploy must succeed");
    receipt.contract_address.expect("no contract_address in deploy receipt")
}

#[tokio::test]
async fn tx_send_reverted_receipt_status_zero() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("{SKIP}");
        return;
    };
    let client = RpcClient::new(&anvil.endpoint).unwrap();
    let signer = EthSigner::from_key(helpers::accounts::ANVIL_ACCOUNT0_KEY).unwrap();

    // Deploy the always-revert contract.
    let reverter_addr = deploy_reverter(&client, &signer).await;

    // Now send a tx to it — any call will revert.
    let from = signer.address();
    let chain_id = client.chain_id().await.unwrap();
    let nonce = client.get_nonce(from).await.unwrap();
    let gas_price = client.gas_price().await.unwrap();

    let builder = TxBuilder::new(chain_id, from, reverter_addr)
        .nonce(nonce)
        .gas_limit(50_000)
        .max_fee(gas_price * 2, gas_price);

    let config = BroadcastConfig {
        poll_interval: Duration::from_millis(100),
        timeout: Duration::from_secs(10),
    };

    let err = send_transaction(builder, &signer, &client, Some(config))
        .await
        .expect_err("call to reverter must not succeed");

    assert!(
        matches!(err, TxError::Reverted { .. }),
        "expected TxError::Reverted, got {err:?}"
    );

    // The reverted receipt must carry a non-zero tx hash.
    if let TxError::Reverted { hash, .. } = err {
        assert_ne!(hash, alloy_primitives::B256::ZERO, "reverted hash must be non-zero");
    }
}
