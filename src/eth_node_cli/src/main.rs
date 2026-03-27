//! CLI binary — thin wrapper over `eth_node`.
//!
//! Spec ref: FORMAL_SPEC.md §7.1 CLI-to-API mapping, §5.1 observability, NFR-004
//!
//! **Architecture constraint:** No business logic here — only argument parsing,
//! `eth_node::*` function calls, and result formatting.

use std::path::PathBuf;

use alloy_dyn_abi::DynSolValue;
use alloy_primitives::{Address, B256, U256};
use alloy_rpc_types::Filter;
use clap::{Parser, Subcommand};
use eth_node::{
    contract::ContractCaller,
    events::Listener,
    primitives::event_selector,
    rpc::RpcClient,
    signer::EthSigner,
    tx::{Broadcaster, TxBuilder},
};
use futures::StreamExt;
use serde_json::{json, Value};
use tracing::{error, info};

// ── CLI schema ────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "eth_node_cli",
    about = "Ethereum toolkit CLI (Phase 1)",
    version
)]
struct Cli {
    /// RPC endpoint URL.
    #[arg(long, env = "ETH_ENDPOINT", default_value = "http://127.0.0.1:8545")]
    endpoint: String,

    /// Suppress all output below error level.
    #[arg(long, conflicts_with = "log_level")]
    quiet: bool,

    /// Log level: trace, debug, info, warn, error.
    #[arg(long, default_value = "info", value_name = "LEVEL")]
    log_level: String,

    /// Write operation result as JSON to this path on success.
    #[arg(long, value_name = "PATH", global = true)]
    dump_state: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the ETH balance of an address.
    Balance {
        /// Ethereum address (0x-prefixed hex).
        address: String,
    },

    /// Send ETH to an address.
    Send {
        /// Recipient address (0x-prefixed hex).
        to: String,

        /// Amount to send in Wei (decimal).
        amount_wei: String,

        /// Sender private key hex (or set ETH_PRIVATE_KEY env var).
        #[arg(long, env = "ETH_PRIVATE_KEY")]
        private_key: String,
    },

    /// Watch and print logs emitted by a contract.
    Watch {
        /// Contract address to filter logs from.
        contract: String,

        /// Optional topic-0 filter: full event signature like
        /// "Transfer(address,address,uint256)" or a 0x-prefixed 32-byte hash.
        #[arg(value_name = "EVENT")]
        event: Option<String>,
    },

    /// Call a view function on a deployed contract and print decoded results.
    Call {
        /// Contract address (0x-prefixed hex).
        contract: String,

        /// Solidity function name.
        function: String,

        /// Function arguments (parsed as address, uint256, bool, bytes, or string).
        args: Vec<String>,

        /// ABI as an inline JSON string.
        /// Tip: if your shell mangles the quotes, use --abi-file instead.
        #[arg(long, value_name = "JSON", conflicts_with = "abi_file")]
        abi: Option<String>,

        /// Path to a file containing the ABI JSON.
        /// Avoids shell-quoting problems with inline JSON on Windows/PowerShell.
        #[arg(long, value_name = "PATH", conflicts_with = "abi")]
        abi_file: Option<PathBuf>,
    },

    /// Print the receipt for a transaction, or "pending" if not yet mined.
    TxStatus {
        /// Transaction hash (0x-prefixed hex).
        hash: String,
    },
}

// ── Entrypoint ────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_logging(&cli);

    let result = run(&cli).await;

    match &result {
        Ok(state) => {
            if let Some(path) = &cli.dump_state {
                let json_str = serde_json::to_string_pretty(state)
                    .unwrap_or_else(|_| "{}".to_string());
                if let Err(e) = std::fs::write(path, json_str) {
                    error!(%e, "failed to write dump-state file");
                }
            }
        }
        Err(e) => {
            error!(error = %e, "command failed");
            std::process::exit(1);
        }
    }
}

async fn run(cli: &Cli) -> Result<Value, String> {
    let client = RpcClient::new(&cli.endpoint).map_err(|e| e.to_string())?;

    match &cli.command {
        Commands::Balance { address } => cmd_balance(address, &client).await,
        Commands::Send { to, amount_wei, private_key } => {
            cmd_send(to, amount_wei, private_key, &client).await
        }
        Commands::Watch { contract, event } => {
            cmd_watch(contract, event.as_deref(), &cli.endpoint).await
        }
        Commands::Call { contract, function, args, abi, abi_file } => {
            let abi_json = resolve_abi(abi.as_deref(), abi_file.as_deref())?;
            cmd_call(contract, function, args, &abi_json, &client).await
        }
        Commands::TxStatus { hash } => cmd_tx_status(hash, &client).await,
    }
}

