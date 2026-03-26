//! Anvil subprocess fixture for integration tests.
//!
//! Starts a fresh Anvil instance on a random available port, waits until the
//! RPC endpoint is ready, then tears down the process on `Drop`.
//!
//! Design decisions (see chronicle/CHR-002-test-infrastructure.md):
//! - One `AnvilInstance` per integration test binary (not per test function).
//! - Random port per instance prevents collisions under parallel `cargo test`.
//! - Readiness polling (HTTP GET, not fixed sleep) keeps startup deterministic.
//! - `Drop` sends SIGKILL/TerminateProcess to prevent orphan processes.
//! - Tests are skipped gracefully when Anvil is not on PATH.
//!
//! Spec ref: FORMAL_SPEC.md §9, §6.3

use std::{
    io,
    net::{TcpListener, TcpStream},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[allow(unused_imports)]
pub use super::accounts::{ANVIL_ACCOUNT0_ADDRESS, ANVIL_ACCOUNT0_KEY, ANVIL_CHAIN_ID};

/// A running Anvil instance.  Killed on drop.
#[allow(dead_code)]
pub struct AnvilInstance {
    child: Child,
    pub port: u16,
    pub endpoint: String,
}

impl AnvilInstance {
    /// Spawn a fresh Anvil instance on a random port.
    ///
    /// Returns `None` if `anvil` is not on PATH — callers should skip the test.
    /// Returns `Err` if the process started but never became ready within 10 s.
    pub fn spawn() -> io::Result<Option<Self>> {
        // Check that anvil exists on PATH before trying to spawn.
        if which_anvil().is_none() {
            return Ok(None);
        }

        let port = free_port()?;
        let child = Command::new("anvil")
            .args(["--port", &port.to_string(), "--silent"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let endpoint = format!("http://127.0.0.1:{port}");

        // Poll until RPC is accepting connections (up to 10 s).
        wait_for_rpc(&endpoint, Duration::from_secs(10))?;

        Ok(Some(AnvilInstance { child, port, endpoint }))
    }

    /// WebSocket endpoint for this Anvil instance.
    ///
    /// Anvil serves WebSocket on the same port as HTTP.
    #[allow(dead_code)]
    pub fn ws_endpoint(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }
}

impl Drop for AnvilInstance {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find a free TCP port by binding to port 0 and reading the assigned port.
fn free_port() -> io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
    // listener drops here, freeing the port.
}

/// Poll the RPC endpoint until a TCP connection succeeds or the timeout expires.
fn wait_for_rpc(endpoint: &str, timeout: Duration) -> io::Result<()> {
    // Parse host:port from the URL (no external HTTP library needed at this stage).
    let addr = endpoint
        .trim_start_matches("http://")
        .trim_end_matches('/');

    let deadline = Instant::now() + timeout;
    loop {
        if TcpStream::connect(addr).is_ok() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("anvil at {addr} did not become ready within {timeout:?}"),
            ));
        }
        thread::sleep(Duration::from_millis(50));
    }
}

/// Returns `Some(path)` if `anvil` is found on PATH, else `None`.
fn which_anvil() -> Option<std::path::PathBuf> {
    // Use `where` on Windows, `which` on Unix — but std::process::Command does
    // its own PATH search, so a dry-run probe is the portable approach.
    std::env::var_os("PATH").and_then(|path_var| {
        std::env::split_paths(&path_var).find_map(|dir| {
            let candidate = dir.join(if cfg!(windows) { "anvil.exe" } else { "anvil" });
            if candidate.is_file() { Some(candidate) } else { None }
        })
    })
}
