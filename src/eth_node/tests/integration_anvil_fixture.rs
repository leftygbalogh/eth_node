//! Integration test: Anvil fixture lifecycle.
//!
//! Verifies that AnvilInstance starts, accepts a TCP connection on its port,
//! and shuts down cleanly.  The test is skipped if `anvil` is not on PATH.
//!
//! Spec ref: FORMAL_SPEC.md §9 (T-002 DoD item 1)

mod helpers;
use helpers::anvil_fixture::AnvilInstance;
use std::net::TcpStream;

#[test]
fn anvil_fixture_starts_and_accepts_connections() {
    let instance = AnvilInstance::spawn().expect("failed to spawn anvil");

    // If anvil is not installed, skip gracefully.
    let Some(anvil) = instance else {
        eprintln!("SKIP: anvil not on PATH");
        return;
    };

    // The fixture should be ready — a TCP connect must succeed immediately.
    let addr = format!("127.0.0.1:{}", anvil.port);
    assert!(
        TcpStream::connect(&addr).is_ok(),
        "expected RPC port {addr} to be open"
    );

    // Dropping `anvil` here kills the subprocess.
    drop(anvil);

    // After drop, the port should be closed.
    assert!(
        TcpStream::connect(&addr).is_err(),
        "expected port {addr} to be closed after drop"
    );
}

#[test]
fn anvil_fixture_two_instances_use_different_ports() {
    let a = AnvilInstance::spawn().expect("spawn a");
    let b = AnvilInstance::spawn().expect("spawn b");

    let (Some(a), Some(b)) = (a, b) else {
        eprintln!("SKIP: anvil not on PATH");
        return;
    };

    assert_ne!(a.port, b.port, "two AnvilInstances must use distinct ports");
}
