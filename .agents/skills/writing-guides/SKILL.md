---
name: writing-guides
description: Use when an approved proposal has failing tests and its user-facing behavior must be explained before production code is written. Do not use for internal implementation notes or prose written from completed code.
---

# Writing Guides

A guide is an independent, user-facing account of behavior. Write it after failing tests and before code.

Writing the interface as prose from the proposal forces it to be explainable independently of how it was built. A guide written from finished code documents whatever got implemented, including its accidents. Do not reverse that direction.

## Audience and location

Write for someone who has never read the proposal and never will. Explain what they can do, what they observe, and any action they must take; do not make them reconstruct intent from implementation details or record history.

Guides live at `guides/<slug>.md` and use [`.agents/templates/GUIDE.md`](../../templates/GUIDE.md). A guide may describe several proposals over time. The usual act is editing the existing guide that already owns the user-facing workflow, not creating another page.

| Action         | Use it when                                                                                                            |
| -------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Extend a guide | The proposal changes a workflow, interface, or concept the guide already explains. Keep one coherent source of truth.  |
| Add a guide    | No guide covers the workflow, or combining it with an existing guide would make either audience or behavior ambiguous. |

Do not duplicate overlapping instructions across guides. Extend the `describes` list when the guide becomes an account of another proposal.

## Frontmatter contract

Every guide begins with the template's required frontmatter:

```yaml
id: guide.<slug> # matches guides/<slug>.md
title: <Title Case>
describes: [proposal.<NNNN>] # one or more
```

`describes` is deliberately a list. Preserve existing proposal ids and add the current one when you extend a guide. The guide explains behavior; it does not restate the proposal's design process.

## Required sequence

1. Read the approved proposal. Extract every observable behavior, input, outcome, error, constraint, and compatibility consequence relevant to a user.
2. Locate the guide that owns that workflow. Extend it unless the add-guide condition above applies.
3. Write the guide from the proposal, not from code. State the interface, prerequisites, observable results, failure behavior, and examples needed to use it.
4. Run every worked example against the project's real interface before including it. If an example cannot run exactly as printed, correct it or remove it. A copied transcript, invented output, or unexecuted command is not a worked example.
5. **Compare the guide with the failing tests before writing code.** Read them side by side and list every behavior each mentions. Anything mentioned by one but not the other is a defect in one of them. Resolve every difference — by correcting the guide, correcting the test, or revising the proposal with the human — before code begins.
6. Give the resulting guide and comparison to the implementer. Code implements the agreement; it does not decide it.

The comparison is the load-bearing step. Without it, a guide is decoration and tests are an unreviewed interpretation.

## Write observable behavior

| Write                                                                     | Do not write                                                                                         |
| ------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Inputs, prerequisites, user actions, outputs, errors, and durable effects | Private data structures, control flow, file layout, or implementation rationale users cannot observe |
| Exact constraints and defaults                                            | Hedges that make a tested rule optional                                                              |
| Executed examples with their real prerequisites                           | Copied, imagined, stale, or destructive examples without an executable safe path                     |

When a detail changes what a user can observe or do, the guide names it. When a detail exists only to make the code work, leave it out unless it changes an observable contract.

## Guides and previews

Guides are the content a documentation-shaped preview publishes. Review the rendered guide through that preview as part of Phase 4. Guides and previews are private by default; do not turn a preview into a public publication unless the project deliberately opts in.

## Red flags

| Red flag                                             | Required correction                                                                  |
| ---------------------------------------------------- | ------------------------------------------------------------------------------------ |
| Describes internals the user cannot observe          | Replace it with the observable result, or remove it.                                 |
| Hedges ("should generally") where tests are exact    | State the exact behavior, then make the guide and tests agree.                       |
| Documents behavior no test covers                    | Add the missing failing test or remove behavior that the proposal does not require.  |
| Tests cover behavior the guide never mentions        | Document the user-relevant behavior or remove the test if it exceeds the proposal.   |
| Written or rewritten after the implementation landed | Discard the code-shaped prose and derive it from the proposal; rerun the comparison. |
| Contains an example nobody has executed              | Execute it verbatim, fix it, or remove it.                                           |

Any red flag means the guide cannot proceed to code or review until corrected.

## Related skills

- [`.agents/skills/implementing-a-proposal/SKILL.md`](../implementing-a-proposal/SKILL.md) — enforces the tests → guides → code order.
- [`.agents/skills/test-driven-development/SKILL.md`](../test-driven-development/SKILL.md) — supplies the failing tests compared here.
- [`.agents/skills/cross-artifact-review/SKILL.md`](../cross-artifact-review/SKILL.md) — checks the agreement after implementation.
