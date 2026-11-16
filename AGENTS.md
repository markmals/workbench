# Agent Contract

This project uses the **Workbench** development process. A human approves a change in a proposal; agents write it into the project's current specifications, then tests, guides, and code; an independent review checks them against each other before the human reviews the result. [`PROCESS.md`](PROCESS.md) describes the whole loop for humans, including the artifact roles and where current requirements live; [`guides/current-specifications.md`](guides/current-specifications.md) shows how to keep them. The skills named below hold the procedures.

Run `mise run status` at the start of every session and whenever you are unsure where the work stands. It reads the branch, the pull request, and the active proposal, and prints the phase and the next step. It also syncs Beads with the other machines and shows ready work, the items in progress, and the project's memories. The human is never expected to remember a command or skill name; "keep going" or a plain description of a problem is enough to orient from it.

## Does this change need a proposal?

Ask one question: **does this change decide something?** Look and feel — how an interface looks and how it is operated — is the exception: the human decides it by looking at the running UI, not in a proposal.

- If intent is established by a current specification, the approved change currently being implemented on this branch, or an unambiguous correction (a typo, a dead link, a bug whose fix is obvious), implement it directly and say which case applies. Completed proposals are historical evidence; they cannot authorize restoring obsolete behavior. Raise any conflict with current specifications before acting.
- If the human is shaping look and feel — layout, sizes, spacing, type, color, motion, which control presents a choice, how it is operated, and wording whose exact words are not themselves a requirement — make the changes directly with `.agents/skills/polishing-a-ui/`. Say so in one line before you start.
- If the change decides something, ask the human whether they want a proposal for it. Only after an explicit yes: create a branch, push it, open a draft pull request, and write the proposal with `.agents/skills/writing-a-proposal/`. A request to "start a feature" or "write a proposal" is not that yes, and neither is `mise run status` output.
- If a small fix turns out to need a design decision, or polish turns out to change what the feature does, stop and ask the proposal question then.

Proposal status belongs to the human. They move a proposal from `draft` to `awaiting-implementation`, and from `active-review` to `accepted`, `returned-for-revisions`, or `rejected`. You move it to `active-review` when you hand finished work to them, and to `implemented` in the accepted pull request before merge (`.agents/skills/completing-a-feature/`).

## Implementing an approved proposal

Use `.agents/skills/implementing-a-proposal/`. In short:

1. Read the proposal, the current sources it links, and the code it touches. Completed proposals are history; current requirements live in the sources that own them.
2. Revise the affected current specifications (API contract documentation, schemas, reference prose, or feature files) and any agreed policy or decision so they state the approved change. When authoring or maintaining `.feature` files, or when the human chooses Gherkin for a surface, use `.agents/skills/specifying-behavior-with-gherkin/`.
3. Write failing tests for the behavior those specifications now require, and watch them fail for the right reason.
4. When the change alters what a user does, sees, or must know, write or extend the guide in `guides/` from the intended contract, not from the code, link the source that owns that contract, and check the guide against the tests.
5. Write the code.
6. Verify with `mise run check`. Report the commands you ran and what they printed. Once the complete implementation has passed the applicable checks and is pushed, wait for CI to pass on that revision (`gh pr checks --watch`), then mark the pull request ready for maintainer review; it stays a draft only while it holds just the proposal. When CI fails, fix the failure and push again. The proposal stays `awaiting-implementation` until the `active-review` handoff.
7. Get one independent review with `.agents/skills/reviewing-an-implementation/` and fix what it confirms. Then post the readiness report and move the proposal to `active-review`; the pull request is already ready.

Which of specifications, tests, guides, and code a change needs is a judgement; the order is not. A change with no user-facing surface needs no guide, and a change with nothing meaningfully testable needs no test, but say so rather than skipping quietly.

Work inline by default. Use subagents only when they buy something specific: slices that are independent, touch separate files, and save real time by running in parallel, or a fresh-context reviewer that has not seen your reasoning. In Oh My Pi, `.omp/tools/dynamic-workflows/` registers a `workflow` tool for agent work with real dependencies between branches; a fixed batch of independent slices uses `task`.

## Rules

