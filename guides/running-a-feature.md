---
id: guide.running-a-feature
title: Running a Feature
describes: [proposal.0001, proposal.0004]
---

# Running a Feature

## What actually happens from idea to merged, and what is your part?

Follow one small feature: project members need to export a report. You describe it, shape its proposal, review a real result, and accept or return it. The agent does the branch work, implementation, checks, reviews, preview, and release preparation.

Use one command whenever you want to orient yourself:

```sh
mise run status
```

It takes no arguments. It derives the current phase from the branch, pull request, and active proposal, then names the next actor.

## What you actually do

1. Describe the work you want done.
2. Iterate on the proposal until it captures the intended design.
3. Review the readiness report, diff, and preview.
4. Return it for revisions or accept it.

The agent owns the work between those decisions. Comment on the pull request or commit directly to its branch; the agent reads a direct commit as instruction.

## Not every change needs all of this

What follows describes substantial work — new or changed behavior, anything where a reasonable person could build it differently. Plenty of changes are not that.

A typo, a dead link, or a bug whose correct behavior was never in question does not need a proposal, and often does not need a pull request either. The useful question is whether the change **decides** something. If the intended behavior already exists — because a proposal promised it, or because the contract is too obvious to write down — the agent is correcting an implementation, not defining one, and it should just fix it.

The agent judges this and tells you which it thinks the change is. You can ask for the full process on anything, at any time, without explaining why. Escalating is always available; the agent quietly deciding your feature is a small fix is the failure mode to watch for.

Polishing an interface does not need a proposal either. A proposal says what a feature does: what a user can do and find out, what happens, and what they are told. Its look and feel — layout, sizes, spacing, type, color, motion, which control presents a choice, how it is operated, and the wording of labels — is yours to decide by looking at the running UI. Ask for changes as you see them. The agent keeps the UI running where you can see it, makes one change at a time, commits what you keep, and lands the work the way the project lands other changes made without a proposal. If a request would change what the feature does, such as keeping a provider's models listed after signing out, it stops and asks whether you want a proposal.

Here is the status command run in this repository while its proposal was ready for implementation:

```text
$ node tools/status.ts ⊘ cache disabled
proposal.0001  Workbench 2.0  ·  awaiting-implementation
PR #1 (draft)  https://github.com/markmals/workbench/pull/1
branch workbench-2

Phase 3 — Implementation
  The proposal is approved for implementation.

Next (agent): Write failing tests, then guides, then code, and pass the gates. Self-check against the proposal, get one independent review, then post the readiness report and mark the pull request ready.
              Skill: .agents/skills/implementing-a-proposal/
```

The exact branch, proposal, and pull-request details change. The phase and next actor are the parts to use.

## Phase 1 — Preparation

### You

Tell the agent: “Project members need to export a report.” Then do nothing while it creates the workspace.

### The agent

The agent creates a branch, pushes it, and opens a draft pull request before writing the proposal or implementation.

The pull request exists this early because it is the durable workspace for one feature: its proposal, implementation, review discussion, and readiness report accumulate there as one reviewable thing.

### What status reports

On the default branch, or on a branch without a pull request, it reports **Phase 1 — Preparation** and tells the agent to ask whether you want a proposal before it creates a branch or pull request for one; corrections and look-and-feel polish go ahead without one. You are not expected to create the branch or the pull request yourself.

## Phase 2 — Proposal

### You

React to the proposal until it represents the change you intend. Comment on the pull request, or edit `proposals/<NNNN>-<slug>.md` directly on the branch. Both work. This is where nearly all of the leverage is, and it can take many rounds. Spend it on what the feature does; how it looks is decided on the preview, not here.

Do not leave an uncertainty for the agent to infer. It records one as:

```text
[NEEDS CLARIFICATION: Which users can export a report?]
```

Answer the question or revise the design. The marker is not permission for the agent to guess. A proposal cannot move beyond `draft` with an unresolved marker.

When the proposal is settled, you decide that it may become `awaiting-implementation`. You may ask the agent to make the edit, but the judgement remains yours. See [Writing a Good Proposal](./writing-a-good-proposal.md).

### The agent

The agent writes `proposals/<NNNN>-<slug>.md` and pushes it as soon as it is coherent enough to react to — not after it is polished. It incorporates your comments and direct commits, preserving the durable design record.

### What status reports

With a draft pull request but no proposal, it reports **Phase 2 — Proposal** and tells the agent to ask whether you want one, unless you already answered; corrections and look-and-feel polish continue without one. Once a `draft` proposal exists, it still reports Phase 2 and tells you to edit it or request changes. It remains `draft` for every round until you settle it.

