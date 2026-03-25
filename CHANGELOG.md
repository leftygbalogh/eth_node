# Changelog

## 1.6.0 - 2026-03-26

Pass 4 simplification — continued reduction without loss of control:

- **A** (`01_DECISION_POLICY.md`): Removed `Conflict Resolution Policy` section (exact duplicate of `00_INTERACTION_GUARDRAILS.md` §7).
- **B** (`01_DECISION_POLICY.md`): Condensed `Detailed Brief Handling Policy` from a 5-item checklist to 2 sentences.
- **C** (`02_WORKFLOW_STAGES.md`): Removed final easter egg DoD line missed in Pass 1 (Stage 1 DoD section 1.3 reference).
- **D** (`02_WORKFLOW_STAGES.md`): Stage Gate Enforcement — replaced 4 duplicate bullets with a single cross-reference to `01_DECISION_POLICY.md` Stage Gate Policy; kept only operationally unique items.
- **E** (`02_WORKFLOW_STAGES.md`): Condensed all specialist agent invocation entries (Stages 1–5) from multi-sentence paragraphs with quoted prompts to single-line challenge focus + block authority; ~60 lines removed.
- **F** (`02_WORKFLOW_STAGES.md`): Universal DoR items 8–10 moved under a `### Conditional Extensions` sub-header; each item prefixed with its trigger condition in italics, making the conditional nature immediately visible.

## 1.5.0 - 2026-03-26

Three-pass simplification of the governance template:

**Pass 1 — Stale references and project-specific cruft removed:**
- Fixed 2× `templates/feedback.json` → `examples/feedback.json` in `02_WORKFLOW_STAGES.md` and `RUNBOOK_KNOWN_FAILURES_TEMPLATE.md`.
- Replaced stale `REQUIREMENTS_SPEC_MANIFEST.md` / `DELIVERABLES_MANIFEST.md` root-file references with template-path references in Stage 2/6 DoD.
- Removed Easter Egg–specific DoD items from Stage 1, 5, and 6.
- Repaired broken "Joint post-mortem" DoD line.
- Removed Easter Egg references from `07_QUALITY_DIMENSIONS.md` §9 and §10.
- Expanded `00_INTERACTION_GUARDRAILS.md` §10 from placeholder to 4 concrete examples; collapsed inline version history → single version line pointing to `CHANGELOG.md`.

**Pass 2 — Structural duplication removed:**
- Deleted duplicated `Clarification Routing Protocol` from `03_TEAM_MODEL_HANDOFFS.md`; replaced with cross-reference to `06`.
- Simplified `01_DECISION_POLICY.md` "Assumption and Clarification Policy" to principle + cross-reference.
- Deleted `01_DECISION_POLICY.md` "Itemized Workflow Policy" (exact duplicate of `00` §3).

**Pass 3 — Content and structure simplified:**
- Merged `§3 Communication Style` + `§7 Response Formatting` in `00` into single `§3 Communication Style and Formatting`; sections renumbered, pre-existing duplicate §13 fixed, §14 Easter Egg Logging Guardrail removed.
- `04_PERSONA_DIRECTORY.md`: removed hardcoded language priority; collapsed agent file index into compact category table referencing `agents/`.
- `05_IDLE_AUTOMATION.md`: removed "Optional Timing Overrides" section; added one-line note directing to script headers.
- `06_COMMAND_CHAIN_AND_PERSONALITY.md`: condensed Personality Archetypes to role/focus table.

## 1.3.1 - 2026-03-25

Applied improvements from `examples/feedback.json` (Competitor Spy V1 and V2 post-mortems):

1. **Chronicle template** — Added mandatory `§7.1 Verification Artifact` field. Entries without this field are structurally incomplete; acceptable evidence forms listed explicitly.
2. **Stage 4 DoD** (`governance/02_WORKFLOW_STAGES.md`) — Added conditional DoD item: for projects with external live/paid APIs, a successful live E2E run with non-zero records is required before Stage 4 closes. Mock-green is not sufficient.
3. **Release checklist** (`templates/RELEASE_CHECKLIST_TEMPLATE.md`) — Added repo layout audit step at Stage 6: verify no stray files in root, `output/` and `build/` are gitignored, folders are clearly separated.
4. **Formal spec template** — Added `§5.1 Observability and CLI UX`: real-time stderr logging default for V1 CLI projects with suppression mechanism. Added `§6.4 Credential Model`: passphrase acquisition order, test suppression requirement, stdin-blocking acceptance criterion.
5. **Task list T-000** — Added `output/` and `build/` (both gitignored) to default layout convention and all language-specific overrides.
6. **PowerShell agent** — `$LASTEXITCODE` over `try/catch` rule confirmed present from v1.2; no change needed.

## 1.3.0 - 2026-03-25

- Restructured repository layout to cleanly separate concerns:
  - Governance policy docs (`00`–`07`) moved to `governance/`.
  - Operational runbooks moved to `templates/` as fillable templates.
  - New directories: `src/` (source code), `config/` (configuration), `build/` (gitignored), `output/` (gitignored).
  - `build/` and `output/` added to `.gitignore`.
- Removed all snake-game project artifacts that had leaked into the template root.
- `templates/feedback.json` duplicate removed; canonical copy is `examples/feedback.json`.
- Updated README with full layout diagram and revised discovery order.
- Updated GETTING_STARTED with layout table and revised quick-start steps.
- Updated `copilot-instructions.md` discovery order and bootstrap rules to reference new paths.
- Added T-000 layout convention to bootstrap rules: `src/`, `config/`, `output/`, `build/` must be established before source files are created.

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
