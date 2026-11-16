---
id: guide.adapting-your-harness
title: Adapting Your Harness
---

# Adapting Your Harness

## How do I wire this into the agent I actually use?

Workbench ships its infrastructure harness-neutral in `.agents/` and comes wired for [Oh My Pi](https://omp.sh). Any other harness needs a deliberate one-time adaptation: point it at the contract and make its skills discoverable. Workbench wires no harness lifecycle events, so there is nothing else to adapt. Commit that configuration, then leave `.agents/` as the source of truth.

The checks that matter most do not depend on a harness noticing anything. The validator, continuous integration, and the commit hook run independently of the agent's instructions.

The [current process](../PROCESS.md) owns the workflow. Your checkout's `AGENTS.md` owns its agent instructions; [Workbench's default-branch agent contract](https://github.com/markmals/workbench/blob/main/AGENTS.md) is the template source. When reviewing an unmerged change, use the contract from that change's branch.

Ask your agent to make the harness-specific changes below. You should review and commit the result, then run a small feature through the process before relying on it for larger work.

## What ships

| Path                    | What it contains                                                |
| ----------------------- | --------------------------------------------------------------- |
| `AGENTS.md`             | The agent contract — the one file every harness must load.      |
| `PROCESS.md`            | The full process description.                                   |
| `VISION.md`             | Your project's vision; replace Workbench's.                     |
| `proposals/`            | One document per change, preserved as history.                  |
| `policies/`             | Rules future work must keep enforcing.                          |
| `decisions/`            | Architectural choices and the context that produced them.       |
| `guides/`               | User-facing documentation.                                      |
| `docs/`                 | The site that publishes the vision, record, and guides.         |
| `tools/`                | The artifact validator and the small scripts the tasks run.     |
| `.agents/skills/`       | One skill directory per process phase or cross-cutting concern. |
| `.agents/rules/`        | Code quality, commit discipline, and enforcement hierarchy.     |
| `.agents/templates/`    | The canonical shape of every artifact.                          |
| `.agents/commit-scopes` | The commit scopes the `commit-msg` hook accepts.                |
| `mise.toml`             | Every pinned tool and every task.                               |
| `hk.pkl`                | Every git hook, including the Scoped Commits check.             |
| `.github/workflows/`    | Continuous integration and the preview workflow.                |
| `.omp/`                 | Oh My Pi wiring: the `workflow` tool.                           |

When a harness needs its own configuration, generate it from `.agents/` rather than replacing the source. This prevents two copies of the process from becoming two different processes.

## If you use Oh My Pi

Nothing needs adapting. Oh My Pi loads `AGENTS.md` from the repository root and discovers `.agents/skills/` on its own, and `.omp/tools/dynamic-workflows/` registers the `workflow` tool.

Continue at [Configure the readiness-report account](#configure-the-readiness-report-account).

## Point the harness at the contract

Most current harnesses load `AGENTS.md` from the repository root without configuration. If yours does, this step is complete.

If it expects another filename, add a thin file that defers to `AGENTS.md`; do not copy the contract. For Claude Code, create this shim at the repository root:

```sh
printf '@AGENTS.md\n' > CLAUDE.md
```

If your harness has no import mechanism, create a symlink using the filename it expects:

```sh
ln -s AGENTS.md .harness-instructions.md
```

The shim or symlink is harness configuration, not an alternative contract. Keep it small enough that a reader can see it delegates to `AGENTS.md`.

## Make skills discoverable

Each `.agents/skills/<name>/SKILL.md` has `name` and `description` frontmatter followed by its procedural instructions. The description is what an agent uses when deciding whether to load it.

If your harness discovers skills from a fixed directory, point that directory at `.agents/skills` with a symlink. Keep `.agents/skills` canonical; the harness-specific path is configuration, not a second copy.

A harness without a skill mechanism can still use Workbench. `AGENTS.md` names each skill at the point where it applies, so the agent can read it directly. Confirm that it follows those paths. If it cannot, put the two or three most important skill bodies in `AGENTS.md` rather than maintaining a parallel skills tree.

## Git hooks need no adaptation

Git hooks are not harness wiring. [hk](https://hk.jdx.dev) owns them, `mise run setup` installs them with `hk install --mise`, and they run for every commit regardless of who or what made it — including the `commit-msg` step that holds every subject to Scoped Commits against the scopes in `.agents/commit-scopes`, the `pre-commit` step that formats staged files, and the steps that sync Beads on `git push` and `git pull`.

No hook runs the full gate. Continuous integration runs `mise run check` on every push and checks every commit in a pull request wherever it was made, and the agent waits for it before marking a pull request ready or merging. Orientation and formatting are commands the agent runs itself, `mise run status` and `mise run fmt -- <files>`, so a harness needs no event to trigger them.

## Configure the readiness-report account

The readiness report comes from a service account so the pull request distinguishes statements the agent made from statements you made.

Create a bot account or GitHub App, give it write access to the repository, and expose its token to the agent as `SERVICE_ACCOUNT_TOKEN`:

```sh
export SERVICE_ACCOUNT_TOKEN=<the bot's token>
```

The name matters. The agent sets `GH_TOKEN` from `SERVICE_ACCOUNT_TOKEN` only when posting its own readiness-report comment. Marking the pull request ready happens after the complete implementation is verified and pushed, with the ordinary `gh pr ready` identity. Exporting the service token as a global `GH_TOKEN` would attribute unrelated GitHub actions to the bot.

Without a service account the agent identifies the available account in the report, subject to your repository's identity rules.

## Start from your own project record

Workbench includes its own record as an example. To replace it and write your project vision, follow [Getting Started](./getting-started.md). Keep `policies/` and `decisions/` empty until a feature actually establishes one; speculative policies are the ones nobody follows.

A preview must be the cheapest realistic artifact through which a human can exercise a change. To choose and wire the one for your repository shape, see [Choosing a Preview](./choosing-a-preview.md).

## Verify the adoption

Provision the toolchain, bootstrap the repository, and run the full gate:

```sh
mise install
mise run setup
mise run check
```

Then run one small feature end to end: branch, draft pull request, proposal, current specification, tests, guides, code, review, and readiness report. The first pass exposes what your harness does not do well while the change is still cheap to correct.

## Add another harness without forking

Repeat the contract and skill steps for the new harness. Do not fork `.agents/`; derive the new harness configuration from it.

If two harnesses need genuinely different instructions, make that difference an explicit note in `AGENTS.md`. Do not leave two divergent copies of the contract or skills for future maintainers to reconcile.

For the feature lifecycle the harness supports, see [Running a Feature](./running-a-feature.md).
