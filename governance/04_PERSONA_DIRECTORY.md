# 04 Persona Directory

This file indexes persona definitions used by the virtual software development team.

## Persona Tracks by Project Mode

- Greenfield track: scope refinement, architecture foresight, formal specification, pseudocode, and implementation-from-spec roles.
- Brownfield track: implemented behavior extraction, parity testing, decomposition into tiny migration units, and controlled rewrite/refactor roles.

## Language Selection

The implementing team declares the primary language during Stage 1 Discover and selects agent personas accordingly. All supported languages have agent files in `agents/`. No language is pre-configured as a default; the project brief drives selection.

## Agent Persona Library

All persona files live in `agents/`. Categories available:

- **Specification and discovery**: greenfield-domain-discovery-expert, greenfield-evolution-architect, greenfield-formal-spec-author, human-spec-writer, traceability-mapper
- **Test and quality**: tdd-driver, tdd-navigator, unit-test-completeness-engineer, property-based-test-engineer, readability-reviewer, maintainability-reviewer, refactoring-steward
- **Documentation**: technical-writer-live-examples, meta-prompter
- **Brownfield specialists**: brownfield-behavior-baseline-analyst, brownfield-incremental-rewrite-planner, brownfield-parity-test-engineer
- **Language implementers**: rust-backend, rust-cli, rust-database, rust-defensive-programming, rust-systems-performance, rust-async-concurrency-reliability, rust-api-contract-serialization, rust-api-ergonomics, rust-documentation-quality, rust-unsafe-code-auditor, rust-webassembly-frontend; python-backend, python-cli, python-database, python-defensive-programming; go-backend, kotlin-jvm, typescript-fullstack, javascript-legacy-migration, c-systems, sql-database, powershell-automation; plus backend, cli, database, and defensive-programming specialist variants
- **Governance agents**: oracle-agent, claire-voyant-agent, team-lead

Use `agents/AGENT_TEMPLATE.md` to define new personas.

Rust reviewer micro-personas:

- `agents/rust-unsafe-code-auditor.md`
- `agents/rust-api-ergonomics-reviewer.md`
- `agents/rust-documentation-quality-reviewer.md`

Python secondary specialty pack:

- `agents/python-backend-specialist.md`
- `agents/python-database-specialist.md`
- `agents/python-cli-specialist.md`
- `agents/python-defensive-programming-specialist.md`

Brownfield core migration trio:

- `agents/brownfield-behavior-baseline-analyst.md`
- `agents/brownfield-parity-test-engineer.md`
- `agents/brownfield-incremental-rewrite-planner.md`

Required sections:

- Mission
- In-scope responsibilities
- Out-of-scope boundaries
- Authority and rights
- Inputs required
- Outputs produced
- Documentation obligations
- Handoff expectations
- Quality bar
- Escalation triggers

Build-capable personas must explicitly state:

- That implementation decisions and trade-offs are recorded in `templates/IMPLEMENTATION_CHRONICLE_TEMPLATE.md`
- That chronicle entries link to source spec sections and task IDs
- That task completion can be blocked if the chronicle entry is missing