// ── Commands ──────────────────────────────────────────────────────────────────

async fn cmd_balance(address: &str, client: &RpcClient) -> Result<Value, String> {
    let addr: Address = address
        .parse()
        .map_err(|_| format!(
            "invalid address '{}' — addresses must start with 0x and be exactly 42 hex characters (e.g. 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266)",
            address
        ))?;

    let wei = client.get_balance(addr).await.map_err(|e| e.to_string())?;

    info!(%address, %wei, "balance queried");

    let wei_str = wei.to_string();
    println!("Balance: {wei_str} wei");

    Ok(json!({ "address": address, "balance_wei": wei_str }))
}

async fn cmd_send(
    to: &str,
    amount_wei: &str,
    private_key: &str,
    client: &RpcClient,
) -> Result<Value, String> {
    let to_addr: Address = to
        .parse()
        .map_err(|e| format!("invalid address: {e}"))?;

    let value: U256 = amount_wei
        .parse()
        .map_err(|_| format!("invalid amount: {amount_wei}"))?;

    let signer = EthSigner::from_key(private_key).map_err(|e| e.to_string())?;

    let chain_id = client.chain_id().await.map_err(|e| e.to_string())?;

    let unsigned = TxBuilder::new(chain_id, signer.address(), to_addr)
        .value(value)
        .build(client)
        .await
        .map_err(|e| e.to_string())?;

    let signed = signer.sign(unsigned).map_err(|e| e.to_string())?;

    info!(hash = %signed.hash, to = %to_addr, "sending transaction");

    let receipt = Broadcaster::new()
        .send(&signed, client)
        .await
        .map_err(|e| e.to_string())?;

    let hash = format!("{:?}", receipt.transaction_hash);
    let block = receipt.block_number.unwrap_or(0);
    let status = if receipt.status() { "success" } else { "reverted" };

    println!("Transaction {status}: {hash} in block {block}");

    Ok(json!({
        "transaction_hash": hash,
        "block_number": block,
        "status": status,
    }))
}

async fn cmd_watch(
    contract: &str,
    event: Option<&str>,
    endpoint: &str,
) -> Result<Value, String> {
    let addr: Address = contract
        .parse()
        .map_err(|e| format!("invalid address: {e}"))?;

    let mut filter = Filter::new().address(addr);

    // Optional topic-0 filter.
    if let Some(sig) = event {
        let topic0 = event_selector(sig);
        filter = filter.event_signature(topic0);
        info!(%contract, %topic0, "watching with topic-0 filter");
    } else {
        info!(%contract, "watching all events");
    }

    let listener = Listener::new(endpoint);
    let mut stream = listener.subscribe(filter);

    println!("Watching contract {contract} for events (Ctrl-C to stop)...");

    let mut count = 0u64;

    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl-C")
    };
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            biased;
            _ = &mut ctrl_c => {
                println!("\nStopped. Received {count} event(s).");
                break;
            }
            item = stream.next() => {
                match item {
                    Some(Ok(log)) => {
                        count += 1;
                        let tx_hash = log
                            .transaction_hash
                            .map(|h| format!("{h:?}"))
                            .unwrap_or_default();
                        println!(
                            "Event #{count}: tx={tx_hash} topics={}",
                            log.topics().len()
                        );
                        info!(tx_hash, topics = log.topics().len(), "event received");
                    }
                    Some(Err(e)) => {
                        error!(error = %e, "stream error");
                        return Err(e.to_string());
                    }
                    None => {
                        println!("Stream ended.");
                        break;
                    }
                }
            }
        }
    }

    Ok(json!({ "contract": contract, "events_received": count }))
}

/// Resolve the ABI JSON from either `--abi` (inline string) or `--abi-file` (path).
///
/// Returns an error if neither is provided or the file cannot be read.
fn resolve_abi(abi: Option<&str>, abi_file: Option<&std::path::Path>) -> Result<String, String> {
    if let Some(json) = abi {
        return Ok(json.to_owned());
    }
    if let Some(path) = abi_file {
        return std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read --abi-file {}: {e}", path.display()));
    }
    Err("one of --abi or --abi-file is required".to_owned())
}

