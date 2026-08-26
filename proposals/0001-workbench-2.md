---
id: proposal.0001
title: Workbench 2.0
status: accepted
pull-request:
supersedes: []
---

# Workbench 2.0

## Motivation

Workbench 1.0 was a spec-driven multiplatform application template. It carried nine platform
skills, four device-driver skills, eleven per-platform task scaffolds, a 42 KB toolchain catalog,
reverse-pointer and drift-detection machinery, and a `/sdd-*` command surface — all in service of
fanning one specification out across web, iOS, Android, Windows, Linux, a CLI, and a Convex
backend.

That answered a question about _how to build the same product several times_. The question that
actually matters is _how to keep an agent honest about any one change_, and 1.0 buried it. The
process was the small part of a large repository about stacks.

Three specific failures made this worth rewriting rather than extending:

**The process had no durable workspace.** Work happened on whatever branch was checked out. Three
separate documents stated the repository did not use feature branches, and commit discipline
prohibited pushing. There was nowhere for a proposal, its implementation, review discussion, and a
status report to accumulate as one reviewable unit.

**Documentation was not an artifact.** The repository rendered its own internal specifications and
produced nothing a user would read. Behavior therefore had two representations — specification and
code — where the process depends on having more than two, written independently, so they can
disagree.

**Nothing was verifiable outside the agent's own session.** There was no continuous integration,
no release path, and no preview. Every quality claim rested on the agent reporting that it had run
something.

## Intended behavior

Workbench becomes a platform-agnostic, harness-agnostic process kit. Cloning it gives a project
the structure described in `PROCESS.md` and nothing about a technology stack.

**Artifacts.** A living `VISION.md`; `proposals/`, `policies/`, `decisions/`, and `guides/` as
numbered or slugged markdown with validated frontmatter; tests and code in whatever form the
project uses; and a preview produced by continuous integration.

**Process.** Five phases — preparation, proposal, implementation, review, completion — encoded as
skills under `.agents/skills/`, one per phase, each invocable by any competent coding agent.
Implementation is strictly ordered: failing tests, then guides written from the proposal, then
code. Review is cross-artifact, then adversarial on a different model, then a preview, then a
readiness report posted to the pull request.

**Enforcement.** A dependency-free validator in `tools/` checks what a machine can check about the
record: frontmatter shape, identifier uniqueness, reference resolution, supersession pairing, and
that no accepted proposal still contains an unresolved question marker. It runs in continuous
integration and locally through `mise run check`. A `commit-msg` git hook enforces commit scoping
independently of any harness.

**Publication.** The existing VitePress site is retained and retargeted: it publishes `guides/`
and nothing else. The design record is read in the repository, not on the web.

**Harness neutrality.** Everything ships in `.agents/` with `AGENTS.md` as the single contract.
Adapting it for a specific harness is a documented one-time step in `ADOPTING.md`, not a fork.

## Acceptance criteria

1. A fresh clone contains no reference to a platform, stack, framework, or device — no `apps/`,
   no `services/`, no reverse pointers, no drift detection, no `/sdd-*` commands.
2. `AGENTS.md` states the five phases, the artifact set, and the hard rules, and every skill it
   names exists at the path given.
3. `mise run check` runs formatting, record validation, the validator's own tests, and the guides
   build, and passes on a clean checkout.
4. The validator rejects each of its five rule violations and accepts a valid record, proven by
   tests that run without network or third-party dependencies.
5. The guides site builds and publishes `guides/` only, and does not fail when `guides/` is empty.
6. Continuous integration runs the same gates as `mise run check` plus commit-message validation
   over the pull request's commits.
7. A preview workflow ships wired for this repository's own shape, with recipes for the other
   shapes present but inert.
8. No surviving document instructs the agent not to use a branch, not to push a feature branch, or
   not to open a pull request.

## Scope

The rewrite covers the repository's own contents: agent contract, process description, skills,
rules, templates, record directories, validator, task definitions, continuous integration, preview
workflow, docs site, and human-facing README.

### Out of scope

