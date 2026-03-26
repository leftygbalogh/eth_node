//! Integration tests for `eth_node::rpc`.
//!
//! These tests start a fresh Anvil instance per test and exercise the typed
//! RPC methods against it. Tests are skipped gracefully when Anvil is not on
//! PATH (local dev without Foundry). CI installs Foundry via
//! `foundry-rs/foundry-toolchain@v1`, so tests run for real there.

mod helpers;

use alloy_primitives::U256;
use eth_node::{
    primitives::parse_address,
    rpc::{RpcClient, RpcError},
};

const SKIP: &str = "SKIP — Anvil not on PATH";

// ── eth_blockNumber ───────────────────────────────────────────────────────────

#[tokio::test]
async fn rpc_block_number_is_zero_on_fresh_anvil() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("{SKIP}");
        return;
    };
    let client = RpcClient::new(&anvil.endpoint).expect("valid Anvil URL");
    let n = client.block_number().await.expect("eth_blockNumber");
    assert_eq!(n, 0, "fresh Anvil starts at block 0");
}

// ── eth_getBalance ────────────────────────────────────────────────────────────

#[tokio::test]
async fn rpc_get_balance_returns_initial_anvil_balance() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("{SKIP}");
        return;
    };
    let client = RpcClient::new(&anvil.endpoint).expect("valid Anvil URL");
    let addr =
        parse_address(helpers::accounts::ANVIL_ACCOUNT0_ADDRESS).expect("valid address constant");
    let balance = client.get_balance(addr).await.expect("eth_getBalance");
    let expected = U256::from(helpers::accounts::ANVIL_INITIAL_BALANCE_WEI);
    assert_eq!(balance, expected, "account 0 should start with 10 000 ETH");
}

// ── eth_chainId ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn rpc_chain_id_is_31337_for_anvil() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("{SKIP}");
        return;
    };
    let client = RpcClient::new(&anvil.endpoint).expect("valid Anvil URL");
    let id = client.chain_id().await.expect("eth_chainId");
    assert_eq!(id, helpers::accounts::ANVIL_CHAIN_ID);
}

// ── eth_getTransactionCount (nonce) ───────────────────────────────────────────

#[tokio::test]
async fn rpc_nonce_is_zero_for_fresh_account() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("{SKIP}");
        return;
    };
    let client = RpcClient::new(&anvil.endpoint).expect("valid Anvil URL");
    let addr =
        parse_address(helpers::accounts::ANVIL_ACCOUNT0_ADDRESS).expect("valid address constant");
    let nonce = client.get_nonce(addr).await.expect("eth_getTransactionCount");
    assert_eq!(nonce, 0, "no transactions yet — nonce should be 0");
}

// ── eth_gasPrice ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn rpc_gas_price_is_nonzero() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("{SKIP}");
        return;
    };
    let client = RpcClient::new(&anvil.endpoint).expect("valid Anvil URL");
    let price = client.gas_price().await.expect("eth_gasPrice");
    assert!(price > 0, "Anvil gas price should be non-zero");
}

// ── Invalid URL ───────────────────────────────────────────────────────────────

#[test]
fn rpc_client_rejects_invalid_url() {
    let err = RpcClient::new("not_a_url").unwrap_err();
    assert!(matches!(err, RpcError::InvalidUrl(_)));
}
