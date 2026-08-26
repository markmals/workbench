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
       → cross-artifact review → adversarial review → readiness report
       → human review → revision → vision update → merge → release → cleanup
```

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

Strictly ordered. Each step is an independent representation of the same intent, and writing them
out of order lets the code decide what "correct" means.

1. **Tests first.** Failing tests for the behavior the proposal's Detailed design specifies.
2. **Guides next.** User-facing docs written from the proposal, not from the code. Then compare
   them against the tests and confirm they describe the same behavior.
3. **Code last.** Implement against proposal, tests, and guides until `mise run check` passes.

Skill: `.agents/skills/implementing-a-proposal/`.

Passing the quality gates is necessary, never sufficient.

### 4. Review

1. **Cross-artifact review** — read proposal, tests, guides, and code together and find where they
   disagree.
2. **Adversarial review** — an independent subagent, **fresh context and a different model**, tries
   to falsify the claim that the work is done. It reports; it does not fix. Findings you decline
   are surfaced in the readiness report, never silently dropped.
   Skill: `.agents/skills/adversarial-review/`.
3. **Preview** — publish the cheapest real artifact through which the human can exercise the
   changed behavior. Review of prose is not review of behavior.
4. **Readiness report** — post it to the PR linking the preview, set the proposal to
   `active-review`, and mark the PR ready. Those three happen together.

Skill for stages 1, 3, and 4: `.agents/skills/reviewing-an-implementation/`.

A preview is keyed to what the repository _is_, not to a fixed mechanism:

| Repository | Preview                                     |
| ---------- | ------------------------------------------- |
| Library    | prerelease package                          |
| Web app    | preview deployment                          |
| API        | preview endpoint deployment                 |
| Docs       | rendered documentation build                |
| CLI        | installable executable                      |
| Desktop    | installable build                           |
| Mobile     | installable build (ad-hoc / internal track) |

### 5. Human review and completion

The human reviews the report, the diff, and the preview. They comment on the PR; they may also
commit directly. When feedback changes the intended design rather than correcting its
implementation, **update the proposal first**, then re-run the implementation loop against it.

Once accepted: update `VISION.md` if the feature changed what the project is, merge, release, then
clean up. Skill: `.agents/skills/completing-a-feature/`.

## Hard rules

- **Never implement without an approved proposal.** A conversational request is not a proposal.
- **Never write production code before its failing test.** If you did, delete it and start over.
- **Never write guides from the finished code.** They are derived from the proposal.
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

A policy that can be enforced by a check **must** be — see
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
different location or filename, adapt it once — see [`ADOPTING.md`](ADOPTING.md) — and commit the
result. `.agents/` stays the source of truth.
