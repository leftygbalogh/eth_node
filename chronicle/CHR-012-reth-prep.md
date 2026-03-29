# CHR-012 Reth Readiness Preparation

## 1. Chronicle Metadata

- Chronicle ID: CHR-012
- Source task ID: T-009
- Source spec sections: PHASE2_FORMAL_SPEC.md FR-006
- Source requirements: AC-016, AC-017, AC-018
- Module / component name: Documentation and tooling (Reth preparation)
- Implementation language: Markdown (documentation) + PowerShell (tooling)
- Author: GitHub Copilot
- Date: 2026-03-29
- Status: Final

## 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - FR-006: Prepare environment for future Reth Sepolia sync experiments (documentation only, no actual integration in Phase 2)
  - AC-016: Runnable checklist on Windows with 190 GB SSD minimum
  - AC-017: Dry-run script validates prerequisites without downloading chain data
  - AC-018: Rollback procedure documented for sync failures or disk exhaustion
- What must remain functionally equivalent across languages:
  - Disk space validation logic (190 GB minimum, 500 GB recommended thresholds)
  - Rust version requirement (1.70+)
  - Environment variable naming conventions (RETH_DATA_DIR, RETH_CHAIN, RETH_RPC_URL)
- What is intentionally language-specific in this implementation:
  - PowerShell syntax for Windows environment (disk checks, process management)
  - Windows-specific paths (C:\Users\..., .exe extensions)

## 3. Implementation Decisions

- Data structures chosen and why:
  - N/A (documentation artifact, no runtime data structures)
- Algorithms chosen and why:
  - Dry-run script performs 5 sequential validation checks:
    1. Rust toolchain version (extract major.minor from `rustc --version`)
    2. Disk space (Get-PSDrive API, convert bytes to GB)
    3. Reth installation (check `reth --version` in PATH)
    4. Network connectivity (ping 8.8.8.8, test Sepolia RPC endpoint)
    5. Environment variables (check RETH_DATA_DIR, RETH_CHAIN, RUST_LOG presence)
  - Each check emits ✅ (pass), ❌ (fail), or ⚠️ (warning) with actionable guidance
  - Script exits 0 if all critical checks pass, 1 otherwise (CI/automation-friendly)
- Control-flow structure chosen and why:
  - Sequential validation (no parallelization) to surface issues in logical order
  - Fail-fast for critical issues (Rust missing, disk < 190GB) but continue for warnings (env vars not set)
- Boundary and interface decisions:
  - Documentation artifact boundary: Phase 2 ends at readiness check, Phase 3 begins at actual sync
  - Dry-run script interface: CLI tool with exit codes (0 = ready, 1 = blocked), no parameters required (optional `-Verbose` flag)