- **Stay inside the proposal.** Behavior it does not describe does not get built; if it should exist, revise the proposal with the human. Look and feel is not behavior a proposal must describe: choose what the proposal leaves open to fit the project's existing interface and its platform's conventions, and let the human refine it on the running UI. A defect outside its scope becomes a Beads item ([Work tracking](#work-tracking), `.agents/skills/tracking-work/`); a defect that stops the proposal from working is your work.
- **Raise ambiguity.** When the proposal, a current specification, the vision, a policy, or a decision is ambiguous or conflicts with the work in a way that changes behavior, stop and raise it rather than choosing silently.
- **Never write production code before its failing test**, when the behavior warrants a test at all.
- **Never write a specification or guide from finished code.**
- **Never claim work is done without running the verifying command this turn** and reading its output ([`.agents/rules/verification.md`](.agents/rules/verification.md)).
- **Find a bug's root cause before fixing it** ([`.agents/skills/systematic-debugging/`](.agents/skills/systematic-debugging/)).
- **The adversarial reviewer reports; it never edits.**
- **Never delete a branch, force-push, publish a release, or deploy without explicit human confirmation.**
- **Add, revise, or delete a policy or decision only after agreeing it with the human.** Revise a changed record in place and delete a retired one; neither gets a replacement record or `supersedes`. Most features establish neither; a record belongs there only when its knowledge outlives the change.

## Quality gates

```sh
mise run setup       # idempotent first-run bootstrap after a clone
mise run check       # the single blocking gate; CI runs it on every push
mise run fmt         # format in place; accepts file paths
mise run lint        # lint, applying the safe fixes
mise run typecheck   # type-check both TypeScript projects
mise run test        # the record tools' and the workflow tool's tests
mise run validate    # record frontmatter, IDs, and cross-references
mise run docs:build  # the guides site builds
mise run status      # phase, next action, who acts; syncs and reports Beads
mise run beads:sync  # sync this machine's Beads replica with the remote
mise run changeset   # add a changeset for this change
```

`mise run check` is the gate: formatting, lint, types, tests, the record rules, and the guides site, in one task. Nothing is done until it passes in this turn. No git hook runs it; CI does, on every push, and a pull request is marked ready or merged only after CI passes on its latest revision. `mise tasks` lists everything available.

## Work tracking

Beads (`bd`) holds the work agents discover, the ready-work queue, and the project's durable memory. It replaces markdown TODO lists and MEMORY files. GitHub Issues and Projects hold the owner's own planning; leave them to the owner. Proposals and Beads answer different questions: the proposal is the record of **intent**, reviewed by a human and preserved as history; Beads is the **work graph** — what is ready, what is blocked, what was discovered and deferred, and what a future session needs to remember.

```sh
mise run status               # syncs Beads, then shows ready work, items in progress, and memories
bd show <id>                  # read one item before touching it
mise run beads:claim -- <id>  # claim before starting
bd create "Title" --description="why this exists" --type=task --priority=2
bd close <id> --reason="Completed"
bd remember "<fact>"          # durable project memory
mise run beads:sync           # publish changes other machines should see
```

Rules that matter:

- Never run `bd edit`; it opens an interactive editor and will hang a non-interactive agent. Use `bd update` flags.
- Priorities are `0`–`4`, `0` highest. Default `2`. Do not invent a scale.
- Claim before starting, only with `mise run beads:claim`, and close only when the work is actually done. Do not start work the claim reports as `held` or `lost`.
- Each machine has its own Beads replica, and other machines see your changes only after a sync. `mise run status`, `git push`, and `git pull` sync on their own; after creating, closing, or updating an item another machine should see, run `mise run beads:sync`.
- Every sync runs through mise. Never run `bd dolt push`, `bd dolt pull`, or `bd sync` directly, and never `git push --mirror`, which deletes the remote's Beads data.
- Resolve a Beads sync conflict yourself, as the skill describes, and report afterwards what you resolved and why.
- Never run `bd hooks install`; hk owns the git hooks and runs `bd hooks run <hook>` as a step.
- `.beads/issues.jsonl` is a passive export that is committed; the database is the source of truth.

A defect that stops the current implementation from satisfying its proposal is **not** a new work item. It is the current work, and it gets fixed. Skill: [`.agents/skills/tracking-work/`](.agents/skills/tracking-work/).

## Tooling

**Mise** owns toolchains, environment, and the task graph. Every command an agent runs routinely is a `mise run <task>`; `mise install` provisions the pinned toolchains from `mise.toml` and `mise.lock`, Bun among them — one test loads the `workflow` tool through Oh My Pi's real loader, which runs on Bun. Add a task rather than documenting a bare command: a task is an enforceable tier, prose is not.

**hk** owns every git hook. `mise run setup` installs them with `hk install --mise`. Never install hooks any other way, and never bypass them with `--no-verify`. The `commit-msg` hook holds every commit to [Scoped Commits](https://scopedcommits.com/) with `tools/check-scoped-commit.ts`, whose allowed scopes live in `.agents/commit-scopes`; `pre-commit` formats and lints staged files; `pre-push` and `post-merge` sync Beads.

**Fnox** owns secrets. A task that needs a secret declares it with `secrets = [...]` in `mise.toml`, and Fnox resolves it from 1Password or the macOS Keychain at task time. Never put a secret in `[env]`, in a file, or in a commit. Workbench itself needs none.

**Beads (`bd`)** owns the work graph and agent memory. See [Work tracking](#work-tracking).

**Changesets** owns the changelog and versioning. `mise run changeset` adds one; `mise run version` applies it. Every pull request carries a changeset.

**Worktrunk (`wt`)** owns worktrees. `wt switch --create <name>` creates a parallel worktree that trusts and installs its own toolchain and copies the ignored files listed in `.worktreeinclude`. Run parallel feature work in a worktree rather than stashing.

**GitHub Actions** runs `mise run check` on every push; it is the only place the gate runs automatically. If it passes locally and fails in CI, the difference is environmental — find it, do not special-case the workflow.

## Harness setup

The process lives harness-neutral in `AGENTS.md` and `.agents/`, which stay the source of truth and which no harness forks. A harness needs only the contract and the skills; Workbench wires no harness lifecycle events. This repository ships the [Oh My Pi](https://omp.sh) setup:

| What            | How Oh My Pi gets it                       |
| --------------- | ------------------------------------------ |
| This contract   | Loads `AGENTS.md` from the repository root |
| Skills          | Discovers `.agents/skills/`                |
| `workflow` tool | `.omp/tools/dynamic-workflows/`            |

Orientation, formatting, and the gate are commands, not harness events: run `mise run status` at the start of a session, `mise run fmt -- <files>` when you want formatting before the `pre-commit` hook applies it, and `mise run check` before claiming work is done. Another harness points at the same contract and skills — see [`guides/adapting-your-harness.md`](guides/adapting-your-harness.md) — and commits the result rather than forking `.agents/`.

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
