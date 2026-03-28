//! T-005 fuzz helpers.
//!
//! Shared strategy/building helpers for feature-gated fuzz properties.

use alloy_primitives::{Address, B256, Bytes, Log as PrimitiveLog, TxKind, U256};
use alloy_rpc_types::{Log, TransactionInput, TransactionRequest};

use crate::executor::SimulationContext;

pub fn make_tx_request(
    from_bytes: [u8; 20],
    to_bytes: [u8; 20],
    calldata: Vec<u8>,
    value_raw: [u8; 32],
    gas: u64,
    gas_price: u128,
    nonce: u64,
    use_create: bool,
) -> TransactionRequest {
    let from = Address::from(from_bytes);
    let to_address = Address::from(to_bytes);

    TransactionRequest {
        from: Some(from),
        to: Some(if use_create {
            TxKind::Create
        } else {
            TxKind::Call(to_address)
        }),
        value: Some(U256::from_be_slice(&value_raw)),
        gas: Some(gas),
        gas_price: Some(gas_price),
        nonce: Some(nonce),
        input: TransactionInput::new(Bytes::from(calldata)),
        chain_id: Some(31337),
        ..Default::default()
    }
}

pub fn make_context(block_number: u64, timestamp: u64, base_fee_per_gas: Option<u64>, gas_limit: u64) -> SimulationContext {
    SimulationContext {
        block_number,
        timestamp,
        base_fee_per_gas,
        gas_limit,
    }
}

pub fn make_log(address_bytes: [u8; 20], topics: Vec<[u8; 32]>, data: Vec<u8>) -> Log {
    let address = Address::from(address_bytes);
    let topics = topics.into_iter().map(B256::from).collect();
    Log {
        inner: PrimitiveLog::new_unchecked(address, topics, Bytes::from(data)),
        ..Default::default()
    }
}
