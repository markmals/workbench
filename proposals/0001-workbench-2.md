---
id: proposal.0001
title: Workbench 2.0
authors: [markmals, Claude]
status: active-review
pull-request: https://github.com/markmals/workbench/pull/1
issues: []
supersedes: []
---

# Workbench 2.0

## Summary

Replaces Workbench's spec-driven multiplatform application template with a platform-agnostic,
harness-agnostic process kit: vision, policies, decisions, proposals, guides, and previews,
enforced by a record validator and continuous integration. Deletes the stack; keeps the process.

## Motivation

Workbench 1.0 was a spec-driven multiplatform application template. It carried nine platform
skills, four device-driver skills, eleven per-platform task scaffolds, a 42 KB toolchain catalog,
reverse-pointer and drift-detection machinery, and a `/sdd-*` command surface — all in service of
fanning one specification out across web, iOS, Android, Windows, Linux, a CLI, and a Convex
backend.

That answered a question about _how to build the same product several times_. The question that
actually matters is _how to keep an agent honest about any one change_, and 1.0 buried it. The
process was the small part of a large repository about stacks.

Three specific failures made this worth rewriting rather than extending.

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

The workaround available today is to ignore most of the repository and use the four process skills
by hand. That works, and it is evidence that the process was the valuable part all along.

## Proposed solution

Delete the stack. Keep the process, and make the parts a machine can check into checks.

Cloning Workbench gives a project the structure described in `PROCESS.md` and nothing about a
technology. The artifacts are a living `VISION.md`, plus `proposals/`, `policies/`, `decisions/`,
and `guides/` as markdown with validated frontmatter. The process is five phases — preparation,
proposal, implementation, review, completion — each encoded as a skill any competent coding agent
can execute.

```text
branch → draft PR → proposal → tests → guides → code → validation
       → cross-artifact review → adversarial review → preview → readiness report
       → human review → revision → vision update → merge → release → cleanup
```

The parts that previously depended on the agent remembering become deterministic: a validator over
the record, continuous integration running the same gates, and a git hook enforcing commit
scoping — none of which depend on a harness noticing anything.

## Detailed design

**Artifacts.** `VISION.md` at the root; `proposals/<NNNN>-<slug>.md`, `policies/<NNNN>-<slug>.md`,
`decisions/<NNNN>-<slug>.md`, and `guides/<slug>.md`. Each carries YAML frontmatter with `id`,
`title`, and `status`, plus kind-specific keys: `authors`, `pull-request`, `issues`, and
`supersedes` on proposals; `established-by` and `supersedes` on policies and decisions;
`describes` on guides. Templates in `.agents/templates/` are derived from the Swift Evolution
templates, carrying instructional prose rather than bare section labels.

**Process skills.** Fourteen under `.agents/skills/`, one per phase plus the cross-cutting
disciplines: `writing-a-proposal`, `writing-acceptance-criteria`, `test-driven-development`,
`writing-guides`, `implementing-a-proposal`, `cross-artifact-review`, `adversarial-review`,
`producing-a-preview`, `reporting-readiness`, `completing-a-feature`,
`recording-policies-and-decisions`, `tracking-defects`, `systematic-debugging`, and
`verification-before-completion`.

**Ordering.** Implementation is strictly failing tests, then guides written from the proposal,
then code. Guides are compared against the tests before code is written; any behavior in one and
not the other is a defect in one of them.

**Validation.** `tools/validate.mjs`, dependency-free Node, enforces five rules over the record:
identifiers match filenames and are unique; every `supersedes`, `established-by`, and `describes`
reference resolves; superseding an entry requires the superseded entry to be marked; required keys
are present and `status` is within the set allowed for that kind; and no proposal at `accepted`,
`implemented`, or `superseded` still contains a `[NEEDS CLARIFICATION:` marker. Pure logic is
separated from filesystem traversal so the rules are testable directly.

**Enforcement surfaces.** `mise run check` runs formatting, validation, the validator's tests, and
the guides build. `.github/workflows/ci.yml` runs the same gates plus commit-message validation
over the pull request's commits. `.agents/hooks/commit-msg` is a POSIX git hook enforcing Scoped
Commits, replacing 1.0's Claude-specific `PreToolUse` hook.