async fn cmd_call(
    contract: &str,
    function: &str,
    args: &[String],
    abi: &str,
    client: &RpcClient,
) -> Result<Value, String> {
    let addr: Address = contract
        .parse()
        .map_err(|e| format!("invalid address: {e}"))?;

    let caller = ContractCaller::new(addr, abi).map_err(|e| e.to_string())?;

    let dyn_args: Vec<DynSolValue> = args.iter().map(|s| parse_dyn_sol_value(s)).collect();

    info!(%contract, function, args_count = dyn_args.len(), "calling contract");

    let tokens = caller
        .call(function, &dyn_args, client)
        .await
        .map_err(|e| e.to_string())?;

    let formatted: Vec<String> = tokens.iter().map(|t| format!("{t:?}")).collect();
    println!("Return: {}", formatted.join(", "));

    Ok(json!({
        "contract": contract,
        "function": function,
        "return_values": formatted,
    }))
}

async fn cmd_tx_status(hash: &str, client: &RpcClient) -> Result<Value, String> {
    let tx_hash: B256 = hash
        .parse()
        .map_err(|e| format!("invalid tx hash: {e}"))?;

    info!(%hash, "querying transaction status");

    let receipt = client
        .get_transaction_receipt(tx_hash)
        .await
        .map_err(|e| e.to_string())?;

    match receipt {
        Some(r) => {
            let block = r.block_number.unwrap_or(0);
            let status = if r.status() { "success" } else { "reverted" };
            println!("Transaction {hash}: {status} in block {block}");
            Ok(json!({ "hash": hash, "status": status, "block_number": block }))
        }
        None => {
            println!("Transaction {hash}: pending (not yet mined)");
            Ok(json!({ "hash": hash, "status": "pending" }))
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Best-effort parse of a CLI string into a [`DynSolValue`].
///
/// Attempts in order: Address (0x + 40 hex chars), Bool, Bytes
/// (0x + even-length hex), Uint256 (decimal), String.
fn parse_dyn_sol_value(s: &str) -> DynSolValue {
    // All "0x"-prefixed values — address or raw bytes.
    if s.starts_with("0x") || s.starts_with("0X") {
        let hex_part = &s[2..];

        // Address: exactly 40 hex chars.
        if hex_part.len() == 40 {
            if let Ok(addr) = s.parse::<Address>() {
                return DynSolValue::Address(addr);
            }
        }

        // Bytes: even-length hex string.
        if hex_part.len().is_multiple_of(2) {
            if let Ok(bytes) = alloy_primitives::hex::decode(hex_part) {
                return DynSolValue::Bytes(bytes);
            }
        }
    }

    // Bool.
    if s.eq_ignore_ascii_case("true") {
        return DynSolValue::Bool(true);
    }
    if s.eq_ignore_ascii_case("false") {
        return DynSolValue::Bool(false);
    }

    // Uint256 (decimal string).
    if let Ok(n) = s.parse::<U256>() {
        return DynSolValue::Uint(n, 256);
    }

    // Fallback: Solidity string.
    DynSolValue::String(s.to_string())
}

/// Initialise `tracing-subscriber` with a JSON layer on stderr.
fn init_logging(cli: &Cli) {
    use tracing_subscriber::{fmt, EnvFilter};

    let level = if cli.quiet { "error" } else { cli.log_level.as_str() };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));

    fmt()
        .json()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_address_arg() {
        let val = parse_dyn_sol_value("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        assert!(matches!(val, DynSolValue::Address(_)));
    }

    #[test]
    fn parse_bool_true_arg() {
        let val = parse_dyn_sol_value("true");
        assert!(matches!(val, DynSolValue::Bool(true)));
    }

    #[test]
    fn parse_bool_false_arg() {
        let val = parse_dyn_sol_value("false");
        assert!(matches!(val, DynSolValue::Bool(false)));
    }

    #[test]
    fn parse_uint256_decimal() {
        let val = parse_dyn_sol_value("1000000000000000000");
        assert!(matches!(val, DynSolValue::Uint(_, 256)));
    }

    #[test]
    fn parse_bytes_hex_arg() {
        let val = parse_dyn_sol_value("0xdeadbeef");
        assert!(matches!(val, DynSolValue::Bytes(_)));
    }

    #[test]
    fn parse_string_fallback_arg() {
        let val = parse_dyn_sol_value("hello");
        assert!(matches!(val, DynSolValue::String(_)));
    }

    #[test]
    fn topic0_from_known_transfer_signature() {
        // keccak256("Transfer(address,address,uint256)") known constant
        let topic = event_selector("Transfer(address,address,uint256)");
        let hex = format!("{topic:?}");
        assert_eq!(
            hex,
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
        );
    }

    #[test]
    fn topic0_passthrough_existing_hash() {
        let hash = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
        let topic = event_selector(hash);
        assert_eq!(format!("{topic:?}"), hash);
    }
}

