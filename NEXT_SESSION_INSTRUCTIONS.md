# Next Session Instructions — Track B Test Contribution (2026-03-29)

## Session Summary

**Completed Work:**
- ✅ Track B test implementation: **17/17 tests for ChainIdFiller** (alloy-rs/alloy)
- ✅ Coverage achieved: **99.24% lines, 99.10% regions, 100% functions**
- ✅ Navigator certification: Test suite complete and approved
- ✅ Oracle/Claire Voyant: 85% PR acceptance forecast
- ✅ Test plan revised from mock-based → Anvil integration approach
- ✅ Created Integration Test Architect agent for pattern analysis

**Implementation Time:** ~210 minutes (~12min/test avg)

---

## Current State

### Git Repository Status

**eth_node repo:** Branch `master`, session work committed (2026-03-29)
- Modified: `.gitignore`, `memory.md`
- Added: `agents/integration-test-architect.md`, `upstream_contrib/plans/`

**alloy fork:** Branch `main`, **1 commit ahead** (commit 6a8568e0)
- Changes: +378 lines in `crates/provider/src/fillers/chain_id.rs`
- **⚠️ NOT PUSHED** — Remote points to upstream (no personal fork configured)

### Test File Location
`upstream_contrib/forks/alloy/crates/provider/src/fillers/chain_id.rs` (test module with 17 tests)

---

## Next Session Tasks

### PRIORITY 1: PR Preparation

**⚠️ GOVERNANCE:** Owner approval required before PR submission

**Steps:**
1. **Verify full test suite:** `cd upstream_contrib\forks\alloy\crates\provider; cargo test --lib`
2. **Generate coverage report:** `cargo llvm-cov --lib --no-cfg-coverage --html`
3. **Create personal fork** at https://github.com/alloy-rs/alloy/fork
4. **Add fork remote:** `git remote add myfork https://github.com/YOUR_USERNAME/alloy.git`
5. **Push:** `git push myfork main`
6. **Submit PR** with description template (see full version in detailed section)

### PR Description Template (Brief)
```markdown
Title: test(provider): add ChainIdFiller test coverage (0% → 99%)

Summary: Adds 17 tests for ChainIdFiller (93 lines, previously untested).
Coverage: 99.24% lines, 100% functions
Pattern: Follows nonce.rs conventions (Anvil integration + MockTransport)
```

### PRIORITY 2: Track B Closure
After PR submitted:
1. Update PHASE2_TASK_LIST.md (mark T-006 complete)
2. Create chronicle: `chronicle/CHR-0XX-track-b-alloy-tests.md`
3. Update memory.md with PR outcome

---

## Key Decisions This Session

1. **Testing Approach Pivot:** Mock-based → Anvil integration (Oracle finding)
   - Increased PR acceptance probability: 20% → 85%

2. **Adapted TDD Workflow:** WRITE → VERIFY → REVIEW (test-after, not test-first)

3. **Field Access Risk (40%):** Tests use `filler.0.get()` — may need `pub(crate)` if reviewers request

---

## Blockers & Risks

**CRITICAL:** No personal fork configured (create at session start)

**GOVERNANCE:** Owner approval required for PR submission

**Residual Risks:**
- Field visibility (40%) — propose solution if raised
- PR rejection (15%) — respond to feedback quickly

---

## Quick Start Commands

```powershell
# Verify eth_node status
cd C:\Users\geb\Documents\VScode\ethereum_node_rust
git status

# Verify alloy tests
cd upstream_contrib\forks\alloy
cargo test --lib --package alloy-provider

# Create fork & push
git remote add myfork https://github.com/YOUR_USERNAME/alloy.git
git push myfork main
```

---

## Questions for User at Session Start

1. **Do you have a personal GitHub fork of alloy-rs/alloy?**
2. **Approve PR submission?** (17 tests passing, 99% coverage, Navigator certified)
3. **Continue Track B closure or pivot to other Phase 2 work?**

---

## Session Metrics

- **Duration:** ~210 min (3.5 hours)
- **Tests:** 17/17 complete
- **Pace:** 12min/test average (ahead of schedule)
- **Coverage:** 99.24% (exceeds 85-90% target)

---

**Status:** Implementation complete, PR preparation pending  
**Next Priority:** Create fork → Push → Submit PR

*Session Date: 2026-03-29 | Resume from "PRIORITY 1: PR Preparation"*
