---
description: When and how to commit, stage, push, and open pull requests in a Workbench project, including Scoped Commits subjects, branch and pull-request names, and change files.
---

# Commit Discipline

Commit often, atomically, and with messages a careful human would write. Commits are durable review units, not an afterthought.

## Natural commit points

Commit when the working tree is one coherent, internally consistent unit:

- A failing test, its implementation, and its verification are complete.
- A self-contained refactor is complete and its checks pass.
- A proposal, guide, policy, decision, or configuration change is complete and verified.

Do not commit half-edited, incomplete, or red work. Finish the thought — or stash and pivot — then commit.

## One logical change

One commit contains one logical change. If its description needs “and also,” split it. Keep unrelated dependency updates, formatting sweeps, and cleanup in separate commits or drop them.

## Scoped Commits

This repository uses [Scoped Commits](https://scopedcommits.com/), not Conventional Commits. Every ordinary subject is:

```text
<scope>: <imperative description>
```

The scope names the feature area the change addresses, not the step of the process that produced it, so a reader scanning the log sees where each change landed. Work on a proposal takes the scope of the feature it changes.

Write an imperative, present-tense description: `add`, `fix`, `remove` — not `added` or `adds`. Keep the subject under roughly 72 characters, specific, and without a trailing period. Use an optional body for motivation or non-obvious trade-offs; wrap it at roughly 72 characters.

Do not write Conventional Commits `type(scope):` subjects. Scoped Commits lead with the area being changed.

`workbench commits check` accepts three kinds of scope:

- Workbench's process scopes: `agents`, `ci`, `decision`, `deps`, `docs`, `guides`, `policy`, `release`, `rules`, `skills`, `templates`, `treewide`, and `vision`;
- the project's product scopes, one per line in `.agents/commit-scopes`, which is optional and grows with the project;
- `<NNNN>-<slug>` for work on that real proposal, from its draft to its implementation, such as `0007-export-queue`.

`workbench commits scopes` lists every accepted scope and where it comes from, and a rejected subject lists them too. Prefer the narrowest scope that accurately describes the change; use `treewide` only for a true sweep. A change that needs a product scope no list has adds it to `.agents/commit-scopes` in the same commit.

Examples:

- `exports: retry a failed report export`
- `guides: explain the readiness report`
- `ci: check change files on every pull request`
- `0007-export-queue: propose queuing report exports`
- `0007-export-queue: reject duplicate export jobs`

Mechanical merge, revert, fixup, and squash commits are exempt. hk's `commit-msg` hook runs `workbench commits check`, so every commit is checked regardless of which harness, editor, or human made it, and CI checks every commit in a pull request.

## Branches and pull requests

A branch and a pull request name the feature area they change, as a commit scope does. Title the pull request as a Scoped Commits subject and name the branch for the same area and change in kebab case: `exports: queue report exports` on `export-queue`. A pull request that starts with a proposal is named for the feature it proposes, never `proposal`: that a change starts as a proposal says nothing about what it changes.

## Staging

- Never use `git add .` or `git add -A`. Stage explicit paths only.
- Run `git status` before staging. Decide exactly what belongs in this commit.
- Run `git diff --staged` before committing. Unstage anything unexpected.
- Preserve the user's uncommitted changes. Do not discard, overwrite, stage, or reformat work you did not make.

## Commit hooks and history

- hk owns every git hook; `mise run setup` installs them with `hk install --mise`. Never run another tool's hook installer, and never set `core.hooksPath` by hand.
- If a hook fails, the commit did not happen. Fix the issue, re-stage, then create a new commit.
- Never bypass hooks with `--no-verify` unless the human explicitly asks.
- Default to a new commit. Amend only when the previous commit is local, the correction belongs to the same logical change, and amending makes history clearer.
- Never rewrite pushed history. Correct it with a new commit. A force-push is a history rewrite and requires explicit human confirmation.

## Never commit

- Secrets: credentials, tokens, keys, and the `.env` files that hold their values. The committed `.env.schema` declares variables and holds no secret values.
- Build output and generated artifacts the project does not track.
- Personal editor configuration unless the human explicitly asks.
- Large binaries unless the project explicitly tracks them.

If an unexpected secret appears in `git status`, stop and warn the human. Secrets reach tasks through Varlock, never through a tracked file: the project declares them in `.env.schema`, and a task that needs one runs under `varlock run --`.

## Frequency

Prefer many small, focused commits to one large commit. Commit at each natural green boundary before beginning the next independent change.

## Change files

Every pull request adds at least one change file in `.changeset/`, written for the person reading the release notes — not for the reviewer who already saw the diff. Create it with `workbench change-files create`, choosing the change type deliberately; never run `knope document-change`, which prompts. A change file opens with a heading that summarizes the change; a file without one does not count. Commit the change file with the work it describes. A pull request with no user-visible change carries a `Release-Note: none` trailer on one of its commits instead (`git commit --trailer "Release-Note: none"`). CI runs `workbench change-files check` and rejects a pull request with neither; Knope's release pull request is exempt.

## Push policy

Pushing the feature branch to `origin` is expected and routine: it creates and updates the pull request that serves as the durable workspace. It starts draft for proposal development and becomes ready after the complete implementation is verified and pushed. Push early and push often.

Pushing directly to the default branch is prohibited. Work reaches the default branch only by merging its pull request.

Force-pushing, deleting a branch, publishing a release, and merging require explicit human confirmation. Merging happens in phase 5 after human acceptance, not because the agent judges the work acceptable.
