# Changelog

## 1.2.0 - 2026-03-22

- Applied 9 rules abstracted from Competitor Spy V1 feedback:
  1. External integration contract table (section 6.3) added to FORMAL_SPEC_TEMPLATE; spike policy for unknown contracts added to Stage 2 DoD.
  2. Fix-verification rule: agent may not declare a fix complete without observable evidence; added to 00_INTERACTION_GUARDRAILS and copilot-instructions.
  3. Live E2E non-empty data assertion added to Stage 5 DoD and RELEASE_CHECKLIST.
  4. Project mode remote URL step: agent actively asks for target remote URL before any discovery questions; Template Development mode skips this; updated GOVERNANCE_MODE, copilot-instructions bootstrap rules.
  5. Rebuild-before-E2E and rebuild-before-push rules added to Stage 5 DoD, Stage 6 DoD, Commit Cadence Policy, and RELEASE_CHECKLIST.
  6. Continuous stage-gate feedback and Stage 6 joint post-mortem: feedback written at every stage gate; Stage 6 requires joint agent+owner post-mortem before approval; added to Stage-Level Done Criteria, Stage 6 DoD, and RELEASE_CHECKLIST.
  7. Outbound request assertion requirement added to Testing dimension in 07_QUALITY_DIMENSIONS.
  9. CLI real-time stderr logging default added to Observability dimension and Stage 2 DoD.
  10. PowerShell LASTEXITCODE rule added to powershell-automation-engineer operating rules.

## 1.1.0 - 2026-03-20

- Completed Stage 5 Verify with explicit evidence checks.
- Added Stage 6 release readiness artifacts:
  - `RELEASE_CHECKLIST.md`
  - `OPERATIONS_AND_ROLLBACK.md`
  - `POST_RELEASE_MONITORING.md`
  - `RUNBOOK_KNOWN_FAILURES.md`
  - `GETTING_STARTED.md`
- Recorded Stage 5 approval and transition to Stage 6.

## 1.0.0 - 2026-03-19

- Established baseline AI governance template.
- Completed and approved Stage 1 through Stage 4 artifacts.
- Added mode-first startup flow, stage-gate denial logging, task status discipline, cross-agent routing, Brownfield confidence gate, and Verify-stage auditability requirements.
