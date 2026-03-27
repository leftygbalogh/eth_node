//! ABI-driven contract caller.
//!
//! Spec ref: FORMAL_SPEC.md §4 FR-007
//!
//! # Design summary
//!
//! [`ContractCaller`] holds a deployed-contract address and a parsed
//! [`alloy_json_abi::JsonAbi`].  Two call modes are exposed:
//!
//! - **Read** ([`ContractCaller::call`]) — ABI-encodes the selector + args,
//!   sends `eth_call`, and ABI-decodes the return data into [`DynSolValue`]s.
//!
//! - **Write** ([`ContractCaller::send`]) — same encoding, then builds an
//!   EIP-1559 transaction, signs it with the provided [`EthSigner`], and
//!   broadcasts via [`Broadcaster`].
//!
//! Function overloading is resolved by iterating the ABI's overloads for the
//! requested name and selecting the first whose input types accept the given
//! argument vector.

use alloy_dyn_abi::{DynSolValue, FunctionExt, JsonAbiExt};
use alloy_json_abi::JsonAbi;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types::{TransactionInput, TransactionRequest};
use thiserror::Error;
use tracing::info_span;

use crate::{
    rpc::RpcClient,
    signer::EthSigner,
    tx::{BroadcastConfig, Broadcaster, TxBuilder},
};

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors produced by the contract caller.
#[derive(Debug, Error)]
pub enum ContractError {
    /// The ABI defines no function with the requested name.
    #[error("function not found in ABI: {0}")]
    AbiNotFound(String),

    /// ABI encoding or decoding failed.
    #[error("ABI codec error: {0}")]
    AbiDecodeError(String),

    /// The call reverted on-chain.
    #[error("call reverted: {0}")]
    CallReverted(String),

    /// The supplied ABI JSON cannot be parsed.
    #[error("invalid ABI JSON: {0}")]
    InvalidAbiJson(String),

    /// An RPC error occurred during the call or send.
    #[error("RPC error: {0}")]
    RpcError(String),

    /// Transaction building, signing, or broadcasting failed.
    #[error("transaction error: {0}")]
    TxError(String),
}

// ── ContractCaller ────────────────────────────────────────────────────────────

/// An ABI-aware caller for a deployed Ethereum contract.
///
/// # Example
/// ```no_run
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use eth_node::contract::ContractCaller;
/// use eth_node::rpc::RpcClient;
/// use alloy_dyn_abi::DynSolValue;
///
/// let abi_json = r#"[{"name":"balanceOf","type":"function",
///   "inputs":[{"name":"account","type":"address"}],
///   "outputs":[{"name":"","type":"uint256"}],
///   "stateMutability":"view"}]"#;
///
/// let address = "0x6B175474E89094C44Da98b954EedeAC495271d0F".parse().unwrap();
/// let client = RpcClient::new("http://127.0.0.1:8545")?;
/// let caller = ContractCaller::new(address, abi_json)?;
///
/// let user: alloy_primitives::Address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".parse().unwrap();
/// let tokens = caller.call("balanceOf", &[DynSolValue::Address(user)], &client).await?;
/// println!("{tokens:#?}");
/// # Ok(()) }
/// ```
#[derive(Debug, Clone)]
pub struct ContractCaller {
    address: Address,
    abi: JsonAbi,
}

impl ContractCaller {
    /// Create a new caller for the contract at `address` with the given ABI JSON.
    ///
    /// # Errors
    /// Returns [`ContractError::InvalidAbiJson`] if `abi_json` is not valid
    /// Solidity ABI JSON.
    pub fn new(address: Address, abi_json: &str) -> Result<Self, ContractError> {
        let abi = serde_json::from_str::<JsonAbi>(abi_json)
            .map_err(|e: serde_json::Error| ContractError::InvalidAbiJson(e.to_string()))?;
        Ok(Self { address, abi })
    }

    // ── Read call ─────────────────────────────────────────────────────────