## Phase 3 — Implementation

### You

Do nothing. This phase may take a long time. The agent returns with a reviewable result rather than asking you to supervise its internal sequence.

### The agent

The order is fixed:

1. Write failing tests for the report export the proposal specifies.
2. Write guides from the proposal and compare them with the tests.
3. Implement the code. Where the proposal leaves look and feel open, the agent chooses it to fit the project's existing interface and its platform's conventions, for you to refine in review.
4. Run the gate with `mise run check`.

`mise run check` runs the tests, then `hk check --all` for formatting, lint, and type checks, then `validate` and `docs:build`.

The order protects the design. Tests written after code agree with code that already exists; guides written after implementation document its accidents. Writing both from the proposal gives each an independent claim before code.

Passing the gates is not the end. Still in Phase 3, the agent reviews the proposal, tests, guides, and code together; dispatches an adversarial review to a different model; resolves valid findings; and preserves declined findings.

It then publishes the preview, posts the readiness report, and marks the pull request ready as one coupled action. That action also changes the proposal to `active-review`.

For the preview, the agent chooses the cheapest realistic artifact through which you can exercise the change — a prerelease package, preview deployment or endpoint, deployed documentation site, installable executable, or installable desktop or mobile build. See [Choosing a Preview](./choosing-a-preview.md).

### What status reports

Once you set `awaiting-implementation`, it reports **Phase 3 — Implementation** and names the agent as next. The command output above is a real example of that state. After the coupled final action, it reports **Phase 4 — Human review** and names you as next.

## Phase 4 — Human review

### You

Three things arrive together: a readiness-report comment, a diff, and a preview where you can export a report. Read the report, inspect the diff, and use it.

Pay particular attention to the **Declined adversarial findings** section. It records findings the agent disagreed with rather than silently dropping them — the highest-signal part of the report.

Ask for look-and-feel changes as you use it: tighter spacing, a different control, clearer wording. The agent applies them on the pull request while the work stays in review, so there is no status to change and no proposal to edit.

Then decide. Set `returned-for-revisions` when you want changes to what the feature does or to how it is built, or `accepted` when it is good enough to complete. You may ask the agent to update the status.

Feedback that changes what the feature does sends the proposal back to Phase 2 first. Feedback correcting only implementation returns directly to Phase 3.

### The agent

The agent reads your comments and commits, updates the affected work, and repeats the relevant implementation and review work. A look-and-feel change needs only the code change, the gate, a push, and a refreshed preview; the agent tells you what changed and where to look. It cannot accept the feature.

### What status reports

At `active-review`, it reports **Phase 4 — Human review** and tells you to review the diff and preview, ask for look-and-feel changes as you go, and then comment or accept. At `returned-for-revisions`, it reports **Phase 4 → 3 — Human review → Implementation**, and its next action covers the conditional case: update the proposal first if what the feature does changed. The tool cannot know which your feedback requires, so it names the common path and leaves the judgement in the text.

## Phase 5 — Completion and release

### You

After acceptance, explicitly confirm release and branch deletion when the agent asks. Those actions are never implied by accepting the feature.

### The agent

If the feature changed what the project is, the agent updates `VISION.md`. It records `implemented` in the original feature pull request after acceptance and confirms the record is in the pull request revision before marking a draft pull request ready or merging. Merge and any required release remain separate; a release needs its own authorization. The agent tracks unfinished delivery and performs only authorized cleanup.

### What status reports

At `accepted`, it reports **Phase 5 — Completion** and names the agent for the vision update and implementation record. Once `implemented` is committed in the feature pull request, it still reports Phase 5 while merge or delivery remains. The pull request's merge state and any required release are tracked separately, so no follow-up pull request is needed just to change the proposal status.

## Status reference

| Status                    | Phase                          | Who decides |
| ------------------------- | ------------------------------ | ----------- |
| `draft`                   | 2 — proposal                   | Agent       |
| `awaiting-implementation` | 3 — implementation             | You         |
| `active-review`           | 4 — human review               | Agent       |
| `returned-for-revisions`  | 4 → 3 — back to implementation | You         |
| `accepted`                | 5 — completion and release     | You         |
| `implemented`             | 5 — completion                 | Agent       |
| `rejected`                | closed                         | You         |
| `withdrawn`               | closed                         | Either      |
| `superseded`              | closed                         | Either      |

“Who decides” means the judgement, not the keystroke. You can direct the agent to update a status you decided; it cannot make either approval decision alone.

A policy or decision may emerge, but neither replaces the proposal. Use [Keeping the Record](./keeping-the-record.md) to decide what belongs in one.
