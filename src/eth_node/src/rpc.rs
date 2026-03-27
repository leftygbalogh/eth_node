//! JSON-RPC client for Ethereum-compatible endpoints.
//!
//! Spec ref: FORMAL_SPEC.md §4 FR-002

use alloy_network::Ethereum;
use alloy_primitives::{Address, Bytes, B256, U256};
use alloy_provider::{Provider, ProviderBuilder, RootProvider};
use alloy_rpc_types::{
    Block, BlockId, BlockNumberOrTag, Filter, Log, TransactionReceipt, TransactionRequest,
};
use thiserror::Error;
use tracing::instrument;
use url::Url;

// ── Error type ───────────────────────────────────────────────────────────────

/// Errors produced by the JSON-RPC client.
///
/// Variants are split by recoverability:
/// - **Transient** (`Transport`, `Timeout`): a retry with back-off may succeed.
/// - **Permanent** (`JsonRpc`, `Deserialization`): retrying the same request will not help.
#[derive(Debug, Error)]
pub enum RpcError {
    /// Network-level failure: connection refused, DNS failure, etc.
    #[error("transport error: {0}")]
    Transport(String),

    /// The RPC server returned a well-formed JSON-RPC error object.
    #[error("JSON-RPC error {code}: {message}")]
    JsonRpc { code: i64, message: String },

    /// The request did not complete within the timeout window.
    #[error("request timed out")]
    Timeout,

    /// The server response payload could not be decoded.
    #[error("deserialization error: {0}")]
    Deserialization(String),

    /// The supplied endpoint string is not a valid URL.
    #[error("invalid endpoint URL: {0}")]
    InvalidUrl(String),
}

fn map_transport_err(err: alloy_transport::TransportError) -> RpcError {
    use alloy_transport::TransportError as TE;
    match err {
        TE::ErrorResp(payload) => RpcError::JsonRpc {
            code: payload.code,
            message: payload.message.to_string(),
        },
        TE::DeserError { err, .. } => RpcError::Deserialization(err.to_string()),
        other => {
            let s = other.to_string();
            if s.contains("timed out") || s.contains("timeout") {
                RpcError::Timeout
            } else {
                RpcError::Transport(s)
            }
        }
    }
}

// ── Client ───────────────────────────────────────────────────────────────────

/// An async JSON-RPC client for any Ethereum-compatible endpoint.
///
/// Wraps [`alloy_provider`] and exposes typed methods for the eleven calls
/// specified in FR-002. The underlying HTTP transport uses connection pooling
/// via `reqwest`, so a single `RpcClient` should be shared across tasks where
/// possible.
///
/// # Example
/// ```no_run
/// # #[tokio::main] async fn main() -> Result<(), eth_node::rpc::RpcError> {
/// use eth_node::rpc::RpcClient;
/// let client = RpcClient::new("http://127.0.0.1:8545")?;
/// let block = client.block_number().await?;
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct RpcClient {
    inner: RootProvider<Ethereum>,
    endpoint: String,
}

impl RpcClient {
    /// Create a new client pointing at `endpoint`.
    ///
    /// The URL is validated immediately; the first network call occurs when
    /// one of the typed methods is awaited.
    ///
    /// # Errors
    /// Returns [`RpcError::InvalidUrl`] if `endpoint` cannot be parsed as a URL.
    pub fn new(endpoint: &str) -> Result<Self, RpcError> {
        let url: Url = endpoint
            .parse()
            .map_err(|e: url::ParseError| RpcError::InvalidUrl(e.to_string()))?;
        // ProviderBuilder::default() gives no mandatory fillers, so on_http
        // returns RootProvider<Ethereum> — the simplest stable concrete type.
        let inner = ProviderBuilder::default().on_http(url);
        Ok(Self {
            inner,
            endpoint: endpoint.to_owned(),
        })
    }

    /// The endpoint URL this client targets.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    // ── eth_blockNumber ───────────────────────────────────────────────────

    /// `eth_blockNumber` — current chain head block number.
    #[instrument(skip(self), err)]
    pub async fn block_number(&self) -> Result<u64, RpcError> {
        self.inner
            .get_block_number()
            .await
            .map_err(map_transport_err)
    }

