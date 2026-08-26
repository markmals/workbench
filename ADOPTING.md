# Adopting Workbench

Workbench ships its infrastructure harness-neutral, in `.agents/`. No coding harness reads that
directory natively. Adapting it is a one-time step you do once, commit, and forget.

Ask your agent to adopt the repository. This document is the instruction set it should follow.

## What ships

```
AGENTS.md              the agent contract — the one file every harness must load
PROCESS.md             the full process description
VISION.md              your project's vision (replace ours)
proposals/             one document per change, preserved as history
policies/              rules future work must keep enforcing
decisions/             architectural choices and the context that produced them
guides/                user-facing documentation
docs/                  the site — publishes the vision, the record, and the guides
tools/                 the artifact validator
.agents/skills/        one directory per phase of the process
.agents/rules/         code quality, commit discipline, enforcement hierarchy
.agents/templates/     the canonical shape of every artifact
.agents/hooks/         the commit-message git hook
.github/workflows/     continuous integration and the preview
```

`.agents/` stays the source of truth. When you adapt it for a harness, generate from it rather
than replacing it, so a second harness can be added later without archaeology.

## 1. Point your harness at `AGENTS.md`

Most current harnesses read `AGENTS.md` from the repository root without configuration. If yours
does, you are done with this step.

If yours expects a different filename, create a thin file that defers to it rather than copying
the content — two copies of a contract become two different contracts within a month.

For Claude Code:

```sh
printf '@AGENTS.md\n' > CLAUDE.md
```

For a harness without an import mechanism, symlink instead:

```sh
ln -s AGENTS.md .harness-instructions.md
```

## 2. Make the skills discoverable

`.agents/skills/<name>/SKILL.md` follows the widely-used skill convention: a `name` and a
`description` in frontmatter, procedural instructions in the body. The `description` is what the
agent sees when deciding whether to load the skill.

If your harness auto-discovers skills from a specific directory, make a symlink from that documented directory to `.agents/skills`. Keep `.agents/skills` canonical; the harness-specific location is configuration, not a second copy.

If it has no skill mechanism, that is not fatal — `AGENTS.md` names every skill by path at the
point in the process where it applies, so the agent can read them directly. Confirm your agent
actually follows those paths; if it does not, inline the two or three most important skill bodies
into `AGENTS.md`.

## 3. Wire the lifecycle events

Workbench needs three things to happen automatically. How you attach them is harness-specific;
what they run is not.

| When                           | Run                   |
| ------------------------------ | --------------------- |
| After the agent edits a file   | `mise run fmt <file>` |
| Before the agent ends its turn | `mise run check`      |
| At session start               | Load `AGENTS.md`      |

Install the commit-message hook, which is harness-independent and also catches commits you make
by hand:

```sh
ln -s ../../.agents/hooks/commit-msg .git/hooks/commit-msg
```

Continuous integration runs the same validation, so the hook is a fast local signal rather than
the only line of defence.

## 4. Choose your preview

A preview is the cheapest realistic artifact through which a human can exercise a change. What
that is depends on what your project is:

| Project           | Preview                                            |
| ----------------- | -------------------------------------------------- |
| Library           | prerelease package                                 |
| Web application   | preview deployment                                 |
| API               | preview endpoint deployment                        |
| Documentation     | rendered documentation build                       |
| Command-line tool | installable executable                             |
| Desktop           | installable build                                  |
| Mobile            | installable build, ad-hoc or internal distribution |

`.github/workflows/preview.yml` ships with a documentation deployment wired up, because that is
what Workbench itself is, plus commented recipes for the other shapes. Wire the row matching your
project and delete the rest.

**A preview has to be something you can actually open or install.** A downloadable build artifact
is not one — reviewing it would mean unzipping a file and starting a local server, which is enough
friction that the change gets reviewed as prose instead. The exception is a command-line tool,
where installing the binary is the real act.

The shipped default deploys to GitHub Pages, which requires enabling Pages with its source set to
the `gh-pages` branch, and which is public. **If your guides must stay private, do not use it** —
GitHub Pages serves privately only under Enterprise. Use the Cloudflare Pages recipe in the
workflow instead, which can gate the preview behind Cloudflare Access, and set
`CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`.

If your site is deployed to a subpath, make sure its base path is configurable at build time. The
shipped VitePress config reads `DOCS_BASE` for exactly this reason; without it every asset
resolves against the domain root and the preview loads blank.

## 5. Configure the service account

The readiness report is posted to the pull request by a service account rather than by you, so the
record shows which statements the agent made and which you made.

Create a bot account or a GitHub App, give it write access to the repository, and expose its token
to the agent's environment as `GH_TOKEN`. Without one, the agent posts from whatever identity is
available and says so in the report — degraded, not broken.

## 6. Clear the worked example

This repository carries its own record. Those files are real examples of what good output looks
like, which is the hardest thing to convey about a process — read them before deleting them.

When you are ready:

```sh
rm proposals/0001-*.md
```

Then rewrite `VISION.md` for your project, from `.agents/templates/VISION.md`. Do this with your
agent, in conversation, before any feature work — the vision is what proposals get evaluated
against, and an unwritten one gets silently replaced by the agent's assumptions.

Keep `policies/` and `decisions/` empty until a feature actually establishes one. Speculative
policies are the ones nobody follows.

## 7. Verify

```sh
mise install
mise run check
```

Then run one small feature end to end — branch, draft pull request, proposal, tests, guides, code,
review, readiness report. The first pass will expose whatever your harness does not do well, and
it is much cheaper to discover that on a small change.

## Adding a second harness

Repeat steps 1–3 for the new harness. Do not fork `.agents/`; generate the new harness's
configuration from it. If the two harnesses need genuinely different instructions, that difference
belongs in `AGENTS.md` as an explicit note, not in two divergent copies.