**Publication.** The existing VitePress site is retained and retargeted from `specs/` and
`features/` onto the record and the guides, with navigation ordered vision, decisions, policies,
proposals, guides. Superseded entries move to a collapsed group rather than disappearing.

**Harness neutrality.** Everything ships in `.agents/` with `AGENTS.md` as the single contract.
`ADOPTING.md` documents the one-time adaptation for a specific harness.

## Acceptance criteria

- [ ] A fresh clone contains no reference to a platform, stack, framework, or device — no `apps/`,
      no `services/`, no reverse pointers, no drift detection, no `/sdd-*` commands.
- [ ] `AGENTS.md` states the five phases, the artifact set, and the hard rules, and every skill it
      names exists at the path given.
- [ ] `mise run check` passes on a clean checkout.
- [ ] The validator rejects each of its five rule violations and accepts a valid record, proven by
      tests that run without network access or third-party dependencies.
- [ ] The site builds, publishes the record and the guides, and does not fail when a record
      directory is empty.
- [ ] Continuous integration runs the same gates as `mise run check` plus commit-message
      validation over the pull request's commits.
- [ ] A preview workflow ships wired for this repository's shape, with recipes for the other
      shapes present but inert.
- [ ] No surviving document instructs the agent not to use a branch, not to push a feature branch,
      or not to open a pull request.

## Compatibility

This is a breaking change for Workbench 1.0, and deliberately so. The two versions share almost
nothing: 1.0's spec identifiers, reverse pointers, feature folders, and `/sdd-*` commands have no
counterpart in 2.0. There is no migration path and none is offered.

Existing 1.0 users are unaffected until they choose to move, because a template repository is
consumed by copying rather than by upgrading. 1.0 is preserved at the `v1.0` tag and remains
usable through "Use this template" at that tag.

## Implications on adoption

Adopting 2.0 requires `mise`, Node 24, and — for the pull-request phases — the `gh` CLI
authenticated against the project's remote. A service account for posting readiness reports is
recommended but optional; without one the agent posts from whatever identity is available and
says so in the report.

The harness adaptation in `ADOPTING.md` is a one-time change that is committed to the project. It
is reversible: deleting the generated harness configuration leaves `.agents/` intact.

Adopting the process is not all-or-nothing. A project can use the artifact structure without the
pull-request phases, or the review stages without the validator. The gates are independent, and
partial adoption degrades rather than breaks.

## Scope

- Delete the multiplatform apparatus: platform skills, device-driver skills, `/sdd-*` commands,
  subagents, per-platform task scaffolds, feature and spec templates, `specs/`, the Convex and
  Clerk environment contract, and the platform codegen hooks.
- Write the agent contract, the process description, the adoption guide, and the vision.
- Create the record directories, their templates, and their explanatory READMEs.
- Rewrite the surviving process skills to proposal vocabulary and add the missing ones.
- Rewrite commit discipline and the enforcement hierarchy; convert the commit hook to POSIX git.
- Build the validator with tests; wire `mise` tasks, continuous integration, and the preview.
- Retarget the documentation site.

### Out of scope

- **Slash commands.** 1.0's `/sdd-*` surface is deleted and not replaced. Commands are
  harness-specific and the kit is harness-neutral; the skills carry the procedure instead.
- **Dependent-project automation.** `PROCESS.md` describes updating dependent repositories' draft
  pull requests when a preview becomes a release. The skill documents the procedure; no tooling
  ships. Building it before a second repository exists would be speculative.
- **Migration tooling for 1.0 users.** See Compatibility.
- **Preview mechanisms other than this repository's own.** Recipes ship as comments. Wiring and
  testing all seven would mean maintaining six workflows nobody here runs.
- **Guides for Workbench itself.** `guides/` ships empty. Workbench's user-facing documentation is
  currently `README.md`, `PROCESS.md`, and `ADOPTING.md`; converting them into guides is real work
  and belongs in its own proposal, which will also be the first genuine end-to-end exercise of
  this process.

## Preview

Workbench is a documentation and process repository, so its preview is the rendered site:
`.github/workflows/preview.yml` runs `mise run docs:build` on every pull request and uploads the
built site as a workflow artifact. Access is gated by repository permissions, which is the privacy
model.

