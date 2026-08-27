# Agent Contract

This project uses the **Workbench** development process. [`PROCESS.md`](PROCESS.md) is the full
description; this file is the operative contract. Read it before doing anything substantial.

The governing principle: **no single artifact defines a feature by itself.** A proposal records
intent, tests encode behavior, guides explain behavior, code implements behavior. Each is written
independently so that disagreement between them is visible before release.

You do not move directly from a request to an implementation.

## Start here, every session

```sh
mise run status
```

It reads the branch, the pull request, and the active proposal's status, then prints which phase
the work is in and what happens next. Run it before asking the human where things stand, and run
it again whenever you are unsure.

The human is never expected to remember a command, a skill name, or an argument. "Keep going",
"start a feature for X", or a plain description of a problem is enough — orient yourself from
`mise run status` and proceed.

## Artifacts

| Artifact      | Lives in       | What it is                                                                     |
| ------------- | -------------- | ------------------------------------------------------------------------------ |
| **Vision**    | `VISION.md`    | The project's purpose, MVP target, principles, and direction. Living document. |
| **Policies**  | `policies/`    | Rules that future work must keep enforcing.                                    |
| **Decisions** | `decisions/`   | Architectural choices, with the context that produced them.                    |
| **Proposals** | `proposals/`   | The specification for one change. Preserved as history after it ships.         |
| **Guides**    | `guides/`      | Human-facing prose explaining how a feature behaves and how it is used.        |
| **Previews**  | CI             | The cheapest real artifact through which a human can exercise the change.      |
| **Tests**     | project-native | Executable specifications of the behavior a proposal describes.                |
| **Code**      | project-native | The implementation.                                                            |

Numbered artifacts use `<NNNN>-<slug>.md` and carry `id`, `status`, and `title` frontmatter.
`.agents/templates/` holds the canonical shape of each. `mise run validate` enforces it.

Every proposal is evaluated against the current vision, policies, and decisions. When they
disagree with what you are about to build, stop and raise it — do not quietly deviate.

## The feature loop

```
branch → draft PR → proposal → tests → guides → code → validation
       → cross-artifact review → adversarial review → preview → readiness report
       → human review → revision → vision update → merge → release → cleanup
```

Numbered phases below follow [`PROCESS.md`](PROCESS.md). Agent review is the end of
implementation, not a phase of its own.

**Scale the process to the change.** What follows is written for substantial work — new or
changed behavior, anything where a reasonable person could build it differently. Not every change
is that. A typo, a dead link, or a bug whose correct behavior was never in question does not need
a proposal, and often does not need a pull request either; fixing it is faster than describing it.

Use judgement, and lean on one question: **does this change decide something?** If intent already
exists — because a proposal promised the behavior, or because the contract is too obvious to write
down — you are correcting an implementation, not defining one. Say which you think it is and why,
in a line, before you start. The human can always ask for more process; give it without argument.

When you are genuinely unsure, ask. When you discover partway through a small fix that there was a
real design choice hiding in it, stop and treat it as substantial work from that point.

### 1. Preparation

Create a branch, push it to `origin`, and open a **draft** pull request before writing anything.
The PR is the durable workspace for the feature: it collects the proposal, the implementation,
review discussion, and status reports. It stays a draft until the agent believes the work is ready.

### 2. Proposal

Write `proposals/<NNNN>-<slug>.md` and **push it to the draft pull request as soon as it is
coherent enough to react to**. Do not polish it privately — the pull request is where the human
reads it, comments on it, and edits it directly.

Then iterate. The human requests changes, you revise, and this repeats for as long as it takes.
The proposal stays `status: draft` throughout. It must end up concrete enough to derive tests,
guides, and an implementation from **without** rereading the original conversation. Unresolved
questions are marked `[NEEDS CLARIFICATION: <question>]` and resolved with the human — never
guessed.

The human decides when the proposal is settled and moves it to `status: awaiting-implementation`.
That is the gate for phase 3. They may ask you to make the edit; deciding it yourself is
prohibited.

Skill: `.agents/skills/writing-a-proposal/`.

### 3. Implementation

Strictly ordered, and it ends with review. Each artifact is an independent representation of the
same intent; writing them out of order lets the code decide what "correct" means.

1. **Tests first.** Failing tests for the behavior the proposal's Detailed design specifies.
2. **Guides next.** User-facing docs written from the proposal, not from the code. Then compare
   them against the tests and confirm they describe the same behavior.
3. **Code last.** Implement against proposal, tests, and guides until `mise run check` passes.

**Which of these a change needs is a judgement.** The order is not — when you write two of them,
write them in this order. But a change with no user-visible surface needs no guide, and inventing
one produces documentation nobody reads describing behavior nobody sees. A change with nothing
meaningfully testable needs no test, though say so out loud rather than skipping quietly, because
"hard to test" and "not worth testing" are different claims and only one of them is a reason.

Ask what a reader or a future maintainer would actually need, not what the list contains.

Passing the quality gates is necessary, never sufficient. Then, still within this phase:

