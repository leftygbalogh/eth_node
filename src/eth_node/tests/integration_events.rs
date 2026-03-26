//! Integration tests for the event/log listener module.
//!
//! Spec ref: FORMAL_SPEC.md §4 FR-006, NFR-006
//!
//! # Minimal emitter bytecode
//!
//! `EMITTER_INIT_CODE` is hand-assembled EVM init code that:
//!
//!   1. Emits a `LOG0` with no topics and no data (cheap, dependency-free event).
//!   2. Returns **empty** runtime code so the deployed account has no callable
//!      functions (safe for test purposes).
//!
//! Assembly (10 bytes):
//! ```text
//! PUSH1 0x00   // length = 0 (LOG0 data size)
//! PUSH1 0x00   // offset = 0 (LOG0 data start in memory)
//! LOG0         // emit log — no topics, no data
//! PUSH1 0x00   // length = 0 (runtime code size)
//! PUSH1 0x00   // offset = 0
//! RETURN       // halt, returning empty runtime
//! ```

mod helpers;

use std::time::Duration;

use alloy_consensus::TxEip1559;
use alloy_primitives::{TxKind, U256};
use alloy_rpc_types::Filter;
use eth_node::events::Listener;
use eth_node::rpc::RpcClient;
use eth_node::signer::{EthSigner, UnsignedTx};
use eth_node::tx::Broadcaster;
use futures::StreamExt;
use helpers::accounts::ANVIL_ACCOUNT0_KEY;
use helpers::anvil_fixture::AnvilInstance;

/// EVM init code: emits one LOG0 (no topics, no data), returns empty runtime.
/// See module-level comment for the detailed disassembly.
const EMITTER_INIT_CODE: &[u8] = &[
    0x60, 0x00, // PUSH1 0x00  — data length for LOG0
    0x60, 0x00, // PUSH1 0x00  — memory offset for LOG0
    0xa0, // LOG0        — emit anonymous log
    0x60, 0x00, // PUSH1 0x00  — size of runtime code = 0
    0x60, 0x00, // PUSH1 0x00  — offset in memory
    0xf3, // RETURN      — return empty bytes as runtime
];

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Skip the test if Anvil is not available; return the running instance.
macro_rules! require_anvil {
    () => {{
        match AnvilInstance::spawn().expect("failed to spawn anvil") {
            None => {
                eprintln!("anvil not on PATH — skipping test");
                return;
            }
            Some(a) => a,
        }
    }};
}

