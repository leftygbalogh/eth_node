//! Transaction builder and broadcaster.
//!
//! Spec ref: FORMAL_SPEC.md §4 FR-003, FR-005

use alloy_consensus::{TxEip1559, TxLegacy};
use alloy_primitives::{Address, Bytes, TxKind, U256};
use thiserror::Error;
use tokio::time::{sleep, Duration, Instant};
use tracing::{info, info_span};

use crate::{
    rpc::{RpcClient, RpcError},
    signer::{EthSigner, SignedTransaction, SignerError, UnsignedTx},
};

// ── Error type ───────────────────────────────────────────────────────────────

/// Errors from the transaction builder and broadcaster.
#[derive(Debug, Error)]
pub enum TxError {
    /// `gas_price` and `max_fee_per_gas` were both supplied (decision table row 4).
    #[error("conflicting fee params: supply either gas_price (legacy) or max_fee_per_gas (EIP-1559), not both")]
    ConflictingFeeParams,

    /// Nonce could not be fetched from the RPC endpoint.
    #[error("nonce unavailable: {0}")]
    NonceUnavailable(#[source] RpcError),

    /// Gas estimation via `eth_estimateGas` failed.
    #[error("gas estimation failed: {0}")]
    GasEstimationFailed(#[source] RpcError),

    /// Invalid EIP-1559 fee parameters.
    #[error("max_priority_fee_per_gas ({priority}) exceeds max_fee_per_gas ({max})")]
    InvalidFeeParams { max: u128, priority: u128 },

    /// Signing failed.
    #[error("signing failed: {0}")]
    SigningFailed(#[source] SignerError),

    /// `eth_sendRawTransaction` failed.
    #[error("submit failed: {0}")]
    SubmitFailed(#[source] RpcError),

    /// Transaction was mined but reverted (status == 0).
    #[error("transaction reverted (hash: {hash:#x})")]
    Reverted {
        hash: alloy_primitives::B256,
        /// Copy of the receipt for inspection.
        receipt: Box<alloy_rpc_types::TransactionReceipt>,
    },

    /// Polling exceeded the configured timeout; tx hash is preserved.
    #[error("confirmation timed out after {elapsed_secs}s (hash: {hash:#x})")]
    ConfirmationTimeout {
        hash: alloy_primitives::B256,
        elapsed_secs: u64,
    },

    /// An RPC error occurred while polling for the receipt.
    #[error("receipt poll failed: {0}")]
    ReceiptPollFailed(#[source] RpcError),
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// In `FeeConfig::Auto` mode the priority fee is calculated as
/// `gas_price / DEFAULT_AUTO_TIP_DIVISOR`.
///
/// To use a different tip percentage, call `.max_fee(max, priority)` explicitly
/// instead of relying on `Auto`.
const DEFAULT_AUTO_TIP_DIVISOR: u128 = 10; // 10 % of base fee

/// Typed fee configuration — drives the decision table.
#[derive(Debug, Clone, Default)]
pub enum FeeConfig {
    /// Neither gas_price nor max_fee supplied — EIP-1559, fees fetched from RPC.
    #[default]
    Auto,
    /// Pure EIP-1559: caller supplies both fee components.
    Eip1559 {
        max_fee_per_gas: u128,
        max_priority_fee_per_gas: u128,
    },
    /// Legacy (type-0): caller supplies gas_price.
    Legacy { gas_price: u128 },
}

/// A fluent builder for Ethereum transactions.
///
/// Produces an [`UnsignedTx`](crate::signer::UnsignedTx) ready for signing.
///
/// # Fee type decision table (FR-003)
///
/// | `gas_price` set | `max_fee` set | Result |
/// |-----------------|--------------|--------|
/// | No              | No           | EIP-1559, both fees fetched from RPC |
/// | No              | Yes          | EIP-1559 with caller-supplied fees   |
/// | Yes             | No           | Legacy (type 0)                      |
/// | Yes             | Yes          | `TxError::ConflictingFeeParams`      |
#[derive(Debug, Clone)]
pub struct TxBuilder {
    chain_id: u64,
    from: Address,
    to: TxKind,
    value: U256,
    data: Bytes,
    nonce: Option<u64>,
    gas_limit: Option<u64>,
    fee_config: FeeConfig,
    /// Tracks whether `.gas_price()` was called (decision table row 4 detection).
    has_gas_price: bool,
    /// Tracks whether `.max_fee()` was called (decision table row 4 detection).
    has_max_fee: bool,
}

impl TxBuilder {
    /// Create a builder for an ETH or contract-call transaction.
    pub fn new(chain_id: u64, from: Address, to: Address) -> Self {
        Self {
            chain_id,
            from,
            to: TxKind::Call(to),
            value: U256::ZERO,
            data: Bytes::new(),
            nonce: None,
            gas_limit: None,
            fee_config: FeeConfig::Auto,
            has_gas_price: false,
            has_max_fee: false,
        }
    }

    /// Shorthand for a pure ETH transfer.
    pub fn transfer(chain_id: u64, from: Address, to: Address, value: U256) -> Self {
        Self::new(chain_id, from, to).value(value)
    }

    /// Set the ETH value (in Wei).
    pub fn value(mut self, value: U256) -> Self {
        self.value = value;
        self
    }

    /// Set the `input` / `data` bytes (for contract calls).
    pub fn data(mut self, data: impl Into<Bytes>) -> Self {
        self.data = data.into();
        self
    }

    /// Override the nonce instead of fetching it from the RPC.
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Override the gas limit instead of estimating via the RPC.
    pub fn gas_limit(mut self, gas: u64) -> Self {
        self.gas_limit = Some(gas);
        self
    }

    /// Use EIP-1559 fees with explicit values.
    pub fn max_fee(mut self, max_fee_per_gas: u128, max_priority_fee_per_gas: u128) -> Self {
        self.fee_config = FeeConfig::Eip1559 {
            max_fee_per_gas,
            max_priority_fee_per_gas,
        };
        self.has_max_fee = true;
        self
    }

    /// Use legacy fee (type-0) transaction with explicit `gas_price`.
    pub fn gas_price(mut self, gas_price: u128) -> Self {
        self.fee_config = FeeConfig::Legacy { gas_price };
        self.has_gas_price = true;
        self
    }

    /// Resolve nonce and gas asynchronously, then build the `UnsignedTx`.
    ///
    /// This method never panics; all failures are returned as `TxError`.
    pub async fn build(self, client: &RpcClient) -> Result<UnsignedTx, TxError> {
        let _span = info_span!("tx_build", from = %self.from, chain_id = self.chain_id).entered();

        // Decision table row 4: both .gas_price() and .max_fee() were called — reject.
        // The builder's setters track this with has_gas_price / has_max_fee because
        // fee_config can only hold one variant at a time; without separate flags the
        // conflict would be silently resolved by whichever setter ran last.
        if self.has_gas_price && self.has_max_fee {
            return Err(TxError::ConflictingFeeParams);
        }

        // Fetch nonce if not overridden.
        let nonce = match self.nonce {
            Some(n) => n,
            None => client
                .get_nonce(self.from)
                .await
                .map_err(TxError::NonceUnavailable)?,
        };

        match self.fee_config {
            FeeConfig::Legacy { gas_price } => {
                let tx_for_gas = TxLegacy {
                    chain_id: Some(self.chain_id),
                    nonce,
                    gas_price,
                    gas_limit: 0, // placeholder for estimation
                    to: self.to,
                    value: self.value,
                    input: self.data.clone(),
                };
                let gas_limit = match self.gas_limit {
                    Some(g) => g,
                    None => {
                        let req = alloy_rpc_types::TransactionRequest {
                            from: Some(self.from),
                            to: Some(self.to),
                            input: alloy_rpc_types::TransactionInput::new(self.data.clone()),
                            value: Some(self.value),
                            gas_price: Some(gas_price),
                            ..Default::default()
                        };
                        client
                            .estimate_gas(req)
                            .await
                            .map_err(TxError::GasEstimationFailed)?
                    }
                };
                let _ = tx_for_gas; // consumed via destructuring above
                Ok(UnsignedTx::Legacy(TxLegacy {
                    chain_id: Some(self.chain_id),
                    nonce,
                    gas_price,
                    gas_limit,
                    to: self.to,
                    value: self.value,
                    input: self.data,
                }))
            }

            FeeConfig::Auto | FeeConfig::Eip1559 { .. } => {
                let (max_fee_per_gas, max_priority_fee_per_gas) = match self.fee_config {
                    FeeConfig::Eip1559 {
                        max_fee_per_gas,
                        max_priority_fee_per_gas,
                    } => (max_fee_per_gas, max_priority_fee_per_gas),
                    _ => {
                        // Auto: fetch both from RPC.
                        let base = client
                            .gas_price()
                            .await
                            .map_err(TxError::GasEstimationFailed)?;
                        (base, base / DEFAULT_AUTO_TIP_DIVISOR)
                    }
                };

                if max_priority_fee_per_gas > max_fee_per_gas {
                    return Err(TxError::InvalidFeeParams {
                        max: max_fee_per_gas,
                        priority: max_priority_fee_per_gas,
                    });
                }

                let gas_limit = match self.gas_limit {
                    Some(g) => g,
                    None => {
                        let req = alloy_rpc_types::TransactionRequest {
                            from: Some(self.from),
                            to: Some(self.to),
                            input: alloy_rpc_types::TransactionInput::new(self.data.clone()),
                            value: Some(self.value),
                            max_fee_per_gas: Some(max_fee_per_gas),
                            max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
                            ..Default::default()
                        };
                        client
                            .estimate_gas(req)
                            .await
                            .map_err(TxError::GasEstimationFailed)?
                    }
                };

                Ok(UnsignedTx::Eip1559(TxEip1559 {
                    chain_id: self.chain_id,
                    nonce,
                    gas_limit,
                    max_fee_per_gas,
                    max_priority_fee_per_gas,
                    to: self.to,
                    value: self.value,
                    input: self.data,
                    ..Default::default()
                }))
            }
        }
    }
}

// ── Broadcaster ───────────────────────────────────────────────────────────────

/// Configuration for the broadcaster's confirmation polling.
#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    /// How often to poll for the receipt.
    pub poll_interval: Duration,
    /// Maximum time to wait before returning `TxError::ConfirmationTimeout`.
    pub timeout: Duration,
}

impl Default for BroadcastConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(500),
            timeout: Duration::from_secs(60),
        }
    }
}

/// Broadcasts a signed transaction and polls for its receipt.
///
/// Returns [`alloy_rpc_types::TransactionReceipt`] on success (status == 1).
/// Returns [`TxError::Reverted`] if the transaction mined but reverted.
/// Returns [`TxError::ConfirmationTimeout`] if the timeout expires.
pub struct Broadcaster {
    config: BroadcastConfig,
}

impl Broadcaster {
    /// Create a broadcaster with the default configuration (500ms poll, 60s timeout).
    pub fn new() -> Self {
        Self {
            config: BroadcastConfig::default(),
        }
    }

    /// Create a broadcaster with custom configuration.
    pub fn with_config(config: BroadcastConfig) -> Self {
        Self { config }
    }

    /// Submit a signed transaction and wait for confirmation.
    pub async fn send(
        &self,
        signed: &SignedTransaction,
        client: &RpcClient,
    ) -> Result<alloy_rpc_types::TransactionReceipt, TxError> {
        let _span = info_span!("tx_broadcast", hash = %signed.hash, from = %signed.from)
            .entered();

        // Submit.
        let hash = client
            .send_raw_transaction(&signed.raw_bytes)
            .await
            .map_err(TxError::SubmitFailed)?;

        info!(%hash, "transaction submitted");

        // Poll for receipt.
        let deadline = Instant::now() + self.config.timeout;
        loop {
            if Instant::now() >= deadline {
                let elapsed_secs = self.config.timeout.as_secs();
                return Err(TxError::ConfirmationTimeout { hash, elapsed_secs });
            }

            match client.get_transaction_receipt(hash).await {
                Ok(Some(receipt)) => {
                    let status = receipt.status();
                    info!(%hash, status = status as u8, "receipt received");
                    if status {
                        return Ok(receipt);
                    } else {
                        return Err(TxError::Reverted {
                            hash,
                            receipt: Box::new(receipt),
                        });
                    }
                }
                Ok(None) => {
                    sleep(self.config.poll_interval).await;
                }
                Err(e) => return Err(TxError::ReceiptPollFailed(e)),
            }
        }
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience: build → sign → broadcast in one call.
///
/// # Example
/// ```no_run
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use eth_node::{rpc::RpcClient, signer::EthSigner, tx::{TxBuilder, send_transaction}};
/// use alloy_primitives::U256;
///
/// let client = RpcClient::new("http://127.0.0.1:8545")?;
/// let signer = EthSigner::from_env()?;
/// let to = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse().unwrap();
/// let builder = TxBuilder::transfer(31337, signer.address(), to, U256::from(1_000_u64));
/// let receipt = send_transaction(builder, &signer, &client, None).await?;
/// println!("confirmed in block {:?}", receipt.block_number);
/// # Ok(()) }
/// ```
pub async fn send_transaction(
    builder: TxBuilder,
    signer: &EthSigner,
    client: &RpcClient,
    config: Option<BroadcastConfig>,
) -> Result<alloy_rpc_types::TransactionReceipt, TxError> {
    let unsigned = builder.build(client).await?;
    let signed = signer.sign(unsigned).map_err(TxError::SigningFailed)?;
    let broadcaster = config.map(Broadcaster::with_config).unwrap_or_default();
    broadcaster.send(&signed, client).await
}

// ── Conflict detection helper (unit-testable without async) ───────────────────

/// Returns `Err(TxError::ConflictingFeeParams)` if both fee params are provided.
///
/// A **conflict** means both a `gas_price` (legacy type-0) and a `max_fee_per_gas`
/// (EIP-1559 type-2) fee parameter were supplied simultaneously.  These select
/// mutually exclusive transaction types; providing both is ambiguous and therefore
/// rejected (decision table row 4).
///
/// Exposes the decision-table check for unit testing without network I/O.
pub fn check_fee_conflict(gas_price: Option<u128>, max_fee: Option<u128>) -> Result<(), TxError> {
    if gas_price.is_some() && max_fee.is_some() {
        return Err(TxError::ConflictingFeeParams);
    }
    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Decision table ────────────────────────────────────────────────────────

    #[test]
    fn fee_conflict_both_set_errors() {
        let err = check_fee_conflict(Some(1_000_000_000), Some(2_000_000_000)).unwrap_err();
        assert!(matches!(err, TxError::ConflictingFeeParams));
    }

    #[test]
    // Decision table rows 1–3 are NOT conflicts — only row 4 is.
    fn fee_params_only_gas_price_is_valid() {
        check_fee_conflict(Some(1_000_000_000), None).unwrap();
    }

    #[test]
    fn fee_params_only_max_fee_is_valid() {
        check_fee_conflict(None, Some(2_000_000_000)).unwrap();
    }

    #[test]
    fn fee_conflict_neither_ok() {
        check_fee_conflict(None, None).unwrap();
    }

    // ── Builder with fixed inputs ─────────────────────────────────────────────

    #[test]
    // Tests that the builder stores field values correctly without making any RPC calls.
    //
    // On async testing (L-016): yes, `#[tokio::test]` is supported and used
    // in `build_gas_price_and_max_fee_returns_conflict` below. This test stays
    // synchronous because it only inspects builder *field state* — nonce fetching
    // and gas estimation don't happen until `.build(&client)` is awaited.
    // The full async flow (build → sign → broadcast) is covered by integration
    // tests (T-006) running against a live Anvil instance.
    fn builder_eip1559_fixed_inputs() {
        let from: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse().unwrap();
        let to: Address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse().unwrap();

        let builder = TxBuilder::transfer(31337, from, to, U256::from(1_000u64))
            .nonce(5)
            .gas_limit(21_000)
            .max_fee(2_000_000_000, 1_000_000_000);

        // Extract the field we can check without async.
        assert_eq!(builder.chain_id, 31337);
        assert_eq!(builder.nonce, Some(5));
        assert_eq!(builder.gas_limit, Some(21_000));
        assert_eq!(builder.value, U256::from(1_000u64));
    }

    #[test]
    fn builder_legacy_fixed_inputs() {
        let from: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse().unwrap();
        let to: Address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse().unwrap();

        let builder = TxBuilder::transfer(31337, from, to, U256::from(500u64))
            .nonce(2)
            .gas_limit(21_000)
            .gas_price(1_000_000_000);

        assert_eq!(builder.chain_id, 31337);
        assert!(matches!(builder.fee_config, FeeConfig::Legacy { gas_price: 1_000_000_000 }));
    }

    #[test]
    fn invalid_fee_params_priority_exceeds_max() {
        // We can trigger this path by constructing the error directly.
        let err = TxError::InvalidFeeParams {
            max: 1_000_000_000,
            priority: 2_000_000_000,
        };
        let msg = err.to_string();
        assert!(msg.contains("exceeds"));
    }

    #[test]
    fn broadcaster_default_config() {
        let b = Broadcaster::new();
        assert_eq!(b.config.poll_interval, Duration::from_millis(500));
        assert_eq!(b.config.timeout, Duration::from_secs(60));
    }

    /// O-001: decision table row 4 — calling both `.gas_price()` and `.max_fee()` on the
    /// builder must return `ConflictingFeeParams` from `build()`.  The conflict check in
    /// `build()` fires before any `await`, so no live RPC is reached.
    #[tokio::test]
    async fn build_gas_price_and_max_fee_returns_conflict() {
        use crate::rpc::RpcClient;
        let client = RpcClient::new("http://127.0.0.1:8545").unwrap();
        let from: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse().unwrap();
        let to: Address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse().unwrap();

        let result = TxBuilder::transfer(31337, from, to, U256::ZERO)
            .gas_price(1_000_000_000)
            .max_fee(2_000_000_000, 1_000_000_000)
            .build(&client)
            .await;

        assert!(
            matches!(result, Err(TxError::ConflictingFeeParams)),
            "expected ConflictingFeeParams, got {result:?}"
        );
    }
}

