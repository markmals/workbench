# Agent Contract

This project uses the **Workbench** development process. A human approves intent in a proposal; agents implement it as tests, guides, and code; an independent review checks them against each other before the human reviews the result. [`PROCESS.md`](PROCESS.md) describes the whole loop and its artifacts for humans. The skills named below hold the procedures.

Run `mise run status` at the start of every session and whenever you are unsure where the work stands. It reads the branch, the pull request, and the active proposal, and prints the phase and the next step. The human is never expected to remember a command or skill name; "keep going" or a plain description of a problem is enough to orient from it.

## Does this change need a proposal?

Ask one question: **does this change decide something?**

- If intent already exists — an approved proposal promises the behavior, or the correct behavior is obvious (a typo, a dead link, a bug whose fix is unambiguous) — implement it directly. Say which case it is, in one line, before you start.
- If the change decides something, ask the human whether they want a proposal for it. Only after an explicit yes: create a branch, push it, open a draft pull request, and write the proposal with `.agents/skills/writing-a-proposal/`. A request to "start a feature" or "write a proposal" is not that yes, and neither is `mise run status` output.
- If a small fix turns out to need a design decision, stop and ask the proposal question then.

Proposal status belongs to the human. They move a proposal from `draft` to `awaiting-implementation`, and from `active-review` to `accepted`, `returned-for-revisions`, or `rejected`. You move it to `active-review` when you hand finished work to them, and to `implemented` in the accepted pull request before merge (`.agents/skills/completing-a-feature/`).

## Implementing an approved proposal

Use `.agents/skills/implementing-a-proposal/`. In short:

1. Read the proposal's Detailed design and the code it touches.
2. Write failing tests for the behavior Detailed design specifies, and watch them fail for the right reason.
3. When the change alters what a user does, sees, or must know, write or extend the guide in `guides/` from the proposal, not from the code, and check it against the tests.
4. Write the code.
5. Verify with `mise run check`. Report the commands you ran and what they printed.
6. Get one independent review with `.agents/skills/reviewing-an-implementation/` and fix what it confirms. Then post the readiness report, move the proposal to `active-review`, and mark the pull request ready, together.

Which of tests, guides, and code a change needs is a judgement; the order is not. A change with no user-facing surface needs no guide, and a change with nothing meaningfully testable needs no test, but say so rather than skipping quietly.

Work inline by default. Use subagents only when they buy something specific: slices that are independent, touch separate files, and save real time by running in parallel, or a fresh-context reviewer that has not seen your reasoning. In Oh My Pi, `.omp/tools/dynamic-workflows/` registers a `workflow` tool for agent work with real dependencies between branches; a fixed batch of independent slices uses `task`.

## Rules

