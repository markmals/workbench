---
id: proposal.0007
title: Depend on Workbench Instead of Copying It
authors: [markmals, anthropic/claude-opus-5-5]
status: awaiting-implementation
pull-request: https://github.com/markmals/workbench/pull/10
issues: [workbench-0ky]
supersedes: []
---

# Depend on Workbench Instead of Copying It

## Summary

Ship Workbench as a versioned package: a Rust `workbench` CLI plus the plugin content that Claude Code, Oh My Pi, and later Pico load. Projects pin it in `mise.toml` and take updates through that pin, with no Node, TypeScript, or harness extensions.

## Motivation

Workbench reaches a project by being copied. A project starts from the GitHub template, or from a starter made with `cp -R`. From then on its `.agents/`, `tools/`, `.omp/`, `hk.pkl`, and `mise.toml` evolve separately from Workbench. Every later improvement has to be carried into each project by hand, and only into the projects that happen to be cloned on the machine doing the carrying.

The copies have drifted. An inventory of `~/Developer` on 2026-11-15 found nine Workbench projects:

- the three `agentic-projects` starters (web, cli, apple);
- Pico, Trove, Recon, Pitlane, Hive, and Bento.

Compared by size against 49 files the template ships, each project matched between 12 and 28 of them. The rest differed or were missing. The drift runs three ways:

- **Projects lag.** The starters' and Pico's `tools/status.ts` predate the template's current step model. Hive and Bento still run the tooling generation before Workbench 2.
- **Projects customize shared files in place:**
  - Trove appends its conventions to `code-quality.md`.
  - Trove and Pico add project rows to `verification.md`.
  - Pitlane and Bento add sections to `PROPOSAL.md`.
  - Recon replaced `check-before-stop.ts` with a no-op.
- **Projects improve shared files without sending the improvement back.** Pico's, Recon's, and Trove's validators each handle cases the template's does not.

None of this is visible from any single repository. Copying again cannot fix it without discarding the customizations.

The copy carries three further costs:

- **Node everywhere.** Workbench's own tooling requires Node, pnpm, and Bun in every project, including Swift and Rust projects that need none of them otherwise. That tooling is TypeScript scripts in `tools/`, a VitePress site, Oxfmt, Changesets, and an OMP tool whose test runs OMP's loader on Bun.
- **OMP packages everywhere.** The harness wiring is OMP extension code in TypeScript, so every project inherits OMP-specific packages.
- **A slower agent loop.** That wiring ran the full gate at the end of every turn. [Proposal 0006](0006-drop-harness-hooks-and-sync-beads.md) removed it, and this proposal keeps it out of the package.

The owner wants skills, rules, templates, the agent contract, and tools to update the way a dependency does, including in repositories that are not cloned locally. Changes to the record format should remain explicit migrations. They want Workbench's tooling in Rust, like their other CLIs, and want Workbench to require no TypeScript and no OMP-specific packages.

Workbench is also the process shared by the owner's holotype templates in `~/Developer/Templates/agentic-projects`: one template per kind of project the owner builds, with more planned. Each template owns its project's tooling, and Workbench supplies the process. Whatever tools Workbench does name must match the owner's stack (`~/Developer/Notes/sprinkles-stack.md`). In particular, Varlock replaces Fnox for environment variables and secrets.

## Domain grounding

### Established context

Claude Code (documentation read 2026-11-15, Claude Code 2.1.325):

