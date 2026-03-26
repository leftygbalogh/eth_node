# Requirements and Specification Manifest — eth_node (Phase 1)

| Title | Location | Core Contents | Purpose |
|-------|----------|---------------|---------|
| Stage 1 Discovery Record | memory.md | Mode, goal, scope, infra choice, approval delegation, component roadmap, non-goals | Commander's intent and discovery decisions |
| Formal Specification | FORMAL_SPEC.md | FR-001–FR-007, NFR-001–NFR-006, statecharts, contracts, integration contracts, architecture (Q3-ARCH-01), test strategy, traceability matrix | Behavioral contract for all Phase 1 implementation |
| Task List | TASK_LIST.md | Numbered tasks T-000 through T-NNN, dependencies, owners, DoR/DoD status | Implementation planning artifact (Stage 3) |
| Implementation Chronicle | IMPLEMENTATION_CHRONICLE.md | Per-module decisions, tradeoffs, test vectors, alternatives rejected | Audit trail for future maintainers and contributors |
| Release Checklist | RELEASE_CHECKLIST.md | AC-001–006 verification, security check, build artifact, changelog | Stage 6 release gate artifact |
| Runbook Known Failures | RUNBOOK_KNOWN_FAILURES.md | Known failure modes, recovery steps, workarounds | Operations reference |
| Operations and Rollback | OPERATIONS_AND_ROLLBACK.md | Rollback procedure, environment setup, dependency install | Operational safety |
| Post Release Monitoring | POST_RELEASE_MONITORING.md | Observability targets, alert conditions | Monitoring guidance |
| Feedback File | examples/feedback.json | Template improvement proposals raised during this project | Fed back to governance template at Stage 6 close |

## Artifact Status

| Artifact | Status | Required for |
|----------|--------|-------------|
| Stage 1 Discovery Record | ✓ Complete | Stage 2 start |
| Formal Specification | ✓ Draft (pending Stage 2 approval) | Stage 3 start |
| Requirements and Specification Manifest (this file) | ✓ Draft | Stage 2 close |
| Task List | Not started | Stage 4 start |
| Implementation Chronicle | Not started | Stage 5 start |
| Release Checklist | Not started | Stage 6 start |
| Runbook Known Failures | Not started | Stage 6 start |
| Operations and Rollback | Not started | Stage 6 start |
| Post Release Monitoring | Not started | Stage 6 start |

## Usage Rules

1. Update this manifest whenever spec/delivery artifacts change.
2. Stage 2 cannot close if required spec artifacts are missing.
3. Stage 6 cannot close if delivered artifacts diverge from this manifest.
