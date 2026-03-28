//! Deploy-driven live decode tests for T-004 gate closure.
//!
//! These tests compile Solidity emitters with Forge, deploy them to Anvil,
//! trigger event emission via transactions, capture logs from receipts, and
//! run the quality decoders on the captured logs.

mod helpers;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use alloy_consensus::TxEip1559;
use alloy_dyn_abi::DynSolValue;
use alloy_primitives::{Address, TxKind, U256};
use eth_node::contract::ContractCaller;
use eth_node::quality::{
    decode_erc1155_approval_for_all, decode_standard_nft_event, DecodedEvent,
};
use eth_node::rpc::RpcClient;
use eth_node::signer::{EthSigner, UnsignedTx};
use eth_node::tx::Broadcaster;
use helpers::accounts::ANVIL_ACCOUNT0_KEY;
use helpers::anvil_fixture::AnvilInstance;
use serde_json::Value;

struct BuiltContracts {
    erc721_init_code: Vec<u8>,
    erc721_abi_json: String,
    erc1155_init_code: Vec<u8>,
    erc1155_abi_json: String,
}

static BUILT_CONTRACTS: OnceLock<Result<BuiltContracts, String>> = OnceLock::new();

macro_rules! require_anvil {
    () => {{
        match AnvilInstance::spawn().expect("spawn anvil") {
            None => {
                eprintln!("anvil not on PATH — skipping test");
                return;
            }
            Some(a) => a,
        }
    }};
}

fn build_contracts_once() -> Result<&'static BuiltContracts, String> {
    let result = BUILT_CONTRACTS.get_or_init(build_contracts);
    result.as_ref().map_err(Clone::clone)
}

fn build_contracts() -> Result<BuiltContracts, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let contracts_dir = resolve_contracts_dir(&manifest_dir)?;
    let out_dir = std::env::temp_dir()
        .join("eth_node_forge_artifacts")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&out_dir)
        .map_err(|e| format!("failed to create forge artifact directory {}: {e}", out_dir.display()))?;

    let erc721_source = contracts_dir.join("TestERC721.sol");
    let erc1155_source = contracts_dir.join("TestERC1155.sol");

    if Command::new("forge")
        .arg("--version")
        .output()
        .is_err()
    {
        return Err("forge not on PATH".to_string());
    }

    let build_output = Command::new("forge")
        .current_dir(&manifest_dir)
        .arg("build")
        .arg(erc721_source.as_os_str())
        .arg(erc1155_source.as_os_str())
        .args(["--out"])
        .arg(out_dir.as_os_str())
        .args(["--cache-path"])
        .arg(out_dir.join("cache").as_os_str())
        .output()
        .map_err(|e| format!("failed to execute forge build: {e}"))?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        return Err(format!("forge build failed: {stderr}"));
    }

    let erc721_artifact = out_dir
        .join("TestERC721.sol")
        .join("TestERC721.json");
    let erc1155_artifact = out_dir
        .join("TestERC1155.sol")
        .join("TestERC1155.json");

    let erc721_json = read_artifact_json(&erc721_artifact)?;
    let erc1155_json = read_artifact_json(&erc1155_artifact)?;

    let erc721_init_code = read_init_code(&erc721_json, &erc721_artifact)?;
    let erc1155_init_code = read_init_code(&erc1155_json, &erc1155_artifact)?;
    let erc721_abi_json = read_abi_json(&erc721_json, &erc721_artifact)?;
    let erc1155_abi_json = read_abi_json(&erc1155_json, &erc1155_artifact)?;

    Ok(BuiltContracts {
        erc721_init_code,
        erc721_abi_json,
        erc1155_init_code,
        erc1155_abi_json,
    })
}

fn resolve_contracts_dir(manifest_dir: &Path) -> Result<PathBuf, String> {
    let candidates = [
        manifest_dir.join("tests").join("contracts"),
        manifest_dir.join("src").join("eth_node").join("tests").join("contracts"),
    ];

    for dir in candidates {
        let has_erc721 = dir.join("TestERC721.sol").is_file();
        let has_erc1155 = dir.join("TestERC1155.sol").is_file();
        if has_erc721 && has_erc1155 {
            return Ok(dir);
        }
    }

    Err(format!(
        "could not locate TestERC721.sol and TestERC1155.sol from manifest dir {}",
        manifest_dir.display()
    ))
}

