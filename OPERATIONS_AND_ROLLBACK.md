# Operational Notes and Rollback Plan

## Operational Notes

- Primary runtime target is local Anvil devnet for deterministic verification.
- CLI capture helpers are the first-line diagnostic tools:
  - scripts/capture-session.sh for single-command sessions
  - scripts/capture-multi.sh for scripted multi-step runs
- Session artifacts are written to output/sessions/<timestamp>/ including screen.log and state JSON files.
- Governance records must remain current in prompts.md and memory.md before and after stage transitions.

## Rollback Triggers

- Stage action executed without required explicit approval.
- Release evidence missing or contradictory.
- Regression discovered in CLI command behavior or output contract.
- Traceability mismatch between tasks, tests, and spec references.

## Rollback Procedure

1. Identify the last known-good commit with passing verification evidence.
2. Create a recovery branch from current HEAD.
3. Revert offending commits using non-destructive git revert.
4. Re-run targeted tests and manual session capture scenarios.
5. Update memory.md with blocker, impact, and recovery decision.
6. Re-open stage gate and request explicit re-approval.

## Safety Rule

- Do not use destructive history rewrite on shared branches.
