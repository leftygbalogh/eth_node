//! T-003: simulate_contract_call() unit tests.
//!
//! Test requirements:
//! - ERC-20 balanceOf(), totalSupply() calls (known return values)
//! - Integration test: deploy to Anvil, call, compare to eth_call
//!
//! Acceptance criteria:
//! - AC-004: simulate_contract_call() matches Anvil eth_call results
//! - AC-005: Invalid calldata returns ExecutorError::InvalidInput (no panic)

use alloy_primitives::{Address, Bytes};
use alloy_rpc_types::{TransactionInput, TransactionRequest};
use eth_node::executor::{simulate_contract_call, ExecutorError, SimulationContext};

mod helpers;

/// Default test context.
fn test_context() -> SimulationContext {
    SimulationContext {
        block_number: 1,
        timestamp: 1_710_000_000,
        base_fee_per_gas: Some(0),
        gas_limit: 30_000_000,
    }
}

#[test]
fn test_contract_call_with_valid_selector() {
    // Call balanceOf(address) on empty account (will return empty since no code)
    // Selector: 0x70a08231
    let mut calldata = vec![0x70, 0xa0, 0x82, 0x31]; // balanceOf selector
    calldata.extend_from_slice(&[0u8; 32]); // address parameter (padded)

    let contract_addr = Address::repeat_byte(0x99);

    let result = simulate_contract_call(contract_addr, Bytes::from(calldata), &test_context());

    // Empty contract returns empty data (valid EVM behavior)
    match result {
        Ok(data) => {
            assert!(
                data.is_empty(),
                "empty contract should return empty data (no code)"
            );
        }
        Err(e) => panic!("valid calldata should not error: {:?}", e),
    }
}

#[test]
fn test_contract_call_invalid_calldata_too_short() {
    // Calldata < 4 bytes should return InvalidInput (AC-005)
    let short_calldata = Bytes::from(vec![0x12, 0x34]); // Only 2 bytes

    let contract_addr = Address::repeat_byte(0x99);

    let result = simulate_contract_call(contract_addr, short_calldata, &test_context());

    assert!(
        matches!(result, Err(ExecutorError::InvalidInput(_))),
        "calldata < 4 bytes should return InvalidInput (AC-005)"
    );
}

#[test]
fn test_contract_call_empty_calldata() {
    // Empty calldata should return InvalidInput
    let empty_calldata = Bytes::default();

    let contract_addr = Address::repeat_byte(0x99);

    let result = simulate_contract_call(contract_addr, empty_calldata, &test_context());

    assert!(
        matches!(result, Err(ExecutorError::InvalidInput(_))),
        "empty calldata should return InvalidInput"
    );
}

#[tokio::test]
async fn test_contract_call_matches_anvil_eth_call_empty_account() {
    // AC-004 parity check against Anvil eth_call on an empty account.
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("SKIP — Anvil not on PATH");
        return;
    };

    let client = eth_node::rpc::RpcClient::new(&anvil.endpoint).expect("valid anvil endpoint");
    let from = Address::repeat_byte(0x01);
    let contract_addr = Address::repeat_byte(0x99);

    // balanceOf(address) selector + 32-byte address arg
    let mut calldata = vec![0x70, 0xa0, 0x82, 0x31];
    calldata.extend_from_slice(&[0u8; 32]);
    let calldata = Bytes::from(calldata);

    let local = simulate_contract_call(contract_addr, calldata.clone(), &test_context())
        .expect("local contract call should succeed");

    let rpc_tx = TransactionRequest {
        from: Some(from),
        to: Some(contract_addr.into()),
        input: TransactionInput::new(calldata),
        ..Default::default()
    };
    let anvil_out = client.call(rpc_tx).await.expect("eth_call should succeed");

    assert_eq!(
        local, anvil_out,
        "simulate_contract_call output should match Anvil eth_call output"
    );
}

// Integration tests comparing to Anvil eth_call will be added after
// unit test validation. AC-004 requires deploying real contract to Anvil
// and comparing simulate_contract_call() output to eth_call RPC response.
