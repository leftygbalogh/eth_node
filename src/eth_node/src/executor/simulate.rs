//! Transaction simulation engine.
//!
//! Provides simulate_tx() for full transaction execution and simulate_contract_call() for read-only calls.
//! See PHASE2_FORMAL_SPEC.md FR-001 for behavioral contracts.

use alloy_primitives::{Address, Bytes, TxKind, U256};
use alloy_rpc_types::TransactionRequest;
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{
        AccountInfo, BlockEnv, Env, ExecutionResult as RevmExecutionResult, Output, TxEnv,
        TransactTo,
    },
    Evm,
};
use std::collections::HashMap;
use thiserror::Error;

/// Executor-specific errors.
#[derive(Debug, Error)]
pub enum ExecutorError {
    /// Invalid transaction input (signature, nonce, gas limit).
    #[error("invalid transaction input: {0}")]
    InvalidInput(String),

    /// Revm execution failure (revert, out-of-gas, precompile error).
    #[error("revm execution failure: {0}")]
    RevmFailure(String),

    /// Context-carrying error variant (per architect recommendation R2).
    /// Used when additional diagnostic context is needed beyond error message.
    #[error("execution error: {message}")]
    Context {
        message: String,
        /// Optional fields for debugging (e.g., block number, tx hash, gas used).
        context: HashMap<String, String>,
    },
}

/// Simulation context (decouples from revm BlockEnv per architect recommendation R1).
#[derive(Debug, Clone)]
pub struct SimulationContext {
    pub block_number: u64,
    pub timestamp: u64,
    pub base_fee_per_gas: Option<u64>,
    pub gas_limit: u64,
}

impl From<SimulationContext> for BlockEnv {
    fn from(ctx: SimulationContext) -> Self {
        BlockEnv {
            number: U256::from(ctx.block_number),
            timestamp: U256::from(ctx.timestamp),
            basefee: U256::from(ctx.base_fee_per_gas.unwrap_or(0)),
            gas_limit: U256::from(ctx.gas_limit),
            ..Default::default()
        }
    }
}

/// Result of transaction simulation.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// Gas used by the transaction.
    pub gas_used: u64,
    /// Return data (empty for transfers, ABI-encoded for contract calls).
    pub return_data: Bytes,
    /// Emitted logs (raw revm logs).
    pub logs: Vec<revm::primitives::Log>,
    /// Success flag (true if execution succeeded, false if reverted).
    pub success: bool,
}

/// Simulate transaction execution using revm.
///
/// # Arguments
/// - `tx`: TransactionRequest to simulate (from alloy).
/// - `context`: SimulationContext with block number, timestamp, base fee, gas limit.
///
/// # Returns
/// - `Ok(SimulationResult)` if simulation completes (even if tx reverts).
/// - `Err(ExecutorError)` if input is invalid or revm fails catastrophically.
///
/// # Example
/// ```ignore
/// use eth_node::executor::{simulate_tx, SimulationContext};
/// use alloy_rpc_types::TransactionRequest;
/// use alloy_primitives::{Address, U256};
///
/// let tx = TransactionRequest {
///     from: Some(Address::ZERO),
///     to: Some(Address::ZERO.into()),
///     value: Some(U256::from(1000)),
///     ..Default::default()
/// };
/// let context = SimulationContext {
///     block_number: 1,
///     timestamp: 1710000000,
///     base_fee_per_gas: Some(10),
///     gas_limit: 30_000_000,
/// };
/// let result = simulate_tx(&tx, &context)?;
/// assert!(result.success);
/// ```
pub fn simulate_tx(
    tx: &TransactionRequest,
    context: &SimulationContext,
) -> Result<SimulationResult, ExecutorError> {
    // Validate required fields
    let from = tx
        .from
        .ok_or_else(|| ExecutorError::InvalidInput("missing 'from' address".into()))?;
    let gas_limit = tx
        .gas
        .ok_or_else(|| ExecutorError::InvalidInput("missing 'gas' limit".into()))?;

    if gas_limit == 0 {
        return Err(ExecutorError::InvalidInput("gas limit cannot be zero".into()));
    }

    // Initialize revm with in-memory database
    let mut cache_db = CacheDB::new(EmptyDB::default());

    // Fund sender account (for testing; Phase 3 will use proper state provider)
    let sender_balance = U256::from(1_000_000_000_000_000_000_u128); // 1 ETH in wei
    cache_db.insert_account_info(
        from,
        AccountInfo {
            balance: sender_balance,
            nonce: tx.nonce.unwrap_or(0),
            ..Default::default()
        },
    );

    // If deploying or calling a contract, ensure target account exists
    if let Some(TxKind::Call(to)) = tx.to {
        // For contract calls, ensure target exists (empty account is fine)
        if !cache_db.accounts.contains_key(&to) {
            cache_db.insert_account_info(
                to,
                AccountInfo {
                    balance: U256::ZERO,
                    nonce: 0,
                    ..Default::default()
                },
            );
        }
    }

    // Build transaction environment
    let tx_env = TxEnv {
        caller: from,
        transact_to: match tx.to {
            Some(TxKind::Call(addr)) => TransactTo::Call(addr),
            Some(TxKind::Create) | None => TransactTo::Create,
        },
        value: tx.value.unwrap_or(U256::ZERO),
        data: tx.input.input().cloned().unwrap_or_default(),
        gas_limit: gas_limit,
        gas_price: U256::from(tx.gas_price.unwrap_or(0)),
        nonce: tx.nonce,
        ..Default::default()
    };

    // Build environment
    let mut env = Env::default();
    env.block = context.clone().into();
    env.tx = tx_env;

    // Execute transaction
    let mut evm = Evm::builder()
        .with_db(cache_db)
        .with_env(Box::new(env))
        .build();

    let result = evm
        .transact()
        .map_err(|e| ExecutorError::RevmFailure(format!("transaction execution failed: {:?}", e)))?;

    // Map revm result to SimulationResult
    match result.result {
        RevmExecutionResult::Success {
            gas_used,
            logs,
            output,
            ..
        } => {
            let return_data = match output {
                Output::Call(data) => data,
                Output::Create(data, _) => data,
            };
            Ok(SimulationResult {
                gas_used,
                return_data,
                logs,
                success: true,
            })
        }
        RevmExecutionResult::Revert { gas_used, output } => Ok(SimulationResult {
            gas_used,
            return_data: output,
            logs: Vec::new(),
            success: false,
        }),
        RevmExecutionResult::Halt { reason, gas_used } => {
            Err(ExecutorError::RevmFailure(format!(
                "transaction halted: {:?}, gas used: {}",
                reason, gas_used
            )))
        }
    }
}

