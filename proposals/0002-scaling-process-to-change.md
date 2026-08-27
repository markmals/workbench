---
id: proposal.0002
title: Scaling Process to the Change
authors: [markmals, Claude]
status: draft
pull-request: https://github.com/markmals/workbench/pull/6
issues: []
supersedes: []
---

# Scaling Process to the Change

## Summary

The process has one setting, and it is sized for a feature. Every change pays for a proposal or
smuggles itself past the rule. This adds three tiers with an objective test for choosing between
them, and a one-way escalation rule so the cheapest tier cannot be chosen for convenience.

## Motivation

`AGENTS.md` says _never implement without an approved proposal_. Taken literally, fixing a typo in
a guide requires a proposal. Nobody does that, so the rule is broken constantly, quietly, by
judgement — which is the worst state for a rule to be in. It cannot be enforced, and it gives no
guidance at the moment it is needed.

Four changes landed today. Weighed against the rule:

| Change                        | Size                                     | What happened                               | Correct?     |
| ----------------------------- | ---------------------------------------- | ------------------------------------------- | ------------ |
| The guides                    | 6 files, 857 lines, a new artifact class | Full proposal                               | Yes          |
| Preview teardown              | 1 workflow, ~20 lines                    | Proposal drafted, then dropped as too heavy | No           |
| Favicon base prefix           | 1 line                                   | No ceremony                                 | Yes, by luck |
| Restore a workflow permission | 1 line                                   | No ceremony                                 | Yes, by luck |

Two of four were right by accident. The teardown fix produced a full proposal for a bug whose
intended behavior was never in question, and the only thing that stopped it shipping was you
saying it felt wrong. A process that depends on the human noticing ceremony is not a process.

The deeper problem is that _size_ is the wrong axis. The teardown fix was twenty lines and had a
genuinely non-obvious root cause; the guides were 857 lines of prose with no design choice left to
make once the set was agreed. What actually distinguishes them is whether **intent already
exists**. A bug has its intent specified already — by the proposal that promised the behavior, or
by a contract too obvious to write down. "Teardown should be final" was never in doubt. "Workbench
should have guides" was a decision someone had to make.

That is the axis, and it is objective enough to test against.

## Proposed solution

Three tiers. The test is not size, effort, or confidence — it is whether the change decides
something.

**Tier 1 — Correction.** No behavior change at all. Typos, dead links, formatting, comments,
renaming a local. Commit it. No branch, no pull request, no record. If reading the diff is faster
than reading a description of it, describing it is waste.

**Tier 2 — Fix.** Behavior is wrong and the correct behavior is not in question. Branch, pull
request, root cause, fix, and a test that would have caught it. **No proposal.** The pull request
body carries the root cause; that is the durable record, and it is enough because there was no
decision to preserve.

**Tier 3 — Change.** New behavior, changed behavior, or a real choice between approaches. The full
five phases, unchanged from today.

## Detailed design

**The test, applied in order.** A change is Tier 3 if _any_ of these is true:

1. A user could notice behavior that was never promised.
2. There is more than one defensible approach and the choice has consequences.
3. It establishes a policy or a decision.
4. It changes the vision, or contradicts an existing policy or decision.

Otherwise it is Tier 2 if it changes behavior at all, and Tier 1 if it does not.

Question 2 is the load-bearing one and the one to be honest about. "I can see how to do this" is
not the same as "there is only one way to do this." The teardown fix passes: given the root cause,
guarding on live state is the only approach that closes the race, and the two alternatives were
demonstrably broken rather than merely worse.

**Escalation is one-way and automatic.** Discovering mid-fix that a design choice exists means the
change was Tier 3 all along. Stop, write the proposal, and do not finish the fix first. Nothing
ever moves down a tier — not when the fix turns out smaller than expected, not when a proposal
feels like overkill in hindsight.

**The human can escalate anything, at any time, without justifying it.** They cannot be asked to
justify wanting a proposal.

**The agent proposes the tier; it never simply assumes one.** State the tier and the reason before
starting, in one line. Tier 1 needs no announcement — announcing a typo fix costs more than the
fix.

