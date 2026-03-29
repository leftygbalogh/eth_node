# Track B Team Composition - Chain ID Filler Test Contribution

**Project:** Upstream test contribution to alloy-rs/alloy  
**Module:** `crates/provider/src/fillers/chain_id.rs`  
**Goal:** 0% → 90%+ test coverage  
**Mode:** XP Pair Programming + TDD

---

## Recommended Team Structure

### **Core Pair: TDD Cycle Execution**

#### **Agent 1: TDD Driver**
- **Primary Role:** Write failing tests first, implement minimal code to pass
- **Responsibilities:**
  - Implement test cases from [CHAIN_ID_TEST_PLAN.md](./CHAIN_ID_TEST_PLAN.md) in red-green-refactor order
  - Start with simplest tests (AC-1: caching) → complex (AC-3: concurrency)
  - Write minimal mock infrastructure needed per test
  - Ensure tests fail before any implementation changes
  - Document test execution results in chronicle
- **Key Skills:**
  - Test-first mindset
  - Rust async/await patterns
  - Mock provider construction
  - Incremental progress discipline
- **Handoff Triggers:**
  - After each test reaches GREEN → Navigator reviews
  - After REFACTOR → Navigator approves before next test
  - Blocked on mock complexity → Escalate to Specialist

#### **Agent 2: TDD Navigator**
- **Primary Role:** Review tests, challenge coverage, guide driver decisions
- **Responsibilities:**
  - Review each test BEFORE implementation (RED phase validation)
  - Challenge: "Does this test prove the assumption?"
  - Question edge cases: "What if chain_id is u64::MAX?"
  - Approve refactoring decisions
  - Flag when tests are testing mocks instead of production code
  - Monitor coverage metrics after each GREEN cycle
- **Key Skills:**
  - Critical thinking on test quality
  - Assumption validation
  - Coverage gap identification
  - Rust idiom knowledge
- **Handoff Triggers:**
  - Test quality concerns → Driver revises
  - Coverage gaps identified → Add to test plan
  - Architecture questions → Escalate to Specialist

---

### **Supporting Roles**

#### **Agent 3: Rust Backend Specialist**
- **Primary Role:** Technical advisor, architecture guidance
- **Responsibilities:**
  - Review mock provider architecture (AC-3 requires sophisticated async mocking)
  - Advise on `Arc<OnceLock>` concurrent access patterns
  - Validate test infrastructure doesn't couple to alloy internals
  - Review final PR for Rust idioms before submission
  - Resolve "this doesn't compile" escalations from Driver
- **Involvement:** On-demand consultation (not full-time)
- **Handoff Triggers:**
  - Driver blocked on Arc/async lifetime issues
  - Navigator questions concurrent access test validity
  - PR review checkpoint before submission

#### **Agent 4: Team Lead**
- **Primary Role:** Coordinate workflow, manage stage gates, resolve blockers
- **Responsibilities:**
  - Maintain task list from test plan (20 test cases → task breakdown)
  - Track progress: X/20 tests passing, Y% coverage achieved
  - Facilitate Driver ↔ Navigator handoffs
  - Escalate blockers to Specialist or User
  - Manage stage transitions (Test → Implementation → PR)
  - Ensure chronicle documentation maintained
  - Prepare PR submission materials (assumptions tested, coverage proof)
- **Involvement:** Full-time coordination
- **Handoff Triggers:**
  - Coverage target met (90%+) → Approval gate for PR submission
  - Blocker persists >30min → Escalate to User
  - Team disagreement on test approach → Facilitate decision

---

## Workflow Pattern

### **Iteration Loop (per test case):**

```
1. [Navigator] Reviews test case pseudocode from CHAIN_ID_TEST_PLAN.md
   ├─ Approves test approach
   └─ Identifies edge cases to add

2. [Driver] Writes failing test (RED phase)
   ├─ Creates minimal mock if needed
   └─ Runs: cargo test --lib -- chain_id
   
3. [Navigator] Reviews RED test
   ├─ Validates: test actually fails for right reason
   ├─ Confirms: test proves the assumption
   └─ Approves: proceed to implement

4. [Driver] Implements minimal code (GREEN phase)
   ├─ Runs: cargo test --lib -- chain_id
   └─ Confirms: test passes

5. [Navigator] Reviews GREEN implementation
   ├─ Checks: test now passes
   ├─ Verifies: no false positives
   └─ Approves: proceed to refactor

6. [Driver] Refactors (REFACTOR phase)
   ├─ Improves test readability
   ├─ Extracts common mock patterns
   └─ Runs: cargo test --lib -- chain_id

7. [Navigator] Approves refactor
   └─ Tests still pass, cleaner code

8. [Team Lead] Updates task list
   ├─ Marks test case complete
   ├─ Updates coverage metrics
   └─ Triggers next test case

9. [Specialist] (if consulted)
   ├─ Reviews mock architecture
   ├─ Advises on concurrent testing patterns
   └─ Returns to standby
```