- **Slash commands.** 1.0's `/sdd-*` surface is deleted and not replaced. Commands are
  harness-specific, and the kit is harness-neutral; the skills carry the procedure instead.
- **Dependent-project automation.** `PROCESS.md` describes updating dependent repositories' draft
  pull requests when a preview becomes a release. The skill documents the procedure; no tooling
  ships for it. Building that before a second repository exists would be speculative.
- **Migration tooling for 1.0 users.** The `v1.0` tag remains usable. There is no upgrade path
  between the two shapes because almost nothing is shared.
- **Preview mechanisms other than this repository's own.** Recipes ship as comments. Wiring and
  testing all seven would mean maintaining six workflows nobody here runs.

Defects discovered while writing this proposal that fall outside it are filed as issues rather
than folded in.

## Design decisions and rejected alternatives

**A process kit, not an application template.** _Rejected: keeping the multiplatform apparatus as
an optional layer._ `PROCESS.md` mentions no platform anywhere. Retaining a second, undocumented
half would mean maintaining the thing that buried the process in the first place. _Rejected:
splitting into two repositories._ The `v1.0` tag preserves the multiplatform work at zero
maintenance cost; a live second repository would not.

**`AGENTS.md` plus a neutral `.agents/` tree, rather than harness-specific configuration.**
_Rejected: shipping configuration for all four harnesses named in `PROCESS.md`._ Only one can be
genuinely verified here, and three untested configurations are worse than an honest adaptation
step. The cost is losing automatic lifecycle enforcement out of the box, which is why the
`commit-msg` hook and continuous integration were made harness-independent — the checks that
matter most do not depend on the harness noticing.

**One proposal document, replacing the seven-file feature folder.** 1.0 required a narrative,
stories, use-cases, a user flow, domain models, view models, and error definitions per feature.
That taxonomy served cross-platform fan-out. With one implementation, it is overhead that makes
small changes expensive enough to skip the process entirely.

**The site publishes the record as well as the guides.** Navigation is vision, decisions,
policies, proposals, guides. _Rejected: publishing guides alone._ The record is the part of a
project that is hardest to reconstruct later and hardest to navigate as raw files in a directory
— cross-references between proposals, policies and decisions become links, superseded entries
stay reachable without cluttering the index, and the vision sits where anyone can find it. A
project whose guides must be public and whose record must not can exclude the record directories
in its own configuration; that is a narrower problem than making the record unreadable for
everyone by default.

**A hand-written frontmatter reader rather than a YAML dependency.** The frontmatter is flat
scalars and inline lists. A parser that rejects what it does not understand is around thirty lines
and keeps the validator dependency-free, which keeps it runnable in any environment with Node.

**Previews keyed to repository shape.** _Rejected: a single fixed mechanism._ The purpose is that
a human can exercise the change. What makes that cheapest and realistic differs completely between
a library and a mobile application, so the kit specifies the property and provides the menu.

## Preview

Workbench is a documentation and process repository, so its preview is the rendered guides build:
`.github/workflows/preview.yml` runs `mise run docs:build` on every pull request and uploads the
site as a workflow artifact. Access is gated by repository permissions, which is the privacy
model.

This is the cheapest realistic artifact for this repository. A deployment would cost hosting and
an access-control decision to expose the same static output; a screenshot would not let a reviewer
navigate the site.

## Policies and decisions checked

`policies/` and `decisions/` are both empty at the time of writing — this is the first proposal in
the repository, and 1.0 recorded neither as a first-class artifact. Nothing to check.

Two constraints were nonetheless inherited from 1.0 and deliberately preserved:

- `.agents/rules/code-quality.md` — retained essentially unchanged; the validator's separation of
  pure logic from I/O is written to satisfy it.
- `.agents/rules/enforcement-hierarchy.md` — retained and retiered. This proposal's decision to
  put record invariants in a validator rather than in prose is an application of it.

Neither is promoted to `policies/` here. They govern this repository rather than emerging from a
feature, and speculative policies are the ones nobody follows.

## Open questions

None. Scope, harness strategy, preview model, defect tracking, and the docs-site surface were all
resolved with the human before this proposal was accepted.
