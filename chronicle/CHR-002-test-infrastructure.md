# Implementation Chronicle — Test Infrastructure

- Chronicle ID: CHR-002
- Source task ID: T-002
- Source spec sections: §8 (Test Strategy), §9 (Integration Contract with Anvil)
- Module / component name: Test infrastructure / Anvil fixture
- Implementation language: Rust
- Status: Draft — awaiting T-002 implementation

## Key Design Constraints (pre-recorded from Claire Voyant review)

- Each integration test binary must spawn its own Anvil instance on a unique random port.
- Port must be confirmed as bound before client connection is attempted (no fixed sleep).
- Anvil process must be terminated via `Drop` — no orphan processes after test completion.
- Private key `0xac0974...` is hard-coded only in test helpers, never in log output (NFR-002).