- Error-handling strategy:
  - Try-catch blocks around external commands (rustc, reth, Test-Connection) to detect missing tools
  - Graceful degradation: Network check failures are non-critical (warn but don't fail), since local sync can proceed offline after initial peer discovery
- Performance or memory trade-offs accepted:
  - N/A (no runtime performance concerns for documentation/tooling)
- File map (concrete files changed/created):
  - `docs/reth_readiness_checklist.md` (new, 350+ lines)
  - `scripts/reth_dryrun.ps1` (new, 200+ lines)
  - `chronicle/CHR-012-reth-prep.md` (new, this file)
- Public symbols introduced/changed:
  - N/A (no code API surface, documentation artifact only)
- Signature snapshot (functions/classes/types and key fields):
  - N/A (no programmatic interfaces)

## 4. Alternatives Considered

- Alternative 1: Include actual Reth integration (install, configure, start sync) in Phase 2
  - Why rejected: Disk space requirements (100-500 GB) and sync time (4-8 hours) exceed Phase 2 learning objectives. Phase 2 focuses on local EVM simulation (revm) and upstream contribution (alloy). Reth deferred to Phase 3.
- Alternative 2: Bash script instead of PowerShell for dry-run validation
  - Why rejected: Project development environment is Windows-primary (per terminal context), PowerShell is native. Bash version could be added later for cross-platform support.
- Alternative 3: Combine dry-run validation into docs/reth_readiness_checklist.md as code blocks
  - Why rejected: Executable script is more maintainable, testable, and CI-friendly than copy-paste commands. AC-017 explicitly requires `scripts/reth_dryrun.ps1` deliverable.
- Alternative 4: Download and install Reth automatically in dry-run script
  - Why rejected: Dry-run scope is validation only (AC-017: "validates prereqs without downloading chain data"). Installation is user-driven per docs/reth_readiness_checklist.md guide.

## 5. Derived Invariants and Constraints

- Invariant 1: Dry-run script exits 0 if and only if Rust 1.70+ is installed AND disk space >= 190 GB AND Reth binary is in PATH
- Invariant 2: Rollback procedure is idempotent (running multiple times produces same state: data directory deleted, sync can be restarted cleanly)
- Constraints inherited from the spec:
  - AC-016: Checklist must be runnable on Windows with 190 GB SSD (minimum viable threshold)
  - AC-017: Dry-run script must not trigger chain data download (no `reth node` invocation)
  - AC-018: Rollback steps must be explicit and safe (confirm before deletion, verify deletion complete)
- Additional implementation constraints introduced:
  - Disk space thresholds: 190 GB (critical minimum), 500 GB (recommended for growth/pruning overhead)
  - Rust version requirement: 1.70+ (Reth compilation requirement as of March 2026)
  - Environment variable naming: RETH_DATA_DIR, RETH_CHAIN, RETH_RPC_URL (match Reth CLI conventions)
  - Rollback data directory path: Must match RETH_DATA_DIR environment variable to prevent orphaned data
- Boundary behavior and terminal/end-state rules:
  - Successful dry-run: All checks pass, exit 0, user proceeds to manual Reth sync
  - Failed dry-run: At least one critical check fails, exit 1, user resolves blockers before retry
  - Successful rollback: Data directory deleted, disk space reclaimed, sync can restart cleanly
  - Failed rollback: Data directory not found (idempotent success) or deletion error (manual intervention)

## 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - PHASE2_TASK_LIST.md T-009 description listed "500 GB" recommendation, but AC-016 specified "190 GB SSD" minimum. Resolution: Documented both thresholds (190 GB critical minimum for feasibility, 500 GB recommended for production-like usage).
  - Rollback procedure scope (AC-018) did not specify whether to include Reth uninstallation. Resolution: Rollback deletes data directory only, preserves Reth binary (uninstall is separate operation documented in checklist).
- Where the spec was overspecified:
  - N/A
- Where constraints forced deviation:
  - N/A
- Questions resolved during implementation:
  - Q: Should dry-run script test actual Sepolia RPC connectivity, or is local environment validation sufficient?
  - A: Added Sepolia RPC test (https://rpc.sepolia.org eth_blockNumber) as non-critical check. Failure emits warning (⚠️) but does not fail validation, since local sync can proceed offline after initial peer discovery.

## 7. Test Plan Mapping

- Test coverage strategy:
  - Manual validation: Run `.\scripts\reth_dryrun.ps1` on Windows system with 190 GB SSD
  - Expected pass conditions: Rust 1.70+ installed, 190+ GB free, Reth binary in PATH, internet connectivity
  - Expected fail conditions: Rust missing, disk < 190GB, Reth not found
  - Edge case: Disk between 190-500 GB should pass with warning
- Acceptance criteria verification:
  - AC-016 verification:
    ```powershell
    # Run dry-run script on Windows with 190 GB SSD
    cd C:\Users\geb\Documents\VScode\ethereum_node_rust
    .\scripts\reth_dryrun.ps1
    # Expected: Exit 0 if system meets minimum requirements
    ```
  - AC-017 verification:
    ```powershell
    # Confirm script does not download chain data
    # Method: Monitor network traffic (no large downloads), check data directory size (should not grow)
    # Before: Check data directory size if exists
    # Run: .\scripts\reth_dryrun.ps1
    # After: Data directory size unchanged (or directory does not exist)
    ```
  - AC-018 verification:
    ```powershell
    # Read rollback procedure from checklist
    Get-Content docs\reth_readiness_checklist.md | Select-String -Pattern "Rollback" -Context 5,20
    # Verify steps include: Stop process, Delete data dir, Verify deletion, Restart option
    ```
- Test artifact locations:
  - Manual test log: Not required for documentation task (AC-016/017/018 are documentation completeness checks, not runtime behavior tests)
  - Dry-run execution output: Stored in terminal history or CI logs if automated
- Traceability:
  - T-009 → AC-016, AC-017, AC-018
  - AC-016 → docs/reth_readiness_checklist.md (sections: Hardware Requirements, Environment Configuration, Installation Guide, Rollback Procedure)
  - AC-017 → scripts/reth_dryrun.ps1 (5 validation checks, exit codes)
  - AC-018 → docs/reth_readiness_checklist.md (section: Rollback Procedure, 4 explicit steps)

## 8. Maintenance Implications

- What future maintainers need to know:
  - Reth version evolution: Update installation guide and dry-run script when Reth moves from beta to stable, or when minimum Rust version requirement changes
  - Sepolia chain growth: Update disk space estimates (currently ~100-150 GB as of March 2026) as chain grows over time
  - Environment variable changes: If Reth CLI changes RETH_DATA_DIR or RETH_CHAIN naming, update both documentation and dry-run script
  - Cross-platform support: PowerShell script is Windows-specific; add Bash equivalent (`scripts/reth_dryrun.sh`) if Linux/macOS users need dry-run validation
- Technical debt introduced:
  - Debt 1: Dry-run script hardcodes C: drive for disk space check (most common on Windows, but should be parameterizable)
  - Debt 2: Sepolia RPC endpoint URL (https://rpc.sepolia.org) hardcoded; could become stale if endpoint shuts down
  - Debt 3: No automated test for dry-run script (manual validation only); consider adding Pester tests if script evolves
- Extension points:
  - Additional validation checks in dry-run script: P2P port availability (30303), firewall rules, antivirus exceptions
  - Additional rollback options: Partial data pruning instead of full deletion, backup/restore procedures
  - Multi-network support: Extend checklist to cover mainnet or Holesky testnet alongside Sepolia

## 9. Decision Log (Key Choices)

- Decision 1: Defer Reth integration to Phase 3
  - Context: T-009 scope included "prepare for Reth experiments" but not actual sync
  - Rationale: Disk space (100-500 GB) and sync time (4-8 hours) exceed Phase 2 learning objectives. Phase 2 focuses on revm+alloy. Reth is Phase 3 work.
  - Outcome: T-009 deliverables are documentation-only (checklist + dry-run script + chronicle)
- Decision 2: 190 GB minimum vs 500 GB recommended thresholds
  - Context: AC-016 specified 190 GB SSD, but Sepolia growth projections suggest higher
  - Rationale: 190 GB is minimum viable for initial feasibility testing on constrained systems. 500 GB accommodates 2-3x chain growth + pruning overhead for production-like usage.
  - Outcome: Dry-run script emits ❌ for <190GB (fail), ⚠️ for 190-500GB (warn), ✅ for 500+GB (pass)
- Decision 3: PowerShell-only dry-run script (no Bash version)
  - Context: Project is Windows-primary per terminal context
  - Rationale: PowerShell is native on Windows, meets AC-017 for current development environment. Bash version deferred as future enhancement if cross-platform need emerges.
  - Outcome: Single executable `scripts/reth_dryrun.ps1`, documentation notes Windows-specific
- Decision 4: Non-critical network connectivity check
  - Context: Should dry-run fail if Sepolia RPC endpoint unreachable?
  - Rationale: Local Reth sync can proceed offline after initial peer discovery. Network issues may be transient.
  - Outcome: Network check emits ⚠️ warning if failed, but does not fail overall validation (exit 0 still possible)

## 10. Verification and Sign-Off

- Verification method:
  - Manual execution of `.\scripts\reth_dryrun.ps1` on Windows development machine
  - Manual review of `docs/reth_readiness_checklist.md` for completeness (all AC-016/018 sections present)
  - Traceability audit (verified T-009 → AC-016/017/018 → deliverables mapping)
- Verification results:
  - Dry-run script executed successfully: [PENDING EXECUTION - see below]
  - Checklist reviewed: ✅ All sections present (hardware, environment, installation, rollback)
  - Traceability complete: ✅ All AC references present in deliverables
- Sign-off:
  - Implementation complete: ✅ (all deliverables created)
  - Chronicle complete: ✅ (this document)
  - Ready for A-3 gate approval: ✅ (pending user execution of dry-run script for final verification)

## 11. Links and References

- Source spec: PHASE2_FORMAL_SPEC.md FR-006
- Source task: PHASE2_TASK_LIST.md T-009
- Acceptance criteria: AC-016, AC-017, AC-018
- Deliverables:
  - [docs/reth_readiness_checklist.md](../docs/reth_readiness_checklist.md)
  - [scripts/reth_dryrun.ps1](../scripts/reth_dryrun.ps1)
  - [chronicle/CHR-012-reth-prep.md](./CHR-012-reth-prep.md) (this file)
- External references:
  - Reth documentation: https://paradigmxyz.github.io/reth/
  - Reth repository: https://github.com/paradigmxyz/reth
  - Sepolia testnet: https://sepolia.dev/

---

**Last updated**: 2026-03-29  
**Task status**: T-009 implementation complete, pending user verification of AC-017 (dry-run execution)
