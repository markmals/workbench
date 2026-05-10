# Commit Discipline

> **This file is `@included` from the root `CLAUDE.md`.** It loads on every session.
>
> **Note for users who copied this template:** the default policy here is that the agent commits at natural commit points. If you'd rather hold commit boundaries yourself, override this with a feedback memory like `"Don't run git commits — I'll handle them"` and the agent will stop committing.

The agent commits often, atomically, and with messages a human would write. Commits are how work becomes durable; treat them as a primary deliverable, not an afterthought.

## When to commit

A commit lands at a **natural commit point** — a moment where the working tree represents one coherent, internally consistent unit of work. Typical natural commit points:

- A failing test was written, then the implementation made it pass, and the test now passes. (Two commits if the test-then-implementation distinction matters; one commit if they're tightly bound.)
- A self-contained refactor is complete and all tests still pass.
- A spec was edited and the affected docs/templates re-checked.
- A configuration change was made and verified in the relevant tool.
- A feature folder was authored or extended and the agent ran `/sdd-analyze` against it.

Do **not** commit mid-task work. If tests are red, if the file is half-edited, if the change is incomplete — keep going (or stash and pivot). A WIP commit is the wrong answer; finish the thought, then commit.

## What goes in one commit

**One logical change per commit.** If you can describe the commit as "X and also Y", it's two commits.

Counter-examples that are _not_ one logical change:

- "Add the items list view model and also bump the Tailwind version"
- "Fix the duplicate-email bug and reformat the file"
- "Implement story.item.create and story.item.edit"

In each case, split. The Tailwind bump is its own commit. The reformat is either its own commit or, ideally, dropped because it has nothing to do with the bug.

## What goes in a good message

A commit message has three parts: **subject**, optional **body**, optional **footer**.

### Subject

- Imperative, present tense: "Add", "Fix", "Remove" — not "Added" or "Adds".
- Under ~72 characters.
- No trailing period.
- Specific. "Fix bug" is useless; "Reject duplicate emails in item creation" is useful.

Conventional Commits format is _recommended but not required_. If you use it, the prefixes that apply here:

| Prefix      | Use for                                          |
| ----------- | ------------------------------------------------ |
| `feat:`     | New user-visible capability                      |
| `fix:`      | Bug fix                                          |
| `refactor:` | Restructuring without behavior change            |
| `test:`     | Adding or improving tests                        |
| `docs:`     | Spec / README / comment-only changes             |
| `chore:`    | Config, deps, tooling                            |
| `spec:`     | Edits to files under `specs/` or `features/<n>/` |

Examples:

- `feat: add items list view model with empty / loaded / error states`
- `fix: reject item creation when email is already present`
- `spec: clarify duplicate-email handling in story.item.create`
- `refactor: split ItemsListView render logic into list and row components`

### Body

Optional. Use it when the WHY isn't obvious from the subject. Wrap at ~72 characters. Explain:

- The motivation (what user-facing problem or spec change this addresses)
- Any non-obvious trade-offs
- Anything a future reader would want to know that the diff doesn't show

Skip the body for trivial changes.

### Footer

Optional. Use for:

- Cross-references: `Refs: story.item.create`, `Spec: vm.items.list`
- Breaking changes: `BREAKING CHANGE: <description>`
- Co-authorship (if collaborating)

## Staging

- **Never `git add .` or `git add -A`.** Both will sweep up files you didn't intend to include — untracked artifacts, env files, build outputs. Stage by explicit path.
- **`git status` before staging.** Look at what's in the working tree. Decide what belongs in this commit and stage exactly that.
- **Review the diff.** `git diff --staged` before committing. If something is in there you don't recognize or didn't intend, take it out.

## Pre-commit hooks

- If a pre-commit hook fails, **the commit didn't happen**. Fix the issue, re-stage, create a new commit. Do **not** `--amend` after a failed hook — there's nothing to amend.
- Never bypass with `--no-verify` unless the user explicitly asks. Hook failures are usually telling you something real.

## When to amend vs. new commit

- **Default: new commit.** Easier to reason about, easier to revert, easier to review.
- **Amend only when:** the previous commit is local (not pushed), the new change is genuinely part of the same logical unit, and amending makes the history clearer (not just smaller).

## What never to commit

- Secrets (`.env`, credential files, API keys). If you see these in `git status`, stop and warn the user.
- Build outputs (`dist/`, `.output/`, `build/`). The `.gitignore` should already exclude these — if it doesn't, fix the gitignore in its own commit.
- Personal IDE config (`.vscode/`, `.idea/`). Unless the user explicitly asks.
- Large binaries unless the project explicitly tracks them.

## Frequency

Prefer **many small commits** over a few large ones. Five focused commits with clear messages beat one giant "implement the items feature" commit every time. Small commits are easier to review, revert, cherry-pick, and reason about months later.

## Push policy

Committing is one thing; pushing is another. **Do not push unless the user asks** — even if commits are clean. The user controls when work goes upstream.
