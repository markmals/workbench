---
id: guide.adopting-workbench
title: Adopting Workbench
---

# Adopting Workbench

## How do I add Workbench to a project?

Workbench is a package: the `workbench` command-line tool plus the process contract, rules, skills, templates, and documentation-site theme your harness loads. A project depends on one pinned release through [mise](https://mise.jdx.dev). `workbench project init` then adds the files Workbench owns, and the package itself stays out of the repository behind `.workbench`, a gitignored link that `mise install` keeps pointed at the pinned release.

This guide covers a new repository, a project made from a holotype template, and an existing project, including one copied from the old Workbench template repository. A new project needs only step 1 and step 7. An existing project works through all of them.

You need mise activated in your shell (see [mise's guide](https://mise.jdx.dev/getting-started.html)), a Git repository, and the [GitHub CLI](https://cli.github.com). If mise is not activated, prefix each `workbench` command below with `mise x --`.

## 1. Pin Workbench and run `project init`

Pin a release in `mise.toml`. `mise use` writes the pin and installs that release:

```sh
mise use github:markmals/workbench@3.0.0
```

Preview what `project init` will do. It writes nothing with `--dry-run`:

```sh
workbench project init --dry-run --output text
```

The receipt lists what it would create, what it would add to existing files and which entries, what is already in place, anything that needs your attention, every Workbench file the project still vendors, and the project-owned pieces Workbench relies on but does not create. Without `--output text` the same receipt is JSON. `workbench schema project init` states the complete contract.

Then run it, and run `mise install` so the new `postinstall` hook links `.workbench` and the tools `project init` pinned are installed:

```sh
workbench project init
mise install
```

`project init` never overwrites or deletes a file. It creates what is missing and, in files that exist, adds only missing keys and entries:

| File                                                                                                                       | What it adds                                                                                                                                               |
| -------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `mise.toml`                                                                                                                | The `postinstall` hook that links `.workbench`; pins for hk, gh, and Beads; the `status`, `validate`, `beads:sync`, `beads:claim`, and `beads:close` tasks |
| `.gitignore`                                                                                                               | `.workbench`                                                                                                                                               |
| `.claude/settings.json`                                                                                                    | The `workbench` marketplace and the `workbench@workbench` plugin                                                                                           |
| `.omp/config.yml`                                                                                                          | `./.workbench` in `extensions:`                                                                                                                            |
| `.claude/skills`, `.claude/rules`                                                                                          | Links to `../.agents/skills` and `../.agents/rules`, when those directories exist                                                                          |
| `hk.pkl`                                                                                                                   | Created when absent: the Scoped Commits check on `commit-msg`, and the Beads hook steps and syncs                                                          |
| `.github/workflows/workbench.yml`                                                                                          | Created when absent: the commit and change-file checks on every pull request except Knope's release pull request                                           |
| `AGENTS.md`                                                                                                                | Created when absent, as a stub stating that Workbench loads from `.workbench`                                                                              |
| `VISION.md`, the `proposals/`, `policies/`, `decisions/`, and `guides/` READMEs, `.agents/commit-scopes`, `docs/hugo.toml` | Created when absent                                                                                                                                        |

When an entry it would add already exists with other content, such as a `status` task that still runs an old script, it leaves the entry alone and lists it under `attention`. Running it again is safe; a second run reports `unchanged`.

Check the result. `context show` reads the setup back, and `project validate` reports anything still wrong with it, as [Record Validation](./record-validation.md#validate-the-projects-workbench-setup) describes:

```sh
workbench context show --output text
workbench project validate --output text
```

Last, add the project-owned pieces the receipt lists under `missing`. Workbench relies on a `setup` task, a `check` task that is the project's whole gate, and an `fmt` task, but their contents depend on the project's language and tools, so `project init` writes none of them. A `setup` task usually restores this machine's Beads replica and installs the Git hooks:

```toml
[tasks.setup]
description = "Idempotent first-run bootstrap: this machine's Beads replica and the Git hooks"
run = [
    "if [ -f .beads/config.yaml ]; then bd bootstrap --yes; else bd init --non-interactive --prefix myproject --skip-agents --skip-hooks; fi",
    "bd config set export.auto true",
    "bd config set dolt.auto-push false",
    "hk install --mise",
]
```

Replace `myproject` with the short name Beads puts in every item's ID. `--skip-hooks` matters: hk owns every Git hook, and `bd init` would otherwise point Git at its own. The receipt's last missing piece, a release flow, is step 4.

## 2. Move project additions out of shared files

A project that copied Workbench's files has often edited them. Move each addition to a place the project owns before deleting the copies:

- **An edited skill or rule** becomes a project skill in `.agents/skills/<name>/` or a project rule in `.agents/rules/<name>.md`, under a name of its own. Never reuse a Workbench skill or rule name: Oh My Pi ranks the package's skill above the project's while Claude Code loads both, so the same name behaves differently in each harness, and `workbench project validate` reports it.
- **An edited template** stays at `.agents/templates/<name>`. A project template overrides the package's template of the same name, and the receipt marks it `different`. A template identical to the package's is marked `identical`, and you delete it in step 3.
- **`AGENTS.md`** keeps only what is true of this project: its purpose and layout, commands beyond the task interface, platform rules, and how to preview a change. Delete the process contract you copied from Workbench; the package supplies it. Keep the line saying that Workbench loads from `.workbench`, a gitignored link `mise install` restores, so an agent that cannot see the contract still learns the fix. When `AGENTS.md` lacks it, the receipt says so.
- **A `CLAUDE.md`** you keep must contain the line `@AGENTS.md`, or Claude Code ignores `AGENTS.md`.

## 3. Delete the vendored copies

Everything in the receipt's `vendored` list is a copy of something the package now supplies, except templates marked `different`. Before deleting the TypeScript tools, point whatever runs them at `workbench`:

- **mise tasks.** `attention` names each `status`, `validate`, `beads:sync`, `beads:claim`, and `beads:close` task that differs from Workbench's. Make each one match the task `project init` writes, which you can see by running it in an empty repository, or replace it with this:

  ```toml
  [tasks.status]
  description = "Where the work stands: current phase, next action, who acts, and Beads"
  run = "workbench workflow status --output text"
  ```

- **`hk.pkl`.** Its `commit-msg` step becomes `workbench commits check --message-file {{commit_msg_file}}`. To see the hk configuration `project init` writes, move yours aside, run `project init`, then carry your project's own steps, such as formatters, into the new file and delete the old one:

  ```sh
  git mv hk.pkl hk.old.pkl
  workbench project init
  ```

- **CI.** `.github/workflows/workbench.yml` now runs the commit and change-file checks. Delete the jobs that ran `tools/check-pr-commits.ts` and `tools/require-changeset.ts`.
- **Node.** If Workbench's tools were the project's only use of Node, remove the `node`, `pnpm`, and `bun` pins and `package.json`.

Then delete the copies. With [jq](https://jqlang.org) installed, this removes every vendored path except template overrides:

```sh
workbench project init --dry-run | jq -r '.vendored[] | select(.template != "different") | .path' | xargs git rm -r --quiet
```

`workbench project validate --output text` lists whatever remains.

## 4. Convert pending change entries and release with Knope

Workbench projects describe each user-visible change in a change file in `.changeset/` and release with [Knope](https://knope.tech). Every pull request adds a change file, or carries a `Release-Note: none` commit trailer when nothing user-visible changed; `.github/workflows/workbench.yml` checks this.

Turn each pending Changesets entry into a change file, then delete the old entry. In a single-package project, `--package` is unnecessary:

```sh
workbench change-files create --type minor --summary "Add the export command" --body "Exports the record as JSON."
```

`.changeset/` must then hold only change files, because Knope fails on any other Markdown file there: delete `.changeset/README.md` and `.changeset/config.json`. A pull request with no user-visible change carries the trailer instead:

```sh
git commit --trailer "Release-Note: none"
```

The release flow is the project's own. Write a `knope.toml` that sets `ignore_conventional_commits = true` under `[changes]`, because Scoped Commits subjects are not Conventional Commits and versions come only from change files, and follow Knope's [preview releases with pull requests](https://knope.tech/recipes/1-preview-releases-with-pull-requests/) recipe. Keep its release branch named `release`, which `workbench.yml` exempts from the change-file check. Merging the release pull request is the release authorization. Delete the Changesets release workflow and dependencies.

## 5. Move a published site to Hugo

Workbench's documentation site is a Hugo theme in the package; `docs/hugo.toml` selects it and mounts the vision, records, and guides where they live. A project publishes it to GitHub Pages or to Cloudflare. `project init` never replaces an existing file, so first remove what the VitePress site used beyond `docs/.vitepress/`, which step 3 deleted: its workflows, and its `docs:build` and `docs:dev` tasks in `mise.toml`. VitePress serves static files from `docs/public/`, which is Hugo's build output, so move them to `docs/static/`. Then let `project init` add the site:

```sh
git rm --quiet .github/workflows/docs.yml .github/workflows/preview.yml
git mv docs/public docs/static
workbench project init --site github-pages
mise install
mise run docs:build
```

Use `--site cloudflare` to publish to Cloudflare instead. Either adds the `hugo` and `github:CloudCannon/pagefind` pins, the `docs:build` and `docs:dev` tasks, `docs/.gitignore` for the build output in `docs/public/`, and two workflows: `.github/workflows/docs.yml` publishes `main`, and `.github/workflows/preview.yml` publishes each pull request at its own URL, comments the link on the pull request, and removes the preview when it closes. A deployed build takes its base URL from `HUGO_BASEURL`, so a preview builds for its own path; a local build needs nothing set. Set `params.repository` in `docs/hugo.toml` when `project init` could not derive it from `origin`: links to files the site does not publish resolve there.

- **GitHub Pages** serves the `gh-pages` branch, which the first publish creates. `docs.yml` publishes only while the repository is public. Point Pages at the branch once:

  ```sh
  gh api --method POST "repos/{owner}/{repo}/pages" -f "source[branch]=gh-pages" -f "source[path]=/"
  ```

- **Cloudflare** serves the site from a Worker that `docs/wrangler.jsonc` names after the project, with a Worker Preview for each pull request. Put the Worker and its Previews behind Cloudflare Access if the guides are private. The workflows need two repository secrets, an API token allowed to edit Workers Scripts and the account ID; each command prompts for the value:

  ```sh
  gh secret set CLOUDFLARE_API_TOKEN
  gh secret set CLOUDFLARE_ACCOUNT_ID
  ```

Delete the VitePress dependencies. [Choosing a Preview](./choosing-a-preview.md) covers previews for everything else a project ships.

## 6. Replace Fnox with Varlock

Workbench projects declare environment variables and secrets with [Varlock](https://varlock.dev) instead of Fnox. Declare each variable in `.env.schema` and commit it; if `.gitignore` ignores `.env.*`, add `!.env.schema` after that line. Run each task that needs a secret under `varlock run --`, for example `run = "varlock run -- ./scripts/deploy.sh"`. Then delete `fnox.toml`, the `fnox` and `1password` pins, the `[secrets.fnox]` block, and every task's `secrets` grant. A project made from a holotype template gets this from its template.

## 7. Run the gate

Bootstrap the repository on this machine and run the project's whole gate:

```sh
mise run setup
mise run check
```

Then run one small feature end to end, as [Running a Feature](./running-a-feature.md) describes. The first pass exposes what your harness does not do well while the change is still cheap.

## 8. Optionally let Renovate update the pin

[Renovate](https://docs.renovatebot.com)'s mise manager finds the `github:markmals/workbench` pin in `mise.toml` and opens a pull request for each release. When such a pull request changes the pin without updating `mise.lock`, regenerate the lock on its branch and push:

```sh
mise lock
```

After the pin changes and `mise install` runs, new sessions load the new release. A running session keeps what it loaded.

## Harness notes

- **Claude Code** loads the process contract as the `workbench` output style, forced while the plugin is enabled, so it replaces your own output style in Workbench projects.
  - It applies the project's marketplace and plugin declarations only in a folder you trust. Accept the trust prompt in the first session.
  - It ignores `AGENTS.md` when a `CLAUDE.md` exists that does not import it with `@AGENTS.md`.
  - In a Git worktree it can resolve the project to the main checkout, so `./.workbench` may be the main checkout's link. Run `mise install` in the main checkout as well whenever the pin changes.
- **Oh My Pi** lists the package's rules and skills itself. A project's `extensions:` list replaces your user-level list rather than adding to it, so repeat any user-level extension you rely on in `.omp/config.yml`.
- **Pico** loads `AGENTS.md` and `.agents/` but not the package until it supports project-declared plugins ([pico#61](https://github.com/markmals/pico/issues/61)).

If `.workbench` is missing or dangling, a harness loads no Workbench content and tasks that call `workbench` fail with mise's missing-tool error; `mise install` restores both.

## Configure the readiness-report account

The readiness report comes from a service account so the pull request distinguishes statements the agent made from statements you made. Create a bot account or GitHub App, give it write access to the repository, and expose its token in the agent's environment as `SERVICE_ACCOUNT_TOKEN`.

The agent sets `GH_TOKEN` from `SERVICE_ACCOUNT_TOKEN` only when posting its own readiness-report comment. Marking the pull request ready happens with the ordinary `gh pr ready` identity, after the complete implementation is verified and pushed. Exporting the service token as a global `GH_TOKEN` would attribute unrelated GitHub actions to the bot. Without a service account, the agent identifies the account it used in the report.