4. **Cross-artifact review** — read proposal, tests, guides, and code together and find where they
   disagree.
5. **Adversarial review** — an independent subagent, **fresh context and a different model**, tries
   to falsify the claim that the work is done. It reports; it does not fix. Findings you decline
   are surfaced in the readiness report, never silently dropped.
   Skill: `.agents/skills/adversarial-review/`.
6. **Preview** — publish the cheapest real artifact through which the human can exercise the
   changed behavior. Review of prose is not review of behavior.
7. **Readiness report** — post it to the PR linking the preview, set the proposal to
   `active-review`, and mark the PR ready for review. Those three are one action.

Skills: `.agents/skills/implementing-a-proposal/` for steps 1–3,
`.agents/skills/reviewing-an-implementation/` for steps 4, 6, and 7.

A preview is keyed to what the repository _is_, not to a fixed mechanism:

| Repository | Preview                                     |
| ---------- | ------------------------------------------- |
| Library    | prerelease package                          |
| Web app    | preview deployment                          |
| API        | preview endpoint deployment                 |
| Docs       | deployed documentation site                 |
| CLI        | installable executable                      |
| Desktop    | installable build                           |
| Mobile     | installable build (ad-hoc / internal track) |

### 4. Human review

The human reviews the report, the diff, and the preview. They comment on the PR; they may also
commit directly. When feedback changes the intended design rather than correcting its
implementation, **update the proposal first**, then re-run phase 3 against it.

Their verdict sets the status: `returned-for-revisions` to send it back, `accepted` to take it.

### 5. Completion and release

Update `VISION.md` if the feature changed what the project is, merge, release, update dependent
projects, then clean up. Skill: `.agents/skills/completing-a-feature/`.

## Hard rules

- **Never implement substantial work without an approved proposal.** A conversational request is
  not a proposal. Corrections and fixes with unambiguous intent are not substantial work.
- **Never write production code before its failing test.** If the behavior warrants a test at all
  and you wrote the code first, delete it and start over.
- **Never write guides from the finished code.** When a change warrants a guide, it is derived
  from the proposal. Whether it warrants one is a judgement; the direction of derivation is not.
- **Never claim work is done without running the verifying command this turn** and reading its
  output. See [`.agents/rules/verification.md`](.agents/rules/verification.md).
- **Never fix a bug before finding its root cause.**
  Skill: `.agents/skills/systematic-debugging/`.
- **Never let the adversarial reviewer edit code.** It reports; the implementer fixes.
- **Never delete a branch, force-push, or publish a release without explicit human confirmation.**
- **Never expand scope.** Behavior not in the proposal does not get implemented. If it should
  exist, revise the proposal. A defect you discovered is not a licence to widen the work.
- **Never file an issue to defer work the proposal already requires.** Fix it.

## Policies and decisions

When a feature establishes a rule future work must keep enforcing, record a **policy**. When it
establishes an architectural choice future work must understand, record a **decision**. Discuss
both with the human before writing them; neither is a substitute for a proposal.

**Most features establish neither, and that is the normal case.** The test is whether the
knowledge outlives the change: if it does not, it belongs in the proposal and nowhere else. Both
directories being empty is a healthy state, not a gap to fill. A record written speculatively is
one nobody follows and everybody has to read.

A policy that can be enforced by a check should be — see
[`.agents/rules/enforcement-hierarchy.md`](.agents/rules/enforcement-hierarchy.md). Prose is the
tier most likely to be missed.

Skill: `.agents/skills/completing-a-feature/`.

## Defects

You will find defects that are not this proposal's problem — pre-existing on the default branch,
belonging to another feature, or simply outside scope. Those are recorded as GitHub issues so they
outlive the pull request.

A defect that stops the current implementation from satisfying its proposal is **not** one of
these. It is the current work, and it gets fixed.

Search open and closed issues before filing. Write the issue so a reader who was never in this
conversation can reproduce it. Link it from the PR when it was deliberately deferred, so the
decision is visible. Close it if later work fixes it incidentally.

Skill: `.agents/skills/tracking-defects/`.

## Quality gates

```sh
mise run check       # everything below, in order
mise run fmt         # format
mise run validate    # artifact frontmatter, IDs, and cross-references
mise run docs:build  # the guides site builds
```

Run project-specific gates (type check, unit, integration, end-to-end) as they exist. `mise tasks`
lists everything available.

## Also binding

- [`.agents/rules/code-quality.md`](.agents/rules/code-quality.md) — what good code looks like here.
- [`.agents/rules/commit-discipline.md`](.agents/rules/commit-discipline.md) — commits, staging, push
  and PR policy.
- [`.agents/rules/enforcement-hierarchy.md`](.agents/rules/enforcement-hierarchy.md) — where a new
  rule belongs.

## Harness setup

This repo ships harness-neutral infrastructure in `.agents/`. If your coding harness expects a
different location or filename, adapt it once — see [`guides/adapting-your-harness.md`](guides/adapting-your-harness.md) — and commit the
result. `.agents/` stays the source of truth.