fn read_artifact_json(artifact_path: &Path) -> Result<Value, String> {
    let raw = std::fs::read_to_string(artifact_path)
        .map_err(|e| format!("failed to read artifact {}: {e}", artifact_path.display()))?;
    serde_json::from_str(&raw)
        .map_err(|e| format!("failed to parse artifact {}: {e}", artifact_path.display()))
}

fn read_init_code(json: &Value, artifact_path: &Path) -> Result<Vec<u8>, String> {
    let hex_bytecode = json
        .pointer("/bytecode/object")
        .and_then(Value::as_str)
        .or_else(|| json.pointer("/bytecode").and_then(Value::as_str))
        .ok_or_else(|| format!("missing bytecode in artifact {}", artifact_path.display()))?;

    let trimmed = hex_bytecode.trim_start_matches("0x");
    hex::decode(trimmed)
        .map_err(|e| format!("invalid hex bytecode in {}: {e}", artifact_path.display()))
}

fn read_abi_json(json: &Value, artifact_path: &Path) -> Result<String, String> {
    let abi = json
        .get("abi")
        .ok_or_else(|| format!("missing abi in artifact {}", artifact_path.display()))?;
    serde_json::to_string(abi)
        .map_err(|e| format!("failed to serialize abi from {}: {e}", artifact_path.display()))
}

async fn deploy_contract(init_code: &[u8], client: &RpcClient, signer: &EthSigner) -> Address {
    let from = signer.address();
    let nonce = client.get_nonce(from).await.expect("get nonce");
    let gas_price = client.gas_price().await.expect("gas price");
    let chain_id = client.chain_id().await.expect("chain id");

    let tx = TxEip1559 {
        chain_id,
        nonce,
        max_fee_per_gas: gas_price * 2,
        max_priority_fee_per_gas: gas_price,
        gas_limit: 2_000_000,
        to: TxKind::Create,
        value: U256::ZERO,
        input: init_code.to_vec().into(),
        ..Default::default()
    };

    let signed = signer.sign(UnsignedTx::Eip1559(tx)).expect("sign deploy");
    let receipt = Broadcaster::new().send(&signed, client).await.expect("deploy");
    assert!(receipt.status(), "deploy tx reverted");
    receipt
        .contract_address
        .expect("deploy receipt missing contract_address")
}

async fn send_contract_call(
    to: Address,
    abi_json: &str,
    function: &str,
    args: &[DynSolValue],
    client: &RpcClient,
    signer: &EthSigner,
) -> alloy_rpc_types::TransactionReceipt {
    let caller = ContractCaller::new(to, abi_json).expect("build contract caller");
    let receipt = caller
        .send(function, args, signer, client, None)
        .await
        .expect("send");
    assert!(receipt.status(), "event tx reverted");
    receipt
}

#[tokio::test]
async fn live_decode_erc721_transfer_from_anvil_receipt() {
    let anvil = require_anvil!();
    let built = match build_contracts_once() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e} — skipping test");
            return;
        }
    };

    let client = RpcClient::new(&anvil.endpoint).expect("rpc client");
    let signer = EthSigner::from_key(ANVIL_ACCOUNT0_KEY).expect("signer");

    let contract = deploy_contract(&built.erc721_init_code, &client, &signer).await;

    let from = Address::repeat_byte(0x21);
    let to = Address::repeat_byte(0x22);
    let token_id = U256::from(42);
    let args = vec![
        DynSolValue::Address(from),
        DynSolValue::Address(to),
        DynSolValue::Uint(token_id, 256),
    ];

    let receipt = send_contract_call(
        contract,
        &built.erc721_abi_json,
        "emitTransfer",
        &args,
        &client,
        &signer,
    )
    .await;
    assert_eq!(receipt.logs().len(), 1, "expected exactly one emitted log");

    let decoded = decode_standard_nft_event(&receipt.logs()[0]).expect("decode transfer");
    match decoded {
        DecodedEvent::Erc721Transfer(e) => {
            assert_eq!(e.from, from);
            assert_eq!(e.to, to);
            assert_eq!(e.token_id, token_id);
        }
        other => panic!("unexpected decoded variant: {other:?}"),
    }
}

