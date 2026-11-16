---
name: tracking-work
description: Use when finding ready work, claiming or closing a task, recording a defect discovered during a proposal or pull request, deciding whether discovered work is in scope or should be deferred, handing off across sessions, syncing Beads between machines or resolving a Beads sync outcome, or deciding between a local checklist and durable project tracking.
---

# Tracking Work

Beads (`bd`) is the project's work graph: the work agents discover, ready work, blockers, deferred follow-ups, and durable memory. The owner's own planning lives in GitHub Issues and Projects, not in Beads. A local checklist tracks the current turn's execution. It is not project state, and it does not survive a context reset.

## Start

```sh
mise run status # syncs Beads, then reports ready work, items in progress, and memories
bd show <id>    # read an item before touching it
```

If the Beads section reports anything other than `synced`, read [Sync outcomes](#sync-outcomes) before relying on it.

## The decisive test

**A defect that prevents the current implementation from satisfying its proposal is current work. Fix it.**

Filing an item to defer work the proposal already requires is the primary failure of this skill. Create or reuse an item only when the work is outside the proposal's scope, already present on the default branch, belongs to another feature or repository, or is otherwise legitimate future work that must outlive this pull request.

## Search before creating

Duplicates are the second failure.

```sh
bd search "<symptom keyword>" --status open,in_progress,blocked,deferred,closed --limit 0
bd list --desc-contains "<symptom keyword>" --status open,in_progress,blocked,deferred,closed --limit 0
```

`bd search` matches titles and IDs; `bd list --desc-contains` reaches the body text a title never mentions. Name every state: a claimed, blocked or deferred item is still a duplicate, and a narrower `--status` hides it. `--limit 0` lifts the 50-result default, which otherwise drops matches status-blind — the duplicate you miss is the one past the cut.

Search by observable symptom, affected area, and reproduction. Reuse the applicable open item. Reopen a closed one when the same defect returns; create a linked successor only for a materially distinct recurrence.

## Write an item that stands alone

A future reader has no access to this conversation. Include the incorrect behavior, the expected behavior, reproduction steps or a minimal reproduction, relevant environment or version information, links to the proposal, PR, test, or code where it was found, known constraints and suspected causes, and why it is outside the current proposal's scope.

```sh
bd create "Import accepts malformed records" \
  --type=bug --priority=1 \
  --description="Expected: the import rejects malformed records with a validation error. Actual: the record is accepted and validation deferred. Repro: run the import with a record missing its identifier. Found under proposal.0007 in PR #42; out of scope because that proposal does not change import validation."
```

Priorities are `0`–`4`, `0` highest, default `2`. Apply the project's existing types and labels. Link a blocking relationship so `bd ready` stays honest about what can actually be started.

## Claim, close, remember

```sh
mise run beads:claim -- <id>   # claim before starting; see Claiming below
mise run beads:close -- <id> "Fixed in PR #42"   # close as this machine's claimant, then sync
bd remember "<durable project fact>"
mise run beads:sync            # publish what you changed
```

Close only when the work is actually done. If later work fixes a deferred defect incidentally, close it with the PR or commit that resolved it rather than leaving it stale.

Durable knowledge — a constraint learned the hard way, a decision too small for a record and too costly to rediscover — goes in `bd remember`, never in a markdown memory file.

## Preserve the path to resolution

When a deferred defect is relevant to the current pull request, link its item from the PR or a review comment, and list it under **Defects discovered and deferred** in the readiness report. The deferral must be explicit and durable.

## Beads across machines

The same project is worked on from several machines. Read this section before you claim, sync, or trust what `bd` shows you.

### The model

- Each machine has one Beads **replica** per project: a full local Dolt database in the main checkout's `.beads/`. Every worktree of that checkout shares it.
- `bd` commands read and write only this replica. Another machine's changes arrive only through a sync, and yours reach it only through one.
- The replicas meet at `refs/dolt/data` on the project's `origin` repository. It is not a branch: it does not appear in branch lists, and `git clone` and `git fetch` do not bring it.
- `git pull` brings Beads changes only because hk syncs after it: `post-merge` after a merging pull, `post-rewrite` after `git pull --rebase` or any other rebase. `git fetch` brings none.
- `.beads/issues.jsonl` is an export for diffs and viewers. It is never a sync channel.
- A **claimant** is `BEADS_ACTOR`, which `mise run beads:claim` sets to `<user>@<short host name>`, one identity per machine. A claim is atomic only within one replica.

### When syncs happen

`mise run status`, `git push`, and `git pull` sync on their own. Run `mise run beads:sync` yourself after creating, closing, or updating an item another machine should see, and before ending a session that changed Beads. Claims sync themselves.

### Claiming

Claim only with `mise run beads:claim -- <id>`. It syncs, claims the item as this machine's claimant, syncs again, and re-reads the item to see who holds it:

| Outcome | Exit | What it means | What you do |
| --- | --- | --- | --- |
| `claimed` | 0 | The item is yours, and other machines can see it. | Start the work. |
| `claimed-unpublished` | 0 | The item is yours on this machine, but a sync failed, so other machines cannot see the claim yet. The output gives the reason. | Start the work, and run `mise run beads:sync` until it succeeds. Say that the claim is unpublished. |
| `held` | nonzero | Another claimant already holds the item. The output names it. | The item is not available. Pick other work, or ask the human. |
| `lost` | nonzero | You claimed it, but after syncing another claimant holds it: a concurrent claim from another machine won. The output names it. | Do not start the work. |

A claim made on two machines before either syncs succeeds on both, and the later write silently replaces the earlier one. `lost` catches that only when the other claim arrives before your second sync, so a claimed item that appears in another claimant's name in `mise run status` is no longer yours either.

A claim made before claimants named machines holds a Git user name, such as `Mark Malstrom`, so `beads:claim` reports it as `held` on every machine. When you know the claim is this session's, or the human confirms it is this machine's, move it to this machine's claimant and publish it: `bd update <id> --if-assignee "<old name>" --assignee <user>@<short host name>`, then `mise run beads:sync`.

### Closing

Close only with `mise run beads:close -- <id> "<reason>"`. `bd close` refuses an item whose assignee is not the actor running it, and a plain `bd` command runs as the Git user name, not as this machine's claimant. The task closes the item as the claimant, then syncs:

| Outcome | Exit | What it means |
| --- | --- | --- |
| `closed` | 0 | The item is closed, and other machines can see it. |
| `closed-unpublished` | 0 | The item is closed on this machine, but the sync failed; the output gives the reason. Run `mise run beads:sync` until it succeeds. |
| `close failed` | nonzero | Beads refused, for example because another claimant holds the item. The output gives Beads' reason. |

### Sync outcomes

`mise run status`, `mise run beads:sync`, and `mise run beads:claim` report one of these. `mise run beads:sync` exits zero only for `synced`.

- **`synced`**: nothing to do.
- **`failed`**: continue, and say that Beads may be stale. The reason says why; a later sync point retries. When the reason says the Beads remote is not this project's `origin`, nothing was pulled or pushed and no later sync will be: stop and tell the human, as [Recovering a Beads sync](recovering-a-sync.md#failed) describes.
- **`conflict`**: Beads halted on a conflict it cannot settle, and nothing was pushed. Resolve it with your best judgement:
    1. Export a backup outside the repository: `bd export --all -o <path>`.
    2. Find the items. With embedded storage, which Workbench uses, the halted merge is aborted, so `bd conflicts list` shows nothing: compare `bd diff main origin/main` with `bd show <id>`. When `bd conflicts list` does show live conflicts, `bd conflicts show` presents them field by field.
    3. Settle an aborted merge with `bd vc merge origin/main --strategy ours` or `--strategy theirs`, and live conflicts with `bd conflicts resolve <id> --ours` or `--theirs`. When the right answer combines both sides, follow up with `bd update`.
    4. Finish with `mise run beads:sync`.
    5. Tell the human what you resolved and why, as a report rather than a request for approval.
- **`blocked`**: a dirty working set stops publishing. Back up first, find what left it dirty and clear it, finish with `mise run beads:sync`, and report afterwards the same way.
- **`unavailable`**: run `mise run setup`. When a read of the replica failed after the sync, the reason names the read and the sync's outcome; a locked or damaged database needs [Recovering a Beads sync](recovering-a-sync.md) rather than setup.

Whatever the resolution, never force a push, and never delete or re-bootstrap the replica to make a conflict go away. Either one discards another machine's work. [Recovering a Beads sync](recovering-a-sync.md) has worked resolutions and what to do when these steps are not enough.

### Never

- Run `bd dolt push`, `bd dolt pull`, or `bd sync` directly. The mise tasks set the environment that keeps Dolt from publishing a `__dolt_remote_info__` branch.
- Run `bd update --claim` outside `mise run beads:claim`, or `bd close` outside `mise run beads:close`.
- Run `bd init` in a project that has `.beads/config.yaml`.
- `bd import` the JSONL export.
- Pass `--force` to a Beads push.
- `git push --mirror`: it deletes `refs/dolt/data` on the remote.
- Enable `dolt.auto-push`.
- Create a Git branch or commit to carry Beads data.
- Run `bd prime`: its generic guidance contradicts these rules. `mise run status` shows the memories.

### A new machine

`mise run setup` restores this machine's replica from the remote with `bd bootstrap`. It never replaces a replica that already exists.

## Rules

- Never run `bd edit`; it opens an interactive editor and hangs a non-interactive agent. Use `bd update` flags.
- Never run `bd hooks install`; hk owns the git hooks and runs `bd hooks run <hook>` as a step.
- Use `--json` when parsing output programmatically.
- Never create a markdown TODO file as the source of truth.

## Red flags

- Filing an item for work the proposal requires.
- Filing without searching every state, including claimed, blocked, deferred and closed items.
- An item body that only makes sense to someone who read the chat.
- Deferring a defect without linking it from the PR.
- Closing a PR with discovered defects neither fixed nor filed.
- Claiming an item you are not about to start, or closing one that is not done.
- Claiming with `bd update --claim`, or starting work after `held` or `lost`.
- Ending a session that changed Beads without a successful sync, or without saying it failed.
- Settling a conflict without a backup first, or without reporting it afterwards.

Any of these means stop and repair the record.

## Related skills

- `reviewing-an-implementation` lists linked, deliberately deferred defects for the readiness report.
- `systematic-debugging` finds root cause before a defect is either fixed or described accurately enough to file.
