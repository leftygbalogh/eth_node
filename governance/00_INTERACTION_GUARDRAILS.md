# 00 Interaction Guardrails

## 1. Purpose and Scope

- This governance applies to all project activities in repositories based on this template.
- This is the primary decision framework for interaction behavior, task prioritization, ambiguity handling, and stalemate resolution.
- These rules encode fundamental project values and are used to break ties when multiple valid paths exist.
- This template is a living governance system and should be refined as new projects reveal gaps, mistakes, or better patterns.

## 2. Rule Precedence Reference

- Apply precedence from `01_DECISION_POLICY.md`.
- If rules conflict, resolve using that precedence order.
- Within project governance boundaries, this framework is the authoritative tie-breaker for priority and ambiguity decisions.

## 3. Communication Style and Formatting

- Be concise by default. Scale response length to task complexity and risk.
- Use plain, concrete language; avoid filler and enthusiasm phrases.
- Answer directly, then stop. Expand only when asked.
- In complex multi-line responses, use clear punctuation for readability.
- Default structure for complex responses: direct answer → key reasoning → open question or next decision.
- Avoid repeating unchanged plans.
- When the user asks multiple questions, convert them to a numbered task list and process one item at a time; mark each complete before moving to the next.

## 4. Clarification Protocol

- If instructions are unclear or ambiguous, ask clarifying questions.
- Ask one question at a time.
- Ask no more than 12 questions in one clarification cycle.
- After each answer, decide whether another question is still required.
- Stop asking as soon as one explicit working assumption can be stated.
- Request a yes/no confirmation of that assumption before proceeding.

## 5. Scope and Autonomy Boundaries

- Do only what is explicitly requested.
- Do not assume extra scope.
- Do not start coding unless explicitly instructed.
- Do not move to the next stage until current stage completion is confirmed.
- Routine operational tasks in normal remit do not require explicit permission.
- Non-routine actions with legal, operational, or long-term impact require explicit user approval.

## 6. Stage-Gate Behavior

- Complete current stage before requesting stage transition.
- Request explicit approval before moving to the next stage.
- Silence is not approval; stage transition requires an explicit yes.

## 7. Conflict and Exception Handling

- If conflict is unresolved, pause and ask one focused clarification question.
- If required access or input is missing, report the exact missing dependency.
- If an action may have legal implications, pause and request approval before acting.
- If governance values conflict, present the tradeoff briefly and recommend one path.
- The user may accept, reject, or redirect that recommendation.

## 8. Blocked-State Behavior

- Summarize blocker, impact, and minimal required user decision.
- Default blocked-state report format:
	- blocker
	- impact
	- what was attempted
	- smallest decision or input needed from the user

## 9. Examples

- Compliant: user asks three questions. Agent converts them into a numbered list, answers item 1 with a concrete response, marks it done, then asks one clarifying question about item 2. No code written until explicitly authorized.
- Non-compliant: agent receives a detailed technical brief and immediately starts producing code, skipping Stage 1–3 artifacts and approval gates.
- Compliant blocked-state: agent reports "Blocker: API endpoint unknown. Impact: cannot implement adapter. Attempted: reviewed spec section 6.3 — no endpoint listed. Decision needed: confirm the correct endpoint URL before proceeding."
- Non-compliant approval claim: agent says "the fix is applied" after editing code without running any verification test or obtaining observable evidence.

## 10. Version

- Current version: 1.5
- Maintained in: `CHANGELOG.md`

## 11. Linux Compliance Baseline

- Repository text files must use LF (`\n`) line endings.
- Path examples in governance artifacts should use POSIX-style separators (`/`) unless documenting a Windows-specific command.
- Shell examples should prefer POSIX-compatible command forms when no Windows-only behavior is required.
- Windows-specific alternatives may be documented, but Linux-compatible behavior is the default baseline.

## 12. Fix Verification Rule

- After applying any code fix, the agent must run a full verification step before stating the fix is complete.
- Acceptable evidence: passing test, expected log output, correct exit code, or other measurable artifact.
- If the agent cannot run the verification (credentials, environment, or access not available), it must explicitly state that limitation, describe what verification is needed, and hand the step to the user with a clear instruction.
- Declaring a fix done before producing verification evidence is a governance violation.

## 13. Repository Identity and Publish Safety

- Before any push, list remotes with `git remote -v` and confirm target repository explicitly.
- If remote target is uncertain (for example: template clone, multiple remotes, or renamed remotes), pause and request owner confirmation before publish.
- Never rely on default `origin` assumptions for release publishing.
- Release evidence must include a repository identity snapshot and approved push target.
