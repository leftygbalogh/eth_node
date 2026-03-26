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
