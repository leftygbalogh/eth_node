# Track B Task List: Chain ID Filler Test Implementation

**Target:** upstream_contrib/forks/alloy/crates/provider/src/fillers/chain_id.rs  
**Test Plan:** upstream_contrib/plans/CHAIN_ID_TEST_PLAN.md (REVISED)  
**Total Tests:** 17 (10 Anvil integration + 7 unit tests)  
**Team:** TDD Driver, TDD Navigator, Rust Backend Specialist (on-demand), Team Lead  
**Oracle Approval:** ✓ (85% PR acceptance forecast)  
**Created:** 2026-03-29

---

## Work Order Rationale

**Sequence:** AC-1 → AC-2 → AC-5 → AC-6 → AC-4 → AC-3 → AC-7

- **AC-1 first:** Simple caching behavior (baseline functionality)
- **AC-2 early:** Error handling (before more complex scenarios)
- **AC-5 mid:** Unit tests for FillerControlFlow (no provider needed)
- **AC-6 mid:** Edge case unit tests (boundary conditions)
- **AC-4 late:** Clone behavior (requires understanding of Arc sharing)
- **AC-3 late:** Concurrency (most complex, requires solid cache foundation)
- **AC-7 last:** Additional edge cases (polish phase)

**Coverage Gates:** Measure after Task 5 (AC-2 complete), Task 10 (AC-6 complete), Task 17 (final)

---

## Risk Mitigation

### Identified Risks (Claire Voyant Analysis)

1. **Internal Field Access Risk (40% probability)**
   - **Issue:** Tests may need `.0.get()` access to verify cache state
   - **Impact:** Requires `pub(crate)` visibility change in chain_id.rs
   - **Mitigation:** Prepare visibility change proposal; verify with Rust Backend Specialist before implementation
   - **Fallback:** Test only observable behavior (avoid cache state inspection)

2. **Stub Provider Trait Complexity (20% probability)**
   - **Issue:** Minimal Provider<Ethereum> trait stub may be verbose
   - **Impact:** Boilerplate overhead for Tests 2.1, 2.2, 4.2
   - **Mitigation:** Create reusable stub template in first unit test (TB-003); reuse for subsequent tests
   - **Example:** See [nonce.rs mock pattern](../forks/alloy/crates/provider/src/fillers/nonce.rs) if needed

3. **Coverage Verification Blocker (25% probability)**
   - **Issue:** llvm-cov may fail on test-only code or external crate
   - **Impact:** Cannot verify 85-90% target achievement
   - **Mitigation:** Run `cargo llvm-cov --lib --no-cfg-coverage` after TB-005, TB-010, TB-017
   - **Fallback:** Use `cargo tarpaulin` or manual line counting if llvm-cov fails

4. **Anvil Startup Failure (10% probability)**
   - **Issue:** Port conflict, permission error, PATH issue
   - **Impact:** All integration tests blocked
   - **Mitigation:** Verify Anvil running before starting TB-001; use explicit port if needed

---

## Task Definitions

### AC-1: Chain ID Immutability (Simple Caching)

---

## Task TB-001: Test 1.1 - Cache Persists Across Multiple Calls

**Type:** Anvil integration test  
**Assumption:** AC-1 (Immutability)  
**Dependencies:** None (first test)  
**Estimated Time:** 45 min (RED 15min, GREEN 10min, REFACTOR 20min)

**DoR:**
- [x] Test plan approved
- [x] Anvil running (verified)
- [x] alloy repository cloned
- [x] chain_id.rs source reviewed

**Test Description:**
Verify that ChainIdFiller caches chain_id from Anvil after first fetch and reuses cached value on subsequent prepare() calls. Uses real Anvil provider (integration test pattern).

**Acceptance Criteria:**
- First prepare() fetches from Anvil, returns 31337
- Second prepare() returns 31337 without RPC call
- Internal cache state is populated (verify via .0.get() if accessible)

**DoD:**
- [ ] Test written and compiles
- [ ] Test RED (fails with expected error)
- [ ] Minimal implementation (GREEN)
- [ ] Navigator approves refactor quality
- [ ] Test passes consistently (3 consecutive runs)
- [ ] Task status updated in this file