This is the cheapest realistic artifact for this repository. A deployment would cost hosting and
an access-control decision to expose the same static output. A screenshot would not let a reviewer
navigate the site, follow a cross-reference between record entries, or confirm that an empty
record directory degrades gracefully.

## Policies and decisions checked

`policies/` and `decisions/` are both empty. This is the first proposal in the repository, and 1.0
recorded neither as a first-class artifact — its `specs/CONVENTIONS.md` explicitly excluded
decision logs, saying to "use a `docs/` directory if you need one". Both directories were read;
there is nothing to check against.

Two constraints were nonetheless inherited from 1.0 and deliberately preserved:

- `.agents/rules/code-quality.md` — retained essentially unchanged. The validator's separation of
  pure logic from filesystem traversal is written to satisfy it.
- `.agents/rules/enforcement-hierarchy.md` — retained and retiered. The decision to put record
  invariants in a validator rather than in prose is an application of it.

Neither is promoted to `policies/` here. They govern this repository rather than emerging from a
feature, and speculative policies are the ones nobody follows.

## Future directions

- Converting `README.md`, `PROCESS.md`, and `ADOPTING.md` into `guides/` entries, which would give
  the kit a worked example of its own guides discipline.
- Mechanical checks beyond frontmatter: proposals with no corresponding tests, guides describing
  behavior no test covers, policies no recent proposal has cited.
- Dependent-project automation, once a second repository consumes a Workbench preview.
- Recording whether the adversarial stage catches defects the confirmatory stages miss, which
  would make it possible to evaluate the process rather than assume it.

None of these are required for the kit to be usable, and each is large enough to deserve its own
proposal.

## Alternatives considered

**Keep the multiplatform apparatus as an optional layer, pruned at setup.** This preserves real
work and was the least disruptive option. Rejected because `PROCESS.md` mentions no platform
anywhere, so the apparatus would be a second, undocumented half of the repository — which is
precisely the condition that buried the process in 1.0. The `v1.0` tag preserves the work at zero
maintenance cost.

**Split into two repositories: `workbench` and `workbench-platforms`.** More respectful of the
platform investment than deletion. Rejected because a live second repository carries ongoing
maintenance, and a tag carries none while remaining fully usable.

**Ship configuration for all four harnesses named in `PROCESS.md`.** Highest fidelity to the claim
of harness-agnosticism. Rejected because only one can be genuinely verified here, and three
untested configurations are worse than an honest adaptation step — they would fail quietly, in the
harness the user actually chose. The cost is losing automatic lifecycle enforcement out of the
box, which is why the commit hook and continuous integration were made harness-independent: the
checks that matter most do not depend on the harness noticing.

**Keep 1.0's seven-file feature folder.** The narrative, stories, use-cases, user flow, domain
models, view models, and error definitions served cross-platform fan-out well. Rejected because
with one implementation the taxonomy is overhead, and overhead on small changes is what causes a
process to be skipped entirely.

**Publish only the guides, not the record.** Simpler, and it sidesteps the question of whether a
public documentation site should expose design history. Rejected because the record is the part of
a project hardest to reconstruct later and hardest to navigate as raw files — publishing it turns
cross-references into links and keeps superseded entries reachable. A project needing public
guides and a private record can exclude the record directories in its own configuration, which is
a narrower problem than making the record unreadable for everyone by default.

**A YAML dependency for the validator.** Rejected as unnecessary. The frontmatter is flat scalars
and inline lists; a reader that rejects what it does not understand is around thirty lines and
keeps the validator runnable anywhere Node is.

## Open questions

None. Scope, harness strategy, preview model, defect tracking, the documentation surface, and the
template lineage were all resolved with the human before implementation began.

## Acknowledgments

The artifact templates are adapted from the
[Swift Evolution](https://github.com/swiftlang/swift-evolution) proposal, policy, and vision
documents. The instructional-prose approach — a template that teaches rather than labels — is
theirs, and it is the single largest improvement to this proposal's design.

The review discipline in `adversarial-review` and `test-driven-development` originated in
[Superpowers](https://github.com/obra/superpowers) and survives from Workbench 1.0.
