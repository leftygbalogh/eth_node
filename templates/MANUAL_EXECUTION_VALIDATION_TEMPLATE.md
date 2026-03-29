# Manual Execution Validation Charter

**Project:** [Project Name]  
**Documentation Target:** [Document name, e.g., CLI_REFERENCE.md]  
**Validation Date:** [YYYY-MM-DD]  
**Executor:** [Agent name or role]  
**Platform:** [Windows/macOS/Linux + shell type]  
**Session Duration:** [Estimated hours]  

---

## 1. Validation Charter

**Purpose:** Verify that every documented example in [document name] can be executed literally (copy-paste) by a beginner user and produces the documented behavior, output format, and user experience on [target platform].

**Scope:**
- All command examples in the target document
- All multi-step workflows and scenarios
- All output format examples
- All prerequisite setup instructions

**Out of Scope:**
- Automated test execution (covered separately)
- Performance benchmarking
- Non-documented edge cases

**Success Criteria:**
- Zero CRITICAL blockers (commands that fail to execute or produce errors)
- Fewer than 3 HIGH-severity issues (output mismatches, missing documentation, unclear instructions)
- All findings documented with severity, platform context, and reproduction steps
- Platform-specific quirks identified and documented

---

## 2. Platform Test Matrix

Document the test environment for this validation session:

| Platform Component | Version/Details | Notes |
|-------------------|----------------|-------|
| Operating System | [e.g., Windows 11 23H2] | |
| Primary Shell | [e.g., PowerShell 7.4] | |
| Secondary Shell | [e.g., Git Bash 2.43] | |
| Language Runtime | [e.g., Rust 1.75, Node 20.x] | |
| Required Tools | [e.g., forge 0.2.0, anvil, cast] | |
| Terminal | [e.g., Windows Terminal, iTerm2] | |

**Required Dependencies:** (check each before starting)
- [ ] Development tools installed and accessible in PATH
- [ ] Test network/service running (e.g., Anvil on port 8545)
- [ ] Required fixtures and contracts present in repository
- [ ] Environment variables set per documentation
- [ ] Working directory confirmed correct

---

## 3. Execution Protocol

### 3.1 Method

Execute each documented example in **strict literal order** as follows:

1. **Copy the command exactly** from the documentation (no retyping, no "smart" fixes)
2. **Paste into appropriate shell** (match the shell type the docs assume or specify platform)
3. **Execute and observe**:
   - Exit code
   - Stdout content and format
   - Stderr content (errors, warnings, logs)
   - Side effects (files created, state changes, network activity)
4. **Compare actual vs. documented behavior**:
   - Does the command succeed? (exit 0 expected)
   - Does the output match what docs show?
   - Are error messages helpful if it fails?
   - Are prerequisites clearly stated?
5. **Record finding** if any discrepancy exists

### 3.2 Finding Severity Levels

| Severity | Definition | Examples |
|----------|-----------|----------|
| **CRITICAL** | Command fails to execute; complete blocker for users | Syntax error, missing flag, nonexistent file, tool not found |
| **HIGH** | Command succeeds but output/behavior differs significantly from docs | Wrong output format, missing expected fields, incorrect data |
| **MEDIUM** | Command works but UX is confusing or requires workaround | Unclear error message, undocumented prerequisite, platform quirk |
| **LOW** | Minor discrepancy or cosmetic issue | Typo in docs, outdated version number, formatting inconsistency |

### 3.3 Evidence Collection

For each executed example, record:
- **Section reference** (e.g., "Section 7.2, Example 3")
- **Command executed** (exact copy-paste)
- **Expected behavior** (from docs)
- **Actual result** (what happened)
- **Exit code**
- **Platform context** (if issue is platform-specific)
- **Screenshot/transcript** (if output format issue)

---

## 4. Findings Log

Use this table format to record all findings during validation:

| ID | Section | Severity | Issue Summary | Platform | Expected | Actual | Notes |
|----|---------|----------|---------------|----------|----------|--------|-------|
| F001 | 7.2 | CRITICAL | Missing --broadcast flag | All | Contract deployed | Dry-run only | forge create without --broadcast performs simulation |
| F002 | 7.4 | CRITICAL | cast send quote failure | Windows/PS | Event emitted | Syntax error | PowerShell→bash quoting breaks function signatures |
| F003 | 7.1 | HIGH | balance output mismatch | All | "Balance: X wei" | JSON log only | --quiet flag required for documented format |

*Continue table for all findings...*

