# Governance Mode

## Current Mode

- Mode: Template Development
- Effective date: 2026-03-20
- Owner: Lefty

## Meaning

This repository is currently operating as the master template under refinement, not as a live software delivery repository.

## Agent Routing Rules

When Mode is `Template Development`, agents should:

1. Improve governance docs, templates, and personas.
2. Process `templates/feedback.json` outputs from real projects.
3. Propose and implement template hardening updates.
4. Avoid starting a real project discovery flow in this repository unless explicitly instructed.

When Mode is `Project`, agents should:

1. Run mode-first project discovery for that project.
2. Execute stages against project artifacts in stage order.
3. Treat this governance folder as the active working governance directory.

## Suggested First Response by Mode

- Template Development: propose next template-improvement task or governance review.
- Project: start Stage 1 Discover with mode selection (Greenfield/Brownfield).

## Switching Mode

To switch from template refinement to live project execution, update this file:

- Change `Mode` to `Project`
- Set project name/context in this file
- Record switch reason in `memory.md`
- Commit mode switch before stage work continues

## Notes

- This file is the first routing check before discovery-order execution.
- Stage gates and approval rules remain active in both modes.
