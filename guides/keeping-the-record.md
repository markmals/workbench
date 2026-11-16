---
id: guide.keeping-the-record
title: Keeping the Record
---

# Keeping the Record

## When does something become a policy, a decision, or neither?

The record holds project context that must outlive the proposal which revealed it: the vision, the rules future work keeps enforcing, and the choices it must understand. It is not a place to restate a feature, capture temporary implementation notes, or collect every review comment. What the product does belongs in its current specifications, described in [Keeping Current Specifications](./current-specifications.md).

## The durable artifacts

| Artifact               | Purpose                                                     | When to update it                                                                        |
| ---------------------- | ----------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| [Vision](../VISION.md) | What the project is, its MVP, principles, and direction.    | At completion, when accepted work changes the project's capabilities, MVP, or direction. |
| Policy                 | A current rule that future work must keep enforcing.        | When agreed work establishes, changes, or retires a durable constraint.                  |
| Decision               | A current architectural choice future work must understand. | When agreed work establishes, changes, or retires a durable choice and its rationale.    |

All three are current documents. Each policy or decision file states the rule or choice that applies now; Git holds its earlier wording, and the proposal that changed it explains why. A proposal explains its own change and is not a current record.

**Most features add nothing here, and that is the point.** Recording nothing is the ordinary outcome, not a step anyone forgot. Both directories being empty means no feature has yet established a rule or a choice worth outliving itself — a healthy state, not a gap to fill.

A policy written speculatively is one nobody follows and everybody has to read. Wait until agreed work actually establishes the rule.

## Apply the test

Ask which sentence describes the knowledge:

| If it says                     | Put it in                                         |
| ------------------------------ | ------------------------------------------------- |
| “Always do X” or “Never do X.” | A policy.                                         |
| “We chose X over Y because Z.” | A decision.                                       |
| “The product does X.”          | The current specification that owns that surface. |
| “This change will do X.”       | The current proposal.                             |

**If it does not outlive the current proposal, it belongs in the proposal and nowhere else.** That includes one-off trade-offs, implementation sequencing, review discussion, and notes useful only while this change is in flight. Do not turn the record into a junk drawer.

A policy constrains future work. State it as an imperative rule that an author can apply without reconstructing the pull request. A decision explains a choice, its alternatives, and the context that made the choice worth keeping. It must answer a future reader who asks why the project works this way.

## Prefer a check to a policy

Before writing a policy, try to make the rule a validator rule, CI step, git hook, task, or template section. A policy that can be a check **must** be a check. Prose is the enforcement tier most likely to be missed and the prose policies are the ones that rot.

Workbench supplies a concrete example. Its proposal template, `.workbench/templates/PROPOSAL.md`, has a “Policies and decisions checked” section. That makes reviewing the current record part of the proposal's shape, rather than creating a policy that says “remember to check policies.” The template is stronger than a reminder.

Read `.workbench/rules/enforcement-hierarchy.md` before adding a rule. Where a check already enforces one, do not duplicate it in prose.

## Create a durable entry only after it is earned

`policies/` and `decisions/` start empty on purpose. Do not write speculative policies or decisions because a future change might need them. Record a rule or choice when agreed work has established it. Optional `established-by: proposal.<NNNN>` names the proposal that introduced it, as provenance; it is not required, and it does not need to change when the record is revised later.

Use the canonical policy template, `.workbench/templates/POLICY.md`, for a narrow rule, or a link to the source that owns it, with its scope and enforcement. Use `.workbench/templates/DECISION.md` for the current arrangement, context, alternatives, and consequences. A project that overrides either in `.agents/templates/` uses its override. Number either file as `<NNNN>-<slug>.md` and use the matching `policy.<NNNN>` or `decision.<NNNN>` ID. Required frontmatter is `id`, `title`, and `status`; [Record Validation and Workflow Status](./record-validation.md) lists the exact checks.

The policy lifecycle is `draft` to `active`; the decision lifecycle is `proposed` to `accepted`. A proposed record stays on its change branch until it is agreed. The human decides whether the feature's outcome merits the record; you can ask the agent to write the entry after that decision.

## Revise or retire in place

When an agreed rule or choice changes, edit its file. Keep the filename and ID, revise the rationale and enforcement in the same change, and update current references to it. When it no longer applies, delete the file. A rule that still binds a supported compatibility promise or an unfinished migration still applies until that obligation ends.

The record needs no replacement file, supersession field, or revision-history table. Git keeps every earlier version, and the proposal that made the change explains it. The validator rejects `supersedes` and `status: superseded` on a policy or decision with a message explaining how to revise or retire it instead.

Before deleting a record, find every link to it. Repoint or remove current references. Where a historical link must keep working, such as a completed proposal that cited the record, point it at an immutable Git revision that still contains the file. Never reuse a retired ID for a different record. [Keeping Current Specifications](./current-specifications.md#worked-example-revising-and-retiring-a-policy) works through one revision and one retirement.

## Migrate a supersession chain

A project adopting this model may already have policies or decisions that supersede one another. For each chain:

1. With the human, determine the complete rule or choice that still applies. Do not assume the newest record already contains every clause the older ones kept.
2. Write the retained clauses into the surviving logical record. Keep the surviving ID where possible.
3. Remove its `supersedes` field and give it a current status: `active` for a policy, `accepted` for a decision.
4. Update current references to the retired records, and replace each historical link that must survive with an immutable Git revision link.
5. Delete the retired files.
6. Run `mise run check`. The validator reports remaining obsolete metadata, and the documentation build reports broken links.

## Keep the vision current

At completion, compare accepted work with `VISION.md`. Update the vision when the work changed what the project is, its capabilities, its MVP, or its direction. Bump `updated` whenever you change the document, and add substantive changes to its revision history.

Do not use the vision as a release plan or a feature inventory. It guides proposals across time; the proposal records the individual change.

## Before you record it

1. State the knowledge in one sentence.
2. Apply the outlives-the-proposal test.
3. For a rule, try a check, task, or template before prose.
4. If it is durable, settle it with the agent, then have the entry written, or the existing entry revised or deleted.
5. At completion, update the vision if accepted work changed it.

For the surrounding feature flow, see [Running a Feature](./running-a-feature.md).
