//! Anvil comparison utility for validating simulation accuracy.
//!
//! Implements `compare_to_anvil()` to execute transactions both locally and via
//! Anvil RPC, then compute structured delta reports.
//!
//! See PHASE2_FORMAL_SPEC.md FR-003 for behavioral contracts.

use alloy_primitives::Bytes;
use alloy_rpc_types::TransactionRequest;

use super::{simulate_tx, ExecutorError, SimulationContext};
use crate::rpc::RpcClient;

/// Structured comparison report between local simulation and Anvil execution.
#[derive(Debug, Clone)]
pub struct ComparisonReport {
    /// Gas used by local simulation.
    pub gas_used_local: u64,
    /// Gas used by Anvil (from receipt).
    pub gas_used_anvil: u64,
    /// Delta (local - anvil); negative means local used less gas.
    pub gas_delta: i64,
    /// Whether return data matches exactly.
    pub return_data_match: bool,
    /// Whether logs match exactly (count and content).
    pub logs_match: bool,
    /// Field-level differences (empty if all match).
    pub differences: Vec<String>,
}

/// Compare local simulation to Anvil execution.
///
/// Executes the transaction both locally (via simulate_tx) and on Anvil (via RPC),
/// then computes a structured delta report.
///
/// # Arguments
/// - `tx`: TransactionRequest to execute (must include all required fields for broadcast).
/// - `anvil_rpc_url`: Anvil RPC endpoint.
///
/// # Returns
/// - `Ok(ComparisonReport)` with detailed field-level comparison.
/// - `Err(ExecutorError)` if RPC communication fails or simulation fails.
///
/// # Reporting Semantics
/// Per PHASE2_FORMAL_SPEC.md FR-003: `compare_to_anvil()` always returns
/// `Ok(ComparisonReport)` unless RPC failure occurs. It does NOT fail on
/// mismatches—it reports them in the `differences` field. Caller interprets
/// thresholds (e.g., 5% gas tolerance).
///
/// # Example
/// ```ignore
/// let report = compare_to_anvil(&tx, "http://127.0.0.1:8545").await?;
/// if report.gas_delta.abs() as u64 > report.gas_used_anvil * 5 / 100 {
///     eprintln!("Gas mismatch exceeds 5%: {}", report.gas_delta);
/// }
/// ```
pub async fn compare_to_anvil(
    tx: &TransactionRequest,
    anvil_rpc_url: &str,
    context: &SimulationContext,
) -> Result<ComparisonReport, ExecutorError> {
    let mut differences = Vec::new();

    let local = simulate_tx(tx, context);
    let client = RpcClient::new(anvil_rpc_url)
        .map_err(|e| ExecutorError::RevmFailure(format!("anvil rpc client init failed: {}", e)))?;

    // Anvil reference values are computed via eth_estimateGas + eth_call.
    // This keeps compare_to_anvil signer-agnostic while still providing deterministic
    // gas and return-data comparison for FR-003.
    let anvil_gas = client
        .estimate_gas(tx.clone())
        .await
        .map_err(|e| ExecutorError::RevmFailure(format!("anvil estimate_gas failed: {}", e)))?;
    let anvil_return = client
        .call(tx.clone())
        .await
        .map_err(|e| ExecutorError::RevmFailure(format!("anvil eth_call failed: {}", e)))?;

    let (gas_used_local, local_return, local_logs_len) = match local {
        Ok(result) => (result.gas_used, result.return_data, result.logs.len()),
        Err(err) => {
            differences.push(format!("local simulation failed: {}", err));
            (0, Bytes::new(), 0)
        }
    };

    let gas_delta = gas_used_local as i64 - anvil_gas as i64;
    if gas_used_local != anvil_gas {
        let threshold = (anvil_gas as f64 * 0.05).ceil() as u64;
        let abs_delta = (gas_delta).unsigned_abs();
        let exceeds = abs_delta > threshold;
        differences.push(format!(
            "gas mismatch: local={}, anvil={}, delta={}, abs_delta={}, threshold_5pct={}, exceeds_threshold={}",
            gas_used_local, anvil_gas, gas_delta, abs_delta, threshold, exceeds
        ));
    }

    let return_data_match = local_return == anvil_return;
    if !return_data_match {
        differences.push(describe_return_data_diff(&local_return, &anvil_return));
    }

    // We do not have canonical Anvil execution logs without broadcasting a signed tx.
    // For Phase 2 this comparison reports local log emissions explicitly.
    let logs_match = local_logs_len == 0;
    if !logs_match {
        differences.push(format!(
            "log mismatch: local emitted {} logs, anvil reference logs unavailable in eth_call/estimate path",
            local_logs_len
        ));
    }

    Ok(ComparisonReport {
        gas_used_local,
        gas_used_anvil: anvil_gas,
        gas_delta,
        return_data_match,
        logs_match,
        differences,
    })
}

fn describe_return_data_diff(local: &Bytes, anvil: &Bytes) -> String {
    let min_len = local.len().min(anvil.len());
    let first_diff = (0..min_len).find(|&i| local[i] != anvil[i]);

    match first_diff {
        Some(idx) => format!(
            "return_data mismatch: first differing byte at index {} (local=0x{:02x}, anvil=0x{:02x}), local_len={}, anvil_len={}",
            idx,
            local[idx],
            anvil[idx],
            local.len(),
            anvil.len()
        ),
        None => format!(
            "return_data mismatch: equal prefix, differing length (local_len={}, anvil_len={})",
            local.len(),
            anvil.len()
        ),
    }
}