/// Deploy the minimal emitter contract and return the contract address.
///
/// The constructor emits one `LOG0` which can be detected by a log filter.
async fn deploy_emitter(client: &RpcClient, signer: &EthSigner) -> alloy_primitives::Address {
    let from = signer.address();
    let nonce = client.get_nonce(from).await.expect("get_nonce");
    let gas_price = client.gas_price().await.expect("gas_price");
    let chain_id = client.chain_id().await.expect("chain_id");

    let tx = TxEip1559 {
        chain_id,
        nonce,
        max_fee_per_gas: gas_price * 2,
        max_priority_fee_per_gas: gas_price,
        gas_limit: 100_000, // generous limit for 10-byte init code
        to: TxKind::Create,
        value: U256::ZERO,
        input: EMITTER_INIT_CODE.to_vec().into(),
        ..Default::default()
    };

    let unsigned = UnsignedTx::Eip1559(tx);
    let signed = signer.sign(unsigned).expect("sign deploy tx");
    let receipt = Broadcaster::new()
        .send(&signed, client)
        .await
        .expect("deploy broadcast");

    assert!(receipt.status(), "deployer constructor reverted");
    receipt.contract_address.expect("no contract_address in deploy receipt")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// HTTP poll: run the stream for ~1 s with no matching contracts deployed;
/// expect no errors and no log items.
#[tokio::test]
async fn test_http_poll_no_errors_when_no_logs() {
    let anvil = require_anvil!();

    let listener =
        Listener::new(&anvil.endpoint).with_poll_interval(Duration::from_millis(300));

    let mut stream = listener.subscribe(Filter::new());

    // Drain the stream for 1 second; any error item is a failure.
    let window = Duration::from_millis(1_000);
    while let Ok(item) = tokio::time::timeout(window, stream.next()).await {
        if let Some(Err(e)) = item {
            panic!("unexpected error from poll stream: {e}");
        }
    }
}

/// WebSocket: stream for ~1 s with no events deployed; expect no errors.
#[tokio::test]
async fn test_ws_subscribe_no_errors_when_no_logs() {
    let anvil = require_anvil!();

    let listener = Listener::new(anvil.ws_endpoint());
    let mut stream = listener.subscribe(Filter::new());

    let window = Duration::from_millis(1_000);
    while let Ok(item) = tokio::time::timeout(window, stream.next()).await {
        if let Some(Err(e)) = item {
            panic!("unexpected error from WS stream: {e}");
        }
    }
}

/// HTTP poll: deploy the log-emitting contract and verify the poll stream
/// delivers the constructor log.
#[tokio::test]
async fn test_http_poll_receives_constructor_log() {
    let anvil = require_anvil!();
    let client = RpcClient::new(&anvil.endpoint).expect("rpc client");
    let signer = EthSigner::from_key(ANVIL_ACCOUNT0_KEY).expect("signer");

    // Note the current block *before* deployment so our filter starts there.
    let from_block = client.block_number().await.expect("block_number");

    let contract_addr = deploy_emitter(&client, &signer).await;

    // Filter: any log from this contract, starting at the deploy block.
    let filter = Filter::new().address(contract_addr).from_block(from_block);

    let listener =
        Listener::new(&anvil.endpoint).with_poll_interval(Duration::from_millis(300));
    let mut stream = listener.subscribe(filter);

    // Expect to receive one log within 5 seconds.
    let log = tokio::time::timeout(Duration::from_secs(5), stream.next())
        .await
        .expect("timed out waiting for log")
        .expect("stream ended unexpectedly")
        .expect("stream yielded an error");

    assert_eq!(log.address(), contract_addr, "log came from wrong address");
}

/// WebSocket: subscribe before deployment, deploy the emitter, then verify the
/// WS subscription delivers the constructor log.
///
/// # Why the background task?
///
/// `ws_subscription_stream` uses `async_stream::stream!` which is **lazy** —
/// the WS TCP connection and `eth_subscribe` call only execute when the stream
/// is first polled.  If we awaited on `stream.next()` only after deploying the
/// emitter the subscription would open *after* the LOG0 was already mined and
/// would never see it.
///
/// Spawning the stream onto its own task drives the first poll immediately, so
/// the WS handshake and subscription complete before `deploy_emitter` is called.
#[tokio::test]
async fn test_ws_subscribe_receives_constructor_log() {
    let anvil = require_anvil!();
    let client = RpcClient::new(&anvil.endpoint).expect("rpc client");
    let signer = EthSigner::from_key(ANVIL_ACCOUNT0_KEY).expect("signer");

    // Create the subscription stream.
    let filter = Filter::new(); // match any log
    let listener = Listener::new(anvil.ws_endpoint());
    let mut stream = listener.subscribe(filter);

    // Drive the stream on a background task so the WS connection and
    // `eth_subscribe` call are established BEFORE we deploy the emitter.
    let (tx, mut rx) = tokio::sync::mpsc::channel(8);
    tokio::spawn(async move {
        while let Some(item) = stream.next().await {
            let _ = tx.send(item).await;
        }
    });

    // Give the WS handshake and eth_subscribe time to complete.
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Deploy — constructor emits one LOG0.
    let _contract_addr = deploy_emitter(&client, &signer).await;

    // Expect to receive the log within 5 seconds.
    let result = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("timed out waiting for WS log")
        .expect("background task channel closed");

    assert!(result.is_ok(), "stream yielded an error: {:?}", result.err());
}

