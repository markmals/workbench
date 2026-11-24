---
name: Workbench
description: Follow the Workbench process contract in every Workbench project.
force-for-plugin: true
keep-coding-instructions: true
---

# Workbench Process Contract

This project uses the **Workbench** development process. A human approves a change in a proposal; agents write it into the project's current specifications, then tests, guides, and code; an independent review checks them against each other before the human reviews the result. This contract ships in the Workbench package that the project pins in `mise.toml` and links at `.workbench/`. The project's own `AGENTS.md` holds facts about the project; this contract holds the process. Skills are named by name, and package files by their `.workbench/` paths.

Run `workbench workflow status` at the start of every session and whenever you are unsure where the work stands. It reads the branch, the pull request, and the active proposal, and reports the phase and the next step. It also syncs Beads with the other machines and reports ready work, the items in progress, and the project's memories. Humans run `mise run status` for the same report as text. The human is never expected to remember a command or skill name; "keep going" or a plain description of a problem is enough to orient from it.

## Using the `workbench` CLI

`workbench` writes JSON results to stdout and diagnostics to stderr. A failure writes an `error` object with a stable `code`, whether it is `retryable`, and `suggestions` for the fix. Do not guess its syntax:

1. **Discover.** `workbench cli search "<task>"` ranks the commands that fit a task. `workbench docs search "<topic>"` finds this contract, the rules, the skills, and the templates.
2. **Inspect.** `workbench schema <resource> <operation>` returns a command's arguments, defaults, effects, error codes, and examples.
3. **Preview.** Run a modifying command with `--dry-run` first.
4. **Execute, then verify** from the receipt it returns, or by reading the result back.

## Does this change need a proposal?

Ask one question: **does this change decide something?** Look and feel — how an interface looks and how it is operated — is the exception: the human decides it by looking at the running UI, not in a proposal.

- If intent is established by a current specification, the approved change currently being implemented on this branch, or an unambiguous correction (a typo, a dead link, a bug whose fix is obvious), implement it directly and say which case applies. Completed proposals are historical evidence; they cannot authorize restoring obsolete behavior. Raise any conflict with current specifications before acting.
- If the human is shaping look and feel — layout, sizes, spacing, type, color, motion, which control presents a choice, how it is operated, and wording whose exact words are not themselves a requirement — make the changes directly with the `polishing-a-ui` skill. Say so in one line before you start.
- If the change decides something, ask the human whether they want a proposal for it. Only after an explicit yes: create a branch, push it, open a draft pull request, and write the proposal with the `writing-a-proposal` skill. Name the branch and the pull request for the feature area the change addresses, never `proposal`, and once the proposal is pushed, link it on the branch at the top of the pull request description. A request to "start a feature" or "write a proposal" is not that yes, and neither is the output of `workbench workflow status`.
- If a small fix turns out to need a design decision, or polish turns out to change what the feature does, stop and ask the proposal question then.

Proposal status belongs to the human. They move a proposal from `draft` to `awaiting-implementation`, and from `active-review` to `accepted`, `returned-for-revisions`, or `rejected`. You move it to `active-review` when you hand finished work to them, and to `implemented` in the accepted pull request before merge (the `completing-a-feature` skill).

## Implementing an approved proposal

Use the `implementing-a-proposal` skill. In short:

1. Read the proposal, the current sources it links, and the code it touches. Completed proposals are history; current requirements live in the sources that own them.
2. Revise the affected current specifications (API contract documentation, schemas, reference prose, or feature files) and any agreed policy or decision so they state the approved change. For `.feature` files, use the `specifying-behavior-with-gherkin` skill.
3. Write failing tests for the behavior those specifications now require, and watch them fail for the right reason.
4. When the change alters what a user does, sees, or must know, write or extend the guide in `guides/` from the intended contract, not from the code, link the source that owns that contract, and check the guide against the tests.
5. Write the code.
6. Verify with `mise run check`. Report the commands you ran and what they printed. Once the complete implementation has passed the applicable checks and is pushed, wait for CI to pass on that revision (`gh pr checks --watch`), then mark the pull request ready for maintainer review; it stays a draft only while it holds just the proposal. When CI fails, fix the failure and push again. The proposal stays `awaiting-implementation` until the `active-review` handoff.
7. Get one independent review with the `reviewing-an-implementation` skill and fix what it confirms. Then post the readiness report and move the proposal to `active-review`; the pull request is already ready.

Which of specifications, tests, guides, and code a change needs is a judgement; the order is not. A change with no user-facing surface needs no guide, and a change with nothing meaningfully testable needs no test, but say so rather than skipping quietly.

## Delegation

Work inline by default. Delegate, with the harness's own delegation, only slices that are independent and touch separate files, or a review that needs fresh context. Give each delegate bounded ownership: the files it owns, the current sources it touches, its constraints, and how to verify its work. A delegate that fails returns no result; treat a missing result as missing, never as success.

## Rules

- **Stay inside the proposal.** Behavior it does not describe does not get built; if it should exist, revise the proposal with the human. Look and feel is not behavior a proposal must describe: choose what the proposal leaves open to fit the project's existing interface and its platform's conventions, and let the human refine it on the running UI. A defect outside its scope becomes a Beads item; a defect that stops the proposal from working is your work.
- **Raise ambiguity.** When the proposal, a current specification, the vision, a policy, or a decision is ambiguous or conflicts with the work in a way that changes behavior, stop and raise it rather than choosing silently.
- **Never write production code before its failing test**, when the behavior warrants a test at all.
- **Never write a specification or guide from finished code.**
- **Never claim work is done without running the verifying command this turn** and reading its output (`.workbench/rules/verification.md`).
- **Find a bug's root cause before fixing it** (the `systematic-debugging` skill).
- **The adversarial reviewer reports; it never edits.**
- **Never delete a branch, force-push, publish a release, or deploy without explicit human confirmation.**
- **Add, revise, or delete a policy or decision only after agreeing it with the human.** Revise a changed record in place and delete a retired one; neither gets a replacement record or `supersedes`. Most features establish neither; a record belongs there only when its knowledge outlives the change.
- **Templates.** Start every artifact from its template in `.workbench/templates/`. A project file `.agents/templates/<name>` overrides `.workbench/templates/<name>`.

