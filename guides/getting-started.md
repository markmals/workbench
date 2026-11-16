---
id: guide.getting-started
title: Getting Started
---

# Getting Started

## I want to use Workbench. What do I do?

Add Workbench to your repository, write your project's vision, and use it for one small feature. You and the agent have different jobs, but you use the same orientation command whenever the work needs a next step.

## What Workbench is

Workbench is a process kit for developing a project with a coding agent. No single artifact defines a feature: intent is written more than once in forms that can disagree. Each requirement has one current owner, such as an API's contract documentation, a schema, or a feature file. Tests encode it, guides explain it, and code implements it; disagreement between them is the signal to investigate. A proposal records each change to those sources and why it was made.

Workbench ships as a package: the `workbench` command-line tool, plus the process contract, rules, skills, and templates your coding agent's harness loads. Your project depends on one pinned release and keeps only its own records and configuration. For the argument behind this structure, read [Workbench's README](https://github.com/markmals/workbench#readme). This guide starts with using it.

## Install the tools

[Mise](https://mise.jdx.dev) owns the toolchain and every task. Install it once per machine and activate it in your shell, following the [mise guide](https://mise.jdx.dev/getting-started.html):

```sh
curl https://mise.run | sh
```

The commands below assume mise is activated. If it is not, prefix each `workbench` command with `mise x --`.

## Add Workbench to your repository

Work from the root of a Git repository: a new one (`git init`), one made from a holotype template, or an existing project. An existing project, especially one copied from the old Workbench template, has more to move; follow [Adopting Workbench](./adopting-workbench.md) instead.

Pin Workbench. `mise use` writes the pin to `mise.toml` and installs that release:

```sh
mise use github:markmals/workbench@3.0.0
```

Preview what Workbench will add, then add it. `project init` never overwrites or deletes a file, and its receipt lists what it created, what it added to, and what it left alone:

```sh
workbench project init --dry-run --output text
workbench project init
```

Install everything it pinned. This also links `.workbench` to the installed package, which is where your harness loads Workbench from. The link is machine-local and gitignored, and every `mise install` restores it:

```sh
mise install
```

The receipt's `missing` list names what Workbench relies on but leaves to your project, because it depends on your language and tools: a `setup` task, a `check` task that is your project's whole gate, an `fmt` task, and a release flow. [Adopting Workbench](./adopting-workbench.md#1-pin-workbench-and-run-project-init) shows a typical `setup` task and how to release with Knope.

**`gh` needs your credentials.** The process runs on pull requests, so authenticate it against your remote:

```sh
gh auth login
```

Without it, `mise run status` cannot see the pull request and will say so rather than guess.

## Verify the setup

Read the setup back, and check it:

```sh
workbench context show --output text
mise run validate
```

`context show` reports the pinned version, where `.workbench` points, which harnesses declare the package, and the Beads and site configuration. `mise run validate` checks your records and your Workbench setup; it passes with no violations in a fresh project. Then bootstrap the repository on this machine and run the complete gate:

```sh
mise run setup
mise run check
```

`mise tasks` lists everything available. The agent runs `mise run check` before claiming a feature is ready.

### If the gate fails

Do not begin a feature with a failing baseline. Give the output to the agent and have it find the root cause; changing a gate or suppressing its failure is not a repair.

## Your harness

`project init` declared the package for [Claude Code](https://code.claude.com) in `.claude/settings.json` and for [Oh My Pi](https://omp.sh) in `.omp/config.yml`. Start a session in either and it loads the process contract and lists Workbench's skills. Claude Code applies the declaration once you trust the folder. [Adopting Workbench](./adopting-workbench.md#harness-notes) covers each harness's caveats.

## Read the worked example

Workbench's own repository keeps its record and guides, which are the clearest available answer to what good output looks like. Read them before you write your own: [its vision](https://github.com/markmals/workbench/blob/main/VISION.md) for the shape of a vision, [its first proposal](https://github.com/markmals/workbench/blob/main/proposals/0001-workbench-2.md) for a real proposal, [Running a Feature](./running-a-feature.md) for the loop you are about to run, and [Keeping Current Specifications](./current-specifications.md) for where your project's requirements will live.

## Write your project's vision

`project init` created `VISION.md` from the package's template. Fill it in, in conversation with the agent. Describe your project's purpose, minimum viable product, principles, direction, and non-goals — not a list of the first feature's tasks.

The vision is the standard against which later proposals are evaluated. Without it, the agent silently substitutes its assumptions for the direction you did not write down. Keep it current when accepted work changes what the project is.

Keep `policies/` and `decisions/` empty until a real feature establishes one; speculative policies are the ones nobody follows.

## Run your first feature

Pick something small for the first pass. It will expose the parts your harness handles badly, and learning that on a small change is much cheaper than learning it on a large one.

The [Running a Feature](./running-a-feature.md) guide takes a change from a branch and draft pull request through proposal, implementation, review, release, and cleanup. The agent handles the ordered implementation and review work; you establish intent, make the proposal decisions reserved for you, and review the preview before accepting the feature.

## The one command

```sh
mise run status
```

Run it with no arguments at the start of a session and whenever you need to orient yourself. It reads the branch, pull request, and proposal record, then reports the current phase, the next action, and who acts. It also syncs the Beads work tracker with your other machines and lists the ready work, the items in progress and who holds them, and the project's memories.

You are not expected to memorise commands, skill names, or process steps. “Keep going” is a complete instruction to the agent because it orients itself with the same report, `workbench workflow status`.

## Where things live

| Path                                           | What it holds                                                                    | When you use it                                                                                      |
| ---------------------------------------------- | -------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `VISION.md`                                    | Your project's purpose, minimum viable product, and direction                    | Write it before feature work; update it after accepted work changes the project.                     |
| `proposals/`                                   | One numbered record of each change and its reasons                               | Write and settle a proposal before implementation.                                                   |
| Contract documentation, schemas, feature files | The current specifications, wherever each surface keeps them                     | The agent revises the affected ones first, once you approve a proposal.                              |
| `guides/`                                      | Reader-facing behavior and usage guides                                          | The agent writes or updates these from the intended contract before code.                            |
| `policies/`                                    | Current rules future work must keep enforcing                                    | Add one when agreed work establishes a durable rule; revise or delete it when that rule changes.     |
| `decisions/`                                   | Current architectural choices and their context                                  | Add one when agreed work establishes a durable choice; revise or delete it when that choice changes. |
| `.workbench`                                   | The pinned Workbench package: the process contract, rules, skills, and templates | Never edit it; change the pin in `mise.toml` and run `mise install`.                                 |
| `.agents/`                                     | Your project's own skills, rules, template overrides, and commit scopes          | Add to it when the project needs instructions of its own.                                            |

## Continue from here

- [Run a feature](./running-a-feature.md)
- [Write a good proposal](./writing-a-good-proposal.md)
- [Choose a preview](./choosing-a-preview.md)
- [Keep the record](./keeping-the-record.md)
- [Keep current specifications](./current-specifications.md)
- [Adopt Workbench in an existing project](./adopting-workbench.md)
