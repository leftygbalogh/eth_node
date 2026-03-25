# Getting Started

## Goal

Use this governance template as the active policy and workflow layer for a new software project.

## Quick Start

1. Clone or copy this repository into your target project.
2. Set `GOVERNANCE_MODE.md` to `Project`.
3. Run `git remote -v` and confirm or repoint origin to the project's own repository before any commits.
4. Read files in the discovery order listed in `README.md`.
5. First discovery question must select project mode: Greenfield or Brownfield.
6. Copy the required templates from `templates/` to the project root and fill them in, in order:
   - `PROJECT_BRIEF_TEMPLATE.md` → `PROJECT_BRIEF.md`
   - `FORMAL_SPEC_TEMPLATE.md` → `FORMAL_SPEC.md`
   - `TASK_LIST_TEMPLATE.md` → `TASK_LIST.md`
   - `IMPLEMENTATION_CHRONICLE_TEMPLATE.md` → `IMPLEMENTATION_CHRONICLE.md`
   - `RELEASE_CHECKLIST_TEMPLATE.md` → `RELEASE_CHECKLIST.md`
7. Enforce stage gates: explicit approval required; stage-completion commit required after each stage.

## Repository Layout — What Goes Where

| Location | Purpose |
|---|---|
| `governance/` | Policy and process documents. Read-only during projects; amend via template PR. |
| `agents/` | Agent persona definitions. Use as-is or extend for project-specific roles. |
| `templates/` | Blank fillable documents. Copy to project root and complete per stage. |
| `src/` | All application source code, crates, modules, and libraries. |
| `config/` | Environment and application configuration files; secrets templates. |
| `build/` | Compiled binaries and packages. Gitignored — never commit build output. |
| `output/` | Reports, CSVs, PDFs, and any runtime-generated files. Gitignored. |
| `scripts/` | Automation helpers (idle guard, setup scripts). |
| `examples/` | Worked examples and reference data. |
| Root | `memory.md`, `prompts.md`, `GOVERNANCE_MODE.md`, `README.md`, `CHANGELOG.md`. |

## Repository Identity Setup (Mandatory for Project Mode)

1. Immediately after cloning, run `git remote -v`.
2. Confirm the remote URL points to the project's own repository — not this template.
3. If origin still points to the template repository, repoint it:
   - `git remote set-url origin <project-repository-url>`
4. Record the confirmed remote URL in `memory.md` before any branch work.

No push is allowed until repository identity is explicitly verified and recorded.

## Linux Compliance Setup

1. Ensure `.gitattributes` enforces LF (`* text=auto eol=lf`).
2. Ensure `.editorconfig` sets `end_of_line = lf`.
3. Use POSIX-style paths (`/`) in governance examples unless platform-specific behavior is being documented.

## Required Logs

- Append every user prompt to `prompts.md`.
- Keep `memory.md` updated with current status, decisions, blockers, and next step.

