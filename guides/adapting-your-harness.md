---
id: guide.adapting-your-harness
title: Adapting Your Harness
describes: [proposal.0001]
---

# Adapting Your Harness

## How do I wire this into the agent I actually use?

Workbench ships its infrastructure harness-neutral in `.agents/`. No coding harness reads that
directory natively. Adapting it is a deliberate one-time trade: point your harness at the contract,
make its skills discoverable, and wire the lifecycle events it supports. Commit that configuration,
then leave `.agents/` as the source of truth.

The checks that matter most do not depend on a harness noticing anything. The validator, continuous
integration, and the commit hook run independently of the agent's instructions.

Ask your agent to make the harness-specific changes below. You should review and commit the result,
then run a small feature through the process before relying on it for larger work.

## What ships

| Path                 | What it contains                                                |
| -------------------- | --------------------------------------------------------------- |
| `AGENTS.md`          | The agent contract — the one file every harness must load.      |
| `PROCESS.md`         | The full process description.                                   |
| `VISION.md`          | Your project's vision; replace Workbench's.                     |
| `proposals/`         | One document per change, preserved as history.                  |
| `policies/`          | Rules future work must keep enforcing.                          |
| `decisions/`         | Architectural choices and the context that produced them.       |
| `guides/`            | User-facing documentation.                                      |
| `docs/`              | The site that publishes the vision, record, and guides.         |
| `tools/`             | The artifact validator.                                         |
| `.agents/skills/`    | One skill directory per process phase or cross-cutting concern. |
| `.agents/rules/`     | Code quality, commit discipline, and enforcement hierarchy.     |
| `.agents/templates/` | The canonical shape of every artifact.                          |
| `.agents/hooks/`     | The commit-message git hook.                                    |
| `.github/workflows/` | Continuous integration and the preview workflow.                |

When a harness needs its own configuration, generate it from `.agents/` rather than replacing the
source. This prevents two copies of the process from becoming two different processes.

## Point the harness at the contract

Most current harnesses load `AGENTS.md` from the repository root without configuration. If yours
does, this step is complete.

If it expects another filename, add a thin file that defers to `AGENTS.md`; do not copy the
contract. For Claude Code, create this shim at the repository root:

```sh
printf '@AGENTS.md\n' > CLAUDE.md
```

If your harness has no import mechanism, create a symlink using the filename it expects:

```sh
ln -s AGENTS.md .harness-instructions.md
```

The shim or symlink is harness configuration, not an alternative contract. Keep it small enough
that a reader can see it delegates to `AGENTS.md`.

## Make skills discoverable

Each `.agents/skills/<name>/SKILL.md` has `name` and `description` frontmatter followed by its
procedural instructions. The description is what an agent uses when deciding whether to load it.

If your harness discovers skills from a fixed directory, point that directory at `.agents/skills`
with a symlink. Keep `.agents/skills` canonical; the harness-specific path is configuration, not a
second copy.

A harness without a skill mechanism can still use Workbench. `AGENTS.md` names each skill at the
point where it applies, so the agent can read it directly. Confirm that it follows those paths. If
it cannot, put the two or three most important skill bodies in `AGENTS.md` rather than maintaining
a parallel skills tree.

## Wire lifecycle events

Workbench needs these events attached where your harness allows them:

| When                           | Run                   |
| ------------------------------ | --------------------- |
| After the agent edits a file   | `mise run fmt <file>` |
| Before the agent ends its turn | `mise run check`      |
| At session start               | Load `AGENTS.md`      |

Install the commit-message hook as well. It is harness-independent and catches commits made by
hand:

```sh
ln -s ../../.agents/hooks/commit-msg .git/hooks/commit-msg
```

Continuous integration runs the same validation, so the hook is a fast local signal rather than
the only line of defence.

## Configure the readiness-report account

The readiness report comes from a service account so the pull request distinguishes statements the
agent made from statements you made.

Create a bot account or GitHub App, give it write access to the repository, and expose its token to
the agent as `GH_TOKEN`. Without one, the agent uses the available identity and says so in the
report. That is degraded, not broken.

## Start from your own project record

Workbench includes its own record as an example. To replace it and write your project vision, follow
[Getting Started](./getting-started.md). Keep `policies/` and `decisions/` empty until a feature
actually establishes one; speculative policies are the ones nobody follows.

A preview must be the cheapest realistic artifact through which a human can exercise a change. To
choose and wire the one for your repository shape, see [Choosing a Preview](./choosing-a-preview.md).

## Verify the adoption

Install the declared tools and run the full gate:

```sh
mise install
mise run check
```

Then run one small feature end to end: branch, draft pull request, proposal, tests, guides, code,
review, and readiness report. The first pass exposes what your harness does not do well while the
change is still cheap to correct.

## Add another harness without forking

Repeat the contract, skill, and lifecycle steps for the new harness. Do not fork `.agents/`; derive
the new harness configuration from it.

If two harnesses need genuinely different instructions, make that difference an explicit note in
`AGENTS.md`. Do not leave two divergent copies of the contract or skills for future maintainers to
reconcile.

For the feature lifecycle the harness supports, see [Running a Feature](./running-a-feature.md).
