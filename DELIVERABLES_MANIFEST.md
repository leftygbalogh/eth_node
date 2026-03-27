# Deliverables Manifest

Project: eth_node (Rust)
Date: 2026-03-27
Stage: Stage 6 - Release (in progress)

## Present Deliverables

- Source code: present under src/
- Test suites: present under src/**/tests and crate test modules
- Build/dependency config: Cargo.toml and Cargo.lock present
- FORMAL_SPEC.md: present
- TASK_LIST.md: present
- CHANGELOG.md: present
- GETTING_STARTED.md: present
- CLI_REFERENCE.md: present
- prompts.md and memory.md governance logs: present
- Session capture scripts: scripts/capture-session.sh, scripts/capture-multi.sh
- Session examples: scripts/examples/scenario-send-and-check.txt
- Stage 5 environment validation matrix: output/S5-terminal-env-matrix.md
- Release remote proof: docs/evidence/release-remote-proof.md
- Release checklist: RELEASE_CHECKLIST.md
- Operational notes and rollback: OPERATIONS_AND_ROLLBACK.md
- Post-release monitoring plan: POST_RELEASE_MONITORING.md
- Known failures runbook: RUNBOOK_KNOWN_FAILURES.md

## Conditionally Required / Deferred Deliverables

- PROJECT_BRIEF.md: not present in this repository root; governance scope for this repo uses existing approved artifacts and memory log snapshots.
- REQUIREMENTS_SPEC_MANIFEST.md: present (project-specific equivalent of template requirement manifest).
- IMPLEMENTATION_CHRONICLE.md: replaced by chronicle/ modular chronicle entries.
- docs/user/README.md: deferred; user-facing guidance currently covered by GETTING_STARTED.md and CLI_REFERENCE.md.
- docs/admin/: deferred; operational details captured in OPERATIONS_AND_ROLLBACK.md and RUNBOOK_KNOWN_FAILURES.md.
- docs/api/guide.md: deferred; API usage currently documented in CLI_REFERENCE.md and FORMAL_SPEC.md.
- docs/evidence/pair-programming-log.md: deferred; pair-programming evidence is in chronicle/ PPL logs.
- ADR set: deferred; architecture decisions captured in chronicle/ entries.
- API contract docs: covered by FORMAL_SPEC.md section mappings.
- Traceability report: covered by FORMAL_SPEC.md section 10 and TASK_LIST.md references.
- Security review report / threat model / performance benchmark / SBOM-license report: not yet generated for this release iteration; no production deployment target.

## Release Gating Notes

- This manifest reflects current repository artifacts and explicitly records deferred items.
- Production-security documentation items remain deferred because this iteration targets local Anvil-based development and verification.
- If a production target is introduced, deferred security and SBOM items become mandatory before release approval.
