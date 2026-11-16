# Workbench

![Workbench](./docs/static/workbench-hero.png)

A development process for working with coding agents, packaged as a versioned dependency your project pins.

Workbench is built on one idea: **no single artifact defines a feature by itself.** A proposal records the change and why, a current specification states what the project now requires, tests encode it, guides explain it, code implements it, and a preview lets a human exercise it. Each is written independently, so when they disagree, you find out before release instead of after.

It assumes nothing about your language, framework, platform, or coding harness, and it needs no Node.

## Why

An agent that goes straight from a request to an implementation produces code nobody specified, tests that agree with whatever got written, and documentation reverse-engineered from the result. Everything corroborates everything else, and none of it is evidence.

The fix is not more review. It is making the agent commit to what it intends to build, in a form that can be checked against what it actually built.

## Quick start

1. Add Workbench to a Git repository, new or existing:

    ```sh
    mise use github:markmals/workbench@3   # pin Workbench in mise.toml and install it
    workbench project init                 # add what Workbench owns
    mise install                           # link .workbench and install the tools init pinned
    ```

    `workbench project init` creates what is missing and never overwrites a file; run it with `--dry-run` first to see what it will do. Its receipt lists what your project still has to supply, such as the `setup`, `check`, and `fmt` tasks. [Getting Started](guides/getting-started.md) walks through a new project, and [Adopting Workbench](guides/adopting-workbench.md) moves a project off a copy of Workbench.

