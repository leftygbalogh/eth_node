# Post-Release Monitoring Plan

## Monitoring Window

- Immediate: first working session after release commit.
- Short-term: first three manual CLI walkthroughs after release.
- Ongoing: every stage-gate cycle for governance drift detection.

## Signals to Monitor

- Unexpected CLI output contract changes (stdout/stderr format drift).
- Failures in capture-session or capture-multi helper scripts.
- Terminal/environment compatibility regressions (PowerShell vs Git Bash behavior).
- Missing prompt or memory log entries in governance files.

## Alert Thresholds

- Critical: stage transition attempted without explicit approval.
- High: release artifact or evidence mismatch.
- Medium: repeated environment-specific CLI failures.
- Low: documentation drift requiring non-breaking corrections.

## Alerting Route

1. Record issue and impact in memory.md.
2. Add corresponding user prompt context to prompts.md if user-triggered.
3. Capture failing session artifacts under output/sessions/.
4. Create a focused remediation task and verification evidence before closure.
