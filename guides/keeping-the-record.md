---
id: guide.keeping-the-record
title: Keeping the Record
describes: [proposal.0001]
---

# Keeping the Record

## When does something become a policy, a decision, or neither?

The record preserves project knowledge that must survive the proposal which revealed it. It is not a
place to restate a feature, capture temporary implementation notes, or collect every review comment.

## The durable artifacts

| Artifact               | Purpose                                                  | When to update it                                                                        |
| ---------------------- | -------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| [Vision](../VISION.md) | What the project is, its MVP, principles, and direction. | At completion, when accepted work changes the project's capabilities, MVP, or direction. |
| Policy                 | A rule that future work must keep enforcing.             | When completed work establishes a durable constraint.                                    |
| Decision               | An architectural choice future work must understand.     | When completed work establishes a durable choice and its rationale.                      |

The vision is living. Policies and decisions preserve a specific rule or choice with the proposal
that established it. A proposal is still the specification for its own change; the record does not
replace it.

## Apply the test

Ask which sentence describes the knowledge:

| If it says                     | Put it in             |
| ------------------------------ | --------------------- |
| “Always do X” or “Never do X.” | A policy.             |
| “We chose X over Y because Z.” | A decision.           |
| “This change will do X.”       | The current proposal. |

**If it does not outlive the current proposal, it belongs in the proposal and nowhere else.**
That includes one-off trade-offs, implementation sequencing, review discussion, and notes useful
only while this change is in flight. Do not turn the record into a junk drawer.

A policy constrains future work. State it as an imperative rule that an author can apply without
reconstructing the pull request. A decision explains a choice, its alternatives, and the context
that made the choice worth keeping. It must answer a future reader who asks why the project works
this way.

## Prefer a check to a policy

Before writing a policy, try to make the rule a validator rule, CI step, git hook, task, or template
section. A policy that can be a check **must** be a check. Prose is the enforcement tier most likely
to be missed and the prose policies are the ones that rot.

This repository supplies a concrete example. The
`.agents/templates/PROPOSAL.md` has a “Policies and decisions checked”
section. That makes reviewing the current record part of the proposal's shape, rather than creating
a policy that says “remember to check policies.” The template is stronger than a reminder.

Use the `.agents/rules/enforcement-hierarchy.md` before adding a rule.
Where a check already enforces one, do not duplicate it in prose.

## Create a durable entry only after it is earned

`policies/` and `decisions/` start empty on purpose. Do not write speculative policies or decisions
because a future change might need them. Record a rule or choice when a feature has established it,
and link it to that feature with `established-by`.

Use the canonical `.agents/templates/POLICY.md` for a narrow rule and its
enforcement tier. Use the `.agents/templates/DECISION.md` for the context,
choice, alternatives, and consequences. Number either file as `<NNNN>-<slug>.md` and use the
matching `policy.<NNNN>` or `decision.<NNNN>` ID.

The policy lifecycle is `draft` to `active`; the decision lifecycle is `proposed` to `accepted`.
Both can become `superseded`. The human decides whether the feature's outcome merits the record;
you can ask the agent to write the entry after that decision.

## Supersede; do not rewrite history

Never edit a policy or decision into a different meaning. Never delete one. Write a new entry with
`supersedes: [<old id>]`, then set the old entry's `status: superseded`.

The validator requires both halves of that change and rejects supersession cycles. Superseded
entries stay reachable in the documentation site's collapsed group. That history lets future work
understand which rule or choice applied at the time and what replaced it.

A correction that leaves the meaning intact can be an ordinary revision. When the substantive rule
or choice changes, supersede it instead.

## Keep the vision current

At completion, compare accepted work with `VISION.md`. Update the vision when the work changed what
the project is, its capabilities, its MVP, or its direction. Bump `updated` whenever you change the
document, and add substantive changes to its revision history.

Do not use the vision as a release plan or a feature inventory. It guides proposals across time;
the proposal records the individual change.

## Before you record it

1. State the knowledge in one sentence.
2. Apply the outlives-the-proposal test.
3. For a rule, try a check, task, or template before prose.
4. If it is durable, settle it with the agent, then have the entry written and linked to the proposal that established it.
5. At completion, update the vision if accepted work changed it.

For the surrounding feature flow, see [Running a Feature](./running-a-feature.md).
