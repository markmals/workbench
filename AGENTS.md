# Workbench

This repository is Workbench itself: the `workbench` CLI and the plugin content that Workbench projects pin in `mise.toml` and load from `.workbench/`. The process contract every session follows ships in that package, at `.workbench/rules/workbench.md`; this file holds only facts about this repository.

Workbench loads from `.workbench`, a gitignored link that `mise run setup` (here) or `mise install` (in a consumer) restores. Here it links to `plugin/`, the working tree's own package content, so sessions in this repository load the process as it stands on the branch.

## Layout

| Path                                                                          | What it is                                                                                                                                                                                                                                                           |
| ----------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Cargo.toml`, `src/`, `tests/`                                                | The Rust `workbench` CLI. `src/registry.rs` defines a command's declaration; each resource's commands live in `src/commands/<resource>`; `tests/support/` holds the fixture projects the integration tests run the real binary in.                                   |
| `build.rs`                                                                    | Records the names of the skills and rules `plugin/` ships, for the name-collision check.                                                                                                                                                                             |
| `plugin/`                                                                     | The package content: `.claude-plugin/` manifests, `rules/` (the process contract `workbench.md` and four rules), `output-styles/workbench.md` (the contract for Claude Code), `skills/`, `templates/`, `reference/cli.md` (generated), and `site/` (the Hugo theme). |
| `docs/`                                                                       | Workbench's own site configuration; `docs/hugo.toml` mounts the records where they live.                                                                                                                                                                             |
| `VISION.md`, `PROCESS.md`, `proposals/`, `policies/`, `decisions/`, `guides/` | Workbench's records and its process description.                                                                                                                                                                                                                     |
| `knope.toml`, `.changeset/`, `CHANGELOG.md`                                   | Releases, change files, and the changelog.                                                                                                                                                                                                                           |
| `.agents/commit-scopes`                                                       | Commit scopes only this repository uses.                                                                                                                                                                                                                             |
| `.claude/settings.json`, `.omp/config.yml`                                    | The harness declarations, the same as a consumer's.                                                                                                                                                                                                                  |

## Commands

```sh
mise run setup        # link .workbench to plugin/, build the CLI, restore Beads, install hooks
mise run check        # the gate: tests, hk check --all (dprint, rustfmt, Clippy, actionlint), validate, docs:build
mise run fmt          # format in place; accepts file paths
mise run test         # cargo test
mise run validate     # workbench records validate and workbench project validate
mise run status       # workbench workflow status, as text
mise run docs:build   # build the site with Hugo and index it with Pagefind
mise run docs:dev     # serve the site with reload
mise run cli:reference   # regenerate plugin/reference/cli.md from the registry
mise run package -- <target>   # build one release archive into artifacts/
```

In mise contexts `workbench` resolves to this working tree's build in `target/debug`, which the tasks build first; in a consumer it is the pinned release.

## Changing the CLI

- A command's registry entry in `src/commands/<resource>` is its specification: summary, description (defaults, errors, side effects), arguments, effect, requirements, error codes, examples, synonyms, and the output type whose `JsonSchema` is its result schema. Write or revise the entry first, then its failing tests, then the code.
- `tests/registry.rs` fails when the parser, runtime defaults, or examples drift from the registry; every example runs in `tests/support::example_project()` and its output must match its schema.
- After changing the registry, run `mise run cli:reference`; the reference test fails until `plugin/reference/cli.md` matches.
- `plugin/output-styles/workbench.md` repeats the body of `plugin/rules/workbench.md` for Claude Code. Edit both; a test fails when their bodies differ.

## Releases and site

Knope releases Workbench from change files: merging its release pull request builds the four archives (`.github/workflows/build-archives.yml`), attests them, tags `v<version>`, and publishes the release. `prerelease.yml` publishes a `3.0.0-rc.<n>` preview from any branch by manual dispatch; it is public, so it needs the owner's confirmation. The site publishes `main` to GitHub Pages, and each pull request gets a preview under `/workbench/pr-preview/pr-<N>/`.