- **Project-declared plugins.** A repository's `.claude/settings.json` can declare a marketplace in `extraKnownMarketplaces` and enable a plugin in `enabledPlugins`. Enabling does not install, with one exception: "Contributors don't need the install when a local `directory` or `file` marketplace lists the plugin with a relative-path `source`." Such a plugin loads in place at session start. A relative `directory` path "resolves against your repository's main checkout", so worktrees share it ([organization plugins](https://code.claude.com/docs/en/plugins/org), [plugin loading](https://code.claude.com/docs/en/plugins/loading)).
- **Trust.** Project marketplace entries are honored only after the user accepts the workspace trust dialog for that folder ([settings reference](https://code.claude.com/docs/en/settings-reference)).
- **No plugin rules or memory.** A plugin has no rules component. "Claude Code doesn't load a `CLAUDE.md` at the plugin root" ([plugin components](https://code.claude.com/docs/en/plugins/components)).
- **Output styles.** A plugin output style with `force-for-plugin: true` applies whenever the plugin is enabled. Claude Code sends the active style's instructions with every request. `keep-coding-instructions: true` keeps Claude Code's built-in engineering instructions ([output styles](https://code.claude.com/docs/en/output-styles)).
- **AGENTS.md.** Since 2.1.277 Claude Code reads `AGENTS.md` as project instructions when no `CLAUDE.md` exists, or through an `@AGENTS.md` import. It reads nothing under `.agents/` ([memory](https://code.claude.com/docs/en/memory)).
- **SessionStart limits.** SessionStart hook context over 10,000 characters is moved to a file behind a short preview. The docs advise writing hook context as factual statements rather than imperative instructions ([hooks](https://code.claude.com/docs/en/hooks)).

Oh My Pi (OMP 18.20.2 source and bundled documentation):

- **Project extension directories.** A committed `.omp/config.yml` `extensions:` entry naming a directory loads that directory's `skills/`, `rules/`, `hooks/pre|post/*.ts`, and `tools/`. Relative entries resolve against the session's working directory. A project `extensions:` list replaces the user-level list rather than extending it (`extension-loading.md`).
- **Ignored Claude configuration.** OMP does not read Claude's `hooks/hooks.json` or `extraKnownMarketplaces`.
- **Rule visibility.** A rule reaches the model only with `alwaysApply: true` (whole body in the system prompt), a `description` (listed, read on demand), or a trigger condition. Skills from an extension directory outrank project `.agents/skills`.
- **Native orchestration.** OMP ships workflow orchestration natively, through the `eval` kernel's `agent()` and `workpool()` helpers and the `workflowz` keyword (`magic-keywords.md`).

Pico (repository at `85a629f`, [pico#61](https://github.com/markmals/pico/issues/61), [pico#62](https://github.com/markmals/pico/issues/62)):

- **What loads today.** Pico embeds the OMP SDK. It loads a project's `AGENTS.md`, `.agents/skills/`, `.agents/rules/`, and project `.claude/` content. It loads no OMP extensions, custom tools, or hooks.
- **#61, plugin support (open).** Proposes installing Claude-format plugins, with project declarations, plugin-provided steering and rules, and translation of supported hook events.
- **#62, native workflows.** Builds dynamic workflows into Pico natively.

Tooling:

- **mise github backend.** mise's `github:` backend installs GitHub release assets and records per-platform URLs and checksums in `mise.lock` ([backends](https://mise.jdx.dev/dev-tools/backends/), [lockfile](https://mise.jdx.dev/dev-tools/mise-lock.html)).
- **Renovate.** Its mise manager updates `github:` backend pins. Its `mise.lock` refresh is documented, but users report it failing on hosted Renovate. jdx publishes a workflow that regenerates the lock on Renovate branches ([manager](https://docs.renovatebot.com/modules/manager/mise/), [renovate-config#19](https://github.com/jdx/renovate-config/pull/19)).
- **Knope change files.** Knope reads change files from `.changeset/` and deletes them after use. Each file maps packages to change types. Knope does not use a `config.json` ([change file](https://knope.tech/reference/concepts/change-file/)).
  - `PrepareRelease` bumps versioned files, writes the changelog, and stages the result. `[changes] ignore_conventional_commits` limits version sources to change files ([PrepareRelease](https://knope.tech/reference/config-file/steps/prepare-release/)).
  - Release notes support extra sections and change templates ([customizing release notes](https://knope.tech/recipes/customizing-release-notes/)).
  - The release-PR recipe keeps one release pull request open from `main`. Merging it runs `Release`, which creates the GitHub release and uploads assets before publishing ([recipe](https://knope.tech/recipes/1-preview-releases-with-pull-requests/), [Release](https://knope.tech/reference/config-file/steps/release/)).
  - `knope document-change` prompts interactively.
- **Single binaries.** [Hugo](https://gohugo.io), [Pagefind](https://pagefind.app), and [dprint](https://dprint.dev) are single binaries with no Node requirement.
- **Varlock** keeps a project's environment contract in a committed, typed `.env.schema`, with decorators such as `@sensitive` and `@required`. Values come from gitignored `.env` files or from plugins such as 1Password's, and `varlock run -- <command>` resolves and injects them ([Varlock](https://varlock.dev), [1Password plugin](https://varlock.dev/plugins/1password/)). It ships as a standalone binary in its GitHub releases. This machine runs 1.25.0 as `github:dmno-dev/varlock` through mise. Gourmand, Trove, and Vice already use it; Gourmand pins `npm:varlock`, which needs Node.
- **Agent-first CLIs.** The owner's guide (`~/Documents/Documentation/Programming/agentic-dev/agent-first-clis.md`) draws on Cloudflare's `cf` CLI. Its principle: "Make capabilities discoverable, inputs predictable, outputs structured, and effects verifiable."
  - One authoritative command registry generates the parser, help, discovery, and documentation.
  - Discovery and inspection commands (`cli search`, `schema`, `docs search`) never mutate anything.
  - The grammar is `<resource> <operation>`.
  - JSON is the default output on stdout, with diagnostics on stderr.
  - Errors are structured and carry stable codes.
  - Mutations offer `--dry-run`, and the current context can be inspected.
  - Commands never prompt.
  - The CLI is evaluated with agents that start with no knowledge of it.

  Pico's `pico` CLI already follows this shape (`pico schema`, `pico cli search`).

### Repository observations

Throwaway fixtures exercised the load-bearing claims on 2026-11-15 and were then deleted.

- **OMP 18.20.2 through `extensions:`.** OMP loaded a skill, an `alwaysApply` rule, and a `hooks/post/*.ts` factory from a directory named in `.omp/config.yml`. This held both when the directory sat behind a symlink inside a gitignored `node_modules` and when it was a `.workbench` symlink into mise's install directory. A rule without frontmatter, in a plugin's `rules/` or a project's `.agents/rules/`, never reached the model. Workbench's four rules have no frontmatter (Beads `workbench-0ky`).
- **Claude Code 2.1.325 with a directory marketplace.** The marketplace was declared at `./.workbench`, a symlink to a mise-installed directory outside the project.
  - `claude plugin details` listed its skill and SessionStart hook.
  - After workspace trust, a `-p` run executed that hook, a plain executable, with `CLAUDE_PLUGIN_ROOT` set to the symlink path. No install step ran.
  - Whether the model saw the plugin's content was not observed: Claude Code was not logged in.
- **mise 2026.11.8.**
  - It installed an archive with its directory tree intact and put its `bin/` on PATH.
  - It locked checksums for seven platforms.
  - A project `[hooks] postinstall` ran on every `mise install`, including when nothing new installed, and re-pointed a symlink after a version bump.
  - The `github:` backend stripped the top-level directory of a cargo-dist release archive (`github:max-sixty/worktrunk@0.81.0`).
- **Hugo 0.170.0.**
  - It built Workbench's proposals and guides in place, through module mounts of `../proposals`, `../guides`, and `../VISION.md`, in 13 ms.
  - It exposed custom frontmatter such as `id` and `status`.
  - A ten-line link template resolved `.md` links under a PR-preview base URL and failed the build on a dead link.
  - It loaded a theme through a symlinked `.workbench`.
- **Pagefind 1.6.0.** Installed as `github:CloudCannon/pagefind`, it indexed the output.
- **Zola 0.23.6** silently dropped custom top-level frontmatter.
- **dprint 0.64.3.** With Markdown `textWrap: "never"`, it unwrapped hand-wrapped prose and normalized frontmatter. hk 2.8.1 has builtins for `dprint`, `cargo_fmt`, `cargo_clippy`, and `taplo`.

### Relevant constraints and principles

The concepts this change introduces or renames:

| Concept             | Identity and lifetime                                                                                                                                                                                                                                                               |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Workbench package   | One published, immutable release of Workbench, identified by its version and tag `v<version>`: the CLI plus the plugin content, one archive per platform.                                                                                                                           |
| Plugin content      | The process contract, skills, rules, templates, and site theme inside the package. Harness-neutral Markdown plus the manifests that let harnesses find it.                                                                                                                          |
| Process contract    | The always-loaded agent instructions for the Workbench process. Ships in the package and changes with it.                                                                                                                                                                           |
| Project contract    | A project's `AGENTS.md`: facts about that project only. Owned and edited by the project.                                                                                                                                                                                            |
| Package pin         | The project's `github:markmals/workbench` version in `mise.toml`, with checksums in `mise.lock`. The single place a project names the Workbench version it uses.                                                                                                                    |
| Package link        | `.workbench`: a gitignored, machine-local symlink from the project root to the installed package for the pinned version. Derived state, recreated by `mise install`.                                                                                                                |
| Harness declaration | Committed `.claude/settings.json` and `.omp/config.yml` entries that point a harness at `.workbench`.                                                                                                                                                                               |
| Task interface      | The project tasks the process contract invokes: `setup`, `check`, and `fmt`, which each project implements. Workbench's own operations are `workbench` commands; the `status`, `validate`, `beads:sync`, `beads:claim`, and `beads:close` tasks wrap them for humans and git hooks. |
| Change file         | One Knope change file in `.changeset/`, describing one change for release notes. Knope calls the collection a changeset; Workbench says "change file" for one file.                                                                                                                 |

The ontology review is REVISE. Today the process and a project's copy of it are the same files. Nothing identifies which version of Workbench a project uses, so "update Workbench" has no referent. The missing concept is the package pin. The package link is derived from it and is never committed.

Other constraints:

- **Vision.** The [vision](../VISION.md) requires harness neutrality, mechanical enforcement over prose, and ceremony that earns its cost. It keeps the human as the final authority.
- **Record model.** [Proposal 0005](0005-current-specifications.md) defines the current-record model, and this proposal leaves it unchanged. [Record Validation and Workflow Status](../guides/record-validation.md) is the current contract for what record validation and workflow status detect.
- **Error handling.** Failures are surfaced, not hidden behind fallbacks.

### Quality bar

- A fresh clone with mise installed gets the exact Workbench version its pin names from `mise install`, plus workspace trust in Claude Code. Nothing else.
- One version identifies everything Workbench supplies to a project.
- A Workbench release reaches every project as a one-line, reviewable change that CI checks before merge.
- No tool Workbench requires needs Node or TypeScript.
- Workbench adds no work to the agent's turn loop.

### Remaining uncertainty

- **Claude Code delivery.** Not yet observed in a real model session:
  - a forced plugin output style delivering the contract;
  - plugin skill listings arriving through it;
  - whether the first session after trust needs `/reload-plugins`.
- **Claude and project symlinks.** Whether Claude Code follows committed `.claude/skills` and `.claude/rules` symlinks into `.agents/` is unverified.
- **Renovate lock updates.** Whether Renovate's `mise.lock` update succeeds for the `github:` pin, or needs the lock-regeneration workflow.
- **Locked platforms without assets.** How `mise lock` treats platforms with no matching asset, such as Windows. Projects may need to narrow the platforms they lock.
- **Pico.** Pico's support depends on #61, which is undesigned.

## Existing baseline

Workbench today is a GitHub template repository (`markmals/workbench`, `isTemplate: true`), last tagged `v1.0`. Its Workbench 2 generation was never released. The closest baseline is the template itself and the nine copies made from it.

Preserve:

- **The record model and lifecycle** from proposals 0001, 0003, 0004, and 0005.
- **The task names** every consumer already shares (`setup`, `check`, `status`, `validate`, `fmt`) and the single gate `mise run check`.
- **The record and status contract** in `guides/record-validation.md`, and the behavior covered by `tools/*.test.ts`.
- **The tool owners:** hk owns git hooks, including the Scoped Commits check against `.agents/commit-scopes`. Beads owns the work graph. Worktrunk owns worktrees.
- **CI** running the same gate.
- **The published site:** home, vision, process, guides, and the proposal, policy, and decision records with their metadata headers, historical proposal navigation, search, and PR previews under `/pr-preview/pr-<N>/`. The site build is the dead-link check.

Incidental, and replaced here:

- The Node, pnpm, and Bun toolchain.
- TypeScript tools and their Vitest suites, including the workflow tool's.
- The VitePress and VoidZero theme, and Oxfmt and Oxlint.
- Changesets with `@changesets/changelog-github`, and `tag-release.ts` and `project-version.ts`, which exist only to serve Changesets.
- Fnox, with `fnox.toml`, the `[secrets.fnox]` task grants, and the 1Password CLI pin it shells out to.
- The OMP `workflow` tool. [Proposal 0006](0006-drop-harness-hooks-and-sync-beads.md) removed the three OMP extensions and the pre-push check.
- The TypeScript Beads tools from proposal 0006 (`tools/bd.ts` and `tools/beads*.ts`). Their behavior is preserved; their implementation is not.
- The vendored `.agents/` copies.
- `guides/adapting-your-harness.md`, which asks each project to hand-wire its harness.

Compromised: the four rules lack frontmatter, so OMP never loads them (Beads `workbench-0ky`). Today the agent reaches them only through the path links in `AGENTS.md`.

## Proposed solution

Publish Workbench as releases of one package, and make every project a consumer of a pinned release rather than the owner of a copy.

```text
New project
mise use github:markmals/workbench@3 && mise install && workbench project init
  → pin in mise.toml, harness declarations, Workbench's tasks and checks, empty record
  → `mise run status` and `mise run validate` pass; the holotype template supplies the rest

Fresh clone of any Workbench project
mise install
  → .workbench links the pinned release → Claude Code and OMP load the contract, skills, and rules

Workbench release
Renovate bumps "github:markmals/workbench" in each repository → CI runs that release's validator
  → merge → git pull && mise install → the next session loads the new release
```

The package carries everything Workbench owns:

- the Rust `workbench` CLI, which replaces `tools/*.ts`;
- the process contract, skills, rules, and templates;
- the Hugo theme for the record site.

The project keeps everything it owns:

- its `AGENTS.md`, vision, records, and guides;
- its own skills and rules under `.agents/`;
- its mise tasks, hk hooks, CI, and Knope configuration.

Three committed lines of harness declaration replace per-harness wiring. Harness hooks, the workflow tool, Changesets, VitePress, and Oxfmt go away, and so does every Node requirement.

## Detailed design

### 1. The Workbench package

A Workbench release is identified by a semantic version and the Git tag `v<version>`. It is published as a GitHub release of `markmals/workbench`. The release carries one archive per supported target, named `workbench-<target>.tar.gz`, where `<target>` is `aarch64-apple-darwin`, `x86_64-apple-darwin`, `aarch64-unknown-linux-musl`, or `x86_64-unknown-linux-musl`. Each archive holds one top-level directory, `workbench-<target>/`, whose contents are the package root:

| Path                                                                                                             | Content                                                                                                                                                                              |
| ---------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `workbench`                                                                                                      | The CLI ([section 5](#5-the-workbench-cli)).                                                                                                                                         |
| `.claude-plugin/plugin.json`                                                                                     | Claude Code manifest: `name: "workbench"`, `version` equal to the release version, `description`, `repository`.                                                                      |
| `.claude-plugin/marketplace.json`                                                                                | A marketplace named `workbench` listing one plugin, `workbench`, with source `"./"`.                                                                                                 |
| `rules/workbench.md`                                                                                             | The process contract (section 4), with frontmatter `alwaysApply: true`.                                                                                                              |
| `rules/code-quality.md`, `rules/commit-discipline.md`, `rules/enforcement-hierarchy.md`, `rules/verification.md` | Today's four rules, each given a one-sentence `description` frontmatter field.                                                                                                       |
| `output-styles/workbench.md`                                                                                     | The process contract for Claude Code: the same body as `rules/workbench.md`, with frontmatter `name`, `description`, `force-for-plugin: true`, and `keep-coding-instructions: true`. |
| `skills/<name>/`                                                                                                 | Today's twelve skills and their supporting files.                                                                                                                                    |
| `templates/`                                                                                                     | Today's artifact templates.                                                                                                                                                          |
| `site/`                                                                                                          | The Hugo theme ([section 9](#9-the-documentation-site)).                                                                                                                             |

Every archive in a release carries identical content apart from the `workbench` binary. Each archive has a GitHub build-provenance attestation.

**Versioning.**

- **Major** for any change a project must act on:
  - the record schema `validate` accepts;
  - the task interface;
  - the files a project must hold, or their layout;
  - a removed or redefined CLI command, flag, exit status, or JSON field.
- **Minor** for added commands, skills, rules, templates, or contract guidance.
- **Patch** for corrections.

A major release's notes describe the migration in their Breaking changes section, and `validate` rejects obsolete forms with migration guidance, as [proposal 0005](0005-current-specifications.md) established. The first package release is `3.0.0`, after the `v1.0` tag and the unreleased Workbench 2 generation.

### 2. Declaring the dependency in a project

A project commits these entries:

```toml
# mise.toml
[tools]
"github:markmals/workbench" = "3.0.0"

[hooks]
postinstall = 'ln -sfn "$(mise where github:markmals/workbench)" "{{config_root}}/.workbench"'
```

```text
# .gitignore
.workbench
```

`.claude/settings.json`:

```json
{
    "extraKnownMarketplaces": {
        "workbench": { "source": { "source": "directory", "path": "./.workbench" } }
    },
    "enabledPlugins": { "workbench@workbench": true }
}
```

```yaml
# .omp/config.yml
extensions:
    - ./.workbench
```

`mise.lock` records the URL and checksum of each locked platform's archive.

- **`.workbench` is derived and machine-local.** Every `mise install` re-points it at the install for the pinned version. That includes the one inside `mise run setup`, Worktrunk's `pre-start` hook, and CI's `jdx/mise-action`.
- **Changing the pin.** After the pin changes and `mise install` runs, new sessions load the new release. A running session keeps what it loaded.
- **When the link is missing.** If `.workbench` is missing or dangling, a harness loads no Workbench content. Tasks that call `workbench` fail with mise's missing-tool error. The project's `AGENTS.md` therefore states that Workbench loads from `.workbench` and that `mise install` restores it, so an agent without the contract still learns the fix.
- **No other files.** Workbench requires no `package.json`, no Node runtime, and no harness-specific package.

### 3. How each harness loads the package

| Content                                | Claude Code                                                      | Oh My Pi                                                 | Pico                                  |
| -------------------------------------- | ---------------------------------------------------------------- | -------------------------------------------------------- | ------------------------------------- |
| Process contract                       | `output-styles/workbench.md`, forced while the plugin is enabled | `rules/workbench.md`, `alwaysApply` in the system prompt | Plugin rules, once #61 supports them  |
| Workbench rules                        | Linked by path from the contract (`.workbench/rules/<name>.md`)  | Listed by description; read on demand                    | Plugin rules, once #61 supports them  |
| Workbench skills                       | `workbench:<name>`, descriptions listed                          | Bare names, descriptions listed                          | Plugin skills, once #61 supports them |
| Templates and skill resources          | Read at `.workbench/...`                                         | Read at `.workbench/...`                                 | Read at `.workbench/...`              |
| Project `AGENTS.md`                    | Native, when no `CLAUDE.md` exists                               | Native                                                   | Native                                |
| Project skills and rules in `.agents/` | Through committed `.claude/skills` and `.claude/rules` links     | Native                                                   | Native                                |

- The package declares no hooks, tools, commands, MCP servers, or agents for any harness.
- Pico needs no Pico-specific file. When #61 lands, it reads the same `.claude/settings.json` declaration.
- Skills and templates name their resources by project-relative path (`.workbench/templates/PROPOSAL.md`, `.workbench/skills/specifying-behavior-with-gherkin/examples.md`). They name other skills by name. A project override `.agents/templates/<name>` takes precedence over `.workbench/templates/<name>`, and the skills say so.
- **Name collisions.** A project skill or rule must not reuse a Workbench skill or rule name. OMP ranks the package above `.agents/skills`, while Claude Code loads both, so an override would behave differently per harness. `workbench project validate` enforces this ([section 5](#5-the-workbench-cli)).

### 4. The process contract and the project's AGENTS.md

`rules/workbench.md` holds what today's `AGENTS.md` says about the process for any project:

- the proposal question and the look-and-feel exception;
- proposal status ownership;
- the implementation steps and the rules;
- Beads usage;
- the task interface;
- the roles of mise, hk, Varlock, Beads, Knope, and Worktrunk;
- delegation guidance;
- links to the four rules and the reasoning skills.

It names skills by name and package files by `.workbench/...` paths. It contains no harness-specific instruction. Its first instruction is to run `workbench workflow status` at the start of every session; humans run `mise run status` for the same report as text. It teaches the CLI's workflow (discover with `workbench cli search` and `workbench schema`, preview with `--dry-run`, execute, then verify from the receipt) and does not reproduce the command reference.

A project's `AGENTS.md` holds only that project:

- its purpose and layout;
- commands beyond the task interface;
- platform rules and preview;
- the `.workbench` line from [section 2](#2-declaring-the-dependency-in-a-project).

Workbench's own `AGENTS.md` becomes the facts of this repository: its crate, package layout, release, and site.

The body of `output-styles/workbench.md` after its frontmatter must equal the body of `rules/workbench.md`. Workbench's test suite fails when they differ. A project that keeps a `CLAUDE.md` must import `@AGENTS.md` from it, or Claude Code ignores `AGENTS.md`. The adoption guide says so.

### 5. The `workbench` CLI

`workbench` is a Rust binary, statically linked for each target. Following the owner's Rust CLI stack, it is built with Clap, tested with `cargo test`, formatted with rustfmt, and linted with Clippy. It needs `git` at run time, plus `gh` or `bd` for the commands that use them. It resolves the project root as the Git top level of the current directory, never from its own location.

Its primary consumer is an agent, so it follows the owner's agent-first CLI guide: capabilities are discoverable, inputs predictable, outputs structured, and effects verifiable. Humans get text output on request, and through the mise tasks.

#### One command registry

Every operation is declared once, in a registry inside the crate. Each entry gives:

- the operation's stable identifier and command path;
- a one-line summary and a description;
- each argument and option, with its type, whether it is required, its default, and its accepted values;
- a JSON Schema for its output;
- whether it reads or modifies state, and what;
- what it requires (`git`, `gh` authentication, `bd`, the network);
- whether it supports `--dry-run` and whether it is idempotent;
- the error codes it can return;
- at least one example.

The registry generates the Clap parser, `--help`, shell completions, `workbench schema`, the `workbench cli search` index, and the CLI reference page in Workbench's guides. Tests fail when these drift apart:

- every Clap argument appears in the registry, and every registry entry parses;
- every registered example runs against a fixture project, and its output validates against the declared output schema;
- every declared default is the runtime default.

#### Command tree

The grammar is `workbench <resource> <operation> [arguments] [options]`, in kebab-case. Collections are plural (`records`, `commits`, `change-files`). Singletons are singular (`project`, `workflow`, `context`). `beads` names the Beads database.

| Command                                                                                                                 | Effect                                                       | Replaces                                                       |
| ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------ | -------------------------------------------------------------- |
| `workbench cli search <query>`                                                                                          | Reads                                                        | —                                                              |
| `workbench cli completions <shell>`                                                                                     | Reads                                                        | —                                                              |
| `workbench schema [<command>]`                                                                                          | Reads                                                        | —                                                              |
| `workbench docs search <query>`                                                                                         | Reads                                                        | —                                                              |
| `workbench context show`                                                                                                | Reads                                                        | —                                                              |
| `workbench workflow status [--no-sync]`                                                                                 | Reads; syncs Beads unless `--no-sync`                        | `tools/status.ts` and proposal 0006's Beads section            |
| `workbench records validate`                                                                                            | Reads                                                        | `tools/validate.ts`                                            |
| `workbench project validate`                                                                                            | Reads                                                        | —                                                              |
| `workbench project init [--site github-pages\|cloudflare] [--dry-run]`                                                  | Creates files ([section 11](#11-adopting-workbench))         | The GitHub template and Adapting Your Harness                  |
| `workbench commits check --message-file <path>` and `workbench commits check <base> <head>`                             | Reads                                                        | `tools/check-scoped-commit.ts` and `tools/check-pr-commits.ts` |
| `workbench change-files create --type <type> --summary <text> [--package <name>] [--body <text\|@file\|->] [--dry-run]` | Creates one file ([section 8](#8-change-files-and-releases)) | Hand-written change files                                      |
| `workbench change-files check <base> <head>`                                                                            | Reads                                                        | `tools/require-changeset.ts`                                   |
| `workbench beads sync [--dry-run]`                                                                                      | Syncs the Beads replica                                      | Proposal 0006's `beads:sync`                                   |
| `workbench beads claim <id> [--dry-run]`                                                                                | Claims an item and syncs                                     | Proposal 0006's `beads:claim`                                  |
| `workbench beads close <id> --reason <text> [--dry-run]`                                                                | Closes an item as the claimant and syncs                     | Proposal 0006's `beads:close`                                  |

The mise tasks wrap these commands for humans and hooks:

- `status` runs `workbench workflow status --output text`.
- `validate` runs `workbench records validate --output text` and `workbench project validate --output text`.
- `beads:sync`, `beads:claim`, and `beads:close` call their commands.

hk's `commit-msg` hook runs `workbench commits check --message-file {{commit_msg_file}}`.

#### Discovery and progressive disclosure

- **`workbench --help`** orients: what Workbench is, the discovery commands, and the resource groups. It is not the full option reference.
- **`workbench cli search "<task>"`** runs locally without credentials. It ranks commands by name, summary, description, option metadata, and synonyms the registry declares. It returns at most five matches by default (`--limit` changes that), as JSON `items` with the exact `command`, its `summary`, its `effect`, and a `score`.
- **`workbench schema`** with no argument lists every command path with its summary. `workbench schema <command>` returns that command's full registry entry.
- **`workbench docs search "<query>"`** searches the package's Markdown: the contract, rules, skills, templates, and the CLI reference. It returns `items` with a `path` under `.workbench/`, a `title`, and a `snippet`; the agent reads the file for detail.
- **`workbench --version`** prints the version.

None of these mutates anything.

#### Output

- **JSON by default.** Results are JSON on stdout: compact when stdout is not a terminal, indented when it is. `--output text` gives a human-formatted rendering of the same result. The JSON contract does not depend on color or terminal width.
- **Standard streams.** stdout carries only the result. Progress, warnings, and diagnostics go to stderr.
- **Collections** return `items`, plus `hasMore` and `nextCursor` whenever the command bounds the result. No command truncates a collection silently.
- **Receipts.** A modifying command returns a receipt with:
  - its `status`;
  - its target: the project root, plus the Beads remote for Beads commands;
  - the identifiers it produced (file paths, item IDs);
  - the fields that matter for the operation, such as a claim's `claimant` and `published`, or a close's `published`.

#### Errors and exit statuses

A failure writes `{"error": {"code", "message", "retryable", "details", "suggestions"}}` to stderr. A check that ran and found violations is a failure too, but its result, listing the violations, still goes to stdout.

| Code                  | Exit | Retryable | Meaning                                                                                                                                                                                                                      |
| --------------------- | ---- | --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `CHECK_FAILED`        | 1    | No        | A check ran and found violations.                                                                                                                                                                                            |
| `INVALID_ARGUMENT`    | 2    | No        | Input is missing or malformed. `details` lists the missing fields and accepted values.                                                                                                                                       |
| `NOT_FOUND`           | 3    | No        | A named record, item, commit, or file does not exist.                                                                                                                                                                        |
| `PRECONDITION_FAILED` | 3    | No        | The project is not ready: not a Git repository, no pin, no `.workbench` link, Beads not initialized, or a Beads remote that is not the checkout's `origin` (or no `origin` to compare it with). `suggestions` names the fix. |
| `DEPENDENCY_MISSING`  | 3    | No        | A required program (`git`, `gh`, `bd`) is absent.                                                                                                                                                                            |
| `UNAUTHENTICATED`     | 3    | No        | `gh` or Git credentials are missing or rejected.                                                                                                                                                                             |
| `CONFLICT`            | 4    | No        | Another claimant holds the item, so a claim or close was refused; a claim was lost; or a Beads sync stopped on a conflict for the agent to resolve.                                                                          |
| `TIMEOUT`             | 5    | Yes       | An operation exceeded its time limit.                                                                                                                                                                                        |
| `UNAVAILABLE`         | 5    | Yes       | The remote or the network was unreachable.                                                                                                                                                                                   |
| `INTERNAL`            | 70   | No        | A defect in `workbench` itself.                                                                                                                                                                                              |

- **Exit status 0** means the operation completed.
- **Partial completion is reported, not hidden.** A claim or close made locally whose sync failed exits 0, but its receipt says `published: false`, and stderr says why.
- **`workflow status`** reports a Beads sync failure inside its result and still exits 0, as in proposal 0006.

#### Preview, safety, and verification

- **`--dry-run`** is available on every modifying command. A dry run validates input, resolves the target, and describes the changes without making them:
  - for `project init`, the files it would create or add to;
  - for `change-files create`, the file it would write;
  - for `beads claim`, the item, its current claimant, and the claimant it would claim as;
  - for `beads close`, the item, its current claimant, and the claimant it would close as;
  - for `beads sync`, the remote and any unpublished local commits.

  A dry run states what it cannot predict; a sync, for example, cannot predict merge conflicts.

- **No destructive operations.** `project init` never overwrites or deletes, so no command needs a confirmation flag.
- **Verification.** Receipts return stable identifiers. `beads claim` reads the claimant back after syncing. After `project init`, `context show` and `project validate` read back the resulting setup.

#### Context

`workbench context show` returns:

- the project root, Git branch, and `origin`;
- the pull request, or `unknown` with the reason;
- the pinned Workbench version, the running binary's version, and where `.workbench` points;
- which harness declarations exist;
- the Beads workspace, its remote, and the claimant identity;
- the site host configuration.

`workbench project validate` reports problems in that setup as `CHECK_FAILED` violations:

- a `.workbench` link pointing at a different version than the pin;
- missing harness declarations;
- name collisions;
- vendored Workbench files.

#### Input and unattended use

- **Input.** Scalar inputs are named flags. `--body` accepts text, `@file`, or `-` for stdin.
- **No prompts.** No command prompts or waits for input. Missing required input fails at once with `INVALID_ARGUMENT`, listing the missing fields and the values each accepts.
- **Time limits.** Every network operation has one (20 seconds for a Beads sync, as in proposal 0006), and `--timeout` overrides it.
- **Credentials.** `workbench` takes no credentials as arguments. It relies on `gh`'s and Git's own credential stores, and never prints credentials in results, errors, or dry runs.
- **The Dolt info branch.** The CLI sets `DOLT_REMOTE_INFO_BRANCH` to empty for every `bd` and `dolt` process it starts. No Workbench sync publishes the info branch, whatever the caller's environment, so a project on the package needs no `[env]` entry for it.

#### Behavior carried over

`records validate`, `commits check`, `change-files check`, `workflow status`, and the `beads` commands detect the same conditions as today's tools and proposal 0006. Every case in `tools/*.test.ts` and 0006's tests for a retained behavior becomes a Rust test. Their output follows this section instead of today's formats. `tag-release.ts` and `project-version.ts` are removed; Knope tags releases and owns versions.

- **Commit scopes.** `commits check` accepts three kinds of scope:
  - the built-in process scopes `agents`, `ci`, `decision`, `deps`, `docs`, `guides`, `policy`, `proposal`, `release`, `rules`, `skills`, `templates`, `treewide`, and `vision`;
  - the scopes in `.agents/commit-scopes`, which is optional and keeps its current format;
  - the `<NNNN>-<slug>` scope of any existing proposal file.

  A rejection lists every accepted scope.

- **The collision rule in `project validate`.** A project skill directory `.agents/skills/<name>/` or rule `.agents/rules/<name>.md` whose name matches a skill or rule shipped in the same release is a violation. The message names the file and says to remove or rename it. The binary embeds the names of its release's skills and rules, so the check does not depend on `.workbench`.

- **Beads.** The `beads` commands and the Beads part of `workflow status` keep what proposal 0006 established:
  - **The claimant.** Claims and closes act as `BEADS_ACTOR`, or `<login user>@<short host name>` when it is unset. `bd close` refuses an item claimed by another actor, which is why closing has a command.
  - **Only the project's own remote.** No sync, claim, or close runs when the replica's Dolt remote is not the same repository as the checkout's `origin`, or when there is no `origin`. Hosted URLs compare across SSH, HTTPS, `git+`, credentials, `.git`, and case; local paths compare exactly, resolving relative origins from the checkout.
  - **The first publish.** When `bd sync` fails and the remote positively has no `refs/dolt/data`, the sync publishes with `bd dolt push`.
  - **Conflicts.** Embedded storage aborts a halted merge, so a conflict's `details` name the conflicted tables Beads reports and the items that differ from the remote (`bd diff main origin/main`), within the sync's time limit.
  - **Reads after the sync.** `workflow status` syncs before reading ready work, items in progress, and memories. When a read fails, the result says which, and keeps the sync's outcome.

#### Evaluating the CLI with agents

Before `3.0.0` is released, fresh agent sessions attempt five tasks in a fixture project. Each session starts with no Workbench context beyond the output of `workbench --help`:

1. find and claim the next ready item, then close it;
2. add a change file for a fix;
3. find out why `records validate` fails, and repair the record;
4. set up Workbench in an empty repository and verify the setup;
5. recover from a Beads sync attempted without network access.

Each run records task completion, invalid commands, the number of commands, and whether the agent verified its result. Failures caused by the CLI's interface are fixed before release.

### 6. No harness hooks

[Proposal 0006](0006-drop-harness-hooks-and-sync-beads.md) removed the three OMP extensions and the pre-push check, moved Beads orientation into `mise run status`, and defined the Beads sync points: `mise run status`, hk's `pre-push`, `post-merge`, and `post-rewrite` after a rebase, and the `beads:sync`, `beads:claim`, and `beads:close` tasks. This proposal keeps them. The package declares hooks for no harness.

### 7. No workflow tool

- Delete `.omp/tools/dynamic-workflows/` and its Vitest suites, the Bun pin, and the `@oh-my-pi/pi-coding-agent` dependency.
- Rewrite the "Agent orchestration" section of `PROCESS.md` and the delegation paragraph of the contract without naming a tool:
  - Work inline by default.
  - Delegate independent slices that touch separate files, or a fresh-context review, using the harness's own delegation.
  - Pass a delegate bounded ownership.
  - Treat a failed delegate's missing result as missing, not as success.
- Workflow orchestration now lives in OMP's native helpers and in Pico (#62). A comment on pico#62 records the last revision containing the tool, [`8988abd`](https://github.com/markmals/workbench/tree/8988abd700fad9164f4c597dff761425912d7d02/.omp/tools/dynamic-workflows), for reference.

### 8. Change files and releases

This section replaces the "Changeset" section of `PROCESS.md` for every Workbench project.

- **Every pull request adds at least one change file** in `.changeset/`, describing the change for someone reading release notes rather than for the reviewer of the diff. The only exception is a pull request with no user-visible change, which carries a `Release-Note: none` trailer instead.
  - A change file is Markdown with frontmatter mapping each affected package to a change type. A single-package project uses `default: <major|minor|patch>`; a project's `knope.toml` may add types.
  - The first heading is the summary. Text below it is the detail.
  - Agents create change files with `workbench change-files create`, which validates the type and package against `knope.toml` and writes the file. They never run `knope document-change`, which prompts interactively.
- **`.changeset/` holds only change files**, because Knope fails on any other Markdown file there. Guidance that lives in `.changeset/README.md` today moves to `PROCESS.md` and the guides.
- **`workbench change-files check <base> <head>`** passes when either:
  - the range adds a file under `.changeset/` with valid change-file frontmatter; or
  - a commit in the range carries the Git trailer `Release-Note: none`. The key is case-insensitive and the value is exactly `none`, as `git interpret-trailers` parses it.

  Otherwise it fails with `CHECK_FAILED`, naming both ways to satisfy it. CI runs it on every pull request except Knope's release pull request.

- **A pull request with no user-visible change** carries `Release-Note: none` on one of its commits (`git commit --trailer "Release-Note: none"`) instead of a change file. The trailer replaces Changesets' `--empty` changeset: it can be checked offline, and it stays in history next to the change it describes.
- **A project's `knope.toml` is project-owned.** It sets `[changes] ignore_conventional_commits = true`. Scoped Commits subjects are not Conventional Commits, and versions come from change files only.
- **Releases follow Knope's release pull request recipe:**
  - A workflow on every push to `main` runs Knope's prepare-release workflow. It keeps one release pull request open showing the next version, the changelog entry, and the version-file changes.
  - Merging that pull request runs the release workflow. It builds the project's artifacts, then runs `knope release`, which tags `v<version>`, creates the GitHub release with the changelog entry as its notes, and uploads the artifacts before publishing.
  - Merging the release pull request is the explicit release authorization the process requires.
- **Workbench's own `knope.toml`:**
  - `versioned_files = ["Cargo.toml", "Cargo.lock"]` and `changelog = "CHANGELOG.md"`.
  - `assets` naming the four archives.
  - `[github]` owner `markmals` and repo `workbench`.
  - `[changes] ignore_conventional_commits = true`.
  - GitHub-style release notes:

    ```toml
    [release_notes]
    change_templates = [
        "### $summary\n\n$details",
        "* $summary by @$pr_author_login in #$pr_number",
        "* $summary",
    ]
    ```
- **Getting to the first release.** Workbench's `Cargo.toml` starts at version `2.0.0`, and this proposal's change file is `major`, so the first release is `3.0.0`. The four pending Changesets entries become Knope change files. `.changeset/config.json` and `.changeset/README.md` are removed.

### 9. The documentation site

- **The theme ships in the package.** The package's `site/` directory is a Hugo theme. A project's `docs/hugo.toml` selects it with `themesDir = "../.workbench"` and `theme = "site"`, and mounts the record files where they live. Nothing is copied and no generated content is committed. A project's `docs/layouts/` and `docs/static/` override the theme.
- **Pages.**
  - The site publishes the same pages as today: home, `VISION.md`, `PROCESS.md`, the guides, and the proposal, policy, and decision records. Each directory README serves as its section's index.
  - Record pages show the metadata today's header shows: `id`, `status`, `authors`, `pull-request`, `issues`, `supersedes`, and `established-by`.
  - Proposals are presented as history, with superseded proposals grouped apart.
  - `[NEEDS CLARIFICATION: …]` markers in proposal prose are highlighted.
- **Links.**
  - A relative link to a published Markdown file resolves to its page, keeping any anchor.
  - A relative link to an unpublished repository file resolves to that file on the repository named by `params.repository`.
  - A relative link to a file that does not exist fails the build, naming the file that contains it. This keeps the site build as the dead-link check inside `mise run check`.
- **Builds and tools.**
  - Each build takes its base URL from the build command, so a preview can be served under any path, including GitHub Pages' `/pr-preview/pr-<N>/`.
  - Pagefind indexes the built site, and the theme offers search over that index.
  - A project pins `hugo` and `github:CloudCannon/pagefind`. The theme declares its minimum Hugo version and fails with that requirement when built with an older one.
  - `docs:build` runs Hugo, then Pagefind. `docs:dev` serves the site with reload; search needs a build.
- **Hosting.** A project publishes its site to GitHub Pages or to Cloudflare; each suits different projects, so Workbench supports both.
  - On either host, `main` publishes the site. Each pull request gets a preview at its own URL, linked from the pull request and removed when the pull request closes.
  - `workbench project init --site github-pages` or `--site cloudflare` writes that host's publishing and preview workflows ([section 11](#11-adopting-workbench)).
  - Cloudflare publishing uses Cloudflare's GitHub Action, which runs Wrangler on the CI runner only; the project still needs no Node. It needs `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` as repository secrets.
  - Workbench's own site stays on GitHub Pages.
- **Look and feel** are left to the owner on the preview, per [proposal 0004](0004-make-room-for-ui-polish.md). Matching the VitePress theme is not a requirement.

### 10. Formatting

- **In Workbench.** dprint formats Markdown with `textWrap: "never"`, plus JSON, YAML (including frontmatter), and TOML. hk's `dprint`, `cargo_fmt`, and `cargo_clippy` builtins replace the Oxfmt, Oxlint, and `tsrs` steps. The whitespace, newline, merge-conflict, and `actionlint` steps remain. No formatter checks Hugo templates or CSS.
- **In projects.** Formatting is project-owned. Each holotype template chooses its formatters, for example Oxfmt in TypeScript projects. `workbench project init` writes no formatter configuration.

### 11. Adopting Workbench

`workbench project init` adds Workbench to a Git repository: a new project, a holotype template, or an existing project. It writes only what Workbench owns. Formatters, tests, builds, releases, and environment schemas belong to the project or its holotype template. It runs where `mise.toml` already pins `github:markmals/workbench` (`mise use github:markmals/workbench@<version>` writes that pin). It creates each missing item and never overwrites an existing file. For structured files it adds only missing keys or entries:

- **`mise.toml`:**
  - the `postinstall` link hook;
  - pins for `hk`, `gh`, and Beads when absent, at the versions the release was tested with;
  - the tasks `status`, `validate`, `beads:sync`, `beads:claim`, and `beads:close` when absent.
- **Ignore and harness files:**
  - the `.gitignore` entry;
  - the `.claude/settings.json` keys;
  - the `.omp/config.yml` extension entry, appended to an existing list;
  - `.claude/skills` and `.claude/rules` links to `../.agents/skills` and `../.agents/rules`, when those directories exist and the links do not.
- **Hooks:** `hk.pkl` when absent. Its `commit-msg` hook runs `workbench commits check --message-file`, and Beads gets proposal 0006's sync steps (`pre-push`, `post-merge`, and `post-rewrite` after a rebase), which call `workbench beads sync`. It runs no check before a push.
- **Workbench's checks:** `.github/workflows/workbench.yml` when absent. It runs `workbench commits check` and `workbench change-files check` on every pull request except Knope's release pull request.
- **`AGENTS.md`** when absent, as a project stub with the `.workbench` line. When `AGENTS.md` exists without that line, the receipt says so.
- **Records:**
  - `VISION.md` from the template when absent;
  - `proposals/`, `policies/`, `decisions/`, and `guides/` with their READMEs when absent;
  - `.agents/commit-scopes` when absent;
  - `docs/hugo.toml` when absent.
- **With `--site github-pages` or `--site cloudflare`:** pins for `hugo` and `github:CloudCannon/pagefind`, the `docs:build` and `docs:dev` tasks, and that host's publishing and preview workflows, each when absent.

Its receipt lists:

- what it created or added to;
- what it left unchanged;
- every Workbench file the project still vendors;
- the project-owned pieces Workbench relies on but does not create: the `setup`, `check`, and `fmt` tasks, and a release flow that consumes change files with Knope.

The vendored files it lists cover:

- Workbench-named skills and rules under `.agents/`;
- templates under `.agents/templates/`, marked identical (removable) or different (an override);
- the three OMP extensions and `.omp/tools/dynamic-workflows/`;
- the TypeScript tools the CLI replaces;
- `.changeset/config.json`, `docs/.vitepress/`, and `fnox.toml`.

`project init --dry-run` returns the same receipt without writing anything. `project init` deletes nothing. It exits 0 unless it could not write a file.

`guides/adopting-workbench.md` replaces `guides/adapting-your-harness.md`. It covers starting a project and migrating an existing one:

1. Pin Workbench, run `workbench project init --dry-run` to see what it will do, then run it.
2. Move project additions out of shared files into project rules, project skills, or template overrides.
3. Delete the vendored copies the receipt lists.
4. Convert pending Changesets entries to change files, and move the release flow to Knope.
5. Move a published site to Hugo on GitHub Pages or Cloudflare.
6. Replace Fnox with Varlock: declare variables in `.env.schema`, and run tasks that need secrets under `varlock run --`.
7. Run `mise run check`.
8. Optionally enable Renovate for the pin, with lock regeneration.

It also covers the harness caveats: Claude Code's workspace trust, `CLAUDE.md` imports, and worktrees resolving to the main checkout; OMP's project `extensions:` replacing the user-level list; and Pico's dependence on #61.

The Workbench repository stops being a GitHub template repository; the owner changes that setting. `README.md` and the getting-started guide describe `mise use` and `workbench project init` in its place. Their step for replacing Workbench's `.beads/` in a copy of the template goes with the template.

### 12. The Workbench repository

- **Layout.**
  - The Rust crate sits at the root: `Cargo.toml`, `Cargo.lock`, `src/`, and `tests/`, with the toolchain pinned through mise.
  - Package content lives under `plugin/`: `.claude-plugin/`, `rules/`, `output-styles/`, `skills/`, `templates/`, and `site/`.
  - A release archive is the binary plus the contents of `plugin/`.
- **Dogfooding.**
  - `mise run setup` links `.workbench` to `plugin` and builds the CLI.
  - Workbench's `mise.toml` puts the build's output directory on `PATH`. In mise contexts, `workbench` therefore resolves to the working tree's build here and to the pinned release in a consumer.
  - Its `.claude/settings.json` and `.omp/config.yml` match a consumer's.
  - Its `hk.pkl` matches what `project init` writes, because it calls `workbench` the same way.
- **Removed:**
  - the npm and TypeScript toolchain: `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`, `tsconfig.json`, `tools/`, `.oxfmtrc.jsonc`, `.oxlintrc.jsonc`;
  - `.omp/`;
  - the VitePress site: `docs/.vitepress/` and the Vue home page;
  - Changesets: `.changeset/config.json` and `.changeset/README.md`;
  - the process files under `.agents/`, which move to `plugin/`.
- **Kept.** `.agents/commit-scopes` keeps Workbench's repository-only scopes.
- **`mise.toml`** drops `node`, `pnpm`, `bun`, `fnox`, and `1password`, and its `[secrets.fnox]` block; `fnox.toml` is deleted. It also drops the `DOLT_REMOTE_INFO_BRANCH` `[env]` entry, which the CLI makes redundant. It adds Rust, `hugo`, `github:CloudCannon/pagefind`, `knope`, and `dprint`. It keeps `hk`, `gh`, `actionlint`, `worktrunk`, and Beads. Workbench needs no environment variables or secrets, so it has no `.env.schema`.
- **`mise run check`** runs the Rust tests, `hk check --all`, `validate`, and `docs:build`.
- **Workflows:** CI (check, commit subjects, change files), docs publishing, PR previews, prepare-release, and release.
- **Documents updated with the change:**
  - `AGENTS.md`, `PROCESS.md`, and `README.md`;
  - the guides that name paths, tools, or Changesets;
  - the skills and templates, for paths and skill names;
  - `.github/pull_request_template.md`.

  `VISION.md` is revised at completion.

## Acceptance criteria

This change touches many independent surfaces, so its outcomes are listed for review:

- [ ] In a fixture repository with no Node on PATH, pinning a published release and running `mise install` creates `.workbench`. A new OMP session and a new Claude Code session each receive the process contract and list Workbench's skills, and OMP lists the four rules.
- [ ] Changing the pin and running `mise install` re-points `.workbench`, and the next session in each harness loads the new release.
- [ ] `workbench records validate`, `commits check`, `change-files check`, `workflow status`, and the `beads` commands pass the cases ported from `tools/*.test.ts` and proposal 0006's tests.
- [ ] Every registered command example runs and its output validates against its declared schema. Every Clap argument is in the registry. `cli search`, `schema`, and `docs search` work offline, without credentials. Every failure writes the structured error with its documented code and exit status.
- [ ] `workbench project validate` reports a project skill or rule that reuses a Workbench name, and a `.workbench` link that disagrees with the pin.
- [ ] `workbench change-files check` passes a range that adds a valid change file, passes a range with a `Release-Note: none` trailer, and fails a range with neither.
- [ ] In the agent evaluation, fresh sessions complete all five tasks using only discovery, without guessing at command syntax.
- [ ] Merging Knope's release pull request publishes a GitHub release with Knope-written notes, four attested archives, and the tag. Installing that version through mise on macOS arm64 and on CI's Linux x64 yields a complete package root.
- [ ] Workbench contains no `package.json`, npm lockfile, TypeScript source, or OMP package, pins no Node, pnpm, or Bun, and `mise run check` passes.
- [ ] The Hugo site publishes today's pages, fails on a dead relative link, serves under a PR-preview path, and searches. `project init --site github-pages` and `--site cloudflare` each produce working publishing and preview workflows.
- [ ] In an empty repository, `workbench project init` followed by `mise install` makes `mise run status` and `mise run validate` succeed and lets both harnesses load the package. In a copy of an existing consumer, it changes no existing file, and its receipt lists that consumer's vendored Workbench files and the missing project-owned tasks.

## Compatibility

This breaks every existing Workbench project. Nothing changes in a project until that project adopts the package. Each adoption is its own change, made by following the adoption guide.

- **Records** keep the format of [proposal 0005](0005-current-specifications.md).
  - Record validation detects what it detects today. Project validation, with the collision rule, is new.
  - `change-files check` replaces the changeset check.
  - The `status`, `validate`, `beads:sync`, `beads:claim`, and `beads:close` task names are unchanged.
- **CLI output.** The `workbench` commands output JSON by default, with new result shapes and structured errors. Anything that parsed the old tools' text output or `--json` shape must change.
- **Changesets to Knope.**
  - Pending Changesets entries become change files.
  - Existing `CHANGELOG.md` history stays as written, and new entries use Knope's format.
  - Release workflows are replaced. Projects that publish through another pipeline rewire it by hand, such as the cli starter's cargo-dist release and Pitlane's packages.
- **The workflow tool.** Projects that still carry a copy (Trove, Pitlane, Hive, and Bento) lose it on migration. OMP's native helpers replace it there.
- **Pico.** Until #61 supports declared plugins and their rules, Pico sessions in an adopted project load its `AGENTS.md` and `.agents/` content but not the package. Today's vendored copies do reach Pico. Pico is not the owner's daily harness yet.
- **OMP users with user-level `extensions:`** must repeat those entries in the project list, because a project list replaces the user list.
- **Fnox to Varlock.** Projects that grant secrets through Fnox move them to `.env.schema` and run the affected tasks under `varlock run --`. Their holotype templates own that change.
- **Starting a project.** "Use this template" ends; `workbench project init` replaces it.
- **Platforms.** Windows is not a supported target.

## Implications on adoption

- **Requirements, at the versions this proposal tested** (the implementation sets `min_version` and the documented floors from what it verifies):
  - mise 2026.11.8, `git`, and `gh`;
  - Claude Code 2.1.325 or OMP 18.20.2;
  - for the site, Hugo 0.170.0 and Pagefind 1.6.0.
- **Adoption follows the guide in [section 11](#11-adopting-workbench).** A new project is one `mise use`, one `mise install`, and `workbench project init`. An existing project also moves its additions out of shared files and deletes its vendored copies.
- **Updates** arrive through the pin, from Renovate's mise manager or by hand. Regenerate `mise.lock` where Renovate's lock update fails.
- **Reversal.** Revert the adoption commits; the vendored files return from history. Pinning an older release reverts only package changes.

## Scope

- The Rust `workbench` CLI described in [section 5](#5-the-workbench-cli): its command registry, discovery and documentation commands, output and error contract, Rust tests ported from `tools/*.test.ts` and proposal 0006, and the agent evaluation.
- The package layout under `plugin/`:
  - the process contract extracted from `AGENTS.md`, plus its output-style copy and their equality test;
  - rule frontmatter, which resolves workbench-0ky;
  - updated skill and template paths.
- The release pipeline: Knope configuration, the per-target builds, attestations, and the prepare-release and release workflows. Publishing `3.0.0` requires the owner's explicit authorization.
- The Hugo theme, Workbench's site on it, and PR previews.
- dprint and the revised `hk.pkl`.
- Removing the workflow tool and every Node, TypeScript, and OMP dependency from Workbench.
- `workbench project init` and `guides/adopting-workbench.md`, with updates to `AGENTS.md`, `PROCESS.md`, `README.md`, the guides, the templates, and the pull request template.
- A comment on pico#62 recording the workflow tool's last revision.

### Out of scope

- **Migrating the nine consumer repositories.** Each migration is a change in that repository, using the adoption guide.
- **Upstreaming the validator improvements found in Pico, Recon, and Trove.** Each changes what record validation detects and needs its own specification.
- **Pico's plugin support** ([pico#61](https://github.com/markmals/pico/issues/61)).
- **Renovate configuration in consumer repositories.**
- **Windows targets, and distribution through Homebrew or crates.io.** Nothing asks for them yet.
- **Sharing tool pins through mise `include` fragments, or hk steps through a Pkl package.** Both add a second version to keep in step with the package.
- **Matching the VitePress site's visual design.** That is look and feel.
- **The Gherkin integration libraries.** They remain separate work, as [proposal 0005](0005-current-specifications.md) set out.
- **Any change to the record format.**

## Preview

- **Repository shape:** a CLI, a harness plugin, and a documentation site.
- **Artifact:** a prerelease of the package, `3.0.0-rc.<n>`, published by a manually dispatched workflow from this branch, plus this pull request's documentation preview. Exercising the package means installing it into a real project and opening real sessions, so the prerelease is the cheapest realistic artifact.
- **Access:**
  - The prerelease is a public GitHub prerelease, because the repository is public. Publishing it requires the owner's explicit confirmation.
  - The site preview publishes at `https://markmals.github.io/workbench/pr-preview/pr-10/`, as this repository already does.
- **Exercise:**
  1. In a disposable fixture repository, pin the prerelease and run `mise install`.
  2. Open an OMP session and a Claude Code session and find the contract and the skills.
  3. Run `mise run status` and `mise run validate`.
  4. Bump the pin to the next release candidate and see the next session load it.
  5. Run `workbench project init --dry-run`, then `workbench project init`, in an empty repository, then `mise run check`.
  6. Starting from `workbench --help`, find and run `beads claim` and `change-files create` using only `cli search` and `schema`.
  7. Browse the site preview: record headers, proposal history, and search. A deliberately dead link fails CI.
- **Cleanup:** delete the fixtures. After acceptance, delete the prerelease and its tag, with the owner's confirmation.

## Policies and decisions checked

- **Policies and decisions.** None: `policies/` and `decisions/` hold only their READMEs.
- **[VISION.md](../VISION.md):**
  - Its minimum viable product is "a repository you can clone". This proposal makes it a versioned package plus `workbench project init`.
  - Its non-goal says infrastructure "ships harness-neutral in `.agents/`". The infrastructure becomes harness-neutral content in the package, with small committed harness declarations.
  - Both statements are revised at completion.
  - The vision's principles hold:
    - Enforcement stays mechanical at commit time and in CI.
    - The package content assumes no language, framework, or platform.
- **[Proposal 0001](0001-workbench-2.md)** established the template-repository distribution and the Workbench 2 tooling. This proposal replaces both and leaves that record as history.
- **[Proposal 0004](0004-make-room-for-ui-polish.md)** leaves look and feel to the running product. The site's appearance follows that.
- **[Proposal 0005](0005-current-specifications.md)** defined the current-record model and the record and status contract in `guides/record-validation.md`, both preserved. Its "changesets and changelogs" role passes to change files and Knope's changelog.
- **[Proposal 0006](0006-drop-harness-hooks-and-sync-beads.md)**, merged in PR #11, removed the harness hooks and the pre-push check and added Beads sync, claims, and closes. This proposal depends on it: the package declares no hooks, and `workbench workflow status` and the `beads` commands carry over 0006's behavior, including the refinements made during its implementation (the conflict report for embedded storage, the `post-rewrite` sync, the guard against another repository's Beads, and `beads:close`).
- **`.agents/rules/enforcement-hierarchy.md`.** Moving the checks into the package keeps them deterministic: `commit-msg`, `pre-commit`, CI, `records validate`, and `project validate`. It adds the collision check.

## Future directions

- Pico's project-declared plugins (#61) can read the same `.claude/settings.json` declaration and the package's rules, making the package complete in Pico.
- A Renovate preset could configure Workbench pins and lock regeneration across the owner's repositories.
- The validator improvements in Pico, Recon, and Trove are candidates for upstreaming as separate changes.
- The general reasoning skills (grounding, ontology review, reframing, systematic debugging, adversarial review) appear standalone in about fourteen non-Workbench repositories. They could become a package of their own.
- Shared tool pins through mise `include` fragments could follow if pin drift across projects becomes a problem.
- The command registry could generate an MCP server exposing the same operations, if a harness ever needs tools rather than a CLI.

## Alternatives considered

- **An npm package in `node_modules`.** Tested working in both harnesses, with versions pinned by the lockfile and updated by Renovate. It requires `package.json` and Node in every project. That requirement is what this proposal removes.
- **The harnesses' own marketplaces.** That means a Claude Code GitHub marketplace with `claude plugin install --scope project`, and `omp plugin install --scope project`.
  - Claude Code keeps one marketplace checkout per user, so pins float across projects.
  - Each clone needs an install step per harness.
  - OMP ignores Claude's marketplace declarations, and its project installs record absolute cache paths.
  - CI never sees the version a session uses.
- **A Git submodule at `.workbench/`.** It pins per project without a runtime. The CLI would still need a second pin or a runtime, and submodules add clone and update friction.
- **TypeScript compiled with `bun build --compile`.** It removes Node from projects but keeps TypeScript in Workbench. The owner chose Rust.
- **cargo-dist for builds and releases.** It and Knope both create the GitHub release, and Knope owns the notes the owner asked for. A plain build per target with attestations is simpler.
- **The contract through a Claude SessionStart hook.** It reintroduces a hook. Claude Code moves context over 10,000 characters to a file, and its docs warn that imperative hook text can trip prompt-injection defenses. An output style is sent with every request and survives compaction. Its cost: it overrides a user's own output style in Workbench projects.
- **Converting the rules into skills.** Skills load in every harness. Rules keep on-demand loading in OMP and Pico, and Claude reaches them through the contract's links, so the conversion would add churn without improving any harness.
- **Zola** dropped custom frontmatter in testing and requires a `content/` directory. **mdBook** needs a generated `SUMMARY.md` and a preprocessor for frontmatter. **Keeping VitePress** keeps Node.
- **Knope reading Changesets-format files unchanged.** The owner chose Knope-native change files and release notes.
- **Keeping the template repository alongside the package.** Two distribution paths would drift apart the way the copies have.
- **Keeping today's tool interfaces:** text by default, `--json` on request, and a flat list of commands with free-text errors. Agents then guess at command names, parse prose, and cannot tell a retryable failure from a permanent one. The owner's agent-first CLI guide sets the contract instead.
- **Other ways to mark a pull request with no user-visible change.** A `--none` form of `change-files create` would write a marker file, which Knope would have to ignore and which only exists to satisfy the check. A pull-request label can only be checked in CI, and it is not recorded in Git history.

## Open questions

None. On 2026-11-15 the owner chose the `Release-Note: none` commit trailer for pull requests with no user-visible change.