/// Simulate read-only contract call using revm.
///
/// Executes a static call (like eth_call) without modifying state.
///
/// # Arguments
/// - `contract_address`: Target contract address.
/// - `calldata`: ABI-encoded function call.
/// - `context`: SimulationContext for block environment.
///
/// # Returns
/// - `Ok(Bytes)` with ABI-encoded return data if call succeeds.
/// - `Err(ExecutorError::InvalidInput)` if calldata is invalid.
/// - `Err(ExecutorError::RevmFailure)` if call reverts or halts.
///
/// # Example
/// ```ignore
/// // Call ERC-20 balanceOf(address)
/// let selector = hex::decode("70a08231").unwrap();
/// let account_bytes = account.as_slice();
/// let mut calldata = selector;
/// calldata.extend_from_slice(&[0u8; 12]); // Pad to 32 bytes
/// calldata.extend_from_slice(account_bytes);
///
/// let result = simulate_contract_call(token_address, Bytes::from(calldata), &context)?;
/// // Decode result as uint256
/// ```
pub fn simulate_contract_call(
    contract_address: Address,
    calldata: Bytes,
    context: &SimulationContext,
) -> Result<Bytes, ExecutorError> {
    // Validate calldata (must be at least 4 bytes for function selector)
    if calldata.len() < 4 {
        return Err(ExecutorError::InvalidInput(format!(
            "calldata too short: {} bytes (minimum 4 for selector)",
            calldata.len()
        )));
    }

    // Initialize revm with in-memory database
    let mut cache_db = CacheDB::new(EmptyDB::default());

    // Ensure contract account exists
    // Note: In Phase 2 we use empty code; Phase 3 will load actual contract code
    // via StateProvider. For now, calls to empty contracts will return empty data.
    cache_db.insert_account_info(
        contract_address,
        AccountInfo {
            balance: U256::ZERO,
            nonce: 0,
            ..Default::default()
        },
    );

    // Static call from zero address (standard for eth_call)
    let caller = Address::ZERO;

    // Build transaction environment for static call
    let tx_env = TxEnv {
        caller,
        transact_to: TransactTo::Call(contract_address),
        value: U256::ZERO, // Static calls have no value
        data: calldata.clone(),
        gas_limit: 30_000_000, // High limit for read calls
        gas_price: U256::ZERO, // No cost for static calls
        nonce: None,           // Static calls don't increment nonce
        ..Default::default()
    };

    // Build environment
    let mut env = Env::default();
    env.block = context.clone().into();
    env.tx = tx_env;

    // Execute static call
    let mut evm = Evm::builder()
        .with_db(cache_db)
        .with_env(Box::new(env))
        .build();

    let result = evm.transact().map_err(|e| {
        ExecutorError::RevmFailure(format!("contract call execution failed: {:?}", e))
    })?;

    // Map result
    match result.result {
        RevmExecutionResult::Success { output, .. } => {
            let return_data = match output {
                Output::Call(data) => data,
                Output::Create(_, _) => {
                    return Err(ExecutorError::InvalidInput(
                        "static call should not create contract".into(),
                    ))
                }
            };
            Ok(return_data)
        }
        RevmExecutionResult::Revert { output, .. } => Err(ExecutorError::RevmFailure(format!(
            "contract call reverted: {}",
            alloy_primitives::hex::encode(&output)
        ))),
        RevmExecutionResult::Halt { reason, .. } => Err(ExecutorError::RevmFailure(format!(
            "contract call halted: {:?}",
            reason
        ))),
    }
}


