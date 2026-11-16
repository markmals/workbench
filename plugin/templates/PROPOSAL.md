---
id: proposal.<NNNN>
title: <Title Case>
authors: [<name>]
# Set `draft` on creation. See the lifecycle note below for who decides each transition.
status: draft
pull-request: <url or number, empty while unopened>
issues: []
supersedes: []
---

# <Title>

<!--
Delete this instruction and replace every <placeholder>. A proposal is the durable record
of one change: why it is needed, what it changes in the project's current specifications,
policies, and decisions, and how. Write for someone without the pull-request conversation.
Link the current sources the change affects rather than restating complete contracts; once
the change is complete, the proposal is history and those sources hold the requirements.

Lifecycle:
- `draft` — Phase 2 proposal development with the human; may carry clarification markers.
- `awaiting-implementation` — Phase 3 implementation may begin.
- `active-review` — Phase 4 human evaluation; the readiness report is posted. The pull request became ready earlier, when the complete, verified implementation was pushed.
- `returned-for-revisions` — Phase 4 human feedback returned the work to Phase 2 or Phase 3.
- `accepted` — Phase 5 vision update, merge, and release are underway.
- `implemented` — the accepted implementation and final records are committed in the feature pull request before merge. Merge and any required release remain separate.
- `rejected` — the human declined the change; retained as a record.
- `withdrawn` — the author pulled it; retained as a record.
- `superseded` — a later proposal replaced it; retained as a record.
Normal path: `draft` → `awaiting-implementation` → `active-review` → `accepted` → `implemented`.
Who decides: the human decides `awaiting-implementation`, `returned-for-revisions`, `accepted`, and `rejected`. Once the complete implementation is verified and pushed, mark the pull request ready while the proposal remains `awaiting-implementation`. At the end of Phase 3, `reviewing-an-implementation` moves the proposal to `active-review` and posts the readiness report. After acceptance, the agent moves `implemented` in that same pull request before merge. The human may ask the agent to make an edit — deciding is the part an agent must never do alone. Permission to begin implementing is `awaiting-implementation`, never `accepted`.

-->

## Summary

<!--
Write no more than 50 words. The proposal list displays this out of context, so name
the change and its value plainly. It does not need to flow into the next section.
-->

<What changes and why it matters.>

## Motivation

<!--
Describe the problem: who encounters it, its impact, evidence that it exists, and the
current workaround. Do not propose a solution; establish why the problem is worth solving.
-->

<Concrete problem, affected people, impact, and current workaround.>

## Domain grounding

<!--
Record the knowledge necessary to evaluate this proposal on the problem's own
terms.

Distinguish established facts from repository observations, inferences, and
remaining uncertainties. Link primary or authoritative sources for important
external claims.

This section explains the world the proposed solution must fit. It is not an
implementation plan.

If no independent domain research is needed, state why rather than filling the
subsections with generic claims. Use “None” for remaining uncertainty when resolved.
-->

### Established context

- ...

### Relevant constraints and principles

- ...

### Quality bar

- ...

### Remaining uncertainty

- ...

## Existing baseline

<!--
After grounding in the domain, inspect the closest existing implementation,
prototype, prior art, or workflow that already accomplishes the intended result.

Describe what it demonstrates, what should be preserved, and what appears to be
incidental, compromised, or incorrect. Existing behavior is evidence, not
authority. If there is no relevant baseline, state that explicitly.
-->

<The closest usable baseline and what can be learned from it.>

## Proposed solution

<!--
Make the persuasive case for the chosen direction. Use representative examples where
they clarify the pitch. This is an overview, not a complete specification.
-->
<Concise explanation of the approach and expected result.>

```text
<Representative action or input> → <observable outcome>
```

## Detailed design

<!--
Link each current source this change affects (API contract documentation, declarations or
schemas, feature files, reference prose, policies, decisions) and specify what changes in it:
behavior, interfaces, data, failures, boundary cases, and operational constraints. Name any
necessary information that has no current home. Link requirements that stay the same instead
of restating them. Someone who is not the author must be able to revise those sources and
implement the change without asking questions.
For a user interface, specify what the user can do, find out, and is told; leave its look and feel to the running UI.
-->

- <Current source, linked>: <what this change does to it>.
- When <trigger or input>, <actor or system> <required behavior>.
- If <error or boundary condition>, <observable result>.
- <Data, configuration, migration, or operational requirement.>

## Acceptance criteria

<!--
Usually delete this section. Detailed design already describes the change and the current
specifications state the result; restating either as a checklist creates a copy that drifts.

Keep it only when the change has discrete outcomes that Detailed design cannot express naturally
— a structural change touching many independent things, or a migration with separable steps.
If every criterion is a restatement of a sentence above, delete the section.
-->

- [ ] <Discrete, independently verifiable outcome.>

## Compatibility

<!--
State effects on existing users, callers, stored data, or configuration, including migration,
fallback, and whether a break is opt-in. If none, state “No compatibility impact.” explicitly: considering it is the point.
-->

<Compatibility impact, migration, and opt-in status — or “No compatibility impact.”>

## Implications on adoption

<!--
Explain new requirements, version floors, setup or migration steps, and whether adoption can be reversed.
If nothing changes, state “No implications on adoption.” explicitly rather than deleting this section.
-->

<Adoption requirements and reversibility — or “No implications on adoption.”>

## Scope

<!--
Name the work required to deliver this proposal. Keep it bounded; merely related work
belongs elsewhere unless this change requires it.
-->

- <In-scope deliverable or change.>
- <In-scope validation or migration.>

### Out of scope

<!--
Name deliberate exclusions and why. Defects discovered while writing this proposal that fall outside
scope are filed in Beads, not folded in; use .agents/skills/tracking-work/.
-->

- <Excluded work and reason it is excluded.>

## Preview

<!--
Choose the artifact from the preview table that exercises this change and explain why it is the cheapest
realistic one. A human must exercise changed behavior, not merely inspect prose or a screenshot; use .agents/skills/reviewing-an-implementation/. For a pure refactor with no exercisable surface, state why no preview applies.
-->

- Artifact: <preview artifact>
- Reason: <why this is the cheapest realistic exercise of the change>

## Policies and decisions checked

<!--
Read every relevant entry in policies/ and decisions/, then state how this proposal honors each, or which one it revises or retires.
“None” is permissible only after actually reading both directories; record conflicts rather than silently departing from them.
-->

- <policy-or-decision-id> — <how this proposal honors, revises, or retires it>

## Future directions

<!--
Describe possibilities neutrally; avoid “we will” and “we should.” Ask whether each belongs in this proposal
now rather than using this section to defer necessary work. Remove the placeholder when no direction is useful.
-->

- <Neutral possible extension and why it is not part of this proposal.>

## Alternatives considered

<!--
Reviewers may already prefer another approach. Engage with credible alternatives raised during review; be fair but need not be neutral.
If an alternative wins, change the proposal, explain why the new idea won, and update this section as review continues.
-->

- <Alternative> — <why it was not chosen>

## Open questions

<!--

Put every unresolved question in the exact marker below. Resolve every marker before the human
approves implementation; the validator enforces this. Do not guess an answer merely to make the
proposal appear complete.
-->

- [NEEDS CLARIFICATION: <question>]

## Acknowledgments

<!--
Optional. Credit people, prior work, or review that materially shaped this proposal.
Remove this section when no acknowledgment is useful.
-->
<Contribution or influence.>
