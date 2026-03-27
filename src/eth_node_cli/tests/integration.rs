//! Integration tests for the `eth_node_cli` binary.
//!
//! Tests that do not require a live node (help, version, invalid-arg error
//! propagation) run unconditionally.  Tests that need a live Anvil instance
//! detect availability via `ANVIL_ENDPOINT` or by attempting a connection and
//! skip gracefully when Anvil is not reachable.
//!
//! Spec ref: FORMAL_SPEC.md §7.1, T-009 DoD

use std::process::Command;

/// Path to the compiled `eth_node_cli` binary set by Cargo via test env.
fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_eth_node_cli"))
}

// ── No-network smoke tests ────────────────────────────────────────────────────

/// `--help` exits 0 and prints the binary name.
#[test]
fn test_cli_help_exits_zero() {
    let out = binary().arg("--help").output().expect("run binary");
    assert!(out.status.success(), "help exited non-zero: {:?}", out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("eth_node_cli"),
        "help output missing binary name: {stdout}"
    );
}

/// `--version` exits 0.
#[test]
fn test_cli_version_exits_zero() {
    let out = binary().arg("--version").output().expect("run binary");
    assert!(out.status.success(), "version exited non-zero: {:?}", out.status);
}

/// `balance` with no arguments exits non-zero (missing required arg).
#[test]
fn test_cli_balance_missing_arg_fails() {
    let out = binary().arg("balance").output().expect("run binary");
    assert!(
        !out.status.success(),
        "balance with no arg should fail, got: {:?}",
        out.status
    );
}

/// `balance` with an invalid address exits 1 and prints an error.
#[test]
fn test_cli_balance_invalid_address_fails() {
    // Point at a guaranteed-non-existant endpoint so we fail at parse, not network.
    let out = binary()
        .args(["--endpoint", "http://127.0.0.1:19999", "balance", "not-an-address"])
        .output()
        .expect("run binary");
    assert!(
        !out.status.success(),
        "invalid address should exit non-zero, got: {:?}",
        out.status
    );
}

/// `tx-status` with an invalid hash exits 1.
#[test]
fn test_cli_tx_status_invalid_hash_fails() {
    let out = binary()
        .args(["--endpoint", "http://127.0.0.1:19999", "tx-status", "0xNOTAHASH"])
        .output()
        .expect("run binary");
    assert!(
        !out.status.success(),
        "invalid hash should exit non-zero, got: {:?}",
        out.status
    );
}

// ── Anvil-dependent tests ─────────────────────────────────────────────────────

/// Returns the Anvil endpoint if a connection can be established, or `None`.
///
/// Uses a rapid TCP probe so tests skip instantly when Anvil is absent.
fn anvil_endpoint() -> Option<String> {
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = "127.0.0.1:8545";
    TcpStream::connect_timeout(
        &addr.parse().expect("valid addr"),
        Duration::from_millis(200),
    )
    .ok()
    .map(|_| "http://127.0.0.1:8545".to_string())
}

/// `balance <address>` prints a balance line against a live Anvil node.
///
/// Skips when Anvil is not running on 127.0.0.1:8545.
#[test]
fn test_cli_balance() {
    let endpoint = match anvil_endpoint() {
        Some(e) => e,
        None => {
            eprintln!("anvil not reachable — skipping test_cli_balance");
            return;
        }
    };

    let address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    let out = binary()
        .args(["--endpoint", &endpoint, "balance", address])
        .output()
        .expect("run binary");

    assert!(
        out.status.success(),
        "balance failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("wei"),
        "stdout should mention 'wei': {stdout}"
    );
}

/// `tx-status <hash>` for an unknown hash prints "pending" or "not yet mined".
///
/// Skips when Anvil is not running.
#[test]
fn test_cli_tx_status() {
    let endpoint = match anvil_endpoint() {
        Some(e) => e,
        None => {
            eprintln!("anvil not reachable — skipping test_cli_tx_status");
            return;
        }
    };

    // A zero hash is unknown on a fresh Anvil — should print "pending".
    let hash = "0x0000000000000000000000000000000000000000000000000000000000000000";
    let out = binary()
        .args(["--endpoint", &endpoint, "tx-status", hash])
        .output()
        .expect("run binary");

    assert!(
        out.status.success(),
        "tx-status failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("pending"),
        "stdout should mention 'pending': {stdout}"
    );
}

/// `--dump-state` writes a JSON file on success.
///
/// Skips when Anvil is not running.
#[test]
fn test_cli_dump_state_writes_file() {
    use std::path::PathBuf;

    let endpoint = match anvil_endpoint() {
        Some(e) => e,
        None => {
            eprintln!("anvil not reachable — skipping test_cli_dump_state_writes_file");
            return;
        }
    };

    let tmp_path: PathBuf = std::env::temp_dir().join("eth_node_cli_dump_state_test.json");
    std::fs::remove_file(&tmp_path).ok();

    let address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    let out = binary()
        .args([
            "--endpoint",
            &endpoint,
            "--dump-state",
            tmp_path.to_str().expect("utf8 path"),
            "balance",
            address,
        ])
        .output()
        .expect("run binary");

    assert!(out.status.success(), "balance with dump-state failed");
    assert!(tmp_path.exists(), "dump-state file not created at {tmp_path:?}");

    let contents = std::fs::read_to_string(&tmp_path).expect("read dump-state");
    let parsed: serde_json::Value =
        serde_json::from_str(&contents).expect("dump-state is valid JSON");
    assert!(parsed.get("balance_wei").is_some(), "missing balance_wei in dump: {parsed}");

    std::fs::remove_file(&tmp_path).ok();
}

// ── Anvil account fixture ─────────────────────────────────────────────────────

