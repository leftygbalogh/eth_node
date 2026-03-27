# Release Verification Evidence (2026-03-27)

## Commands Run

- cargo build
- cargo test

## Results

- cargo build: PASS
- cargo test: PASS

## Test Summary

- eth_node unit tests: 79 passed
- integration_anvil_fixture: 2 passed
- integration_contract: 6 passed
- integration_events: 5 passed
- integration_rpc: 6 passed
- integration_tx: 5 passed
- eth_node_cli unit tests: 8 passed
- eth_node_cli integration tests: 20 passed
- doc-tests: 6 passed

Total passed: 137
Total failed: 0

## Notes

- This evidence run validates current Stage 6 release-doc baseline.
- A final rebuild-from-HEAD may still be re-run immediately before final Stage 6 release closure commit/push to satisfy strict timestamp gating.
