---
name: implementing-a-proposal
description: Use when an approved proposal on an existing branch with a draft pull request needs substantive implementation, or after root-cause analysis identifies a nontrivial bug fix. Do not use for proposal authoring or trivial inline edits.
---

# Implementing a Proposal

This is the default Phase 3 workflow and its controlled handoff into Phase 4. A proposal is the unit of work. Its intent must survive independent expression in tests, guides, and code.

**Hard order:** failing tests → guides → code. Do not reverse, combine, or defer these steps.

## Preconditions

- The proposal is approved (`status: awaiting-implementation`) and has no unresolved clarification marker.
- Phase 1 has already created a branch, pushed it to `origin`, and opened a draft pull request.
- You read the proposal and the applicable vision, policies, and decisions. Stop and raise any conflict; do not silently deviate.

## Slice the proposal

Dispatch one implementer per coherent slice. Slice by acceptance criterion unless criteria share state, an interface, or an atomic observable behavior; keep those together.

| Slice shape                            | Dispatch boundary                                                                     |
| -------------------------------------- | ------------------------------------------------------------------------------------- |
| Independent acceptance criterion       | One implementer and one complete tests → guides → code cycle                          |
| Criteria sharing an interface or state | One implementer for the combined behavior                                             |
| Cross-cutting change                   | One owner for the shared boundary; separate callers only after its contract is stable |

Never parallel-dispatch implementers onto the same files. A small proposal with one coherent change gets one implementer; it is not split merely to create concurrency.

## Controller loop

For each slice, in an order that protects shared boundaries:

1. Curate the implementer brief: proposal text, relevant acceptance criteria, governing records, affected tests and guides, nearby project conventions, and the applicable commands from `mise tasks`.
2. Dispatch one implementer. Tell it not to commit.
3. Handle its status. Do not start a later review stage until this slice has completed its ordered implementation cycle and its quality gates are clean.
4. After every slice is clean and integrated, run the proposal-wide review sequence below.

The brief must make expected external behavior and scope explicit. Do not ask the implementer to infer intent from an incomplete conversation or to choose behavior the proposal leaves open.

## Implementer order — non-negotiable

The implementer completes this sequence for its slice:

1. **Failing tests.** Write a focused test for each behavior, name it idiomatically with the proposal and acceptance criterion it verifies, run it, and confirm it fails for the missing behavior rather than setup.
2. **Guides.** Edit or add the relevant guide from the proposal — before production code exists — using [`.agents/skills/writing-guides/SKILL.md`](../writing-guides/SKILL.md).
3. **Compare guides and tests.** Read the guide and failing tests side by side. List every behavior each names. A behavior named by only one is a defect in one of them; resolve it before writing code.
4. **Code.** Write the minimum code that makes the tests pass. Do not add behavior absent from the proposal, tests, and guide.
5. **Quality gates.** Run `mise run check`, then run the applicable project-specific gates discovered through `mise tasks`.
6. **Self-review.** Re-read the proposal, tests, guide, and changed code; fix clear gaps before reporting. Self-review never substitutes for review.

The implementer reports exactly one status:

| Status               | Controller action                                                                                                                                                                              |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `DONE`               | Continue only after the reported gates are clean.                                                                                                                                              |
| `DONE_WITH_CONCERNS` | Resolve correctness or scope concerns before review; record observations for review.                                                                                                           |
| `NEEDS_CONTEXT`      | Supply the missing material and re-dispatch. Do not tell the implementer to guess or search for intent.                                                                                        |
| `BLOCKED`            | Classify the blocker: missing context, reasoning shortfall, oversized slice, or proposal conflict. Supply context, escalate the model for reasoning, re-slice, or stop and raise the conflict. |

Never silently retry the same dispatch.

## Proposal-wide review sequence

Run these stages in order after implementation is integrated. A finding returns work to the implementer; rerun quality gates and the affected review stage. Never start the next stage before the prior stage is clean.

1. **Proposal-compliance.** Dispatch an inline reviewer with the full proposal and changed artifacts. It confirms every acceptance criterion is implemented and tested, checks that no behavior exceeds approved scope, and returns approval or specific gaps. Gaps go to the implementer, then this stage repeats.
2. **Code-quality.** Only after proposal-compliance is clean, dispatch an inline reviewer. It checks project conventions, names, duplication, boundary error handling, unnecessary complexity, and tests that assert observable behavior. It reports concrete important issues; the implementer fixes them and this stage repeats.
3. **Cross-artifact review.** After both inline stages are clean, load and follow [`.agents/skills/cross-artifact-review/SKILL.md`](../cross-artifact-review/SKILL.md). Do not replace it with an abbreviated local review.
4. **Adversarial review.** Only after cross-artifact review is clean, load and follow [`.agents/skills/adversarial-review/SKILL.md`](../adversarial-review/SKILL.md). It is report-only, fresh-context, and uses a different model from the implementer where possible.

After adversarial review converges, continue Phase 4 through [`.agents/skills/producing-a-preview/SKILL.md`](../producing-a-preview/SKILL.md) and [`.agents/skills/reporting-readiness/SKILL.md`](../reporting-readiness/SKILL.md).

## Constraints

- **Never dispatch parallel implementer subagents on the same files.** They will conflict.
- **Never let the implementer commit.** It works in a partial state; the controller follows [`.agents/rules/commit-discipline.md`](../../rules/commit-discipline.md) at the natural boundary.
- **Never let self-review substitute for review.** Proposal-compliance, code-quality, cross-artifact review, and adversarial review are all required.
- **Never start code-quality before proposal-compliance is clean.**
- **Never start cross-artifact review before both inline stages are clean.**
- **Never start adversarial review before cross-artifact review is clean.**
- **Never let code outrun failing tests and guides.** Code written before them is discarded; restart in the required order.

## Model selection

- Use the harness's standard capable model for implementation and the two inline confirmatory reviews.
- Use the strongest available model distinct from the implementer for adversarial review. Cognitive diversity is the point.
- Escalate an implementer only for a reasoning shortfall, never for missing context.
- Do not use a low-reasoning model with untrusted recovery behavior for implementation or review.

## Skip dispatch entirely when

The change is so trivial that dispatch costs more than the work:

- Renaming a constant
- Fixing a one-line typo
- Adding a single missing test case to an existing test file
- Updating a comment

Do it inline. The hard tests → guides → code order still applies when production behavior changes.

## Related skills

- [`.agents/skills/writing-a-proposal/SKILL.md`](../writing-a-proposal/SKILL.md) — creates the approved proposal.
- [`.agents/skills/test-driven-development/SKILL.md`](../test-driven-development/SKILL.md) — controls the failing-test cycle.
- [`.agents/skills/writing-guides/SKILL.md`](../writing-guides/SKILL.md) — creates the independent guide.
- [`.agents/skills/systematic-debugging/SKILL.md`](../systematic-debugging/SKILL.md) — finds a bug's root cause before its fix is implemented.