2. Commit what `project init` wrote, and open the project in [Claude Code](https://code.claude.com) or [Oh My Pi](https://omp.sh). Both load Workbench from `.workbench`, through the harness declarations `project init` wrote. Claude Code asks you to trust the folder first.
3. Write your project's vision with your agent. `project init` starts `VISION.md` from Workbench's template.
4. Start your first feature. Say what you want built. Your agent branches, opens a draft pull request, and writes a proposal before it writes any code.

Updates arrive through the pin. Change the version in `mise.toml`, or let Renovate change it, and run `mise install`; the next session loads the new release.

## You never have to remember a command

There is one, and it takes no arguments:

```sh
mise run status
```

It reads the branch, the pull request, and the proposal's status, then tells you where the work is and what happens next:

```
proposal.0001  Workbench 2.0  ·  draft
PR #1 (draft)  https://github.com/markmals/workbench/pull/1
branch workbench-2

Phase 2 — Proposal
  The proposal is published and being iterated on.

Next (you): Edit the proposal directly, or tell the agent what to change.
            Move it to awaiting-implementation when you are satisfied.
```

Your agent runs the same report, as JSON from `workbench workflow status`, at the start of every session. So "keep going" is a complete instruction — it can work out the rest. Skills exist for the agent to read; you are not expected to know their names.

## The process

Per feature:

```
branch → draft PR → proposal → current specifications → tests → guides → code
       → validation → ready PR → self-check → adversarial review → preview
       → readiness report → human review → revision → records and vision update
       → merge → release → cleanup
```

**Preparation.** The agent branches, pushes, and opens a draft pull request. The pull request is the durable workspace — proposal, implementation, review discussion, and status reports all accumulate in one reviewable place.

**Proposal.** You and the agent write `proposals/<NNNN>-<slug>.md` and iterate until it is right. It links the current specifications the change touches and says what changes in each, concretely enough that the agent can revise them and build without rereading your conversation. Unresolved questions are marked and answered, never guessed. Once the change ships, the proposal is its history; the specifications are where the next change starts.

**Implementation.** Current specifications first: whichever API contract documentation, schema, reference prose, or Gherkin feature already owns the surface is revised to state the approved change. Then failing tests, then guides written from that intended contract, then code. The order matters: tests written after the code agree with the code, and guides written after the implementation document its accidents. Once the complete implementation is verified and pushed, the agent marks the pull request ready for you to review. The proposal remains `awaiting-implementation` while agent review continues.

**Review.** The agent reads proposal, specifications, tests, guides, and code together and fixes where they disagree. Then one adversarial reviewer — fresh context, different model, style ignored — tries to prove the work is broken. It reports; it does not fix. Findings the primary agent declines are surfaced in the readiness report rather than quietly dropped. Then the preview goes up, and the agent posts a readiness report and moves the proposal to `active-review`; the PR was already ready.

**Human review.** You read the report, the diff, and the preview, and comment. Look-and-feel feedback is applied while the work stays in review. If your feedback changes what the feature does, or a design decision the proposal records, rather than correcting the implementation, the proposal is updated first and the loop runs again. When you accept it: records and vision update, merge, release, cleanup.

The full description is in [`PROCESS.md`](PROCESS.md), and [`guides/current-specifications.md`](guides/current-specifications.md) shows where each kind of requirement lives. The process contract your agent loads every session is [`plugin/rules/workbench.md`](plugin/rules/workbench.md); your project's `AGENTS.md` holds only facts about your project.

## Previews

A preview is the cheapest realistic artifact through which you can exercise a change. _Cheapest_ rules out standing up production infrastructure to review a pull request; _realistic_ rules out screenshots and descriptions of what would happen.

| Project           | Preview                                            |
| ----------------- | -------------------------------------------------- |
| Library           | prerelease package                                 |
| Web application   | preview deployment                                 |
| API               | preview endpoint deployment                        |
| Documentation     | rendered documentation build                       |
| Command-line tool | installable executable                             |
| Desktop           | installable build                                  |
| Mobile            | installable build, ad-hoc or internal distribution |

Your project wires the row that matches it, and [Choosing a Preview](guides/choosing-a-preview.md) explains how. Previews are private by default.

## Defects

Defects an agent finds along the way get filed in [Beads](https://github.com/steveyegge/beads), not folded into whatever it happens to be building. A defect that stops the current work from satisfying its proposal is the current work and gets fixed; a defect that is out of scope, pre-existing, or someone else's problem becomes a work item written so a future reader can reproduce it without the conversation it came from. Deferred defects are linked from the pull request, so the decision to defer is visible.

## What's in here

| Path | What it is |
| --- | --- |
| `AGENTS.md` | Facts about this repository for agents: the crate, the package layout, the release, and the site. |
| `PROCESS.md` | The full process description. |
| `VISION.md` | Workbench's purpose and direction. |
| `proposals/` | One document per change, kept as history after it ships. |
| `policies/` | Current rules future work must keep enforcing, revised in place. |
| `decisions/` | Current architectural choices and the context that produced them. |
| `guides/` | User-facing documentation. Start at `getting-started.md`. |
| `Cargo.toml`, `src/`, `tests/` | The `workbench` CLI, a Rust crate. |
| `plugin/.claude-plugin/` | Claude Code's plugin and marketplace manifests. |
| `plugin/rules/` | The process contract, `workbench.md`, and the code quality, commit discipline, enforcement hierarchy, and verification rules. |
| `plugin/output-styles/` | The process contract as a Claude Code output style. |
| `plugin/skills/` | Skills for each phase and cross-cutting concern, including optional Gherkin guidance. For the agent, not you. |
| `plugin/templates/` | The canonical shape of every artifact. |
| `plugin/reference/cli.md` | The CLI reference, generated from the command registry. |
| `plugin/site/` | The Hugo theme for a project's record site. |
| `docs/` | This repository's site, built with Hugo and Pagefind. Publishes the vision, the process, the record, and the guides. |
| `knope.toml`, `.changeset/` | The release configuration, and the pending change files Knope turns into the changelog and release notes. |
| `.agents/commit-scopes` | This repository's commit scopes beyond Workbench's built-in process scopes. |
| `mise.toml` | Every pinned tool and every task. `mise.lock` pins the downloads. |
| `hk.pkl` | Every git hook: formatting, Scoped Commits, and the Beads steps, including the Beads sync on push and pull. |
| `dprint.json` | Formatting for Markdown, JSON, YAML, and TOML. |
| `.github/workflows/` | Continuous integration, the release, the preview, and the published site. |
| `.claude/settings.json`, `.omp/config.yml` | The harness declarations that load `.workbench`, the same as in any project. |

A release is the `workbench` binary plus the contents of `plugin/`, one archive per platform. In this repository, `mise run setup` links `.workbench` to `plugin/` and builds the CLI, so Workbench runs on its own working tree.

## Enforcement

Prose is the tier most likely to be missed, so anything checkable is a check.

```sh
mise run check       # the gate: the Rust tests, hk's checks, records, the site
mise run fmt         # format in place; accepts file paths
mise run validate    # records, and the project's Workbench setup
mise run docs:build  # the site builds, with no dead links
```

`mise run validate` enforces that identifiers match filenames and are unique, that proposal `supersedes` and optional `established-by` references resolve, that superseding a proposal marks the superseded one, and that no proposal approved for implementation still carries an unresolved question. It rejects supersession metadata on policies and decisions, and `describes` on guides, with migration guidance; [`guides/record-validation.md`](guides/record-validation.md) is its full contract. It also reports a `.workbench` link that disagrees with the pin, a missing harness declaration, a project skill or rule that reuses a Workbench name, and Workbench files a project still vendors. [hk](https://hk.jdx.dev) owns the git hooks: `commit-msg` runs `workbench commits check`, which holds every commit to [Scoped Commits](https://scopedcommits.com/) with Workbench's process scopes, the scopes in `.agents/commit-scopes`, and each proposal's own scope, and `pre-commit` formats and lints staged files. Continuous integration runs `mise run check` on every push and checks every pull request's commit subjects and change files. A pull request is marked ready or merged only after CI passes, so no gate depends on the agent remembering.

## Harness support

Workbench's process contract, skills, rules, and templates are harness-neutral Markdown in the package. A project commits two small declarations that point its harness at `.workbench`: `.claude/settings.json` for [Claude Code](https://code.claude.com) and `.omp/config.yml` for [Oh My Pi](https://omp.sh). `workbench project init` writes both. Workbench wires no harness lifecycle events: the agent runs `workbench workflow status`, `mise run fmt`, and `mise run check` itself, and nothing slows its loop between turns.

The checks that matter most — record validation, continuous integration, and the commit hook — do not depend on the harness noticing anything.

## Coming from Workbench 1.0

1.0 was a spec-driven multiplatform application template: platform skills, a toolchain catalog, reverse pointers, drift detection, and a `/sdd-*` command surface. It is preserved at the `v1.0` tag and remains usable.

There is no upgrade path. Today's Workbench shares almost nothing with it: it deletes the stack and keeps the process. If you want the multiplatform apparatus, use the tag.

## Read next

- [`PROCESS.md`](PROCESS.md) — the process in full.
- [`plugin/rules/workbench.md`](plugin/rules/workbench.md) — the process contract your agent is actually bound by.
- [`guides/current-specifications.md`](guides/current-specifications.md) — where current requirements live, with worked library, Gherkin, and policy changes.
- [`proposals/0001-workbench-2.md`](proposals/0001-workbench-2.md) — the Workbench 2 rewrite, proposed through its own process. The most useful thing to read if you want to know what a good proposal looks like.
- [`proposals/0007-depend-on-workbench.md`](proposals/0007-depend-on-workbench.md) — how Workbench became a package projects depend on.
