# Implementation Chronicle

## 1. Chronicle Metadata

- Chronicle ID: CHR-GOV-STARTUP
- Source task ID: T-001
- Source spec sections: FORMAL_SPEC.md section 4 (FR-001)
- Source requirements: FR-001
- Module / component name: Startup discovery flow
- Implementation language: Markdown policy artifacts
- Author: GitHub Copilot
- Date: 2026-03-19
- Status: Final

## 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - Mode-first startup behavior where the first discovery question is always project mode (Greenfield or Brownfield).
- What must remain functionally equivalent across languages:
  - Startup order: read governance, ask mode first, record mode, then continue discovery questions.
- What is intentionally language-specific in this implementation:
  - Rule implementation is via markdown governance docs and instruction files.

## 3. Implementation Decisions

- Data structures chosen and why:
  - N/A (policy text updates only).
- Algorithms chosen and why:
  - N/A.
- Control-flow structure chosen and why:
  - Added explicit "first question" rule in workflow and bootstrap docs.
- Boundary and interface decisions:
  - Behavioral contract examples kept in formal spec FR-001.
- Error-handling strategy:
  - If mode answer ambiguous, ask one clarifying question (already in FR-001).
- Performance or memory trade-offs accepted:
  - N/A.

## 4. Alternatives Considered

- Alternative 1: Keep mode-first only in formal spec.
- Why rejected:
  - Operational docs might drift; runtime behavior would remain implicit.
- Alternative 2: Add separate startup policy file.
- Why rejected:
  - Increases surface area; existing governance files already own this behavior.

## 5. Derived Invariants and Constraints

- Invariant 1:
  - No domain-specific discovery question is asked before mode is selected.
- Invariant 2:
  - Mode is recorded before project brief details are collected.
- Constraints inherited from the spec:
  - Stage-gated progression and explicit approvals remain mandatory.
- Additional implementation constraints introduced:
  - Startup transcript examples must exist for both Greenfield and Brownfield paths.

## 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - Mode-first behavior existed in FR-001 but was not explicit in all bootstrap docs.
- How the ambiguity was resolved in code:
  - Added explicit first-question wording to workflow and bootstrap instructions.
- Any controlled divergence from the spec:
  - None.
- Follow-up needed in the spec or task list:
  - None for T-001.

## 7. Testing Notes

- Unit tests added:
  - Not executable; validated by policy text consistency checks.
- Integration tests added:
  - Not executable; validated startup flow wording across workflow, README, and copilot instructions.
- Property-based tests added:
  - None.
- Edge cases covered:
  - Ambiguous mode answer flow retained in FR-001.
- Failure modes exercised:
  - Missing governance files still block startup before project questions.

## 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
  - Re-implement mode-first startup as explicit rule in workflow + bootstrap docs and include dual startup transcripts in FR-001.
- Order of implementation steps that mattered:
  - Update normative policy docs first, then spec examples, then task status.
- Non-obvious pitfalls discovered during implementation:
  - Behavior can look implemented in spec but still be inconsistent operationally if bootstrap docs lag.
- What not to change without updating the behavioral spec:
  - Do not reorder startup questions to ask domain details before mode selection.

## 9. Known Limitations

- Limitation 1:
  - No executable test harness for startup behavior exists yet.
- Reason accepted:
  - Current project phase is governance artifact build-out.
- Revisit trigger:
  - Build of automation layer and policy checks in later scope.

## 10. Approval / Review

- Reviewed by: Pending
- Review date: Pending
- Notes:
  - Linked to task T-001 and requirement FR-001.

---

## Chronicle Entry: CHR-GOV-STAGE-GATE

### 1. Chronicle Metadata

- Chronicle ID: CHR-GOV-STAGE-GATE
- Source task ID: T-002
- Source spec sections: FORMAL_SPEC.md section 4 (FR-002)
- Source requirements: FR-002, NFR-001
- Module / component name: Stage transition gate checks
- Implementation language: Markdown policy artifacts
- Author: GitHub Copilot
- Date: 2026-03-19
- Status: Final

### 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
  - Explicit approval and stage-completion commit checks before any stage transition.
- What must remain functionally equivalent across languages:
  - Transition denial conditions and denial-reason logging must be consistent.
- What is intentionally language-specific in this implementation:
  - Enforcement represented as governance policy text.

### 3. Implementation Decisions

- Control-flow structure chosen and why:
  - Added denial-reason logging requirement to both policy (`01_DECISION_POLICY.md`) and enforcement (`02_WORKFLOW_STAGES.md`) to avoid single-file drift.
- Error-handling strategy:
  - FR-002 now specifies denial logging for both missing explicit approval and missing stage-completion commit.

### 4. Alternatives Considered

- Alternative 1: Keep denial logging only in formal spec.
- Why rejected:
  - Operational enforcement can be missed without policy-level wording.

### 5. Derived Invariants and Constraints

- Invariant 1:
  - Stage transition denial always includes a recorded reason.
- Invariant 2:
  - Transition cannot proceed without explicit approval and required stage-completion commit.

### 6. Divergences and Clarifications

- Where the spec was ambiguous:
  - Denial behavior existed, but denial-reason recording location was not explicit.
- How the ambiguity was resolved in code:
  - Required logging to `memory.md` and active stage artifact.

### 7. Testing Notes

- Unit tests added:
  - Not executable; policy and spec consistency validated manually.
- Integration tests added:
  - Not executable; cross-document consistency check performed.

### 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
  - Re-add denial-reason logging requirements in both policy and stage enforcement, then align FR-002 error handling.

### 9. Known Limitations

- Limitation 1:
  - No automated lint rule yet to fail on missing denial-reason logging.
- Revisit trigger:
  - Add automation checks during Automation Layer implementation.

### 10. Approval / Review

- Reviewed by: Pending
- Review date: Pending
- Notes:
  - Linked to task T-002 and requirements FR-002/NFR-001.