- **Stay inside the proposal.** Behavior it does not describe does not get built; if it should exist, revise the proposal with the human. A defect outside its scope becomes a Beads item ([Work tracking](#work-tracking), `.agents/skills/tracking-work/`); a defect that stops the proposal from working is your work.
- **Raise ambiguity.** When the proposal, vision, a policy, or a decision is ambiguous or conflicts with the work in a way that changes behavior, stop and raise it rather than choosing silently.
- **Never write production code before its failing test**, when the behavior warrants a test at all.
- **Never write a guide from finished code.**
- **Never claim work is done without running the verifying command this turn** and reading its output ([`.agents/rules/verification.md`](.agents/rules/verification.md)).
- **Find a bug's root cause before fixing it** ([`.agents/skills/systematic-debugging/`](.agents/skills/systematic-debugging/)).
- **The adversarial reviewer reports; it never edits.**
- **Never delete a branch, force-push, publish a release, or deploy without explicit human confirmation.**
- **Record a policy or decision only after discussing it with the human.** Most features establish neither; a record belongs there only when its knowledge outlives the change.

## Quality gates

```sh
mise run setup       # idempotent first-run bootstrap after a clone
mise run check       # the single blocking gate; CI and the pre-push hook both call it
mise run fmt         # format in place; accepts file paths
mise run lint        # lint, applying the safe fixes
mise run typecheck   # type-check both TypeScript projects
mise run test        # the record tools' and the workflow tool's tests
mise run validate    # record frontmatter, IDs, and cross-references
mise run docs:build  # the guides site builds
mise run status      # phase, next action, who acts
mise run changeset   # add a changeset for this change
```

`mise run check` is the gate: formatting, lint, types, tests, the record rules, and the guides site, in one task. Nothing is done until it passes in this turn. `mise tasks` lists everything available.

## Work tracking

Beads (`bd`) is the work tracker, the ready-work queue, and durable agent memory. It replaces GitHub-issue defect tracking, markdown TODO lists, and MEMORY files. Proposals and Beads answer different questions: the proposal is the record of **intent**, reviewed by a human and preserved as history; Beads is the **work graph** — what is ready, what is blocked, what was discovered and deferred, and what a future session needs to remember.

```sh
bd ready                 # what can be worked on now
bd show <id>             # read one item before touching it
bd update <id> --claim   # claim atomically before starting
bd create "Title" --description="why this exists" --type=task --priority=2
bd close <id> --reason="Completed"
bd remember "<fact>"     # durable project memory
```

Rules that matter:

- Never run `bd edit`; it opens an interactive editor and will hang a non-interactive agent. Use `bd update` flags.
- Priorities are `0`–`4`, `0` highest. Default `2`. Do not invent a scale.
- Claim before starting and close only when the work is actually done.
- Never run `bd hooks install`; hk owns the git hooks and runs `bd hooks run <hook>` as a step.
- `.beads/issues.jsonl` is a passive export that is committed; the database is the source of truth.

A defect that stops the current implementation from satisfying its proposal is **not** a new work item. It is the current work, and it gets fixed. Skill: [`.agents/skills/tracking-work/`](.agents/skills/tracking-work/).

## Tooling

**Mise** owns toolchains, environment, and the task graph. Every command an agent runs routinely is a `mise run <task>`; `mise install` provisions the pinned toolchains from `mise.toml` and `mise.lock`, Bun among them — one test loads the `workflow` tool through Oh My Pi's real loader, which runs on Bun. Add a task rather than documenting a bare command: a task is an enforceable tier, prose is not.

**hk** owns every git hook. `mise run setup` installs them with `hk install --mise`. Never install hooks any other way, and never bypass them with `--no-verify`. The `commit-msg` hook holds every commit to [Scoped Commits](https://scopedcommits.com/) with `tools/check-scoped-commit.ts`, whose allowed scopes live in `.agents/commit-scopes`; `pre-push` runs the gate.

**Fnox** owns secrets. A task that needs a secret declares it with `secrets = [...]` in `mise.toml`, and Fnox resolves it from 1Password or the macOS Keychain at task time. Never put a secret in `[env]`, in a file, or in a commit. Workbench itself needs none.

**Beads (`bd`)** owns the work graph and agent memory. See [Work tracking](#work-tracking).

**Changesets** owns the changelog and versioning. `mise run changeset` adds one; `mise run version` applies it. Every pull request carries a changeset.

**Worktrunk (`wt`)** owns worktrees. `wt switch --create <name>` creates a parallel worktree that trusts and installs its own toolchain and copies the ignored files listed in `.worktreeinclude`. Run parallel feature work in a worktree rather than stashing.

**GitHub Actions** runs the same `mise run check` the pre-push hook runs. If it passes locally and fails in CI, the difference is environmental — find it, do not special-case the workflow.

## Harness setup

The process lives harness-neutral in `AGENTS.md` and `.agents/`, which stay the source of truth and which no harness forks. Lifecycle wiring is the only harness-specific part, and this repository ships it for [Oh My Pi](https://omp.sh):

| What | How Oh My Pi gets it |
| --- | --- |
| This contract | Loads `AGENTS.md` from the repository root |
| Skills | Discovers `.agents/skills/` |
| `workflow` tool | `.omp/tools/dynamic-workflows/` |
| Before the first turn | `.omp/extensions/prime-beads.ts` runs `bd prime` and injects the work graph and durable memories into the session |
| After every edit | `.omp/extensions/format-after-edit.ts` runs `mise run fmt -- <files>` on each file the `edit` or `write` tool changed and reports a reformat or a parse failure |
| Before the turn ends | `.omp/extensions/check-before-stop.ts` runs `mise run check`; a failure sends the agent back once with the output |

A harness without extensions loses only the convenience, not the rule: `bd prime` and `mise run check` are commands the agent runs itself, and hk's `pre-push` hook and CI run the gate regardless. Another harness wires the same three events into its own configuration — see [`guides/adapting-your-harness.md`](guides/adapting-your-harness.md) — and commits the result rather than forking `.agents/`.

Git hooks belong to hk, not to a harness: `mise run setup` installs them, and they run for every commit regardless of who or what made it.

## Also binding

- [`.agents/rules/code-quality.md`](.agents/rules/code-quality.md) — what good code looks like here.
- [`.agents/rules/commit-discipline.md`](.agents/rules/commit-discipline.md) — commits, staging, push and PR policy.
- [`.agents/rules/enforcement-hierarchy.md`](.agents/rules/enforcement-hierarchy.md) — where a new rule belongs.

### Cross-cutting reasoning skills

- [`.agents/skills/grounding-a-problem/`](.agents/skills/grounding-a-problem/) — learn the problem domain before deciding on a solution.
- [`.agents/skills/ontology-review/`](.agents/skills/ontology-review/) — review the conceptual model when important domain concepts change.
- [`.agents/skills/reframing-a-problem/`](.agents/skills/reframing-a-problem/) — challenge the problem formulation when the current model stops explaining reality.
- [`.agents/skills/systematic-debugging/`](.agents/skills/systematic-debugging/) — find root cause when the intended behavior is sound but implementation is wrong.
