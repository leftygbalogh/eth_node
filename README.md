# eth_node — Rust Ethereum Toolkit

A learning project exploring the Rust Ethereum ecosystem.
Built on the [Alloy](https://github.com/alloy-rs/alloy) crate suite with [Foundry Anvil](https://github.com/foundry-rs/foundry) as the local devnet.

## Project Status

- Stage 4 (Build) — In Progress
- Current task: T-000 (workspace scaffold)

## Prerequisites

1. **Rust** (stable toolchain): https://rustup.rs
2. **Foundry** (Anvil): https://getfoundry.sh
   ```sh
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   ```
3. Verify both are installed:
   ```sh
   rustc --version
   anvil --version
   ```

## Local Setup

4. Clone the repository:
   ```sh
   git clone https://github.com/leftygbalogh/eth_node.git
   cd eth_node
   ```
5. Build the workspace:
   ```sh
   cargo build
   ```
6. Run the test suite (Anvil starts automatically as a test fixture):
   ```sh
   cargo test
   ```
7. Run the CLI binary:
   ```sh
   cargo run --bin eth_node_cli
   ```

## Project Layout

```
Cargo.toml               — Workspace manifest (alloy ecosystem, pinned)
src/
  eth_node/              — Library crate (all business logic)
    src/
      lib.rs
      primitives.rs      — Address, H256, U256, ABI, RLP helpers
      rpc.rs             — JSON-RPC client (Alloy provider)
      signer.rs          — EIP-1559 transaction signing
      tx.rs              — Transaction builder + broadcaster
      events.rs          — Event/log listener
      contract.rs        — ABI contract caller
    tests/               — Integration tests (require Anvil)
  eth_node_cli/          — CLI binary (thin wrapper over eth_node)
chronicle/               — Implementation decisions per module
output/                  — Session capture artifacts (gitignored)
scripts/                 — Session capture and idle-guard scripts
```

## Configuration

Set these environment variables before running:

| Variable | Purpose | Example |
|---|---|---|
| `ETH_RPC_URL` | RPC endpoint | `http://127.0.0.1:8545` |
| `ETH_PRIVATE_KEY` | Signing key — never commit | see Anvil output |
| `ETH_CHAIN_ID` | Target chain ID | `31337` (Anvil default) |

## Governance

Stages 1–3 approved. Full specification in [FORMAL_SPEC.md](FORMAL_SPEC.md) and task planning in [TASK_LIST.md](TASK_LIST.md).
