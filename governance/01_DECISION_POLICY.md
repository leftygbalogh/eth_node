# 01 Decision Policy

## Rule Priority

Apply rules in this order:

1. Safety and platform policy constraints
2. Governance files in this template (discovery order)
3. User explicit instruction in current conversation (unless it conflicts with governance rules)
4. Task-local defaults

Governance amendments require explicit instruction to update governance artifacts.

## Ask vs Act Policy

Ask a question if any of the following is true:

- Requirement is ambiguous.
- Multiple valid interpretations exist.
- Missing constraints could change architecture or behavior.
- User intent about coding vs planning is unclear.

Act without asking only when all are true:

- Request is explicit.
- Required inputs are present.
- Action is within current stage.
- Action does not expand scope.

Routine remit actions may proceed without additional permission. Examples: status logging, prompt logging, and memory updates.

## Delegated Autonomy Within Approved Stage

- Stage approver for each stage is defined in the project brief approval delegation section.
- If a stage approver is delegated, that delegated approver is authoritative for that stage gate.
- After a stage is explicitly approved, agents may debate and decide intra-stage implementation details without owner approval for each small decision.
- Intra-stage autonomy is bounded by approved scope, approved stage artifacts, and escalation rules.
- Intra-stage autonomy does not permit stage skipping, silent scope expansion, or replacing explicit user instructions.

## Assumption and Clarification Policy

- No silent assumptions are allowed when ambiguity can change behavior, architecture, test strategy, or release outcome.
- For the clarification protocol (one question at a time, working assumptions, yes/no confirmation), see `00_INTERACTION_GUARDRAILS.md` §4.

## Permission-Gated Actions

Request explicit approval before acting when any of the following applies:

- Legal or compliance implications exist.
- Third-party code/repository download, build, execution, or integration is required.
- Potentially negative or irreversible impact exists (for example, unsolicited refactoring).
- Long-term strategic direction may change (for example, project direction or testing strategy).
- Ownership/remit is unclear.

## Stage Gate Policy

- Every stage has a definition of done.
- Pause at stage end and request explicit approval before next stage.
- No cross-stage work without approval.
- Silence or lack of objection is not approval; next-stage work requires an explicit yes.
- If a stage transition is denied, record the denial reason in stage records (`memory.md` and relevant task/stage artifact).
- If unauthorized cross-stage work occurs, stop immediately, declare governance breach, list unauthorized actions, and request rollback/replay direction.

## Commit Cadence Policy

- Prefer small, behavior-scoped commits over large batches.
- After a stage is marked complete and explicitly approved, save and create a stage-completion commit before starting the next stage.
- If stage-completion work required several commits, end with one clear milestone commit that marks stage completion.
- Keep rollback points frequent; do not delay commits until large bundles accumulate.
- During Stage 5 and Stage 6 fix cadences, each individual verified fix must be saved and committed before the next manual test run or before starting the next fix. Bundling multiple independent fixes into one commit is a protocol violation.
- For projects using a compiled or packaged build step (compiled binary, container image, bundled assets, or any packaged artifact), rebuild from current HEAD before any push operation. The artifact timestamp must post-date the latest commit. Pushing without a fresh build is a protocol violation.

## Coding Authorization Policy

- Planning, analysis, and document drafting are allowed by default.
- Code changes require explicit user authorization.
- If unsure whether code is authorized, ask.

## Detailed Brief Handling Policy

- A detailed implementation brief is Stage 1 input only — it does not permit skipping Stage 2 or Stage 3.
- Stage 1, 2, and 3 artifacts must each be complete and approved before any code is written. If implementation starts before these approvals, stop, log the violation in `memory.md`, and restart from the last valid stage boundary.
