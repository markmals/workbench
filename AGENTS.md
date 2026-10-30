# Agent Contract

This project uses the **Workbench** development process. A human approves intent in a proposal; agents implement it as tests, guides, and code; an independent review checks them against each other before the human reviews the result. [`PROCESS.md`](PROCESS.md) describes the whole loop and its artifacts for humans. The skills named below hold the procedures.

Run `vp run status` at the start of every session and whenever you are unsure where the work stands. It reads the branch, the pull request, and the active proposal, and prints the phase and the next step. The human is never expected to remember a command or skill name; "keep going" or a plain description of a problem is enough to orient from it.

## Does this change need a proposal?

Ask one question: **does this change decide something?**

- If intent already exists — an approved proposal promises the behavior, or the correct behavior is obvious (a typo, a dead link, a bug whose fix is unambiguous) — implement it directly. Say which case it is, in one line, before you start.
- If the change decides something, ask the human whether they want a proposal for it. Only after an explicit yes: create a branch, push it, open a draft pull request, and write the proposal with `.agents/skills/writing-a-proposal/`. A request to "start a feature" or "write a proposal" is not that yes, and neither is `vp run status` output.
- If a small fix turns out to need a design decision, stop and ask the proposal question then.

Proposal status belongs to the human. They move a proposal from `draft` to `awaiting-implementation`, and from `active-review` to `accepted`, `returned-for-revisions`, or `rejected`. You move it to `active-review` when you hand finished work to them, and to `implemented` in the accepted pull request before merge (`.agents/skills/completing-a-feature/`).

## Implementing an approved proposal

Use `.agents/skills/implementing-a-proposal/`. In short:

1. Read the proposal's Detailed design and the code it touches.
2. Write failing tests for the behavior Detailed design specifies, and watch them fail for the right reason.
3. When the change alters what a user does, sees, or must know, write or extend the guide in `guides/` from the proposal, not from the code, and check it against the tests.
4. Write the code.
5. Verify with `vp run check` and any project-specific gates. Report the commands you ran and what they printed.
6. Get one independent review with `.agents/skills/reviewing-an-implementation/` and fix what it confirms. Then post the readiness report, move the proposal to `active-review`, and mark the pull request ready, together.

Which of tests, guides, and code a change needs is a judgement; the order is not. A change with no user-facing surface needs no guide, and a change with nothing meaningfully testable needs no test, but say so rather than skipping quietly.

Work inline by default. Use subagents only when they buy something specific: slices that are independent, touch separate files, and save real time by running in parallel, or a fresh-context reviewer that has not seen your reasoning. In Oh My Pi, `.omp/tools/dynamic-workflows/` registers a `workflow` tool for agent work with real dependencies between branches; a fixed batch of independent slices uses `task`.

## Rules

- **Stay inside the proposal.** Behavior it does not describe does not get built; if it should exist, revise the proposal with the human. A defect outside its scope becomes a GitHub issue (`.agents/skills/tracking-defects/`); a defect that stops the proposal from working is your work.
- **Raise ambiguity.** When the proposal, vision, a policy, or a decision is ambiguous or conflicts with the work in a way that changes behavior, stop and raise it rather than choosing silently.
- **Never write production code before its failing test**, when the behavior warrants a test at all.
- **Never write a guide from finished code.**
- **Never claim work is done without running the verifying command this turn** and reading its output ([`.agents/rules/verification.md`](.agents/rules/verification.md)).
- **Find a bug's root cause before fixing it** ([`.agents/skills/systematic-debugging/`](.agents/skills/systematic-debugging/)).
- **The adversarial reviewer reports; it never edits.**
- **Never delete a branch, force-push, or publish a release without explicit human confirmation.**
- **Record a policy or decision only after discussing it with the human.** Most features establish neither; a record belongs there only when its knowledge outlives the change.

## Quality gates

[Vite+](https://viteplus.dev) runs the toolchain. `.node-version` pins Node, which runs the TypeScript in `tools/` directly; `package.json#devEngines` pins pnpm. `vp run test` also needs [Bun](https://bun.sh) on the `PATH`: one test loads the `workflow` tool through Oh My Pi's real loader, which runs on Bun.

```sh
vp install           # dependencies, with the pinned pnpm
vp run check         # the full gate: vp check, validate, test, docs:build
vp fmt               # format
vp check             # format check, lint, and type-check
vp run validate      # artifact frontmatter, IDs, and cross-references
vp run test          # the record tools' and the workflow tool's tests
vp run docs:build    # the guides site builds
```

Run project-specific gates (unit, integration, end-to-end) as they exist. `vp run` with no task lists everything available.

## Harness setup

The process lives harness-neutral in `.agents/`, which stays the source of truth. This repository wires it into [Oh My Pi](https://omp.sh):

| What | How Oh My Pi gets it |
| --- | --- |
| This contract | Loads `AGENTS.md` from the repository root |
| Skills | Discovers `.agents/skills/` |
| `workflow` tool | `.omp/tools/dynamic-workflows/` |
| Format after every edit | `.omp/extensions/format-after-edit.ts` runs `vp fmt` on each file the `edit` or `write` tool changed and reports a reformat or a parse failure |
| Gate before the turn ends | `.omp/extensions/check-before-stop.ts` runs `vp run check`; a failure sends the agent back once with the output |

Another harness wires the same lifecycle events, listed in [`.agents/hooks/README.md`](.agents/hooks/README.md), into its own configuration; adapt it once — see [`guides/adapting-your-harness.md`](guides/adapting-your-harness.md) — and commit the result rather than forking `.agents/`. The commit-message hook is a Git hook that `vp install` installs. It checks only commits made from Oh My Pi, which it recognizes by the `OMPCODE=1` that Oh My Pi exports to every command; another harness needs its own marker added there.

## Also binding

- [`.agents/rules/code-quality.md`](.agents/rules/code-quality.md) — what good code looks like here.
- [`.agents/rules/commit-discipline.md`](.agents/rules/commit-discipline.md) — commits, staging, push and PR policy.
- [`.agents/rules/enforcement-hierarchy.md`](.agents/rules/enforcement-hierarchy.md) — where a new rule belongs.

### Cross-cutting reasoning skills

- [`.agents/skills/grounding-a-problem/`](.agents/skills/grounding-a-problem/) — learn the problem domain before deciding on a solution.
- [`.agents/skills/ontology-review/`](.agents/skills/ontology-review/) — review the conceptual model when important domain concepts change.
- [`.agents/skills/reframing-a-problem/`](.agents/skills/reframing-a-problem/) — challenge the problem formulation when the current model stops explaining reality.
- [`.agents/skills/systematic-debugging/`](.agents/skills/systematic-debugging/) — find root cause when the intended behavior is sound but implementation is wrong.
