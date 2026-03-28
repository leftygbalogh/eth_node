#![cfg(feature = "fuzz")]
//! T-005 feature-gated fuzz properties.
//!
//! Run with:
//! `cargo test --package eth_node --features fuzz --test fuzz_properties`

use std::panic::{catch_unwind, AssertUnwindSafe};

use alloy_primitives::{Address, U256};
use eth_node::{
    executor::simulate_tx,
    primitives::{abi_decode_address, abi_decode_tuple, abi_decode_uint256, abi_encode_address, abi_encode_tuple, abi_encode_uint256, AbiTuple},
    quality::{decode_standard_nft_event, fuzz::{make_context, make_log, make_tx_request}},
};
use proptest::prelude::*;
use proptest::test_runner::Config;

fn fuzz_config() -> Config {
    Config {
        cases: 10_000,
        max_shrink_iters: 0,
        ..Config::default()
    }
}

proptest! {
    #![proptest_config(fuzz_config())]

    #[test]
    fn simulate_tx_never_panics_on_random_inputs(
        from in any::<[u8; 20]>(),
        to in any::<[u8; 20]>(),
        calldata in proptest::collection::vec(any::<u8>(), 0..10_000),
        value_raw in any::<[u8; 32]>(),
        gas in prop_oneof![Just(0u64), Just(u64::MAX), 21_000u64..500_000u64],
        gas_price in prop_oneof![Just(0u128), 1u128..10_000_000_000_000u128],
        nonce in any::<u64>(),
        use_create in any::<bool>(),
        block_number in any::<u64>(),
        timestamp in any::<u64>(),
        base_fee in prop_oneof![Just(None), any::<u64>().prop_map(Some)],
        block_gas_limit in prop_oneof![Just(21_000u64), 21_000u64..30_000_000u64],
    ) {
        let tx = make_tx_request(from, to, calldata, value_raw, gas, gas_price, nonce, use_create);
        let context = make_context(block_number, timestamp, base_fee, block_gas_limit);

        let result = catch_unwind(AssertUnwindSafe(|| simulate_tx(&tx, &context)));
        prop_assert!(result.is_ok(), "simulate_tx panicked on fuzz input");
    }
}

proptest! {
    #![proptest_config(fuzz_config())]

    #[test]
    fn nft_decoder_never_panics_on_random_logs(
        address in any::<[u8; 20]>(),
        topics in proptest::collection::vec(any::<[u8; 32]>(), 0..4),
        data in proptest::collection::vec(any::<u8>(), 0..10_000),
    ) {
        let log = make_log(address, topics, data);
        let result = catch_unwind(AssertUnwindSafe(|| decode_standard_nft_event(&log)));
        prop_assert!(result.is_ok(), "decode_standard_nft_event panicked on fuzz log");
    }
}

proptest! {
    #![proptest_config(fuzz_config())]

    #[test]
    fn abi_roundtrip_preserves_values(
        uint_raw in any::<[u8; 32]>(),
        address_raw in any::<[u8; 20]>(),
        flag in any::<bool>(),
    ) {
        let uint = U256::from_be_slice(&uint_raw);
        let address = Address::from(address_raw);
        let tuple = AbiTuple {
            a: uint,
            b: address,
            c: flag,
        };

        let encoded_uint = abi_encode_uint256(uint);
        let encoded_address = abi_encode_address(address);
        let encoded_tuple = abi_encode_tuple(tuple.clone());

        let decoded_uint = abi_decode_uint256(&encoded_uint).expect("uint256 roundtrip decode should succeed");
        let decoded_address = abi_decode_address(&encoded_address).expect("address roundtrip decode should succeed");
        let decoded_tuple = abi_decode_tuple(&encoded_tuple).expect("tuple roundtrip decode should succeed");

        prop_assert_eq!(decoded_uint, uint);
        prop_assert_eq!(decoded_address, address);
        prop_assert_eq!(decoded_tuple.a, tuple.a);
        prop_assert_eq!(decoded_tuple.b, tuple.b);
        prop_assert_eq!(decoded_tuple.c, tuple.c);
    }
}