**Assigned:** [TDD Driver pending]  
**Status:** Not started  
**Notes:**

---

## Task TB-002: Test 1.2 - Pre-configured Chain ID Never Fetches

**Type:** Anvil integration test  
**Assumption:** AC-1 (Immutability)  
**Dependencies:** TB-001 (cache mechanics understood)  
**Estimated Time:** 30 min (RED 10min, GREEN 5min, REFACTOR 15min)

**DoR:**
- [x] TB-001 complete
- [x] Navigator approved TB-001 refactor

**Test Description:**
Verify ChainIdFiller initialized with pre-configured chain_id=42 never fetches from Anvil (always returns 42, not Anvil's 31337).

**Acceptance Criteria:**
- ChainIdFiller::new(Some(42)) created
- 10 consecutive prepare() calls all return 42
- Cache shows pre-configured value (not Anvil's 31337)

**DoD:**
- [ ] Test written and compiles
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

### AC-2: First Fetch Failure Handling (Error Paths)

---

## Task TB-003: Test 2.1 - First Fetch Timeout Propagates Error

**Type:** Unit test with stub provider  
**Assumption:** AC-2 (Error propagation)  
**Dependencies:** TB-002 (basic provider interaction understood)  
**Estimated Time:** 60 min (RED 20min, GREEN 20min, REFACTOR 20min)

**DoR:**
- [x] TB-002 complete
- [x] AC-1 tests passing

**Test Description:**
Verify timeout error from get_chain_id() propagates to caller on first fetch. Uses minimal stub provider (no Anvil).

**Acceptance Criteria:**
- Stub provider returns TransportError::timeout()
- prepare() returns Err(TransportError::Timeout)
- Cache remains empty (not poisoned)

**Implementation Notes:**
- Create reusable StubProvider template for this and TB-004
- Minimal trait impl (only get_chain_id needed)
- No mock framework (matches alloy conventions)

**DoD:**
- [ ] Test written with stub provider
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves stub design
- [ ] Test passes (3 runs)
- [ ] Stub template documented for reuse
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

## Task TB-004: Test 2.2 - Recovery After Initial Failure

**Type:** Unit test with sequence stub provider  
**Assumption:** AC-2 (Cache not poisoned by error)  
**Dependencies:** TB-003 (stub provider template exists)  
**Estimated Time:** 45 min (RED 15min, GREEN 15min, REFACTOR 15min)

**DoR:**
- [x] TB-003 complete
- [x] Stub provider template available

**Test Description:**
Verify ChainIdFiller recovers from initial fetch error. Second prepare() call succeeds and caches value.

**Acceptance Criteria:**
- Stub provider sequence: Err → Ok(1)
- First prepare() returns Err
- Cache still empty after error
- Second prepare() returns Ok(1)
- Third prepare() uses cached value (no third RPC)

**DoD:**
- [ ] Test written with sequence provider
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator verifies cache not poisoned
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

## Task TB-005: Test 2.3 - Cached Value Persists After Provider Failure

**Type:** Anvil integration test  
**Assumption:** AC-2 (Cache resilience)  
**Dependencies:** TB-004 (error recovery understood)  
**Estimated Time:** 30 min (RED 10min, GREEN 5min, REFACTOR 15min)

**DoR:**
- [x] TB-004 complete
- [x] AC-2 unit tests passing

**Test Description:**
Verify cached chain_id persists and is reused even if provider becomes unavailable (simulated by cache hits, not actual failure with Anvil).

**Acceptance Criteria:**
- First prepare() populates cache from Anvil
- 10 subsequent prepare() calls all return cached 31337
- No error propagation after cache populated

**DoD:**
- [ ] Test written
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves
- [ ] Test passes (3 runs)
- [ ] **COVERAGE GATE 1:** Run `cargo llvm-cov --lib --no-cfg-coverage`
- [ ] **COVERAGE GATE 1:** Record result (target: 40-50% at this stage)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

### AC-5: Pre-set Chain ID = Finished (FillerControlFlow)

---

## Task TB-006: Test 5.1 - Transaction With Chain ID Reports Finished

**Type:** Unit test (no provider)  
**Assumption:** AC-5 (Finished detection)  
**Dependencies:** TB-005 (AC-2 complete)  
**Estimated Time:** 20 min (RED 5min, GREEN 5min, REFACTOR 10min)

**DoR:**
- [x] TB-005 complete
- [x] Coverage Gate 1 passed

**Test Description:**
Verify filler.status(&tx) returns FillerControlFlow::Finished when transaction already has chain_id set.

**Acceptance Criteria:**
- Transaction with chain_id=1
- filler.status(&tx) returns Finished
- No provider interaction needed

**DoD:**
- [ ] Test written
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

## Task TB-007: Test 5.2 - Transaction Without Chain ID Reports Ready

**Type:** Unit test (no provider)  
**Assumption:** AC-5 (Ready detection)  
**Dependencies:** TB-006 (status() logic understood)  
**Estimated Time:** 20 min (RED 5min, GREEN 5min, REFACTOR 10min)

**DoR:**
- [x] TB-006 complete

**Test Description:**
Verify filler.status(&tx) returns FillerControlFlow::Ready when transaction has NO chain_id.

**Acceptance Criteria:**
- Transaction with chain_id=None
- filler.status(&tx) returns Ready
- ChainIdFiller has pre-configured chain_id=42

**DoD:**
- [ ] Test written
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

## Task TB-008: Test 5.3 - Fill Does Not Override Pre-set Chain ID

**Type:** Unit test (no provider)  
**Assumption:** AC-5 (Respect pre-set values)  
**Dependencies:** TB-007 (FillerControlFlow complete)  
**Estimated Time:** 25 min (RED 10min, GREEN 5min, REFACTOR 10min)

**DoR:**
- [x] TB-007 complete

**Test Description:**
Verify fill_sync() does NOT overwrite transaction's existing chain_id, even if filler has different cached value.

**Acceptance Criteria:**
- ChainIdFiller with cached chain_id=42
- Transaction with chain_id=1
- fill_sync() called
- Transaction still has chain_id=1 (not 42)

**DoD:**
- [ ] Test written
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves non-override logic
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

### AC-6: RPC Honesty (Boundary Conditions)

---

## Task TB-009: Test 6.1 - Chain ID Type Bounds

**Type:** Unit test (no provider)  
**Assumption:** AC-6 (Type safety)  
**Dependencies:** TB-008 (AC-5 complete)  
**Estimated Time:** 15 min (RED 5min, GREEN 5min, REFACTOR 5min)

**DoR:**
- [x] TB-008 complete

**Test Description:**
Verify ChainIdFiller handles u64::MAX without panic or overflow.

**Acceptance Criteria:**
- ChainIdFiller::new(Some(u64::MAX))
- Cache stores u64::MAX correctly
- No panic or type conversion issues

**DoD:**
- [ ] Test written
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

## Task TB-010: Test 6.2 - Chain ID Zero Value

**Type:** Unit test (no provider)  
**Assumption:** AC-6 (Accept any u64)  
**Dependencies:** TB-009 (boundary testing understood)  
**Estimated Time:** 15 min (RED 5min, GREEN 5min, REFACTOR 5min)

**DoR:**
- [x] TB-009 complete

**Test Description:**
Verify ChainIdFiller accepts chain_id=0 without validation (documents current behavior).

**Acceptance Criteria:**
- ChainIdFiller::new(Some(0))
- Cache stores 0
- No validation or rejection

**DoD:**
- [ ] Test written
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves
- [ ] Test passes (3 runs)
- [ ] **COVERAGE GATE 2:** Run `cargo llvm-cov --lib --no-cfg-coverage`
- [ ] **COVERAGE GATE 2:** Record result (target: 65-75% at this stage)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

### AC-4: Cloned Providers Share Cache (Arc Semantics)

---

## Task TB-011: Test 4.1 - Cloned Filler Shares Cache

**Type:** Anvil integration test  
**Assumption:** AC-4 (Arc<OnceLock> sharing)  
**Dependencies:** TB-010 (AC-6 complete)  
**Estimated Time:** 35 min (RED 10min, GREEN 10min, REFACTOR 15min)

**DoR:**
- [x] TB-010 complete
- [x] Coverage Gate 2 passed

**Test Description:**
Verify cloning ChainIdFiller shares the same Arc<OnceLock> (pointer equality) and both clones return same cached chain_id.

**Acceptance Criteria:**
- Original filler fetches from Anvil (chain_id=31337)
- Cloned filler also returns 31337
- Arc::ptr_eq(&filler1.0, &filler2.0) == true

**DoD:**
- [ ] Test written
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator verifies Arc semantics
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

## Task TB-012: Test 4.2 - Pre-configured Filler Clone Behavior

**Type:** Unit test (no provider)  
**Assumption:** AC-4 (Clone consistency)  
**Dependencies:** TB-011 (clone behavior understood)  
**Estimated Time:** 20 min (RED 5min, GREEN 5min, REFACTOR 10min)

**DoR:**
- [x] TB-011 complete

**Test Description:**
Verify cloning pre-configured ChainIdFiller shares same internal state (Arc pointer equality).

**Acceptance Criteria:**
- ChainIdFiller::new(Some(42))
- Clone has same cached value
- Arc::ptr_eq() == true

**DoD:**
- [ ] Test written
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

### AC-3: Concurrent Access Safety (Most Complex)

---

## Task TB-013: Test 3.1 - Concurrent First Fetch Single Initialization

**Type:** Anvil integration test (async concurrency)  
**Assumption:** AC-3 (OnceLock race safety)  
**Dependencies:** TB-012 (AC-4 complete)  
**Estimated Time:** 60 min (RED 20min, GREEN 20min, REFACTOR 20min)

**DoR:**
- [x] TB-012 complete
- [x] AC-1, AC-2, AC-4, AC-5, AC-6 tests all passing

**Test Description:**
Verify 10 concurrent prepare() calls during first fetch initialize cache exactly once (OnceLock guarantee). All return same chain_id.

**Acceptance Criteria:**
- ChainIdFiller with no pre-set chain_id
- 10 tokio tasks spawn simultaneously
- All call prepare() with shared filler (Arc)
- All return 31337
- Cache initialized exactly once

**Implementation Notes:**
- Requires Arc<ChainIdFiller> and Arc<Provider>
- Use tokio::spawn and futures::future::join_all
- Reference: nonce.rs:242-268 concurrency test pattern

**DoD:**
- [ ] Test written with real async concurrency
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator verifies OnceLock semantics
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

## Task TB-014: Test 3.2 - Concurrent Access After Cache Populated

**Type:** Anvil integration test (high contention)  
**Assumption:** AC-3 (Cache read contention)  
**Dependencies:** TB-013 (concurrency pattern established)  
**Estimated Time:** 45 min (RED 15min, GREEN 10min, REFACTOR 20min)

**DoR:**
- [x] TB-013 complete
- [x] Concurrent first-fetch working

**Test Description:**
Verify 100 concurrent reads from populated cache cause no contention or errors.

**Acceptance Criteria:**
- Cache pre-populated with chain_id=31337
- 100 tokio tasks read simultaneously
- All return cached value (no RPC calls)
- No deadlocks or panics

**DoD:**
- [ ] Test written with 100 concurrent tasks
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves high-contention scenario
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

### AC-7: Additional Edge Cases (Polish)

---

## Task TB-015: Test 7.1 - Default Construction

**Type:** Anvil integration test  
**Assumption:** AC-7 (Default behavior)  
**Dependencies:** TB-014 (AC-3 complete)  
**Estimated Time:** 20 min (RED 5min, GREEN 5min, REFACTOR 10min)

**DoR:**
- [x] TB-014 complete
- [x] All AC-1 through AC-6 tests passing

**Test Description:**
Verify ChainIdFiller::default() behaves identically to new(None) - fetches from provider.

**Acceptance Criteria:**
- ChainIdFiller::default() created
- prepare() fetches from Anvil
- Returns 31337

**DoD:**
- [ ] Test written
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

## Task TB-016: Test 7.2 - PartialEq Implementation

**Type:** Unit test (no provider)  
**Assumption:** AC-7 (Equality semantics)  
**Dependencies:** TB-015 (default behavior understood)  
**Estimated Time:** 25 min (RED 10min, GREEN 5min, REFACTOR 10min)

**DoR:**
- [x] TB-015 complete

**Test Description:**
Verify PartialEq compares cached values correctly (same values equal, different values not equal).

**Acceptance Criteria:**
- ChainIdFiller::new(Some(1)) == ChainIdFiller::new(Some(1))
- ChainIdFiller::new(Some(1)) != ChainIdFiller::new(Some(2))
- ChainIdFiller::new(None) != ChainIdFiller::new(Some(1))

**DoD:**
- [ ] Test written
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves equality logic
- [ ] Test passes (3 runs)
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

## Task TB-017: Test 7.3 - Debug Format

**Type:** Unit test (no provider)  
**Assumption:** AC-7 (Debug trait)  
**Dependencies:** TB-016 (polish phase)  
**Estimated Time:** 15 min (RED 5min, GREEN 5min, REFACTOR 5min)

**DoR:**
- [x] TB-016 complete

**Test Description:**
Verify Debug implementation doesn't panic and produces reasonable output.

**Acceptance Criteria:**
- format!("{:?}", filler) succeeds
- Output contains "ChainIdFiller"
- No panic

**DoD:**
- [ ] Test written
- [ ] Test RED
- [ ] Implementation GREEN
- [ ] Navigator approves
- [ ] Test passes (3 runs)
- [ ] **COVERAGE GATE 3 (FINAL):** Run `cargo llvm-cov --lib --no-cfg-coverage`
- [ ] **COVERAGE GATE 3 (FINAL):** Record result (target: 85-90%)
- [ ] **COVERAGE GATE 3 (FINAL):** If under 85%, identify uncovered lines
- [ ] Status updated

**Assigned:** [Pending]  
**Status:** Not started  
**Notes:**

---

## Progress Tracking

**Last Updated:** 2026-03-29 (Task list created)

**Tests Completed:** 0/17  
**Coverage:** Not yet measured (gates at TB-005, TB-010, TB-017)  
**Current Phase:** Pre-implementation (task list approved, awaiting Team Lead go-ahead)

**Blockers:** None

**Current Cycle:** None (awaiting start authorization)

**Next Task:** TB-001 (Test 1.1 - Cache Persists)

---

## Coverage Gates Schedule

| Gate | After Task | Expected Coverage | Command |
|------|-----------|-------------------|---------|
| Gate 1 | TB-005 (AC-2 complete) | 40-50% | `cargo llvm-cov --lib --no-cfg-coverage` |
| Gate 2 | TB-010 (AC-6 complete) | 65-75% | `cargo llvm-cov --lib --no-cfg-coverage` |
| Gate 3 | TB-017 (final) | 85-90% | `cargo llvm-cov --lib --no-cfg-coverage` |

**Gate Failure Protocol:**
- If coverage below target at Gate 1 or 2: Continue (early stages)
- If coverage below 85% at Gate 3: Analyze uncovered lines, add targeted tests if needed

---

## Team Coordination Notes

**TDD Workflow per Task:**
1. Team Lead assigns task to TDD Driver
2. Driver writes failing test (RED phase)
3. Driver shares test with Navigator for review
4. Navigator approves test quality OR requests revisions
5. Driver implements minimal code (GREEN phase)
6. Driver runs test (should pass)
7. Driver shares implementation with Navigator
8. Navigator reviews code quality, suggests refactoring if needed
9. Driver refactors (if Navigator requested)
10. Navigator approves final version
11. Team Lead commits with descriptive message
12. Team Lead updates task status in this file
13. Team Lead assigns next task

**Rust Backend Specialist:** On-call for:
- Visibility change proposals (if internal field access needed)
- Complex Provider trait questions
- alloy API usage clarifications

**Team Lead Responsibilities:**
- Track time per test (flag if >60 min for simple tests)
- Enforce Navigator approval gates (no skipping)
- Run coverage gates at TB-005, TB-010, TB-017
- Maintain progress tracking section
- Coordinate with Rust Backend Specialist for blockers

---

## Approval

**Task List Status:** Created, awaiting Team Lead authorization to begin TB-001

**Team Lead:** Ready to assign TB-001 to TDD Driver upon explicit "go" command

**Notes:** 
- All 17 tests from revised test plan included
- Work order optimized for incremental complexity
- Risk mitigation strategies documented
- Coverage verification gates scheduled
