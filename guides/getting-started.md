---
id: guide.getting-started
title: Getting Started
---

# Getting Started

## I just cloned this. What do I do?

Start by making the kit your project, then use it for one small feature. You and the agent have different jobs, but you use the same orientation command whenever the work needs a next step.

## What Workbench is

Workbench is a process kit for developing a project with a coding agent. No single artifact defines a feature: intent is written more than once in forms that can disagree. Each requirement has one current owner, such as an API's contract documentation, a schema, or a feature file. Tests encode it, guides explain it, and code implements it; disagreement between them is the signal to investigate. A proposal records each change to those sources and why it was made.

For the argument behind this structure, read the `README.md`. This guide starts with using it.

## Install and verify

[Mise](https://mise.jdx.dev) owns the toolchain and every task. Install it once per machine, following the [mise guide](https://mise.jdx.dev/getting-started.html):

```sh
curl https://mise.run | sh
```

From the repository root, provision the pinned tools. `mise.toml` pins them and `mise.lock` pins the exact downloads, so every machine and every CI runner gets the same versions — Node, pnpm, [Bun](https://bun.sh) (one test loads the `workflow` tool through Oh My Pi's real loader, and that loader runs on Bun), the [GitHub CLI](https://cli.github.com), [hk](https://hk.jdx.dev), [Fnox](https://fnox.jdx.dev), [Beads](https://github.com/steveyegge/beads) and [Worktrunk](https://worktrunk.dev):

```sh
mise install
```

Then bootstrap the repository. This installs the npm dependencies with the pinned pnpm, initializes the Beads work tracker, and installs every git hook — including the `commit-msg` hook that holds commits to [Scoped Commits](https://scopedcommits.com/). It is idempotent; run it again whenever the tooling changes:

```sh
mise run setup
```

Then run the complete local gate:

```sh
mise run check
```

`mise run check` succeeds only when formatting, lint, and type checks, record validation, tests, and the documentation build all finish without an error. The documentation build also catches dead links.

**`gh` still needs your credentials.** The process runs on pull requests, so authenticate it against your remote:

```sh
gh auth login
```

Without it, `mise run status` cannot see the pull request and will say so rather than guess.

When you need to diagnose the nonformatting parts of the gate, run:

```sh
mise run validate
mise run test
```

This repository produced this trimmed passing output:

```text
$ node tools/validate.ts
Validation passed: 0 violations

$ node --test 'tools/**/*.test.ts'
ℹ tests 120
ℹ pass 120
ℹ fail 0

$ vitest run --dir .omp/tools
 Test Files  4 passed (4)
      Tests  107 passed (107)
```

`mise tasks` lists everything available. Run `mise run check` after changing the project; the agent runs it before claiming a feature is ready.

### If the gate fails

Do not begin a feature with a failing baseline. Give the output to the agent and have it find the root cause; changing a gate or suppressing its failure is not a repair.

## Adapt it to your harness

Workbench comes wired for [Oh My Pi](https://omp.sh): it loads the contract, discovers the skills, and runs the lifecycle extensions in `.omp/extensions/` with no further setup. For any other harness, adapt its contract and skill-loading configuration, commit that one-time setup, and then forget about the wiring; the process material in `.agents/` remains the source of truth. Follow [Adapting Your Harness](./adapting-your-harness.md) before starting feature work.

## Read the worked example

This clone ships Workbench's own record and guides, which are the clearest available answer to what good output looks like. Read them before you write or delete anything: `VISION.md` for the shape of a vision, `proposals/0001-*.md` for a real proposal, [Running a Feature](./running-a-feature.md) for the loop you are about to run, and [Keeping Current Specifications](./current-specifications.md) for where your project's requirements will live.

## Write your project's vision

Now replace `VISION.md`, using `.agents/templates/VISION.md`, in conversation with the agent. Describe your project's purpose, minimum viable product, principles, direction, and non-goals — not a list of the first feature's tasks.

The vision is the standard against which later proposals are evaluated. Without it, the agent silently substitutes its assumptions for the direction you did not write down. Keep it current when accepted work changes what the project is.

## Clear the rest of the example

Remove Workbench's proposals and its guides. **Keep `guides/README.md`** — it documents the frontmatter every future guide needs, and the site's navigation points at it while you have no guides of your own. Keep the `README.md` in each record directory for the same reason.

Keep `policies/` and `decisions/` empty until a real feature establishes one; speculative policies are the ones nobody follows.

## Run your first feature

Pick something small for the first pass. It will expose the parts your harness handles badly, and learning that on a small change is much cheaper than learning it on a large one.

The [Running a Feature](./running-a-feature.md) guide you just read takes a change from a branch and draft pull request through proposal, implementation, review, release, and cleanup. The agent handles the ordered implementation and review work; you establish intent, make the proposal decisions reserved for you, and review the preview before accepting the feature.

## The one command

```sh
mise run status
```

Run it with no arguments at the start of a session and whenever you need to orient yourself. It reads the branch, pull request, and proposal record, then reports the current phase, the next action, and who acts.

You are not expected to memorise commands, skill names, or process steps. “Keep going” is a complete instruction to the agent because it orients itself with the same command.

## Where things live

| Path | What it holds | When you use it |
| --- | --- | --- |
| `VISION.md` | Your project's purpose, minimum viable product, and direction | Write it before feature work; update it after accepted work changes the project. |
| `proposals/` | One numbered record of each change and its reasons | Write and settle a proposal before implementation. |
| Contract documentation, schemas, feature files | The current specifications, wherever each surface keeps them | The agent revises the affected ones first, once you approve a proposal. |
| `guides/` | Reader-facing behavior and usage guides | The agent writes or updates these from the intended contract before code. |
| `policies/` | Current rules future work must keep enforcing | Add one when agreed work establishes a durable rule; revise or delete it when that rule changes. |
| `decisions/` | Current architectural choices and their context | Add one when agreed work establishes a durable choice; revise or delete it when that choice changes. |
| `.agents/` | Harness-neutral process instructions, rules, and templates | Your harness reads the adapted copy; this remains the source of truth. |

## Continue from here

- [Run a feature](./running-a-feature.md)
- [Write a good proposal](./writing-a-good-proposal.md)
- [Choose a preview](./choosing-a-preview.md)
- [Keep the record](./keeping-the-record.md)
- [Keep current specifications](./current-specifications.md)
- [Adapt your harness](./adapting-your-harness.md)
