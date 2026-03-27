//! Anvil comparison utility for validating simulation accuracy.
//!
//! Implements `compare_to_anvil()` to execute transactions both locally and via
//! Anvil RPC, then compute structured delta reports.
//!
//! See PHASE2_FORMAL_SPEC.md FR-003 for behavioral contracts.

use alloy_rpc_types::TransactionRequest;

use super::{ExecutorError, SimulationContext};

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
    _tx: &TransactionRequest,
    _anvil_rpc_url: &str,
    _context: &SimulationContext,
) -> Result<ComparisonReport, ExecutorError> {
    // T-003: Implementation placeholder
    // Will integrate with Phase 1 tx::send_transaction for Anvil broadcast
    // Will use simulate_tx for local execution
    unimplemented!("T-003: compare_to_anvil not yet implemented")
}
