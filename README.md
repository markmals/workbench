# Workbench

![Workbench](./docs/public/workbench-hero.png)

A development process for working with coding agents, packaged as a repository you can clone.

Workbench is built on one idea: **no single artifact defines a feature by itself.** A proposal
records intent, tests encode behavior, guides explain behavior, code implements behavior, and a
preview lets a human exercise it. Each is written independently, so when they disagree, you find
out before release instead of after.

It assumes nothing about your language, framework, platform, or coding harness.

## Why

An agent that goes straight from a request to an implementation produces code nobody specified,
tests that agree with whatever got written, and documentation reverse-engineered from the result.
Everything corroborates everything else, and none of it is evidence.

The fix is not more review. It is making the agent commit to what it intends to build, in a form
that can be checked against what it actually built.

## Quick start

1. Click **Use this template** on GitHub, then clone your repository.
2. Ask your agent to adopt it, pointing it at [`guides/adapting-your-harness.md`](guides/adapting-your-harness.md). That wires Workbench
   into whichever harness you use — a one-time change you commit and forget.
3. Write your project's vision with your agent, from
   [`.agents/templates/VISION.md`](.agents/templates/VISION.md).
4. Start your first feature. Say what you want built. Your agent branches, opens a draft pull
   request, and writes a proposal before it writes any code.

```sh
mise install
mise run check
```

## You never have to remember a command

There is one, and it takes no arguments:

```sh
mise run status
```

It reads the branch, the pull request, and the proposal's status, then tells you where the work
is and what happens next:

```
proposal.0001  Workbench 2.0  ·  draft
PR #1 (draft)  https://github.com/markmals/workbench/pull/1
branch workbench-2

Phase 2 — Proposal
  The proposal is published and being iterated on.

Next (you): Edit the proposal directly, or tell the agent what to change.
            Move it to awaiting-implementation when you are satisfied.
```

Your agent runs it too, at the start of every session. So "keep going" is a complete instruction —
it can work out the rest. Skills exist for the agent to read; you are not expected to know their
names.

## The process

Per feature:

```
branch → draft PR → proposal → tests → guides → code → validation
       → cross-artifact review → adversarial review → preview → readiness report
       → human review → revision → vision update → merge → release → cleanup
```

**Preparation.** The agent branches, pushes, and opens a draft pull request. The pull request is
the durable workspace — proposal, implementation, review discussion, and status reports all
accumulate in one reviewable place.

**Proposal.** You and the agent write `proposals/<NNNN>-<slug>.md` and iterate until it is right.
It has to be concrete enough that tests, guides, and an implementation can be derived from it
without rereading your conversation. Unresolved questions are marked and answered, never guessed.

**Implementation.** Failing tests first, then guides written from the proposal, then code. The
order matters: tests written after the code agree with the code, and guides written after the
implementation document its accidents.

**Review.** A cross-artifact pass looks for disagreement between proposal, tests, guides, and
code. Then an adversarial reviewer — fresh context, different model — tries to prove the work is
broken. It reports; it does not fix. Findings the primary agent declines are surfaced in the
readiness report rather than quietly dropped. Then the preview goes up, and the agent posts a
readiness report and marks the pull request ready.

**Human review.** You read the report, the diff, and the preview, and comment. If your feedback
changes the design rather than correcting the implementation, the proposal is updated first and
the loop runs again. When you accept it: vision update, merge, release, cleanup.

The full description is in [`PROCESS.md`](PROCESS.md). The operative contract your agent reads
every session is [`AGENTS.md`](AGENTS.md).

## Previews

A preview is the cheapest realistic artifact through which you can exercise a change. _Cheapest_
rules out standing up production infrastructure to review a pull request; _realistic_ rules out
screenshots and descriptions of what would happen.

| Project           | Preview                                            |
| ----------------- | -------------------------------------------------- |
| Library           | prerelease package                                 |
| Web application   | preview deployment                                 |
| API               | preview endpoint deployment                        |
| Documentation     | rendered documentation build                       |
| Command-line tool | installable executable                             |
| Desktop           | installable build                                  |
| Mobile            | installable build, ad-hoc or internal distribution |

Wire the row matching your project and delete the rest. Previews are private by default.

## Defects

Defects an agent finds along the way get filed as issues, not folded into whatever it happens to
be building. A defect that stops the current work from satisfying its proposal is the current
work and gets fixed; a defect that is out of scope, pre-existing, or someone else's problem
becomes a GitHub issue written so a future reader can reproduce it without the conversation it
came from. Deferred defects are linked from the pull request, so the decision to defer is visible.

## What's in here

| Path                 | What it is                                                    |
| -------------------- | ------------------------------------------------------------- |
| `AGENTS.md`          | The contract every agent session loads.                       |
| `PROCESS.md`         | The full process description.                                 |
| `guides/`            | User-facing documentation. Start at `getting-started.md`.     |
| `VISION.md`          | The project's purpose and direction. Replace ours with yours. |
| `proposals/`         | One document per change, kept as history after it ships.      |
| `policies/`          | Rules future work must keep enforcing.                        |
| `decisions/`         | Architectural choices and the context that produced them.     |
| `guides/`            | User-facing documentation.                                    |
| `docs/`              | The site. Publishes the vision, the record, and the guides.   |
| `tools/`             | The record validator. Dependency-free.                        |
| `.agents/skills/`    | Seven skills, roughly one per phase. For the agent, not you.  |
| `.agents/rules/`     | Code quality, commit discipline, enforcement hierarchy.       |
| `.agents/templates/` | The canonical shape of every artifact.                        |
| `.github/workflows/` | Continuous integration and the preview.                       |

## Enforcement

Prose is the tier most likely to be missed, so anything checkable is a check.

```sh
mise run check       # everything below
mise run fmt         # format
mise run validate    # record frontmatter, ids, cross-references
mise run test        # the tools' own tests
mise run docs:build  # the site builds
```

`mise run validate` enforces that identifiers match filenames and are unique, that every
`supersedes` / `established-by` / `describes` reference resolves, that superseding an entry marks
the superseded one, and that no accepted proposal still carries an unresolved question. A
`commit-msg` git hook enforces commit scoping. Continuous integration runs all of it, so no gate
depends on the agent remembering.

## Harness support

`AGENTS.md` is the contract and `.agents/` is the infrastructure. Both are harness-neutral, which
means no harness reads them natively without a small one-time adaptation — see
[`guides/adapting-your-harness.md`](guides/adapting-your-harness.md). This is deliberate: shipping configuration for four harnesses when
only one can be genuinely verified is worse than an honest adaptation step.

The checks that matter most — the validator, continuous integration, and the commit hook — do not
depend on the harness noticing anything.

## Coming from Workbench 1.0

1.0 was a spec-driven multiplatform application template: platform skills, a toolchain catalog,
reverse pointers, drift detection, and a `/sdd-*` command surface. It is preserved at the `v1.0`
tag and remains usable.

There is no upgrade path. The two versions share almost nothing — 2.0 deletes the stack and keeps
the process. If you want the multiplatform apparatus, use the tag.

## Read next

- [`PROCESS.md`](PROCESS.md) — the process in full.
- [`AGENTS.md`](AGENTS.md) — what your agent is actually bound by.
- [`proposals/0001-workbench-2.md`](proposals/0001-workbench-2.md) — this rewrite, proposed through
  its own process. The most useful thing to read if you want to know what a good proposal looks
  like.
