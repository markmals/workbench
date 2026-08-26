---
id: guide.running-a-feature
title: Running a Feature
describes: [proposal.0001]
---

# Running a Feature

## What actually happens from idea to merged, and what is your part?

Follow one small feature: project members need to export a report. You describe
it, shape its proposal, review a real result, and accept or return it. The
agent does the branch work, implementation, checks, reviews, preview, and release preparation.

Use one command whenever you want to orient yourself:

```sh
mise run status
```

It takes no arguments. It derives the current phase from the branch, pull
request, and active proposal, then names the next actor.

## What you actually do

1. Describe the work you want done.
2. Iterate on the proposal until it captures the intended design.
3. Review the readiness report, diff, and preview.
4. Return it for revisions or accept it.

The agent owns the work between those decisions. Comment on the pull request
or commit directly to its branch; the agent reads a direct commit as instruction.

Here is the status command run in this repository while its proposal was ready
for implementation:

```text
[status] $ node tools/status.mjs
proposal.0001  Workbench 2.0  ·  awaiting-implementation
PR #1 (draft)  https://github.com/markmals/workbench/pull/1
branch workbench-2

Phase 3 — Implementation
  The proposal is approved for implementation.

Next (agent): Write failing tests, then guides, then code and gates. Complete cross-artifact and adversarial reviews, post the readiness report, and mark the pull request ready.
              Skill: .agents/skills/implementing-a-proposal/
```

The exact branch, proposal, and pull-request details change. The phase and next
actor are the parts to use.

## Phase 1 — Preparation

### You

Tell the agent: “Project members need to export a report.” Then do nothing
while it creates the workspace.

### The agent

The agent creates a branch, pushes it, and opens a draft pull request before
writing the proposal or implementation.

The pull request exists this early because it is the durable workspace for one
feature: its proposal, implementation, review discussion, and readiness report
accumulate there as one reviewable thing.

### What status reports

Before a branch exists, it reports **Phase 1 — Preparation** and tells the
agent to create one. On a feature branch without a pull request, it again
reports Phase 1 and tells the agent to push the branch and open a draft pull
request. You are not expected to perform either action.

## Phase 2 — Proposal

### You

React to the proposal until it represents the change you intend. Comment on the
pull request, or edit `proposals/<NNNN>-<slug>.md` directly on the branch. Both
work. This is where nearly all of the leverage is, and it can take many rounds.

Do not leave an uncertainty for the agent to infer. It records one as:

```text
[NEEDS CLARIFICATION: Which users can export a report?]
```

Answer the question or revise the design. The marker is not permission for the
agent to guess. A proposal cannot move beyond `draft` with an unresolved marker.

When the proposal is settled, you decide that it may become
`awaiting-implementation`. You may ask the agent to make the edit, but the
judgement remains yours. See [Writing a Good Proposal](./writing-a-good-proposal.md).

### The agent

The agent writes `proposals/<NNNN>-<slug>.md` and pushes it as soon as it is
coherent enough to react to — not after it is polished. It incorporates your
comments and direct commits, preserving the durable design record.

### What status reports

With a draft pull request but no proposal, it reports **Phase 2 — Proposal**
and tells the agent to write and push one. Once a `draft` proposal exists, it
still reports Phase 2 and tells you to edit it or request changes. It remains
`draft` for every round until you settle it.

## Phase 3 — Implementation

### You

Do nothing. This phase may take a long time. The agent returns with a reviewable
result rather than asking you to supervise its internal sequence.

### The agent

The order is fixed:

1. Write failing tests for the report export the proposal specifies.
2. Write guides from the proposal and compare them with the tests.
3. Implement the code.
4. Run the gates with `mise run check`.

`mise run check` runs `fmt:check`, `validate`, `test`, and `docs:build`.

The order protects the design. Tests written after code agree with code that
already exists; guides written after implementation document its accidents.
Writing both from the proposal gives each an independent claim before code.

Passing the gates is not the end. Still in Phase 3, the agent reviews the
proposal, tests, guides, and code together; dispatches an adversarial review to
a different model; resolves valid findings; and preserves declined findings.

It then publishes the preview, posts the readiness report, and marks the pull
request ready as one coupled action. That action also changes the proposal to
`active-review`.

For the preview, the agent chooses the cheapest realistic artifact through
which you can exercise the change — a prerelease package, preview deployment
or endpoint, deployed documentation site, installable executable, or
installable desktop or mobile build. See [Choosing a Preview](./choosing-a-preview.md).

### What status reports

Once you set `awaiting-implementation`, it reports **Phase 3 — Implementation**
and names the agent as next. The command output above is a real example of that
state. After the coupled final action, it reports **Phase 4 — Human review**
and names you as next.

## Phase 4 — Human review

### You

Three things arrive together: a readiness-report comment, a diff, and a preview
where you can export a report. Read the report, inspect the diff, and use it.

Pay particular attention to the **Declined adversarial findings** section. It
records findings the agent disagreed with rather than silently dropping them —
the highest-signal part of the report.

Then decide. Set `returned-for-revisions` when you want changes, or `accepted`
when it is good enough to complete. You may ask the agent to update the status.

Feedback that changes the design sends the proposal back to Phase 2 first.
Feedback correcting only implementation returns directly to Phase 3.

### The agent

The agent reads your comments and commits, updates the affected work, and
repeats the relevant implementation and review work. It cannot accept the feature.

### What status reports

At `active-review`, it reports **Phase 4 — Human review** and tells you to
review the diff and preview, then comment or accept. At
`returned-for-revisions`, it reports the return from human review to the
proposal or implementation work the feedback requires.

## Phase 5 — Completion and release

### You

After acceptance, explicitly confirm release and branch deletion when the agent
asks. Those actions are never implied by accepting the feature.

### The agent

If the feature changed what the project is, the agent updates `VISION.md`. It
prepares merge, release, dependent-project updates where relevant, and cleanup.
It merges only after acceptance and sets `implemented` only after release.

### What status reports

At `accepted`, it reports **Phase 5 — Completion** and names the agent for the
vision update, merge, release, and cleanup. Once `implemented` is recorded on a
merged pull request, it reports that the feature is done.

## Status reference

| Status                    | Phase                      | Who decides |
| ------------------------- | -------------------------- | ----------- |
| `draft`                   | 2 — proposal               | Agent       |
| `awaiting-implementation` | 3 — implementation         | You         |
| `active-review`           | 4 — human review           | Agent       |
| `returned-for-revisions`  | 4 → 2 or 3                 | You         |
| `accepted`                | 5 — completion and release | You         |
| `implemented`             | 5 complete                 | Agent       |
| `rejected`                | closed                     | You         |
| `withdrawn`               | closed                     | Either      |
| `superseded`              | closed                     | Either      |

“Who decides” means the judgement, not the keystroke. You can direct the agent
to update a status you decided; it cannot make either approval decision alone.

A policy or decision may emerge, but neither replaces the proposal. Use
[Keeping the Record](./keeping-the-record.md) to decide what belongs in one.
