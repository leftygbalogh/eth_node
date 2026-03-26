# Implementation Chronicle — Workspace Scaffold

- Chronicle ID: CHR-000
- Source task ID: T-000
- Source spec sections: §7.1 (Layered Architecture / Q3-ARCH-01)
- Module / component name: Workspace scaffold
- Implementation language: Rust
- Status: Final

## 2. Intent to Implementation Mapping

- Establishes the Cargo workspace with two members (`eth_node` library, `eth_node_cli` binary) as required by Q3-ARCH-01.
- All subsequent implementation tasks build on this scaffold without modifying workspace manifest structure.

## 3. Implementation Decisions

- Workspace `Cargo.toml` placed at repo root; crate directories under `src/` per governance layout convention.
- All shared dependencies declared as `[workspace.dependencies]` with pinned minor versions (Claire Voyant risk mitigation: alloy API stability).
- Six Phase 1 module stubs declared in `lib.rs` so subsequent tasks can be developed in parallel without merge conflicts.
- **serde pinned to `>=1.0.0, <1.0.215`** (resolves to 1.0.214): serde 1.0.215+ introduced `serde_core` crate split which broke alloy-consensus 0.12.6's direct use of `serde::__private`. This pin must be revisited if alloy is upgraded beyond 0.12.

## 4. Alternatives Considered

- Alternative: Place crates at repo root (`eth_node/`, `eth_node_cli/`). Rejected — governance convention requires source code under `src/`.

## 5. Derived Invariants

- `eth_node_cli` must never contain business logic; all logic lives in `eth_node`.
- Alloy crate versions are pinned in the workspace manifest and must not be bumped without a team lead decision record.
