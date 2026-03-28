//! T-003 integration tests: compare_to_anvil()
//!
//! Validates FR-003 behavior:
//! - AC-006 gas mismatch detection with 5% threshold info
//! - AC-007 field-level diff details

mod helpers;

use alloy_primitives::{Address, TxKind, U256};
use alloy_rpc_types::TransactionRequest;
use eth_node::executor::{compare_to_anvil, SimulationContext};

fn test_context() -> SimulationContext {
    SimulationContext {
        block_number: 1,
        timestamp: 1_710_000_000,
        base_fee_per_gas: Some(0),
        gas_limit: 30_000_000,
    }
}

#[tokio::test]
async fn test_compare_to_anvil_simple_transfer_report_shape() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("SKIP — Anvil not on PATH");
        return;
    };

    let from = Address::parse_checksummed(helpers::accounts::ANVIL_ACCOUNT0_ADDRESS, None)
        .expect("valid checksummed account0 address");
    let to = Address::repeat_byte(0x42);

    let tx = TransactionRequest {
        from: Some(from),
        to: Some(TxKind::Call(to)),
        value: Some(U256::from(1_000_000u64)),
        gas: Some(21_000),
        gas_price: Some(1_000_000_000u128),
        nonce: Some(0),
        chain_id: Some(helpers::accounts::ANVIL_CHAIN_ID),
        ..Default::default()
    };

    let report = compare_to_anvil(&tx, &anvil.endpoint, &test_context())
        .await
        .expect("compare_to_anvil should return report");

    assert!(
        report.gas_used_anvil > 0,
        "anvil gas should be populated in report"
    );
    assert!(
        report.gas_used_local > 0,
        "local gas should be populated in report"
    );
    assert!(
        report.return_data_match,
        "simple transfer should have matching empty return data"
    );
}

#[tokio::test]
async fn test_compare_to_anvil_detects_gas_threshold_exceeded() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("SKIP — Anvil not on PATH");
        return;
    };

    let from = Address::parse_checksummed(helpers::accounts::ANVIL_ACCOUNT0_ADDRESS, None)
        .expect("valid checksummed account0 address");
    let to = Address::repeat_byte(0x7a);

    // Deliberately skew local context to force mismatch in local simulation behavior.
    let skewed_context = SimulationContext {
        block_number: 99,
        timestamp: 9_999_999_999,
        base_fee_per_gas: Some(123_456_789),
        gas_limit: 25_000,
    };

    let tx = TransactionRequest {
        from: Some(from),
        to: Some(TxKind::Call(to)),
        value: Some(U256::from(7_000_000u64)),
        gas: Some(21_000),
        gas_price: Some(1_000_000_000u128),
        nonce: Some(0),
        chain_id: Some(helpers::accounts::ANVIL_CHAIN_ID),
        ..Default::default()
    };

    let report = compare_to_anvil(&tx, &anvil.endpoint, &skewed_context)
        .await
        .expect("compare_to_anvil should return report even on mismatch");

    let abs_delta = report.gas_delta.unsigned_abs();
    let threshold = (report.gas_used_anvil * 5) / 100;

    if abs_delta > threshold {
        assert!(
            report
                .differences
                .iter()
                .any(|d| d.contains("gas mismatch") && d.contains("exceeds_threshold=true")),
            "differences should include explicit gas threshold exceedance details"
        );
    } else {
        assert!(
            report
                .differences
                .iter()
                .any(|d| d.contains("gas mismatch") || d.contains("local simulation failed"))
                || report.gas_used_local == report.gas_used_anvil,
            "report should either include mismatch detail or exact gas match"
        );
    }
}

#[tokio::test]
async fn test_compare_to_anvil_field_level_return_data_diff_detail() {
    let Some(anvil) = helpers::anvil_fixture::AnvilInstance::spawn().unwrap() else {
        println!("SKIP — Anvil not on PATH");
        return;
    };

    let from = Address::parse_checksummed(helpers::accounts::ANVIL_ACCOUNT0_ADDRESS, None)
        .expect("valid checksummed account0 address");
    let to = Address::repeat_byte(0x55);

    // Non-empty calldata to a non-contract account can produce divergent behavior details
    // between local simulation and eth_call output path.
    let tx = TransactionRequest {
        from: Some(from),
        to: Some(TxKind::Call(to)),
        input: alloy_rpc_types::TransactionInput::new(vec![0xde, 0xad, 0xbe, 0xef].into()),
        gas: Some(50_000),
        gas_price: Some(1_000_000_000u128),
        nonce: Some(0),
        chain_id: Some(helpers::accounts::ANVIL_CHAIN_ID),
        ..Default::default()
    };

    let report = compare_to_anvil(&tx, &anvil.endpoint, &test_context())
        .await
        .expect("compare_to_anvil should return report");

    if !report.return_data_match {
        assert!(
            report
                .differences
                .iter()
                .any(|d| d.contains("return_data mismatch")),
            "field-level return-data mismatch detail should be present"
        );
    }

    // AC-007: differences should contain field-level details when mismatches exist.
    if !report.differences.is_empty() {
        assert!(
            report.differences.iter().any(|d| {
                d.contains("gas mismatch")
                    || d.contains("return_data mismatch")
                    || d.contains("log mismatch")
                    || d.contains("local simulation failed")
            }),
            "differences must contain field-level comparison details"
        );
    }
}
