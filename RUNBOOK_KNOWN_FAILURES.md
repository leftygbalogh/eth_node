# Runbook for Known Failure Scenarios

## Scenario 1: Anvil Not Running or Port Conflict

- Symptom: CLI commands fail to connect to http://127.0.0.1:8545.
- Expected action:
1. Check whether an Anvil process is already bound to port 8545.
2. Start a fresh instance when needed.
3. Re-run command through scripts/capture-session.sh to preserve evidence.

## Scenario 2: tx-status Uses Placeholder Hash

- Symptom: tx-status returns pending for copied documentation hash.
- Expected action:
1. Use the transaction hash printed by the preceding send command.
2. Re-run tx-status with the real hash.
3. Capture output in output/sessions/<timestamp>/screen.log.

## Scenario 3: watch Session Shows No Events

- Symptom: watch command connects but no event lines appear.
- Expected action:
1. Confirm watched contract emits events for triggered action.
2. Prefer the Receiver walkthrough contract for plain ETH send demonstrations.
3. Trigger two known test sends and verify Event #N lines appear.

## Scenario 4: Shell Compatibility Differences

- Symptom: command works in one shell but not another.
- Expected action:
1. Check output/S5-terminal-env-matrix.md for known validated combinations.
2. Reproduce in both PowerShell and Git Bash when possible.
3. Record any new shell-specific gap in release notes and memory.md.

## Interactive CLI Diagnostic Procedure

1. Run scripts/capture-session.sh <subcommand> [args...].
2. For multi-step sequences, run scripts/capture-multi.sh <commands-file>.
3. Collect artifacts from output/sessions/<timestamp>/.
4. Attach screen.log and relevant state JSON file to defect notes.

## Known Environment Gaps

- Git Bash path translation and toolchain path discovery can differ from PowerShell.
- jq may be unavailable in default Windows environments; PowerShell ConvertFrom-Json is the documented fallback.
- /tmp path assumptions in docs depend on Git Bash mapping to Windows temp directory.