## Tasks

Every Workbench project implements three tasks, and the process invokes them by name:

```sh
mise run setup   # idempotent first-run bootstrap after a clone
mise run check   # the single blocking gate; CI runs it on every push
mise run fmt     # format in place; accepts file paths
```

`mise run check` is the gate. Nothing is done until it passes in this turn. No git hook runs it; CI does, on every push, and a pull request is marked ready or merged only after CI passes on its latest revision. The `status`, `validate`, `beads:sync`, `beads:claim`, and `beads:close` tasks wrap `workbench` commands for humans and git hooks. `mise tasks` lists everything the project defines.

## Work tracking

Beads (`bd`) holds the work agents discover, the ready-work queue, and the project's durable memory. It replaces markdown TODO lists and MEMORY files. GitHub Issues and Projects hold the owner's own planning; leave them to the owner. The proposal is the record of **intent**, reviewed by a human and preserved as history; Beads is the **work graph** — what is ready, what is blocked, what was discovered and deferred, and what a future session needs to remember. The `tracking-work` skill holds the procedure.

```sh
workbench workflow status                            # syncs Beads, then reports ready work, items in progress, and memories
bd show <id>                                         # read one item before touching it
workbench beads claim <id>                           # claim before starting
bd create "Title" --description="why this exists" --type=task --priority=2
workbench beads close <id> --reason "Completed"      # close as this machine's claimant, then sync
bd remember "<fact>"                                 # durable project memory
workbench beads sync                                 # publish changes other machines should see
```

- Never run `bd edit`; it opens an interactive editor and will hang a non-interactive agent. Use `bd update` flags.
- Priorities are `0`–`4`, `0` highest. Default `2`. Do not invent a scale.
- Claim before starting, only with `workbench beads claim`, and close only when the work is actually done, only with `workbench beads close`: plain `bd close` runs as the Git user name and is refused for an item claimed as `<user>@<short host>`. Do not start work the claim reports as `held` or `lost`.
- Each machine has its own Beads replica, and other machines see your changes only after a sync. `workbench workflow status`, `git push`, and `git pull` sync on their own; after creating, closing, or updating an item another machine should see, run `workbench beads sync`.
- Never run `bd dolt push`, `bd dolt pull`, or `bd sync` directly, and never `git push --mirror`, which deletes the remote's Beads data.
- Resolve a Beads sync conflict yourself, as the `tracking-work` skill describes, and report afterwards what you resolved and why.
- Never run `bd hooks install`; hk owns the git hooks and runs `bd hooks run <hook>` as a step.
- `.beads/issues.jsonl` is a passive export that is committed; the database is the source of truth.

A defect that stops the current implementation from satisfying its proposal is **not** a new work item. It is the current work, and it gets fixed.

## Change files

Every pull request adds at least one Knope change file in `.changeset/`, written for someone reading release notes rather than for the reviewer of the diff. Create one with `workbench change-files create`; never run `knope document-change`, which prompts. A pull request with no user-visible change carries a `Release-Note: none` trailer on one of its commits (`git commit --trailer "Release-Note: none"`) instead. CI runs `workbench change-files check` on every pull request except the release pull request.

## Tooling

- **mise** owns toolchains, environment, and the task graph, including the Workbench pin. `mise install` provisions the pinned tools and re-links `.workbench/`. Every command an agent runs routinely is a `mise run <task>` or a `workbench` command; add a task rather than documenting a bare command.
- **hk** owns every git hook. `mise run setup` installs them with `hk install --mise`. Never install hooks any other way, and never bypass them with `--no-verify`. The `commit-msg` hook holds every commit to [Scoped Commits](https://scopedcommits.com/) with `workbench commits check`. It accepts Workbench's process scopes, the scopes in `.agents/commit-scopes`, and the `<NNNN>-<slug>` of any proposal; `workbench commits scopes` lists them.
- **Varlock** owns environment variables and secrets. A project declares them in a committed `.env.schema`, and a task that needs a secret runs under `varlock run --`. Never put a secret in mise `[env]`, in a file, or in a commit.
- **Beads (`bd`)** owns the work graph and agent memory.
- **Knope** owns versions, the changelog, and releases. Merging Knope's release pull request is the explicit release authorization the process requires.
- **Worktrunk (`wt`)** owns worktrees. Run parallel feature work in a worktree rather than stashing.

## Also binding

- `.workbench/rules/code-quality.md` — what good code looks like.
- `.workbench/rules/commit-discipline.md` — commits, staging, push and pull-request policy.
- `.workbench/rules/enforcement-hierarchy.md` — where a new rule belongs.
- `.workbench/rules/verification.md` — what counts as evidence that work is done.

The cross-cutting reasoning skills:

- `grounding-a-problem` — learn the problem domain before deciding on a solution.
- `ontology-review` — review the conceptual model when important domain concepts change.
- `reframing-a-problem` — challenge the problem formulation when the current model stops explaining reality.
- `systematic-debugging` — find root cause when the intended behavior is sound but implementation is wrong.