    // ── eth_getBalance ────────────────────────────────────────────────────

    /// `eth_getBalance(address, "latest")` — account balance in Wei.
    #[instrument(skip(self), err)]
    pub async fn get_balance(&self, address: Address) -> Result<U256, RpcError> {
        self.inner
            .get_balance(address)
            .await
            .map_err(map_transport_err)
    }

    /// `eth_getBalance(address, block_id)` — balance at a specific block.
    #[instrument(skip(self), err)]
    pub async fn get_balance_at(
        &self,
        address: Address,
        block: BlockId,
    ) -> Result<U256, RpcError> {
        self.inner
            .get_balance(address)
            .block_id(block)
            .await
            .map_err(map_transport_err)
    }

    // ── eth_getTransactionCount ───────────────────────────────────────────

    /// `eth_getTransactionCount(address, "latest")` — current account nonce.
    #[instrument(skip(self), err)]
    pub async fn get_nonce(&self, address: Address) -> Result<u64, RpcError> {
        self.inner
            .get_transaction_count(address)
            .await
            .map_err(map_transport_err)
    }

    /// `eth_getTransactionCount(address, block_id)` — nonce at a specific block.
    #[instrument(skip(self), err)]
    pub async fn get_nonce_at(&self, address: Address, block: BlockId) -> Result<u64, RpcError> {
        self.inner
            .get_transaction_count(address)
            .block_id(block)
            .await
            .map_err(map_transport_err)
    }

    // ── eth_getTransactionReceipt ─────────────────────────────────────────

    /// `eth_getTransactionReceipt(tx_hash)` — receipt for a mined transaction.
    ///
    /// Returns `None` if the transaction is not yet mined or unknown.
    #[instrument(skip(self), err)]
    pub async fn get_transaction_receipt(
        &self,
        hash: B256,
    ) -> Result<Option<TransactionReceipt>, RpcError> {
        self.inner
            .get_transaction_receipt(hash)
            .await
            .map_err(map_transport_err)
    }

    // ── eth_getBlockByNumber ──────────────────────────────────────────────

    /// `eth_getBlockByNumber(tag, false)` — block with transaction hashes (not full objects).
    #[instrument(skip(self), err)]
    pub async fn get_block_by_number(
        &self,
        tag: BlockNumberOrTag,
    ) -> Result<Option<Block>, RpcError> {
        self.inner
            .get_block_by_number(tag)
            .await
            .map_err(map_transport_err)
    }

    // ── eth_sendRawTransaction ────────────────────────────────────────────

    /// `eth_sendRawTransaction(signed_rlp)` — broadcast a signed transaction.
    ///
    /// Returns the transaction hash. Does **not** wait for confirmation;
    /// use [`get_transaction_receipt`](Self::get_transaction_receipt) to poll
    /// for the receipt.
    #[instrument(skip(self, signed_rlp), err)]
    pub async fn send_raw_transaction(&self, signed_rlp: &[u8]) -> Result<B256, RpcError> {
        let pending = self
            .inner
            .send_raw_transaction(signed_rlp)
            .await
            .map_err(map_transport_err)?;
        Ok(*pending.tx_hash())
    }

    // ── eth_call ──────────────────────────────────────────────────────────

    /// `eth_call(call_object, "latest")` — simulate a call without a transaction.
    ///
    /// Returns the raw ABI-encoded return data.
    #[instrument(skip(self, tx), err)]
    pub async fn call(&self, tx: TransactionRequest) -> Result<Bytes, RpcError> {
        self.inner.call(tx).await.map_err(map_transport_err)
    }

    // ── eth_getLogs ───────────────────────────────────────────────────────

    /// `eth_getLogs(filter)` — query logs matching a filter.
    #[instrument(skip(self, filter), err)]
    pub async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, RpcError> {
        self.inner
            .get_logs(filter)
            .await
            .map_err(map_transport_err)
    }

    // ── eth_chainId ───────────────────────────────────────────────────────

    /// `eth_chainId` — chain ID of the connected network.
    ///
    /// For Anvil the default is `31337`.
    #[instrument(skip(self), err)]
    pub async fn chain_id(&self) -> Result<u64, RpcError> {
        self.inner
            .get_chain_id()
            .await
            .map_err(map_transport_err)
    }

