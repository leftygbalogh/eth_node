//! Integration tests for the `decode-receipt` CLI command.

#[path = "../../eth_node/tests/helpers/mod.rs"]
mod helpers;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use alloy_consensus::TxEip1559;
use alloy_dyn_abi::DynSolValue;
use alloy_primitives::{Address, TxKind, U256};
use eth_node::{
    contract::ContractCaller,
    rpc::RpcClient,
    signer::{EthSigner, UnsignedTx},
    tx::Broadcaster,
};
use helpers::accounts::ANVIL_ACCOUNT0_KEY;
use helpers::anvil_fixture::AnvilInstance;
use serde_json::Value;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_eth_node_cli"))
}

struct BuiltContracts {
    erc721_init_code: Vec<u8>,
    erc721_abi_json: String,
    erc1155_init_code: Vec<u8>,
    erc1155_abi_json: String,
}

static BUILT_CONTRACTS: OnceLock<Result<BuiltContracts, String>> = OnceLock::new();

fn build_contracts_once() -> Result<&'static BuiltContracts, String> {
    let result = BUILT_CONTRACTS.get_or_init(build_contracts);
    result.as_ref().map_err(Clone::clone)
}

fn build_contracts() -> Result<BuiltContracts, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let contracts_dir = resolve_contracts_dir(&manifest_dir)?;
    let out_dir = std::env::temp_dir()
        .join("eth_node_cli_forge_artifacts")
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

    let erc721_artifact = out_dir.join("TestERC721.sol").join("TestERC721.json");
    let erc1155_artifact = out_dir.join("TestERC1155.sol").join("TestERC1155.json");

    let erc721_json = read_artifact_json(&erc721_artifact)?;
    let erc1155_json = read_artifact_json(&erc1155_artifact)?;

    Ok(BuiltContracts {
        erc721_init_code: read_init_code(&erc721_json, &erc721_artifact)?,
        erc721_abi_json: read_abi_json(&erc721_json, &erc721_artifact)?,
        erc1155_init_code: read_init_code(&erc1155_json, &erc1155_artifact)?,
        erc1155_abi_json: read_abi_json(&erc1155_json, &erc1155_artifact)?,
    })
}