/// Anvil deterministic account 0 private key (well-known, non-secret).
const ANVIL_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
/// Anvil deterministic account 1 address (used as send recipient).
const ANVIL_ADDR_1: &str = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
/// Anvil deterministic account 0 address.
const ANVIL_ADDR_0: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

/// `send <to> <amount> --private-key <key>` broadcasts 0.001 ETH and prints a tx hash.
///
/// Skips when Anvil is not running on 127.0.0.1:8545.
#[test]
fn test_cli_send() {
    let endpoint = match anvil_endpoint() {
        Some(e) => e,
        None => {
            eprintln!("anvil not reachable — skipping test_cli_send");
            return;
        }
    };

    let out = binary()
        .args([
            "--endpoint",
            &endpoint,
            "send",
            ANVIL_ADDR_1,
            "1000000000000000", // 0.001 ETH in wei
            "--private-key",
            ANVIL_KEY,
        ])
        .output()
        .expect("run binary");

    assert!(
        out.status.success(),
        "send failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The CLI prints "Transaction <status>: 0x<hash> in block <n>".
    assert!(
        stdout.contains("0x"),
        "stdout should contain a tx hash: {stdout}"
    );
}

/// `watch <contract>` connects, prints the watching banner, then is killed.
///
/// The test spawns the process, waits 800 ms (long enough for the banner to
/// appear), kills it, and asserts the banner was printed.
///
/// Skips when Anvil is not running on 127.0.0.1:8545.
#[test]
fn test_cli_watch_prints_banner() {
    let endpoint = match anvil_endpoint() {
        Some(e) => e,
        None => {
            eprintln!("anvil not reachable — skipping test_cli_watch_prints_banner");
            return;
        }
    };

    let mut child = binary()
        .args([
            "--endpoint",
            &endpoint,
            "watch",
            "0x0000000000000000000000000000000000000000",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn binary");

    std::thread::sleep(std::time::Duration::from_millis(800));
    let _ = child.kill();
    let out = child.wait_with_output().expect("wait binary");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Watching"),
        "expected 'Watching' banner in stdout: {stdout}"
    );
}

/// `call <contract> <fn> <arg> --abi <json>` against the zero address.
///
/// A zero address holds no code on Anvil, so eth_call returns zero bytes.
/// The CLI exits either 0 (empty/zero decode) or 1 (ABI decode error) — both
/// are acceptable; the test asserts no panic (exit code 0 or 1, not ≥2 or signal).
///
/// Skips when Anvil is not running on 127.0.0.1:8545.
#[test]
fn test_cli_call_graceful() {
    let endpoint = match anvil_endpoint() {
        Some(e) => e,
        None => {
            eprintln!("anvil not reachable — skipping test_cli_call_graceful");
            return;
        }
    };

    let abi = r#"[{"name":"balanceOf","type":"function","inputs":[{"name":"account","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#;

    let out = binary()
        .args([
            "--endpoint",
            &endpoint,
            "call",
            "0x0000000000000000000000000000000000000000",
            "balanceOf",
            ANVIL_ADDR_0,
            "--abi",
            abi,
        ])
        .output()
        .expect("run binary");

    // Graceful exit: 0 (decoded value) or 1 (ABI decode error on empty bytes).
    // Any other code (≥2 or signal) indicates a panic or crash — unacceptable.
    let code = out.status.code().unwrap_or(99);
    assert!(
        code == 0 || code == 1,
        "call should exit 0 or 1 (graceful), got {code}\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Regression test (Stage 5 escaped-defect): `--dump-state` must be accepted
/// when placed AFTER the subcommand, not only before it.
///
/// Clap `global = true` enables this.  Without it, placing `--dump-state`
/// after `balance <address>` would cause Clap to reject it as an unknown arg.
#[test]
fn test_dump_state_flag_accepted_after_subcommand() {
    // Use a non-existent dump-state path — the binary will fail to connect,
    // but the arg parse must succeed (exit code != 2 which is Clap parse error).
    let out = binary()
        .args([
            "--endpoint",
            "http://127.0.0.1:19999", // nothing listening — instant transport error
            "balance",
            ANVIL_ADDR_0,
            "--dump-state",
            "/tmp/test_dump_state_regression.json",
        ])
        .output()
        .expect("run binary");

    let code = out.status.code().unwrap_or(99);
    // Exit 0 (success) or 1 (transport error) are both fine.
    // Exit 2 means Clap parse error — that would be a regression.
    assert_ne!(
        code, 2,
        "--dump-state after subcommand must parse correctly (exit 2 = Clap reject)\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Regression test (S5-D3 / Claire Voyant fix): --abi-file must be accepted
/// and load ABI from a file, avoiding PowerShell shell-quoting issues.
///
/// Writes a minimal ABI JSON to a temp file, passes it via --abi-file,
/// confirms no Clap parse error (exit 2 = arg rejected) and no panic.
/// A transport error (no server on port 19999) is expected and acceptable.
#[test]
fn test_cli_call_accepts_abi_file() {
    use std::io::Write;
    let abi_json = r#"[{"name":"balanceOf","type":"function","inputs":[{"name":"account","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#;
    let path = std::env::temp_dir().join("test_abi_file.json");
    std::fs::File::create(&path)
        .and_then(|mut f| f.write_all(abi_json.as_bytes()))
        .expect("write temp abi file");

    let out = binary()
        .args([
            "--endpoint",
            "http://127.0.0.1:19999",
            "call",
            "--abi-file",
            path.to_str().unwrap(),
            "0x0000000000000000000000000000000000000000",
            "balanceOf",
            ANVIL_ADDR_0,
        ])
        .output()
        .expect("run binary");

    let code = out.status.code().unwrap_or(99);
    assert_ne!(
        code, 2,
        "--abi-file must parse correctly (exit 2 = Clap reject)\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::remove_file(&path);
}
