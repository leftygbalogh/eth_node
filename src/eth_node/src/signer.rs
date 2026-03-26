//! Transaction signing from a local private key.
//!
//! Spec ref: FORMAL_SPEC.md §4 FR-004

use alloy_consensus::{SignableTransaction, TxEip1559, TxEnvelope, TxLegacy};
use alloy_network::eip2718::Encodable2718;
use alloy_network::TxSignerSync;
use alloy_primitives::{Address, Bytes, B256};
use alloy_signer::Signer as _;
use alloy_signer_local::PrivateKeySigner;
use thiserror::Error;
use tracing::info_span;

// ── Error type ───────────────────────────────────────────────────────────────

/// Errors produced by the signer module.
///
/// `SigningFailed` deliberately uses only the display form of the underlying
/// error — never `Debug`, to avoid any accidental key-material leakage from
/// internal error types.
#[derive(Debug, Error)]
pub enum SignerError {
    /// `ETH_PRIVATE_KEY` env var is absent, or the keystore path was not set.
    #[error("credential not found: {0}")]
    CredentialNotFound(String),

    /// The private key string is malformed (wrong length, invalid hex).
    #[error("invalid key format")]
    InvalidKey,

    /// ECDSA signing produced an error.
    #[error("signing failed: {0}")]
    SigningFailed(String),
}

// ── Output types ─────────────────────────────────────────────────────────────

/// A signed transaction ready for broadcast via `eth_sendRawTransaction`.
#[derive(Debug, Clone)]
pub struct SignedTransaction {
    /// EIP-2718 encoded bytes — pass these directly to `eth_sendRawTransaction`.
    pub raw_bytes: Bytes,
    /// Keccak-256 hash of the encoded envelope (the transaction hash).
    pub hash: B256,
    /// The address that produced the signature.
    pub from: Address,
}

/// An unsigned transaction ready to be signed.
///
/// Two variants covering the Phase 1 scope:
/// - `Eip1559` — type-2 (EIP-1559) transaction.
/// - `Legacy` — type-0 (legacy) transaction with optional EIP-155 chain ID.
#[derive(Debug, Clone)]
pub enum UnsignedTx {
    /// EIP-1559 fee-market transaction (type 2).
    Eip1559(TxEip1559),
    /// Legacy gas-price transaction (type 0).
    Legacy(TxLegacy),
}

// ── Signer ───────────────────────────────────────────────────────────────────

/// A transaction signer backed by a locally held private key.
///
/// # Security
/// The private key is held inside `PrivateKeySigner`, which alloy marks as
/// non-`Debug` — the key does **not** appear in any derived `Debug` output.
/// `SignerError` variants avoid exposing key material in their display strings.
///
/// # Example
/// ```no_run
/// use eth_node::signer::{EthSigner, UnsignedTx};
/// use alloy_consensus::TxEip1559;
/// use alloy_primitives::{TxKind, U256};
///
/// let signer = EthSigner::from_env().expect("ETH_PRIVATE_KEY must be set");
/// let tx = UnsignedTx::Eip1559(TxEip1559 {
///     chain_id: 31337,
///     nonce: 0,
///     gas_limit: 21_000,
///     max_fee_per_gas: 1_000_000_000,
///     max_priority_fee_per_gas: 1_000_000_000,
///     to: TxKind::Call("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".parse().unwrap()),
///     value: U256::from(1_000_000_000_000_000_u64),
///     ..Default::default()
/// });
/// let signed = signer.sign(tx).unwrap();
/// println!("tx hash: {:?}", signed.hash);
/// ```
pub struct EthSigner {
    inner: PrivateKeySigner,
}

/// Custom `Debug` implementation that never reveals the private key.
impl std::fmt::Debug for EthSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EthSigner")
            .field("address", &self.inner.address())
            .finish_non_exhaustive()
    }
}

impl EthSigner {
    /// Load the private key from the `ETH_PRIVATE_KEY` environment variable.
    ///
    /// The variable must be a 32-byte hex string, with or without `0x` prefix.
    ///
    /// # Errors
    /// - [`SignerError::CredentialNotFound`] if the env var is absent.
    /// - [`SignerError::InvalidKey`] if the value is not a valid 32-byte hex key.
    pub fn from_env() -> Result<Self, SignerError> {
        let hex_key = std::env::var("ETH_PRIVATE_KEY").map_err(|_| {
            SignerError::CredentialNotFound("ETH_PRIVATE_KEY env var not set".into())
        })?;
        Self::from_key(&hex_key)
    }

