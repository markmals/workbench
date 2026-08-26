---
name: test-driven-development
description: Use when writing production code for a proposal, bug fix, refactor, or behavior change. Do not use for generated code, configuration, or throwaway work that will be deleted.
---

# Test-Driven Development

Write the test first. Watch it fail. Write the minimal code to pass it. Refactor while green.

**Core principle:** if you didn't watch the test fail, you don't know if it tests the right thing.

## The Iron Law

```
NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST
```

If you wrote code before the test, **delete it** and start over. Don't keep it as "reference". Don't "adapt it" while writing the test. Delete means delete.

## When to use

**Always:**

- New behavior from an approved proposal
- Bug fixes — write a failing regression test first
- Behavior changes
- Refactors that change observable behavior

**Exceptions (ask first):**

- Generated code
- Configuration files
- Throwaway prototypes that will be deleted, not kept

If you're thinking "skip TDD just this once" — stop. That's rationalization.

Tests name the proposal id and the acceptance criterion they verify, in whatever form the project's test framework makes idiomatic.

## Red → Green → Refactor

```
1. RED      Write one failing test for one behavior.
2. Verify   Run the test. Confirm it fails — for the right reason.
3. GREEN    Write the minimal code to make it pass.
4. Verify   Run the test. Confirm it passes. Confirm other tests still pass.
5. Refactor Clean up while staying green.
6. Repeat   Next failing test for next behavior.
```

### RED — write one failing test

- One behavior per test.
- Name the observed result, not an implementation detail.
- Test real code; mock only what you can't control (network, time, randomness).
- A specific scenario is an example test. An acceptance criterion that says "always", "never", "for any", or "for all" is an invariant; its failing test is a property-based test. Do not settle for examples when the proposal is universal.

### Verify RED — watch it fail

**Mandatory. Never skip.** Run the test command. Confirm:

- It **fails**, not errors out (failing assertion ≠ undefined symbol).
- The failure message matches what you expect.
- It fails because the **behavior is missing**, not because of a typo or import error.

If the test passes immediately, you're testing existing behavior — fix the test.
If the test errors, fix the error and re-run until it fails for the right reason.

### GREEN — minimal code

Write the simplest code that makes the failing test pass.

- No defensive `if` for cases not yet tested.
- No options objects, configuration, or hooks for future flexibility (YAGNI).
- No "while I'm here" refactors of unrelated code.

### Verify GREEN — watch it pass

Run the test. Confirm:

- It passes.
- All other tests in the suite still pass.
- Output is pristine — no warnings, no leaked logs, no "test passed but skipped".

### Refactor

Now that you're green, clean up. Remove duplication, improve names, extract helpers. Keep tests green throughout — every change should leave the suite passing. Do not add new behavior.

## Invariants get a property, not just examples

When a proposal has an acceptance criterion phrased as an invariant — something that must hold for **all** valid inputs, not just the ones you happened to think of — the test for it **is a property-based test**. Hand-picked examples alone are insufficient: they prove the cases you imagined. [`.agents/skills/adversarial-review/SKILL.md`](../adversarial-review/SKILL.md) will flag an invariant covered only by examples because it hunts exactly the inputs your examples skipped.

The trigger is concrete: any acceptance criterion using "always", "never", "for any", or "for all". Each such invariant earns a property:

- "Name is non-empty after trimming" → for all strings blank after trimming, validation rejects.
- Round-trip → for all valid values, `decode(encode(value))` equals `value`.
- Idempotence → for all inputs, `normalize(normalize(value))` equals `normalize(value)`.

Discipline that still applies:

- **Watch it fail first.** A property test is a RED test like any other — run it against the missing or wrong behavior and confirm it fails for the right reason before you make it pass.
- **Keep the example tests too.** They document specific behavior clearly. The property is the safety net underneath them, not a replacement for example tests.
- **Do not force a property where there isn't one.** A single mapping, a constant, or formatting is example-shaped. Properties are for universally quantified invariants, not for everything.

## Why order matters

> "I'll write tests after to verify it works."

Tests written after the implementation **pass immediately**. That proves nothing:

- Might test the wrong thing
- Might test what you implemented, not what's required
- Might miss edge cases you forgot
- You never saw it catch a bug

Test-first forces you to **see the test fail**, which proves the test actually tests something.

## Common rationalizations

| Excuse                            | Reality                                                                                             |
| --------------------------------- | --------------------------------------------------------------------------------------------------- |
| "Too simple to test"              | Simple code still breaks. The test takes 30 seconds.                                                |
| "I'll test after"                 | Tests passing immediately prove nothing.                                                            |
| "Already manually tested"         | Manual ≠ automated. No record, can't re-run.                                                        |
| "Deleting work is wasteful"       | Sunk cost. Keeping unverified code is technical debt.                                               |
| "Need to explore first"           | Fine — throw the exploration away, then start with TDD.                                             |
| "Test hard = design unclear"      | Listen to the test. Hard to test means hard to use.                                                 |
| "My examples cover the invariant" | Examples cover the inputs you thought of. "For all" means the ones you didn't — write the property. |

## Red flags — stop and start over

- Code before the test
- Test added "after we ship"
- Test passes immediately
- Can't explain why the test failed
- "Just this once"
- "Keep as reference, write tests, then adapt"

All of these mean: **delete the code, start with TDD**.

## When stuck

| Problem                | Solution                                             |
| ---------------------- | ---------------------------------------------------- |
| Don't know how to test | Write the wished-for API. Write the assertion first. |
| Test too complicated   | Design too complicated. Simplify the interface.      |
| Must mock everything   | Code too coupled. Use dependency injection.          |
| Test setup huge        | Extract helpers. Still big? Simplify the design.     |

## Verification checklist

Before marking work complete:

- [ ] Every behavior has at least one test
- [ ] Every stated invariant ("always" / "never" / "for any" / "for all") has a property-based test, not just examples
- [ ] Watched each test fail before implementing
- [ ] Each test failed for the expected reason
- [ ] Wrote minimal code to pass each test
- [ ] All tests pass
- [ ] Output pristine (no warnings, no leaked logs)
- [ ] Tests use real code (mocks only when unavoidable)
- [ ] Tests idiomatically name the proposal and acceptance criterion they verify

Can't tick every box? You skipped TDD. Start over.

## Commit

Each green-refactor cycle is a natural commit boundary. Once the suite is green and the refactor is clean, follow [`.agents/rules/commit-discipline.md`](../../rules/commit-discipline.md).

Do **not** commit while red. A WIP commit between red and green is the wrong answer — finish the cycle, then commit.

## Related skills

- [`.agents/skills/implementing-a-proposal/SKILL.md`](../implementing-a-proposal/SKILL.md) — the workflow that invokes this skill.
- [`.agents/skills/adversarial-review/SKILL.md`](../adversarial-review/SKILL.md) — challenges weak tests and untested invariants.
- [`.agents/skills/verification-before-completion/SKILL.md`](../verification-before-completion/SKILL.md) — the gate before claiming the proposal is done.
- [`.agents/skills/systematic-debugging/SKILL.md`](../systematic-debugging/SKILL.md) — for a failure whose cause is not yet understood.
