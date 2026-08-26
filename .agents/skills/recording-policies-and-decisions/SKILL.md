---
name: recording-policies-and-decisions
description: Use when an accepted proposal leaves a durable rule or architectural choice for future work. Do not use for temporary proposal context.
---

# Recording Policies and Decisions

Record only knowledge that outlives its proposal.

## The test

| Record   | It answers                                        | Example                      |
| -------- | ------------------------------------------------- | ---------------------------- |
| Policy   | What must future work always or never do?         | Always validate imports.     |
| Decision | What choice must future work understand, and why? | We chose X over Y because Z. |

If it does not outlive this proposal, it belongs in the proposal and nowhere else.

## Before writing

1. Discuss the record with the human. Neither a policy nor a decision is an agent-only conclusion.
2. Confirm the originating proposal id.
3. Use the applicable canonical template. Both records carry `established-by: proposal.<NNNN>`.

## Enforce policies first

A policy that can be enforced by a check MUST be. Before writing prose, try to express it as a validator rule, CI step, or template section. Apply `.agents/rules/enforcement-hierarchy.md`.

Keep prose for judgment a deterministic check cannot decide. Prose policies are the ones that rot.

## Supersede; never rewrite history

Never edit a policy or decision into a different meaning. Never delete one.

1. Write a new record with `supersedes: [<old id>]`.
2. Set the old record's `status: superseded`.
3. Preserve the old rationale and link.

The validator enforces this pairing. A correction that does not change meaning may amend the existing record; a changed rule or rationale requires supersession.

## Red flags

- A one-time implementation detail is becoming a record.
- You wrote a record without the human.
- You wrote a prose policy before checking whether automation can enforce it.
- You changed an old record's meaning in place.
- A new superseding record left the old status active.

Any of these means stop and correct the record boundary.