#[tokio::test]
async fn live_decode_erc721_approval_from_anvil_receipt() {
    let anvil = require_anvil!();
    let built = match build_contracts_once() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e} — skipping test");
            return;
        }
    };

    let client = RpcClient::new(&anvil.endpoint).expect("rpc client");
    let signer = EthSigner::from_key(ANVIL_ACCOUNT0_KEY).expect("signer");

    let contract = deploy_contract(&built.erc721_init_code, &client, &signer).await;

    let owner = Address::repeat_byte(0x31);
    let approved = Address::repeat_byte(0x32);
    let token_id = U256::from(99);
    let args = vec![
        DynSolValue::Address(owner),
        DynSolValue::Address(approved),
        DynSolValue::Uint(token_id, 256),
    ];

    let receipt = send_contract_call(
        contract,
        &built.erc721_abi_json,
        "emitApproval",
        &args,
        &client,
        &signer,
    )
    .await;
    assert_eq!(receipt.logs().len(), 1, "expected exactly one emitted log");

    let decoded = decode_standard_nft_event(&receipt.logs()[0]).expect("decode approval");
    match decoded {
        DecodedEvent::Erc721Approval(e) => {
            assert_eq!(e.owner, owner);
            assert_eq!(e.approved, approved);
            assert_eq!(e.token_id, token_id);
        }
        other => panic!("unexpected decoded variant: {other:?}"),
    }
}

#[tokio::test]
async fn live_decode_erc721_approval_for_all_from_anvil_receipt() {
    let anvil = require_anvil!();
    let built = match build_contracts_once() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e} — skipping test");
            return;
        }
    };

    let client = RpcClient::new(&anvil.endpoint).expect("rpc client");
    let signer = EthSigner::from_key(ANVIL_ACCOUNT0_KEY).expect("signer");

    let contract = deploy_contract(&built.erc721_init_code, &client, &signer).await;

    let owner = Address::repeat_byte(0x41);
    let operator = Address::repeat_byte(0x42);
    let args = vec![
        DynSolValue::Address(owner),
        DynSolValue::Address(operator),
        DynSolValue::Bool(true),
    ];

    let receipt = send_contract_call(
        contract,
        &built.erc721_abi_json,
        "emitApprovalForAll",
        &args,
        &client,
        &signer,
    )
    .await;
    assert_eq!(receipt.logs().len(), 1, "expected exactly one emitted log");

    let decoded = decode_standard_nft_event(&receipt.logs()[0]).expect("decode approvalForAll");
    match decoded {
        DecodedEvent::Erc721ApprovalForAll(e) => {
            assert_eq!(e.owner, owner);
            assert_eq!(e.operator, operator);
            assert!(e.approved);
        }
        other => panic!("unexpected decoded variant: {other:?}"),
    }
}

#[tokio::test]
async fn live_decode_erc1155_transfer_single_from_anvil_receipt() {
    let anvil = require_anvil!();
    let built = match build_contracts_once() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e} — skipping test");
            return;
        }
    };

    let client = RpcClient::new(&anvil.endpoint).expect("rpc client");
    let signer = EthSigner::from_key(ANVIL_ACCOUNT0_KEY).expect("signer");

    let contract = deploy_contract(&built.erc1155_init_code, &client, &signer).await;

    let operator = Address::repeat_byte(0x71);
    let from = Address::repeat_byte(0x72);
    let to = Address::repeat_byte(0x73);
    let id = U256::from(7);
    let value = U256::from(300);
    let args = vec![
        DynSolValue::Address(operator),
        DynSolValue::Address(from),
        DynSolValue::Address(to),
        DynSolValue::Uint(id, 256),
        DynSolValue::Uint(value, 256),
    ];

    let receipt = send_contract_call(
        contract,
        &built.erc1155_abi_json,
        "emitTransferSingle",
        &args,
        &client,
        &signer,
    )
    .await;
    assert_eq!(receipt.logs().len(), 1, "expected exactly one emitted log");

    let decoded = decode_standard_nft_event(&receipt.logs()[0]).expect("decode transferSingle");
    match decoded {
        DecodedEvent::Erc1155TransferSingle(e) => {
            assert_eq!(e.operator, operator);
            assert_eq!(e.from, from);
            assert_eq!(e.to, to);
            assert_eq!(e.id, id);
            assert_eq!(e.value, value);
        }
        other => panic!("unexpected decoded variant: {other:?}"),
    }
}