    /// Execute a read-only contract call (`eth_call`) and decode the return
    /// data.
    ///
    /// Overloaded functions are resolved by selecting the first overload whose
    /// parameter types accept the given `args` vector.
    ///
    /// # Errors
    /// - [`ContractError::AbiNotFound`] — function not in ABI.
    /// - [`ContractError::AbiDecodeError`] — encoding or decoding failure.
    /// - [`ContractError::CallReverted`] — the call reverted on-chain.
    /// - [`ContractError::RpcError`] — transport-level failure.
    pub async fn call(
        &self,
        fn_name: &str,
        args: &[DynSolValue],
        client: &RpcClient,
    ) -> Result<Vec<DynSolValue>, ContractError> {
        let _span =
            info_span!("contract_call", function = fn_name, address = %self.address).entered();

        let func = self.resolve_function(fn_name, args)?;
        let encoded = func
            .abi_encode_input(args)
            .map_err(|e| ContractError::AbiDecodeError(e.to_string()))?;

        let req = TransactionRequest::default()
            .to(self.address)
            .input(TransactionInput::new(Bytes::from(encoded)));

        let raw = client.call(req).await.map_err(|e| {
            let msg = e.to_string();
            if msg.to_lowercase().contains("revert") {
                ContractError::CallReverted(msg)
            } else {
                ContractError::RpcError(msg)
            }
        })?;

        // Empty return data means the address has no deployed contract code.
        // Surfacing the raw "buffer overrun" message from the ABI decoder is
        // confusing; give the user an actionable hint instead.
        if raw.is_empty() {
            return Err(ContractError::AbiDecodeError(format!(
                "contract at {} returned no data — it may not be deployed at this address",
                self.address
            )));
        }

        func.abi_decode_output(&raw, true)
            .map_err(|e: alloy_dyn_abi::Error| ContractError::AbiDecodeError(e.to_string()))
    }

    // ── Write call ────────────────────────────────────────────────────────

    /// Build → sign → broadcast a state-changing contract call.
    ///
    /// Returns the confirmed [`TransactionReceipt`].
    ///
    /// # Errors
    /// - [`ContractError::AbiNotFound`] — function not in ABI.
    /// - [`ContractError::AbiDecodeError`] — argument encoding failure.
    /// - [`ContractError::TxError`] — build, sign, or broadcast failure.
    pub async fn send(
        &self,
        fn_name: &str,
        args: &[DynSolValue],
        signer: &EthSigner,
        client: &RpcClient,
        config: Option<BroadcastConfig>,
    ) -> Result<alloy_rpc_types::TransactionReceipt, ContractError> {
        let _span =
            info_span!("contract_send", function = fn_name, address = %self.address).entered();

        let func = self.resolve_function(fn_name, args)?;
        let encoded = func
            .abi_encode_input(args)
            .map_err(|e| ContractError::AbiDecodeError(e.to_string()))?;

        let chain_id = client
            .chain_id()
            .await
            .map_err(|e| ContractError::RpcError(e.to_string()))?;

        let builder = TxBuilder::new(chain_id, signer.address(), self.address)
            .data(Bytes::from(encoded));

        let unsigned = builder
            .build(client)
            .await
            .map_err(|e| ContractError::TxError(e.to_string()))?;

        let signed = signer
            .sign(unsigned)
            .map_err(|e| ContractError::TxError(e.to_string()))?;

        let broadcaster = config.map(Broadcaster::with_config).unwrap_or_default();
        broadcaster
            .send(&signed, client)
            .await
            .map_err(|e| ContractError::TxError(e.to_string()))
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    /// Find the first function overload that accepts `args`.
    ///
    /// Overloads are tried in definition order.  If no overload matches,
    /// returns [`ContractError::AbiDecodeError`] (wrong argument types).
    fn resolve_function(
        &self,
        fn_name: &str,
        args: &[DynSolValue],
    ) -> Result<&alloy_json_abi::Function, ContractError> {
        let overloads = self
            .abi
            .function(fn_name)
            .ok_or_else(|| ContractError::AbiNotFound(fn_name.to_string()))?;

        overloads
            .iter()
            .find(|f| {
                f.inputs.len() == args.len() && f.abi_encode_input(args).is_ok()
            })
            .ok_or_else(|| {
                ContractError::AbiDecodeError(format!(
                    "no overload of `{fn_name}` accepts the given argument types"
                ))
            })
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use alloy_dyn_abi::DynSolValue;
    use alloy_primitives::{Address, U256};

    use super::*;

    const ADDR: Address = Address::ZERO;

    const BALANCE_OF_ABI: &str = r#"[{
        "name": "balanceOf",
        "type": "function",
        "inputs": [{"name": "account", "type": "address"}],
        "outputs": [{"name": "", "type": "uint256"}],
        "stateMutability": "view"
    }]"#;

