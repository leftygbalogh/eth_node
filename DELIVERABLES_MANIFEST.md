# Deliverables Manifest

Purpose: concrete inventory of produced artifacts for Rust Terminal Snake Game cycle.

## Delivered Artifacts

| Deliverable | Location | Status |
|---|---|---|
| Rust game core | src/lib.rs | Delivered |
| Rust terminal runtime | src/main.rs | Delivered |
| Build config | Cargo.toml | Delivered |
| Bash launcher | run_snake.sh | Delivered |
| Project Brief | PROJECT_BRIEF.md | Delivered |
| Formal Spec | FORMAL_SPEC.md | Delivered |
| Task List | TASK_LIST.md | Delivered |
| Implementation Chronicle | IMPLEMENTATION_CHRONICLE.md | Delivered |
| Release Checklist | RELEASE_CHECKLIST.md | Delivered |
| Operations and rollback | OPERATIONS_AND_ROLLBACK.md | Delivered |
| Known failures runbook | RUNBOOK_KNOWN_FAILURES.md | Delivered |
| Post-release monitoring plan | POST_RELEASE_MONITORING.md | Delivered |
| Getting started guide | GETTING_STARTED.md | Delivered |
| Changelog | CHANGELOG.md | Delivered |
| ADR | docs/adr/ADR-0001-layered-architecture.md | Delivered |
| API guide | docs/api/guide.md | Delivered |
| User guide | docs/user/README.md | Delivered |
| Admin guide | docs/admin/README.md | Delivered |
| Traceability report | docs/evidence/traceability.md | Delivered |
| Environment matrix | docs/evidence/environment-matrix.md | Delivered |
| Manual test report | docs/evidence/manual-test-report.md | Delivered |

## Verification Evidence

- cargo test -q passed.
- cargo build --release passed.

## Known Gaps

- Git Bash smoke execution is validated; Linux/WSL runtime parity checks remain tracked in docs/evidence/environment-matrix.md.