This is the same authority split the status lifecycle already uses: the agent may do the work, but
it does not get to rule on whether its own work needs review. An agent that silently selects Tier 2
for a change with a design choice in it has approved its own scope, which is the failure this
repository has already produced twice.

**`mise run status` reports the tier** once a branch has a pull request, reading it from a
`tier:` line in the pull request body. Absent, it says so rather than guessing.

**Where this lives.** `AGENTS.md` gains a Tiers section and its hard rule changes from _never
implement without an approved proposal_ to _never implement a Tier 3 change without an approved
proposal_. `PROCESS.md` gains a section describing tiers as the process's own scaling mechanism.
`guides/running-a-feature.md` gains the reader-facing version. The skills state which tier they
apply to.

## Compatibility

Existing behavior is a strict subset: everything is Tier 3 today, and Tier 3 is unchanged. Nothing
in flight is affected.

## Implications on adoption

Adopters get a process that no longer demands a proposal for a broken link. The risk is tier
inflation downward — an agent choosing Tier 2 because it is cheaper. The escalation rule and the
requirement to state the tier out loud are the mitigations, and they are prose, which the
enforcement hierarchy says is the weakest tier. Whether this can become a check is an open
question below.

## Scope

- The Tiers section in `AGENTS.md`, and the amended hard rule.
- A tiers section in `PROCESS.md`.
- Tier applicability stated in each skill.
- The reader-facing version in `guides/running-a-feature.md`.
- `tier:` reporting in `mise run status`, with tests.

### Out of scope

- **Retroactively tiering the existing record.** Proposal 0001 was Tier 3 and stays a proposal.
- **A fourth tier for releases.** Releasing is a phase, not a change.

## Preview

The deployed documentation site, showing `guides/running-a-feature.md` with the tier guidance a
reader would actually use to decide. The rest of the change is agent-facing prose and a status
line, neither of which a preview improves on.

## Policies and decisions checked

`policies/` and `decisions/` are both empty; both directories were read. `VISION.md` was read.

One vision principle governs this directly: _ceremony must earn its cost. Every step exists because
skipping it produced a specific, recurring failure. A step that cannot be justified that way gets
deleted._ A proposal for a bug fix cannot be justified that way, so this proposal is the vision
being applied rather than amended.

This is a strong candidate for the first **policy**: _state the tier before starting, and never
move a change down a tier._ Recording it is deferred to completion, when there is evidence the
tiers survive contact.

## Future directions

- A check that fails a pull request whose body declares Tier 2 while touching `VISION.md`,
  `policies/`, or `decisions/` — a cheap mechanical catch for the most obvious kind of tier abuse.
- Recording the tier in the record, so it becomes possible to ask later whether Tier 2 changes
  produce more defects than Tier 3 ones.

## Alternatives considered

**Size thresholds — lines changed, files touched.** Objective and checkable, which is genuinely
attractive. Rejected because it is measuring the wrong thing, and today proves it: a twenty-line
workflow fix with a subtle root cause against 857 lines of prose with no decision in it. A
threshold would have got both backwards.

**Leave the rule and rely on judgement.** This is the status quo, and it produced two correct
outcomes out of four by luck. It also means the written rule and the practised rule differ, which
teaches that the written rules are approximate — the most expensive thing a process can teach.

**A lightweight proposal template for small changes.** Keeps one path with less filling-in.
Rejected because the cost is not the template, it is the approval round trip: writing down what a
typo fix intends, pushing it, waiting for a human to move it to `awaiting-implementation`. Halving
the ceremony still leaves ceremony for changes that warrant none.

**Let the human set the tier every time.** Safest against agent tier inflation. Rejected because it
puts a decision on the human for every change, including the typos — which is the burden this is
trying to remove. The agent proposing and the human overriding gets the same protection at a
fraction of the cost.

## Open questions

- [NEEDS CLARIFICATION: Three tiers, or two? Tier 1 and Tier 2 differ only in whether a pull
  request exists. If you would rather every change go through a pull request — which makes the
  record uniform and costs little given the branch already exists — Tier 1 collapses into Tier 2
  and the model gets simpler.]
- [NEEDS CLARIFICATION: Should the tier be declared in the pull request body, where `mise run
status` can read it and a check could enforce it, or is a stated tier in conversation enough?
  The former is enforceable and the latter is frictionless.]