    const OVERLOADED_ABI: &str = r#"[
        {"name":"transfer","type":"function",
         "inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"}],
         "outputs":[{"name":"","type":"bool"}],"stateMutability":"nonpayable"},
        {"name":"transfer","type":"function",
         "inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"},
                   {"name":"data","type":"bytes"}],
         "outputs":[{"name":"","type":"bool"}],"stateMutability":"nonpayable"}
    ]"#;

    // An ABI with two identical transfer(address,uint256) signatures — used to
    // verify that the first definition is always selected over a duplicate.
    const DUPLICATE_OVERLOAD_ABI: &str = r#"[
        {"name":"transfer","type":"function",
         "inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"}],
         "outputs":[{"name":"","type":"bool"}],"stateMutability":"nonpayable"},
        {"name":"transfer","type":"function",
         "inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"}],
         "outputs":[{"name":"result","type":"bool"}],"stateMutability":"nonpayable"}
    ]"#;

    #[test]
    fn new_succeeds_with_valid_abi() {
        ContractCaller::new(ADDR, BALANCE_OF_ABI).expect("valid ABI");
    }

    #[test]
    // An empty JSON array `[]` is valid Solidity ABI JSON representing a contract
    // with no functions. Construction must succeed; any subsequent function lookup
    // returns AbiNotFound because there is simply nothing in the ABI to match.
    fn new_succeeds_with_empty_abi_array() {
        ContractCaller::new(ADDR, "[]").expect("empty array is valid ABI JSON");
    }

    #[test]
    fn resolve_function_on_empty_abi_returns_not_found() {
        let caller = ContractCaller::new(ADDR, "[]").unwrap();
        let err = caller.resolve_function("transfer", &[]).expect_err("no functions defined");
        assert!(matches!(err, ContractError::AbiNotFound(_)));
    }

    #[test]
    // The Lefty question was about fields that are present but empty. `[]` is the
    // canonical case: the JSON is valid, but there are zero function entries.
    // alloy's JsonAbi parser accepts this without error.
    fn new_rejects_invalid_json() {
        let err = ContractCaller::new(ADDR, "not json{{{").expect_err("should fail");
        assert!(matches!(err, ContractError::InvalidAbiJson(_)));
    }

    #[test]
    fn resolve_function_not_found() {
        let caller = ContractCaller::new(ADDR, BALANCE_OF_ABI).unwrap();
        let err = caller
            .resolve_function("nonexistent", &[])
            .expect_err("should not find");
        assert!(matches!(err, ContractError::AbiNotFound(_)));
    }

    #[test]
    // Overloading means the same function name appears multiple times in the ABI,
    // each time with a different parameter list. `resolve_function` tries each
    // overload in definition order and picks the first whose input count and
    // ABI-encoding both match the supplied argument vector.
    fn resolve_function_finds_correct_overload() {
        let caller = ContractCaller::new(ADDR, OVERLOADED_ABI).unwrap();

        // 2-arg overload
        let f2 = caller
            .resolve_function(
                "transfer",
                &[
                    DynSolValue::Address(Address::ZERO),
                    DynSolValue::Uint(U256::from(1u64), 256),
                ],
            )
            .expect("2-arg overload");
        assert_eq!(f2.inputs.len(), 2);

        // 3-arg overload
        let f3 = caller
            .resolve_function(
                "transfer",
                &[
                    DynSolValue::Address(Address::ZERO),
                    DynSolValue::Uint(U256::from(1u64), 256),
                    DynSolValue::Bytes(vec![]),
                ],
            )
            .expect("3-arg overload");
        assert_eq!(f3.inputs.len(), 3);
    }

    #[test]
    // When two overloads have *identical* signatures, the first definition in the
    // ABI array wins. We distinguish them by their output names ("" vs "result").
    fn resolve_function_duplicate_overload_picks_first() {
        let caller = ContractCaller::new(ADDR, DUPLICATE_OVERLOAD_ABI).unwrap();
        let f = caller
            .resolve_function(
                "transfer",
                &[
                    DynSolValue::Address(Address::ZERO),
                    DynSolValue::Uint(U256::from(1u64), 256),
                ],
            )
            .expect("first overload");
        // The first overload has an unnamed output (""); the second has "result".
        assert_eq!(f.outputs[0].name, "");
    }

    #[test]
    fn error_display_messages() {
        assert!(ContractError::AbiNotFound("foo".into()).to_string().contains("foo"));
        assert!(ContractError::CallReverted("boom".into())
            .to_string()
            .contains("boom"));
        assert!(ContractError::InvalidAbiJson("bad".into())
            .to_string()
            .contains("bad"));
    }

    /// DIT-001: resolve_function finds the function by name but no overload accepts the
    /// given argument types — must return AbiDecodeError (not AbiNotFound).
    #[test]
    fn resolve_function_rejects_wrong_arg_types() {
        let caller = ContractCaller::new(ADDR, OVERLOADED_ABI).unwrap();

        // `transfer` exists, but passing a uint256 where an address is expected
        // should fail with AbiDecodeError, not AbiNotFound.
        let err = caller
            .resolve_function(
                "transfer",
                &[
                    DynSolValue::Uint(U256::from(1u64), 256), // wrong type: uint256 instead of address
                    DynSolValue::Uint(U256::from(1u64), 256),
                ],
            )
            .expect_err("wrong arg types should fail");

        assert!(
            matches!(err, ContractError::AbiDecodeError(_)),
            "expected AbiDecodeError for wrong arg types, got {err:?}"
        );
    }

    #[test]
    /// G6: verify that ABI-encoding `balanceOf(address(0))` produces the correct
    /// 4-byte function selector in the first bytes of the `eth_call` data field.
    fn eth_call_data_contains_correct_balance_of_selector() {
        // keccak256("balanceOf(address)")[:4] = 0x70a08231
        let caller = ContractCaller::new(ADDR, BALANCE_OF_ABI).unwrap();
        let func = caller
            .resolve_function("balanceOf", &[DynSolValue::Address(Address::ZERO)])
            .expect("resolve balanceOf");
        let encoded = func
            .abi_encode_input(&[DynSolValue::Address(Address::ZERO)])
            .expect("ABI encode input");
        // 4-byte selector + 32-byte padded address argument = 36 bytes total.
        assert_eq!(encoded.len(), 36, "encoded call data must be 36 bytes");
        assert_eq!(
            &encoded[..4],
            &[0x70, 0xa0, 0x82, 0x31],
            "first 4 bytes must be keccak256(\"balanceOf(address)\")[:4]"
        );
    }
}

/// G7 variant 3: ABI decoding arbitrary bytes must never panic.
#[cfg(test)]
mod proptests {
    use alloy_dyn_abi::FunctionExt;
    use alloy_json_abi::JsonAbi;
    use proptest::prelude::*;

    const BALANCE_OF_ABI: &str = r#"[{
        "name": "balanceOf",
        "type": "function",
        "inputs": [{"name": "account", "type": "address"}],
        "outputs": [{"name": "", "type": "uint256"}],
        "stateMutability": "view"
    }]"#;

    proptest! {
        #[test]
        fn abi_decode_arbitrary_bytes_never_panics(
            bytes in proptest::collection::vec(any::<u8>(), 0..256)
        ) {
            // ABI decoding arbitrary bytes must never panic — always Ok or Err.
            let abi: JsonAbi = serde_json::from_str(BALANCE_OF_ABI).unwrap();
            let func = abi.function("balanceOf").unwrap().first().unwrap();
            let _ = func.abi_decode_output(&bytes, false);
        }
    }
}