    /// Create a signer from a hex-encoded private key string.
    ///
    /// Accepts keys with or without a `0x` prefix, case-insensitive.
    ///
    /// # Errors
    /// Returns [`SignerError::InvalidKey`] if the string is not a valid 32-byte hex key.
    pub fn from_key(hex_key: &str) -> Result<Self, SignerError> {
        let inner: PrivateKeySigner = hex_key.parse().map_err(|_| SignerError::InvalidKey)?;
        Ok(Self { inner })
    }

    /// The Ethereum address derived from this signer's private key.
    pub fn address(&self) -> Address {
        self.inner.address()
    }

    /// Sign an unsigned transaction and produce a broadcast-ready [`SignedTransaction`].
    ///
    /// Emits a `tracing` span tagged with the sender address (never the key).
    /// For legacy transactions, EIP-155 replay protection is applied using the
    /// chain ID embedded in the [`TxLegacy::chain_id`] field.
    ///
    /// # Errors
    /// Returns [`SignerError::SigningFailed`] if the ECDSA engine reports an error.
    pub fn sign(&self, tx: UnsignedTx) -> Result<SignedTransaction, SignerError> {
        let from = self.inner.address();
        let _span = info_span!("eth_sign", %from).entered();

        let envelope = match tx {
            UnsignedTx::Eip1559(mut tx_inner) => {
                let sig = self
                    .inner
                    .sign_transaction_sync(&mut tx_inner)
                    .map_err(|e| SignerError::SigningFailed(e.to_string()))?;
                TxEnvelope::Eip1559(tx_inner.into_signed(sig))
            }
            UnsignedTx::Legacy(mut tx_inner) => {
                // Apply EIP-155: set the signer's chain_id from the tx field so
                // `sign_transaction_with_chain_id!` can embed it in the signature.
                let inner = self.inner.clone().with_chain_id(tx_inner.chain_id);
                let sig = inner
                    .sign_transaction_sync(&mut tx_inner)
                    .map_err(|e| SignerError::SigningFailed(e.to_string()))?;
                TxEnvelope::Legacy(tx_inner.into_signed(sig))
            }
        };

        let raw_bytes = Bytes::from(envelope.encoded_2718());
        let hash = envelope.trie_hash();

        Ok(SignedTransaction { raw_bytes, hash, from })
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::TxEip1559;
    use alloy_primitives::{TxKind, U256};

    // Anvil account 0 — well-known devnet test key (not a production secret).
    const ANVIL_KEY: &str =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const ANVIL_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    fn test_eip1559_tx() -> TxEip1559 {
        TxEip1559 {
            chain_id: 31337,
            nonce: 0,
            gas_limit: 21_000,
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
            to: TxKind::Call(
                "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
                    .parse()
                    .unwrap(),
            ),
            value: U256::from(1_000_000_000_000_000_u64), // 0.001 ETH
            ..Default::default()
        }
    }

    // ── Construction tests ────────────────────────────────────────────────────

    #[test]
    fn from_key_derives_correct_address() {
        let signer = EthSigner::from_key(ANVIL_KEY).unwrap();
        let expected: Address = ANVIL_ADDRESS.parse().unwrap();
        assert_eq!(signer.address(), expected);
    }

    #[test]
    fn from_key_without_0x_prefix_works() {
        // strip '0x'
        let key_no_prefix = &ANVIL_KEY[2..];
        let signer = EthSigner::from_key(key_no_prefix).unwrap();
        let expected: Address = ANVIL_ADDRESS.parse().unwrap();
        assert_eq!(signer.address(), expected);
    }

    #[test]
    fn from_key_bad_hex_returns_invalid_key() {
        let err = EthSigner::from_key("not-hex-at-all").unwrap_err();
        assert!(matches!(err, SignerError::InvalidKey));
    }

    #[test]
    fn from_key_short_key_returns_invalid_key() {
        let err = EthSigner::from_key("0xdeadbeef").unwrap_err();
        assert!(matches!(err, SignerError::InvalidKey));
    }

    #[test]
    fn from_env_missing_var_returns_credential_not_found() {
        // Unset the var for this test (restore afterward).
        let old = std::env::var("ETH_PRIVATE_KEY").ok();
        std::env::remove_var("ETH_PRIVATE_KEY");

        let err = EthSigner::from_env().unwrap_err();
        assert!(matches!(err, SignerError::CredentialNotFound(_)));

        if let Some(v) = old {
            std::env::set_var("ETH_PRIVATE_KEY", v);
        }
    }

    // ── Signing tests ─────────────────────────────────────────────────────────

    #[test]
    fn sign_eip1559_produces_nonempty_raw_bytes_and_correct_from() {
        let signer = EthSigner::from_key(ANVIL_KEY).unwrap();
        let signed = signer.sign(UnsignedTx::Eip1559(test_eip1559_tx())).unwrap();

        assert!(!signed.raw_bytes.is_empty(), "raw_bytes must not be empty");
        assert_ne!(signed.hash, B256::ZERO, "hash must not be zero");
        let expected_from: Address = ANVIL_ADDRESS.parse().unwrap();
        assert_eq!(signed.from, expected_from);
    }

    #[test]
    // RFC 6979 signing is purely input-deterministic: the output depends only on
    // (private_key, transaction_fields) — there is no timestamp or random nonce.
    //
    // "Same transaction" means identical field values (chain_id, nonce, gas_limit,
    // max_fee, max_priority_fee, to, value, data). Two `TxEip1559` structs with
    // identical content produce bit-for-bit identical EIP-2718 encoded bytes, and
    // therefore the same keccak hash and the same RFC 6979 signature.
    //
    // "A nanosecond apart" only differs if a field differs (typically the nonce).
    // Two transactions with the same nonce are literally the same transaction on-chain;
    // only one can be confirmed. This is intentional replay protection.
    fn sign_eip1559_is_deterministic_rfc6979() {
        // RFC 6979 guarantees deterministic signing — same key + same tx = same signature.
        let signer = EthSigner::from_key(ANVIL_KEY).unwrap();
        let s1 = signer.sign(UnsignedTx::Eip1559(test_eip1559_tx())).unwrap();
        let s2 = signer.sign(UnsignedTx::Eip1559(test_eip1559_tx())).unwrap();
        assert_eq!(s1.raw_bytes, s2.raw_bytes);
        assert_eq!(s1.hash, s2.hash);
    }

    #[test]
    fn sign_legacy_produces_nonempty_raw_bytes() {
        use alloy_consensus::TxLegacy;
        let signer = EthSigner::from_key(ANVIL_KEY).unwrap();
        let tx = TxLegacy {
            chain_id: Some(31337),
            nonce: 0,
            gas_price: 1_000_000_000,
            gas_limit: 21_000,
            to: TxKind::Call(
                "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
                    .parse()
                    .unwrap(),
            ),
            value: U256::from(1_000u64),
            input: Default::default(),
        };
        let signed = signer.sign(UnsignedTx::Legacy(tx)).unwrap();
        assert!(!signed.raw_bytes.is_empty());
        let expected_from: Address = ANVIL_ADDRESS.parse().unwrap();
        assert_eq!(signed.from, expected_from);
    }

    // ── NFR-002: key must not appear in log output ────────────────────────────

    #[test]
    // For Phase 1 (local devnet, env-var key), the private key is the only item
    // that must never reach log output. Items to re-check if Phase 2 adds HD wallets:
    //   - BIP-39 mnemonic seed phrases
    //   - Derivation paths that could narrow key-space guessing
    //   - Raw signature bytes (enable replay analysis in some contexts)
    // None of these are in scope today; revisit this test when HD wallet support is added.
    fn key_not_in_log_output() {
        use std::io;
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::fmt::MakeWriter;

        // A writer that captures bytes into an in-memory buffer.
        #[derive(Clone)]
        struct BufWriter(Arc<Mutex<Vec<u8>>>);

        impl io::Write for BufWriter {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        impl<'a> MakeWriter<'a> for BufWriter {
            type Writer = BufWriter;
            fn make_writer(&'a self) -> BufWriter {
                self.clone()
            }
        }

        let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
        let writer = BufWriter(buf.clone());

        let subscriber = tracing_subscriber::fmt()
            .with_writer(writer)
            .with_max_level(tracing::Level::TRACE)
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let signer = EthSigner::from_key(ANVIL_KEY).unwrap();
            let _ = signer.sign(UnsignedTx::Eip1559(test_eip1559_tx()));
        });

        let guard = buf.lock().unwrap();
        let captured = String::from_utf8_lossy(&guard);
        // The raw key, without the '0x' prefix, must not appear in any log line.
        let key_hex = &ANVIL_KEY[2..]; // strip '0x'
        assert!(
            !captured.contains(key_hex),
            "SECURITY: private key leaked into tracing output!\nCaptured:\n{captured}"
        );
    }
}

