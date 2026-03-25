# AI Governance Template

A structured governance framework for AI-assisted software projects. Provides stage-gated workflows, agent personas, fillable document templates, and operating rules for Greenfield and Brownfield projects.

## Baseline Status

- Baseline version: 1.3
- Baseline state: Approved
- Approved on: 2026-03-25

## Repository Layout

```
/
├── README.md                  ← this file
├── GETTING_STARTED.md         ← onboarding guide for new projects
├── CHANGELOG.md               ← version history
├── GOVERNANCE_MODE.md         ← active mode: Template Development or Project
├── memory.md                  ← live session state (auto-updated by agent)
├── prompts.md                 ← running log of every user prompt
│
├── .github/
│   └── copilot-instructions.md  ← agent startup and behavior rules
│
├── governance/                ← policy and process documents (read-only during projects)
│   ├── 00_INTERACTION_GUARDRAILS.md
│   ├── 01_DECISION_POLICY.md
│   ├── 02_WORKFLOW_STAGES.md
│   ├── 03_TEAM_MODEL_HANDOFFS.md
│   ├── 04_PERSONA_DIRECTORY.md
│   ├── 05_IDLE_AUTOMATION.md
│   ├── 06_COMMAND_CHAIN_AND_PERSONALITY.md
│   └── 07_QUALITY_DIMENSIONS.md
│
├── agents/                    ← agent persona definitions
│   └── *.md
│
├── templates/                 ← blank fillable documents (copy and fill in for each project)
│   ├── PROJECT_BRIEF_TEMPLATE.md
│   ├── FORMAL_SPEC_TEMPLATE.md
│   ├── TASK_LIST_TEMPLATE.md
│   ├── IMPLEMENTATION_CHRONICLE_TEMPLATE.md
│   ├── PAIR_PROGRAMMING_LOG_TEMPLATE.md
│   ├── DELIVERABLES_MANIFEST_TEMPLATE.md
│   ├── REQUIREMENTS_SPEC_MANIFEST_TEMPLATE.md
│   ├── RELEASE_CHECKLIST_TEMPLATE.md
│   ├── OPERATIONS_AND_ROLLBACK_TEMPLATE.md
│   ├── POST_RELEASE_MONITORING_TEMPLATE.md
│   └── RUNBOOK_KNOWN_FAILURES_TEMPLATE.md
│
├── scripts/                   ← automation and tooling helpers
│   ├── idle-guard.ps1
│   └── idle-guard.sh
│
├── src/                       ← application source code and libraries
├── config/                    ← environment and application config files
├── build/                     ← compiled artifacts (gitignored)
├── output/                    ← generated reports and runtime output (gitignored)
│
└── examples/                  ← worked examples and reference data
    └── feedback.json          ← post-mortem feedback log
```

## Discovery Order

At session start, read files in this order:

1. `GOVERNANCE_MODE.md`
2. `governance/00_INTERACTION_GUARDRAILS.md`
3. `governance/01_DECISION_POLICY.md`
4. `governance/02_WORKFLOW_STAGES.md`
5. `governance/03_TEAM_MODEL_HANDOFFS.md`
6. `governance/04_PERSONA_DIRECTORY.md`
7. `governance/05_IDLE_AUTOMATION.md`
8. `governance/06_COMMAND_CHAIN_AND_PERSONALITY.md`
9. `governance/07_QUALITY_DIMENSIONS.md`
10. Use `templates/` documents as needed per stage

## Non-Negotiable Operating Rules

- Clarify ambiguity through questions, one at a time, up to 12 questions per clarification cycle.
- Do not proceed to the next stage until the current stage is complete and approved.
- After each approved stage completion, create a stage-completion commit before starting next-stage work.
- Prefer smaller, behavior-scoped commits to keep rollback points frequent.
- Do not write code unless explicitly instructed to do so.
- Record every user prompt in `prompts.md`.
- Maintain `memory.md` with current status, decisions, blockers, and next step.
- Linux compliance baseline: LF line endings, POSIX-style paths, Linux-compatible shell examples.

## Usage in New Projects

1. Clone or copy this repository into the new project.
2. Set `GOVERNANCE_MODE.md` to `Project`.
3. Run `git remote -v` and confirm or repoint origin to the project repository before any commits.
4. Copy required templates from `templates/` to the project root and fill them in.
5. Place all source code under `src/`, configs under `config/`.
6. Add `build/` and `output/` to `.gitignore` (already included in the template `.gitignore`).
7. Run the stage-gated workflow; do not skip stages.

## Living Template Model

- `governance/` is the master policy layer. Do not edit it mid-project; raise a template amendment instead.
- Improvements discovered during projects are fed back here via `examples/feedback.json` at Stage 6 close.
- Use `CHANGELOG.md` for manual version tracking.

## Notes on Automation

Idle-time save/update/commit behavior is enforced via `scripts/idle-guard.ps1` and `scripts/idle-guard.sh`.
