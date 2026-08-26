---
name: cross-artifact-review
description: Use when Review begins and a proposal, tests, guides, and code must be evaluated together. Do not use for a code-only or adversarial review.
---

# Cross-Artifact Review

**Phase 4, stage 1.** Find disagreement between representations of one change. This is not a code review. A clean implementation can still disagree with its proposal, tests, or guides.

## Inputs

Read the current proposal, changed tests, affected guides, and implementation together. Include the current vision, policies, and decisions when they constrain the change.

Do this after the project's quality gates. Do it before adversarial review.

## Method — behavior inventory

Build one row per behavior. The row label names the behavior; the four evidence columns are the four representations. Tick a column only when that source explicitly covers the behavior. Investigate every row that is not fully ticked.

| Behavior                 | Proposal | Tests | Guides | Code |
| ------------------------ | -------- | ----- | ------ | ---- |
| Reject malformed records | ✓        | ✓     | ✓      | ✓    |
| Preserve input order     | ✓        | ✓     | —      | ✓    |

Do not collapse related behaviors into one row. Boundary cases, errors, defaults, and observable side effects are separate behaviors when they can disagree.

## Required checks

For every inventory row, establish all six:

1. Tests cover the behavior the proposal promises.
2. Guides describe the same behavior the tests exercise.
3. Code implements what the proposal and guides describe.
4. No substantial documented behavior is missing from the implementation.
5. No substantial implemented behavior is absent from the proposal and guides — this catches scope creep.
6. Implementation details have not silently changed scope or intent.

## Resolve every gap

Every non-full row resolves in exactly one way:

| Resolution             | Use when                               | Action                                                                                |
| ---------------------- | -------------------------------------- | ------------------------------------------------------------------------------------- |
| Fix the wrong artifact | Intent has not changed                 | Correct tests, guides, or code; recheck the row.                                      |
| Revise the proposal    | Intent or scope has changed            | Discuss with the human first, then derive the affected tests, guides, and code again. |
| Record the divergence  | It remains deliberate for human review | Name the behavior, affected sources, impact, and reason in the readiness report.      |

You MAY edit tests, guides, and code to remove a disagreement. You MUST NOT change a proposal without the human — a proposal change is a design change.

## Output format

```
CROSS-ARTIFACT REVIEW — <proposal-id>

BEHAVIOR INVENTORY:
  <behavior> — proposal ✓/— | tests ✓/— | guides ✓/— | code ✓/—

GAPS:
  1. <behavior> — <sources that disagree> — <one resolution and action>

VERDICT: ALIGNED | DIVERGENCES RECORDED (n) | REVISION REQUIRED
```

List every non-full row under `GAPS`. `ALIGNED` requires a fully ticked inventory.

## Red flags

- You reviewed code quality instead of comparing representations.
- You ticked a source because it probably implies the behavior.
- You fixed code to match a guide while leaving the proposal contradictory.
- You accepted added implementation behavior because the tests pass.
- You changed a proposal without the human.
- You left a non-full row unexplained.

Any of these means repeat the inventory.

## Related skills

- `adversarial-review` — the independent, report-only stage that follows.
- `reporting-readiness` — records deliberate divergences for human review.