---

## 5. Platform-Specific Discoveries

Document platform-specific behaviors, quirks, or workarounds discovered:

### Windows (PowerShell + Git Bash)

**Quirks Discovered:**
- [e.g., PowerShell→bash -c invocation requires .sh script files for commands with function signature quoting]
- [e.g., alias commands don't persist in single-line bash -c execution]
- [e.g., Path translation required: /c/tmp vs C:\tmp]

**Recommended Workarounds:**
- [e.g., Create temporary .sh files for complex cast send commands]
- [e.g., Use full binary path instead of alias in PowerShell context]

### macOS

**Quirks Discovered:**
- [Document any macOS-specific issues]

**Recommended Workarounds:**
- [Document solutions]

### Linux

**Quirks Discovered:**
- [Document any Linux-specific issues]

**Recommended Workarounds:**
- [Document solutions]

---

## 6. Validation Results Summary

**Total Examples Executed:** [X]  
**Total Findings:** [Y]  
- CRITICAL: [N]
- HIGH: [N]
- MEDIUM: [N]
- LOW: [N]

**Platform Coverage:**
- [ ] Windows (PowerShell + Git Bash)
- [ ] macOS (zsh/bash)
- [ ] Linux (bash)

**Pass/Fail Determination:**

| Criterion | Status | Notes |
|-----------|--------|-------|
| Zero CRITICAL blockers | ✅ PASS / ❌ FAIL | [Details if FAIL] |
| < 3 HIGH-severity issues | ✅ PASS / ❌ FAIL | [Count: X] |
| All findings documented | ✅ PASS / ❌ FAIL | |
| Platform quirks identified | ✅ PASS / ❌ FAIL | [See Section 5] |

**Overall Result:** ✅ PASS / ❌ FAIL / ⚠️ CONDITIONAL PASS

---

## 7. Recommendations

### Immediate Documentation Fixes Required

1. [Fix description] - [Priority] - [Section reference]
2. [Fix description] - [Priority] - [Section reference]

### Platform-Specific Documentation Additions Needed

1. Add "Platform Considerations" section with Windows/PowerShell quirks
2. Add workaround guidance for [specific issue]
3. Clarify prerequisite for [specific requirement]

### Test Infrastructure Improvements

1. [e.g., Add fixture existence validation to automated tests]
2. [e.g., Enhance automated test depth from Tier 1 to Tier 2]

---

## 8. Session Evidence

**Transcript Location:** `[path/to/transcript.txt]`  
**Screenshots/Artifacts:** `[path/to/evidence/]`  
**Commit Hash (if fixes applied):** `[git hash]`  

**Executor Sign-off:**
- [Agent/Role name]
- Date: [YYYY-MM-DD]
- Time spent: [X hours]

**Approval:**
- [ ] Product Owner reviewed findings
- [ ] CRITICAL/HIGH issues resolved or documented as known limitations
- [ ] Documentation updates committed (if applicable)
- [ ] Ready to proceed to Stage 5

---

## Appendix A: Execution Checklist

Use this checklist to track progress through the document:

### Section [X]
- [ ] Example 1: [Brief description] - Result: ✅ / ❌ / ⚠️
- [ ] Example 2: [Brief description] - Result: ✅ / ❌ / ⚠️
- [ ] Example 3: [Brief description] - Result: ✅ / ❌ / ⚠️

### Section [Y]
- [ ] Scenario 1 Step 1: [Brief description] - Result: ✅ / ❌ / ⚠️
- [ ] Scenario 1 Step 2: [Brief description] - Result: ✅ / ❌ / ⚠️

*Continue for all documented examples...*

---

## Appendix B: Quick Reference - Common Issues

| Issue Pattern | Likely Cause | Quick Check |
|---------------|--------------|-------------|
| Command not found | Missing tool in PATH, wrong shell | `which <tool>` or `where <tool>` |
| File not found | Missing fixture, wrong working directory | `ls` or `dir` to verify, check `pwd` |
| Syntax error | Platform-specific quoting, shell mismatch | Try in alternate shell, check quotes |
| Permission denied | File permissions, sudo required | `ls -la` to check perms, retry with sudo |
| Connection refused | Service not running (e.g., Anvil) | Check process list, verify port |
| Output format mismatch | Flag missing, version difference | Check flags, compare tool versions |

---

**Template Version:** 1.0  
**Created:** 2026-03-29  
**Source:** FB-017 - eth_node project manual validation retrospective
