---
id: guide.getting-started
title: Getting Started
describes: [proposal.0001]
---

# Getting Started

## I just cloned this. What do I do?

Start by making the kit your project, then use it for one small feature. You and the agent have
different jobs, but you use the same orientation command whenever the work needs a next step.

## What Workbench is

Workbench is a process kit for developing a project with a coding agent.
No single artifact defines a feature: intent is written more than once in forms that can disagree.
A proposal records intent, tests encode behavior, guides explain it, and code implements it;
disagreement between them is the signal to investigate.

For the argument behind this structure, read the `README.md`. This guide starts with
using it.

## Install and verify

From the repository root, install the tools pinned by the project:

```sh
mise install
```

Then run the complete local gate:

```sh
mise run check
```

`mise run check` succeeds only when formatting verification, record validation, tests, and the
documentation build all finish without an error. The documentation build also catches dead links.

**One tool `mise` does not install: the [GitHub CLI](https://cli.github.com).** The process runs
on pull requests, so `gh` must be present and authenticated against your remote:

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
[validate] $ node tools/validate.mjs
Validation passed: 0 violations

[test] $ node --test tools/*.test.mjs
ℹ tests 55
ℹ pass 55
ℹ fail 0
```

Run `mise run check` after changing the project; the agent runs it before claiming a feature is
ready.

### If the gate fails

Do not begin a feature with a failing baseline. Give the output to the agent and have it find the
root cause; changing a gate or suppressing its failure is not a repair.

## Adapt it to your harness

No coding harness reads `.agents/` natively. Adapt its contract and skill-loading configuration for
the harness you actually use, commit that one-time setup, and then forget about the wiring; the
process material in `.agents/` remains the source of truth. Follow
[Adapting Your Harness](./adapting-your-harness.md) before starting feature work.

## Read the worked example

This clone ships Workbench's own record and guides, which are the clearest available answer to
what good output looks like. Read them before you write or delete anything: `VISION.md` for the
shape of a vision, `proposals/0001-*.md` for a real proposal, and
[Running a Feature](./running-a-feature.md) for the loop you are about to run.

## Write your project's vision

Now replace `VISION.md`, using `.agents/templates/VISION.md`, in conversation with the agent.
Describe your project's purpose, minimum viable product, principles, direction, and non-goals —
not a list of the first feature's tasks.

The vision is the standard against which later proposals are evaluated. Without it, the agent
silently substitutes its assumptions for the direction you did not write down. Keep it current
when accepted work changes what the project is.

## Clear the rest of the example

Remove `proposals/0001-*.md` and the six guides. **Keep `guides/README.md`** — it documents the
frontmatter every future guide needs, and the site's navigation points at it while you have no
guides of your own. Keep the `README.md` in each record directory for the same reason.

Keep `policies/` and `decisions/` empty until a real feature establishes one; speculative
policies are the ones nobody follows.

## Run your first feature

Pick something small for the first pass. It will expose the parts your harness handles badly, and
learning that on a small change is much cheaper than learning it on a large one.

The [Running a Feature](./running-a-feature.md) guide you just read takes a change from a branch
and draft pull request through proposal, implementation, review, release, and cleanup. The agent
handles the ordered implementation and review work; you establish intent, make the proposal
decisions reserved for you, and review the preview before accepting the feature.

## The one command

```sh
mise run status
```

Run it with no arguments at the start of a session and whenever you need to orient yourself. It
reads the branch, pull request, and proposal record, then reports the current phase, the next
action, and who acts.

You are not expected to memorise commands, skill names, or process steps. “Keep going” is a
complete instruction to the agent because it orients itself with the same command.

## Where things live

| Path         | What it holds                                                 | When you use it                                                                  |
| ------------ | ------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| `VISION.md`  | Your project's purpose, minimum viable product, and direction | Write it before feature work; update it after accepted work changes the project. |
| `proposals/` | One numbered specification for each change                    | Write and settle a proposal before implementation.                               |
| `guides/`    | Reader-facing behavior and usage guides                       | The agent writes or updates these from the proposal before code.                 |
| `policies/`  | Rules future work must keep enforcing                         | Add one only when a feature establishes a durable rule.                          |
| `decisions/` | Architectural choices and their context                       | Add one only when a feature establishes a durable choice.                        |
| `.agents/`   | Harness-neutral process instructions, rules, and templates    | Your harness reads the adapted copy; this remains the source of truth.           |

## Continue from here

- [Run a feature](./running-a-feature.md)
- [Write a good proposal](./writing-a-good-proposal.md)
- [Choose a preview](./choosing-a-preview.md)
- [Keep the record](./keeping-the-record.md)
- [Adapt your harness](./adapting-your-harness.md)
