//! Test accounts for Anvil integration tests.
//!
//! Anvil always starts with the same deterministic funded accounts derived
//! from mnemonic: "test test test test test test test test test test test junk"
//!
//! Spec ref: FORMAL_SPEC.md §9.2
#![allow(dead_code)]

/// Private key for Anvil account 0.
/// ONLY for use in local test fixtures — never in production.
pub const ANVIL_ACCOUNT0_KEY: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

/// Address for Anvil account 0 (checksummed).
pub const ANVIL_ACCOUNT0_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

/// Chain ID used by the default Anvil instance.
pub const ANVIL_CHAIN_ID: u64 = 31337;

/// Starting ETH balance of each funded Anvil account (in Wei: 10_000 ETH).
pub const ANVIL_INITIAL_BALANCE_WEI: u128 = 10_000 * 1_000_000_000_000_000_000u128;
