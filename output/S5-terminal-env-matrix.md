# Stage 5 — Terminal Environment Validation Matrix

**Date:** 2026-03-27  
**Prepared by:** Team Lead (Stage 5 — Verify)  
**Spec ref:** FORMAL_SPEC.md §5 (NFR-003 observability, §9 Interactive CLI diagnostics)

---

## 1. Target Environments

| Environment | OS | Shell | Rust | Anvil | Status | Artifact path |
|---|---|---|---|---|---|---|
| Windows 10 (19045) | Windows NT 10.0.19045.0 | PowerShell 5.1.19041.6456 | 1.92.0 | 1.6.0-rc1 | **PASS** | `output/sessions/` (10 session dirs) |
| Linux (CI) | Ubuntu (GitHub Actions) | bash | stable (CI pin) | latest Foundry | **PASS** (CI green) | GitHub Actions log |
| macOS | — | — | — | — | **NOT TESTED** — explicit release risk |

---

## 2. Tested CLI Commands (Windows 10 / PowerShell 5.1)

| Command | Session artifact dir | Outcome | Notes |
|---|---|---|---|
| `balance` | `output/sessions/2026-03-26_21-15-05/` | ✅ PASS | Balance 10000 ETH, state.json written |
| `balance` | `output/sessions/2026-03-27_06-08-55/` | ✅ PASS | Confirmed after S5 script fix |
| `send` | `output/sessions/2026-03-27_06-09-00/` | ✅ PASS | Tx hash + receipt in block 1, state.json written |
| `tx-status` | `output/sessions/2026-03-27_06-09-06/` | ✅ PASS | Receipt confirmed success in block 1, state.json written |
| `call` | `output/sessions/2026-03-27_06-12-31/` | ✅ PASS | balanceOf returned Uint(1000, 256), state.json written |
| `watch` | `output/sessions/watch-2026-03-27_06-13-58/` | ✅ PASS | Banner printed, topic-0 filter shown, gracefully terminated after 3s |
| `balance` (transport error) | `output/sessions/2026-03-26_21-04-17/` | ✅ PASS (graceful error) | Prints structured error, exits 1, no panic |
| `balance` (invalid `--dump-state` position) | `output/sessions/2026-03-27_06-08-32/` | ⚠️ DEFECT FOUND → FIXED | S5 defect: `--dump-state` required `global=true`; fixed in `aa1f0eb` |

---

## 3. Identified Defects and Disposition

| Defect ID | Description | Fix | Regression test |
|---|---|---|---|
| S5-D1 | `--dump-state` rejected when placed after subcommand | Added `global = true` to Clap arg; capture-session.ps1 arg order fixed | `test_dump_state_flag_accepted_after_subcommand` (commit `aa1f0eb`) |
| S5-D2 | Clippy `map_or(false,...)` → `is_some_and(...)` in events.rs | Replaced all 4 occurrences | Clippy enforced in CI (`-- -D warnings`) (commit `10b482c`) |

---

## 4. Release Risks (Untested Environments)

| Risk | Environment | Mitigation |
|---|---|---|
| Shell quoting of `--abi` JSON arg in macOS zsh | macOS | No macOS test environment available. Recommend Lefty test manually on macOS before Phase 2. |
| `capture-session.sh` (Linux/macOS) script | Linux/macOS | Script exists; not tested interactively on this machine. CI tests use direct binary invocation, not the script. |
| `watch` command graceful Ctrl-C behavior | Windows | Tested only via kill signal (SIGKILL proxy). Interactive Ctrl-C behavior not captured in session artifact. |

---

## 5. PPL-001 Pair Programming Log Audit

**Log:** `chronicle/PPL-001-A1-G3-G5-G6-G7.md`

| Requirement | Status |
|---|---|
| All decision cycles have: proposal, critique, resolution, decision owner | ✅ PASS (5/5 cycles: A1-D1, G3-D1, G5-D1, G6-D1, G7-D1) |
| Linked code/test evidence for each cycle | ✅ PASS (all 5 cycles link to source files + test names) |
| No unresolved disagreements | ✅ PASS (section 3: "Unresolved disagreements: None") |
| No escalations unresolved | ✅ PASS ("Escalations made: None") |
| Verified by cargo test with count + date | ✅ PASS ("124 tests, all passing" as of 2026-03-26) |

**Verdict:** PPL-001 log is complete and audit-ready. No blockers.

---

## 6. Maintainability Trend Metric (Phase 1 cycle)

| Metric | Value | Direction |
|---|---|---|
| Test count: Stage 4 close | 124 | — |
| Test count: Stage 5 close | 129 | ▲ +5 (Gap-003 CLI tests + S5 regression test) |
| Clippy warnings (Stage 4 close) | 0 | — |
| Clippy warnings (Stage 5 discovered) | 4 (`map_or`) | ▼ fixed → 0 |
| Technical debt items logged | 2 | Signer trait abstraction, Transport trait abstraction (Phase 2) |
| Chronicle entries complete | 9/9 (CHR-000 through CHR-008) | ✅ |
| Orphan tests | 0 | ✅ |