fn resolve_contracts_dir(manifest_dir: &Path) -> Result<PathBuf, String> {
    let candidates = [
        manifest_dir.join("tests").join("contracts"),
        manifest_dir
            .join("..")
            .join("eth_node")
            .join("tests")
            .join("contracts"),
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
    alloy_primitives::hex::decode(trimmed)
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

#[test]
fn test_cli_decode_receipt_invalid_hash_fails() {
    let out = binary()
        .args(["decode-receipt", "0xNOTAHASH"])
        .output()
        .expect("run binary");

    assert!(
        !out.status.success(),
        "invalid tx hash should fail, got: {:?}",
        out.status
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("invalid tx hash"),
        "stderr should mention invalid tx hash: {stderr}"
    );
}

#[test]
fn test_cli_decode_receipt_pending_when_unknown() {
    let Some(anvil) = AnvilInstance::spawn().expect("spawn anvil") else {
        eprintln!("anvil not on PATH — skipping test_cli_decode_receipt_pending_when_unknown");
        return;
    };

    let hash = "0x0000000000000000000000000000000000000000000000000000000000000000";
    let out = binary()
        .args(["--endpoint", &anvil.endpoint, "--porcelain", "decode-receipt", hash])
        .output()
        .expect("run binary");

    assert!(
        out.status.success(),
        "decode-receipt failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let parsed: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(parsed["status"], "pending");
    assert_eq!(parsed["hash"], hash);
}

#[tokio::test]
async fn test_cli_decode_receipt_decodes_live_erc721_transfer() {
    let Some(anvil) = AnvilInstance::spawn().expect("spawn anvil") else {
        eprintln!("anvil not on PATH — skipping test_cli_decode_receipt_decodes_live_erc721_transfer");
        return;
    };
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
    let receipt = send_contract_call(
        contract,
        &built.erc721_abi_json,
        "emitTransfer",
        &[
            DynSolValue::Address(from),
            DynSolValue::Address(to),
            DynSolValue::Uint(token_id, 256),
        ],
        &client,
        &signer,
    )
    .await;

    let tx_hash = format!("{:?}", receipt.transaction_hash);
    let out = binary()
        .args([
            "--endpoint",
            &anvil.endpoint,
            "--porcelain",
            "decode-receipt",
            &tx_hash,
        ])
        .output()
        .expect("run binary");

    assert!(
        out.status.success(),
        "decode-receipt failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let parsed: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(parsed["status"], "success");
    assert_eq!(parsed["logs"][0]["decode_status"], "decoded");
    assert_eq!(parsed["logs"][0]["standard"], "erc721");
    assert_eq!(parsed["logs"][0]["event_name"], "Transfer");
    assert_eq!(parsed["logs"][0]["fields"]["from"], from.to_string());
    assert_eq!(parsed["logs"][0]["fields"]["to"], to.to_string());
    assert_eq!(parsed["logs"][0]["fields"]["token_id"], token_id.to_string());
}

#[tokio::test]
async fn test_cli_decode_receipt_handles_shared_approval_for_all() {
    let Some(anvil) = AnvilInstance::spawn().expect("spawn anvil") else {
        eprintln!("anvil not on PATH — skipping test_cli_decode_receipt_handles_shared_approval_for_all");
        return;
    };
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
    let receipt = send_contract_call(
        contract,
        &built.erc1155_abi_json,
        "emitApprovalForAll",
        &[
            DynSolValue::Address(account),
            DynSolValue::Address(operator),
            DynSolValue::Bool(false),
        ],
        &client,
        &signer,
    )
    .await;

    let tx_hash = format!("{:?}", receipt.transaction_hash);

    let ambiguous = binary()
        .args([
            "--endpoint",
            &anvil.endpoint,
            "--porcelain",
            "decode-receipt",
            &tx_hash,
        ])
        .output()
        .expect("run binary");
    assert!(
        ambiguous.status.success(),
        "decode-receipt failed: {}",
        String::from_utf8_lossy(&ambiguous.stderr)
    );

    let ambiguous_json: Value = serde_json::from_slice(&ambiguous.stdout).expect("stdout json");
    assert_eq!(ambiguous_json["logs"][0]["decode_status"], "ambiguous");
    assert_eq!(ambiguous_json["logs"][0]["event_name"], "ApprovalForAll");
    assert_eq!(ambiguous_json["logs"][0]["fields"]["subject"], account.to_string());
    assert_eq!(ambiguous_json["logs"][0]["fields"]["operator"], operator.to_string());
    assert_eq!(ambiguous_json["logs"][0]["fields"]["approved"], false);
    assert_eq!(ambiguous_json["logs"][0]["candidate_standards"][0], "erc721");
    assert_eq!(ambiguous_json["logs"][0]["candidate_standards"][1], "erc1155");

    let forced = binary()
        .args([
            "--endpoint",
            &anvil.endpoint,
            "--porcelain",
            "decode-receipt",
            "--approval-for-all-as",
            "erc1155",
            &tx_hash,
        ])
        .output()
        .expect("run binary");
    assert!(
        forced.status.success(),
        "decode-receipt with override failed: {}",
        String::from_utf8_lossy(&forced.stderr)
    );

    let forced_json: Value = serde_json::from_slice(&forced.stdout).expect("stdout json");
    assert_eq!(forced_json["logs"][0]["decode_status"], "decoded");
    assert_eq!(forced_json["logs"][0]["standard"], "erc1155");
    assert_eq!(forced_json["logs"][0]["event_name"], "ApprovalForAll");
    assert_eq!(forced_json["logs"][0]["fields"]["account"], account.to_string());
    assert_eq!(forced_json["logs"][0]["fields"]["operator"], operator.to_string());
    assert_eq!(forced_json["logs"][0]["fields"]["approved"], false);
}