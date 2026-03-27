# Release Checklist

## Stage 5 Verify Results

- [x] Syntax check passes for deliverable binaries and scripts.
- [x] Startup, event handling, and main behaviors verified against formal spec.
- [x] Runtime error and edge-case handling paths implemented and verified.
- [x] Output and artifact requirements implemented as specified.
- [x] No extra out-of-scope features added.
- [x] Fresh release rebuild from HEAD completed for current verification run (see docs/evidence/release-verification-2026-03-27.md).
- [x] External live-data E2E non-empty record check not applicable for local Anvil-only scope.

## Stage 6 Release

- [x] Deliverables manifest created and aligned: DELIVERABLES_MANIFEST.md.
- [x] Logging and artifact requirements documented and present.
- [x] Governance artifacts for stages 1-6 updated.
- [x] Repo layout reviewed: governance/source/output separation retained.
- [x] Fresh rebuild from HEAD completed immediately before final release commit and push.
- [x] Push target verified with git remote evidence.
- [x] Intended remote and branch stated (origin/master).
- [x] Release remote proof snapshot saved: docs/evidence/release-remote-proof.md.
- [x] Joint post-mortem feedback cycle complete in examples/feedback.json with owner response.

## Stage 5 Approval

- Approved by: Lefty
- Approval date: 2026-03-27
- Notes: Stage 5 gate approved explicitly.

## Stage 6 Approval

- Approved by: Lefty
- Approval date: 2026-03-27
- Notes: Stage 6 gate approved after joint post-mortem owner pass and final verification.