#[tokio::test]
async fn live_decode_erc1155_transfer_batch_from_anvil_receipt() {
    let anvil = require_anvil!();
    let built = match build_contracts_once() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e} — skipping test");
            return;
        }
    };

    let client = RpcClient::new(&anvil.endpoint).expect("rpc client");
    let signer = EthSigner::from_key(ANVIL_ACCOUNT0_KEY).expect("signer");

    let contract = deploy_contract(&built.erc1155_init_code, &client, &signer).await;

    let operator = Address::repeat_byte(0x81);
    let from = Address::repeat_byte(0x82);
    let to = Address::repeat_byte(0x83);
    let ids = vec![U256::from(1), U256::from(2), U256::from(3)];
    let values = vec![U256::from(10), U256::from(20), U256::from(30)];
    let id_tokens: Vec<DynSolValue> = ids
        .iter()
        .copied()
        .map(|v| DynSolValue::Uint(v, 256))
        .collect();
    let value_tokens: Vec<DynSolValue> = values
        .iter()
        .copied()
        .map(|v| DynSolValue::Uint(v, 256))
        .collect();
    let args = vec![
        DynSolValue::Address(operator),
        DynSolValue::Address(from),
        DynSolValue::Address(to),
        DynSolValue::Array(id_tokens),
        DynSolValue::Array(value_tokens),
    ];

    let receipt = send_contract_call(
        contract,
        &built.erc1155_abi_json,
        "emitTransferBatch",
        &args,
        &client,
        &signer,
    )
    .await;
    assert_eq!(receipt.logs().len(), 1, "expected exactly one emitted log");

    let decoded = decode_standard_nft_event(&receipt.logs()[0]).expect("decode transferBatch");
    match decoded {
        DecodedEvent::Erc1155TransferBatch(e) => {
            assert_eq!(e.operator, operator);
            assert_eq!(e.from, from);
            assert_eq!(e.to, to);
            assert_eq!(e.ids, ids);
            assert_eq!(e.values, values);
        }
        other => panic!("unexpected decoded variant: {other:?}"),
    }
}

#[tokio::test]
async fn live_decode_erc1155_approval_for_all_from_anvil_receipt() {
    let anvil = require_anvil!();
    let built = match build_contracts_once() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e} — skipping test");
            return;
        }
    };

    let client = RpcClient::new(&anvil.endpoint).expect("rpc client");
    let signer = EthSigner::from_key(ANVIL_ACCOUNT0_KEY).expect("signer");

    let contract = deploy_contract(&built.erc1155_init_code, &client, &signer).await;

    let account = Address::repeat_byte(0x91);
    let operator = Address::repeat_byte(0x92);
    let args = vec![
        DynSolValue::Address(account),
        DynSolValue::Address(operator),
        DynSolValue::Bool(false),
    ];

    let receipt = send_contract_call(
        contract,
        &built.erc1155_abi_json,
        "emitApprovalForAll",
        &args,
        &client,
        &signer,
    )
    .await;
    assert_eq!(receipt.logs().len(), 1, "expected exactly one emitted log");

    let decoded = decode_erc1155_approval_for_all(&receipt.logs()[0])
        .expect("decode erc1155 approvalForAll");
    assert_eq!(decoded.account, account);
    assert_eq!(decoded.operator, operator);
    assert!(!decoded.approved);
}

#[tokio::test]
async fn live_decode_erc1155_uri_from_anvil_receipt() {
    let anvil = require_anvil!();
    let built = match build_contracts_once() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e} — skipping test");
            return;
        }
    };

    let client = RpcClient::new(&anvil.endpoint).expect("rpc client");
    let signer = EthSigner::from_key(ANVIL_ACCOUNT0_KEY).expect("signer");

    let contract = deploy_contract(&built.erc1155_init_code, &client, &signer).await;

    let uri = "ipfs://example/metadata/1".to_string();
    let id = U256::from(777);
    let args = vec![DynSolValue::String(uri.clone()), DynSolValue::Uint(id, 256)];

    let receipt = send_contract_call(
        contract,
        &built.erc1155_abi_json,
        "emitUri",
        &args,
        &client,
        &signer,
    )
    .await;
    assert_eq!(receipt.logs().len(), 1, "expected exactly one emitted log");

    let decoded = decode_standard_nft_event(&receipt.logs()[0]).expect("decode URI");
    match decoded {
        DecodedEvent::Erc1155Uri(e) => {
            assert_eq!(e.value, uri);
            assert_eq!(e.id, id);
        }
        other => panic!("unexpected decoded variant: {other:?}"),
    }
}