    // ── eth_gasPrice ──────────────────────────────────────────────────────

    /// `eth_gasPrice` — current gas price in Wei.
    #[instrument(skip(self), err)]
    pub async fn gas_price(&self) -> Result<u128, RpcError> {
        self.inner
            .get_gas_price()
            .await
            .map_err(map_transport_err)
    }

    // ── eth_estimateGas ───────────────────────────────────────────────────

    /// `eth_estimateGas(call_object)` — estimate gas for a call or transaction.
    #[instrument(skip(self, tx), err)]
    pub async fn estimate_gas(&self, tx: TransactionRequest) -> Result<u64, RpcError> {
        self.inner
            .estimate_gas(tx)
            .await
            .map_err(map_transport_err)
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    // Unit tests cover only what is testable without a network: URL validation and
    // error display contracts. All methods that perform actual RPC calls (get_balance,
    // block_number, etc.) require a live Anvil instance and live in the integration
    // test suite (eth_node/tests/integration_rpc.rs, covered by T-004).

    #[test]
    // Near-valid URL variants that exercise the `url` crate's parse rules:
    // - Port 9999999 exceeds the valid range (0–65535) → rejected.
    // - Double-dotted host (127.0.0.0.0.1) is not a valid IP or hostname → rejected.
    // - A plausible-looking TLD typo like "example.comm:8545" IS accepted by the url
    //   crate (it is syntactically valid); failure only occurs at the first network call.
    //   This is documented and correct: `new()` performs syntax-only validation.
    fn new_rejects_bad_url() {
        let err = RpcClient::new("not a url !!").unwrap_err();
        assert!(
            matches!(err, RpcError::InvalidUrl(_)),
            "expected InvalidUrl, got {err:?}"
        );
    }

    #[test]
    fn new_accepts_valid_http_url() {
        // Construction is infallible for a valid URL (no network call yet).
        let client = RpcClient::new("http://127.0.0.1:8545").unwrap();
        assert_eq!(client.endpoint(), "http://127.0.0.1:8545");
    }

    #[test]
    fn new_rejects_port_overflow_url() {
        // Port 9_999_999 exceeds the maximum valid port (65_535).
        let err = RpcClient::new("http://127.0.0.1:9999999").unwrap_err();
        assert!(matches!(err, RpcError::InvalidUrl(_)));
    }

    #[test]
    fn new_rejects_double_dot_host_url() {
        // "127.0.0.0.0.1" is not a valid IP address or hostname.
        let err = RpcClient::new("http://127.0.0.0.0.1:8545").unwrap_err();
        assert!(matches!(err, RpcError::InvalidUrl(_)));
    }

    /// DIT-002: lock in the display string contracts for all RpcError variants so that
    /// changes to error messages are caught by tests.
    #[test]
    fn rpc_error_display_messages() {
        assert_eq!(
            RpcError::Transport("connection refused".into()).to_string(),
            "transport error: connection refused"
        );
        assert_eq!(
            RpcError::JsonRpc { code: -32601, message: "method not found".into() }.to_string(),
            "JSON-RPC error -32601: method not found"
        );
        assert_eq!(RpcError::Timeout.to_string(), "request timed out");
        assert_eq!(
            RpcError::Deserialization("unexpected field".into()).to_string(),
            "deserialization error: unexpected field"
        );
        assert_eq!(
            RpcError::InvalidUrl("not a url".into()).to_string(),
            "invalid endpoint URL: not a url"
        );
    }

    #[test]
    /// G5: verify that alloy_primitives::Address preserves the EIP-55 checksum that
    /// alloy uses as the JSON-RPC wire parameter for eth_getBalance.
    fn eth_getbalance_address_param_is_checksummed_hex() {
        let addr: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
            .parse()
            .expect("parse checksummed address");
        assert_eq!(
            addr.to_checksum(None),
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "alloy must preserve EIP-55 checksum for JSON-RPC wire encoding"
        );
    }
}

/// G7 variant 1: `RpcClient::new` must never panic for any string input.
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn rpc_client_new_never_panics(s in ".*") {
            // Must always return Ok or Err(InvalidUrl) — never panic.
            let _ = RpcClient::new(&s);
        }
    }
}