---

## Optional: Additional Specialists

### **Agent 5: Rust Async Concurrency Reliability Specialist**
- **When Needed:** AC-3 concurrent access tests (Test 3.1-3.3)
- **Responsibilities:**
  - Design mock provider with atomic counters
  - Validate `Arc<OnceLock>` race condition testing
  - Review tokio::spawn test patterns
  - Ensure no false positives from timing assumptions
- **Involvement:** AC-3 tests only (~3 test cases)

### **Agent 6: Traceability Mapper**
- **When Needed:** Final PR preparation
- **Responsibilities:**
  - Link each test to assumption in commit message
  - Map coverage report to test plan sections
  - Document assumptions validated vs documented behavior
  - Prepare traceability matrix for PR description
- **Involvement:** Post-implementation, pre-PR

---

## Team Size Justification

**Minimum Viable:** 2 agents (Driver + Navigator)
- Core TDD pair can execute test plan
- Missing: coordination overhead, technical escalation path

**Recommended:** 4 agents (Driver + Navigator + Specialist + Lead)
- Driver + Navigator: execution
- Specialist: unblocks technical issues fast
- Lead: maintains visibility, manages handoffs

**Maximum:** 6 agents (add Concurrency Specialist + Traceability Mapper)
- Concurrency Specialist: AC-3 is complex, worth dedicated expertise
- Traceability Mapper: ensures upstream contribution tells clear story

---

## Decision Factors

### **Choose 2-agent team if:**
- User wants to directly coordinate
- Test plan is straightforward (no AC-3 complexity)
- Budget-conscious iteration

### **Choose 4-agent team if:** ⭐ **RECOMMENDED**
- Standard XP workflow
- User delegates coordination
- Need technical unblocking
- Want visible progress tracking

### **Choose 6-agent team if:**
- AC-3 concurrent tests are critical (they are!)
- Contributing upstream with strong traceability requirement
- Maximum efficiency (specialists accelerate, don't duplicate)

---

## Team Communication Protocol

### **Synchronous Handoffs:**
- Driver → Navigator: After each RED, GREEN, REFACTOR
- Navigator → Driver: Test approval or revision request
- Any → Lead: Blocker escalation
- Any → Specialist: Technical question (via Lead)

### **Asynchronous Updates:**
- Lead maintains live task list (visible to all)
- Driver logs test results in chronicle
- Navigator logs review decisions
- Specialist logs advisory responses

### **User Touchpoints:**
- Lead: Weekly progress summary (X/20 tests, Y% coverage)
- Lead: Approval gate before PR submission
- Any: Blocker escalation after internal triage

---

## Success Metrics

**Individual Performance:**
- Driver: Tests written per hour, RED→GREEN→REFACTOR cycle time
- Navigator: Edge cases identified, test quality improvements
- Specialist: Unblock time (should be <30min per escalation)
- Lead: Task completion velocity, coordination overhead

**Team Performance:**
- Coverage increase: 0% → 90%+ within 20 test cycles
- Test quality: 0 false positives, assumptions proven
- PR acceptance: Maintainer feedback positive, merged
- Knowledge transfer: eth_node can apply pattern to future contributions

---

## Recommended Team: 4 Agents

**Rationale:**
1. **TDD Driver + Navigator:** Core pair programming model (XP standard)
2. **Rust Backend Specialist:** AC-3 concurrent testing is non-trivial, need expert unblocking
3. **Team Lead:** 20 test cases + stage gates + PR submission = non-trivial coordination

**Optional additions based on user preference:**
- Add **Rust Async Concurrency Specialist** for AC-3 focus (Test 3.1-3.3 critical)
- Add **Traceability Mapper** for upstream contribution storytelling

---

## Alternative: Lean Team (2 agents)

If user prefers minimal team:
- **TDD Driver** (implements + some navigation)
- **Team Lead** (coordinates + technical questions to user)

Tradeoff: Slower (no dedicated Navigator), user becomes blocker resolver.

---

## Proposed Team Assignment

**Role** | **Agent** | **Time Commitment**
---------|-----------|--------------------
TDD Driver | TDD Driver | Full-time
TDD Navigator | TDD Navigator | Full-time  
Technical Advisor | Rust Backend Specialist | On-demand
Coordinator | Team Lead | Full-time

**Next Step:** User approves team composition, Team Lead initializes task list from test plan.
