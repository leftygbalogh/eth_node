//! eth_node — Rust Ethereum toolkit library
//!
//! Phase 1 modules: primitives, rpc, signer, tx, events, contract.
//! Phase 2 modules: executor, quality, upstream_contrib (T-001 onwards).
//! See FORMAL_SPEC.md for behavioral contracts.

// Phase 1 modules
pub mod primitives;
pub mod rpc;
pub mod signer;
pub mod tx;
pub mod events;
pub mod contract;

// Phase 2 modules (uncomment as implemented)
// pub mod executor;
// pub mod quality;
// pub mod upstream_contrib;
