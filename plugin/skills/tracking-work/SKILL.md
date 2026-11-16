---
name: tracking-work
description: Use when finding ready work, claiming or closing a task, recording a defect discovered during a proposal or pull request, deciding whether discovered work is in scope or should be deferred, handing off across sessions, syncing Beads between machines or resolving a Beads sync outcome, or deciding between a local checklist and durable project tracking.
---

# Tracking Work

Beads (`bd`) is the project's work graph: the work agents discover, ready work, blockers, deferred follow-ups, and durable memory. The owner's own planning lives in GitHub Issues and Projects, not in Beads. A local checklist tracks the current turn's execution. It is not project state, and it does not survive a context reset.

## Start

```sh
workbench workflow status # syncs Beads, then reports ready work, items in progress, and memories
bd show <id>              # read an item before touching it
```

The Beads part of the result is its `beads` field. If `beads.sync.outcome` is anything other than `synced`, or `beads` holds `unavailable`, read [Sync outcomes](#sync-outcomes) before relying on it. Humans run `mise run status` for the same report as text, and the `beads:sync`, `beads:claim`, and `beads:close` tasks wrap the commands below for them and for the git hooks.

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
workbench beads claim <id>                             # claim before starting; see Claiming below
workbench beads close <id> --reason "Fixed in PR #42"  # close as this machine's claimant, then sync
bd remember "<durable project fact>"
workbench beads sync                                   # publish what you changed
```

A claim or close returns a receipt with its `status`, the `project` and Beads `remote` it acted on, the item `id`, the `claimant` it acted as, and whether the change is `published`; a sync's receipt has its `status`, `project`, and `remote`. Preview any of them with `--dry-run`: a claim or close reports the item, its current claimant, and the claimant it would act as; a sync reports the remote and the changes not yet published, and every dry run lists what it `cannotPredict`. Every sync is limited to 20 seconds; `--timeout <seconds>` changes that.

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
- A **claimant** is `BEADS_ACTOR`, or `<login user>@<short host name>` when it is unset: one identity per machine. `workbench beads claim` and `workbench beads close` act as it. A claim is atomic only within one replica.

### When syncs happen

`workbench workflow status`, `git push`, and `git pull` sync on their own. Run `workbench beads sync` yourself after creating, closing, or updating an item another machine should see, and before ending a session that changed Beads. Claims and closes sync themselves.

### Claiming

Claim only with `workbench beads claim <id>`. It syncs, claims the item as this machine's claimant, syncs again, and re-reads the item to see who holds it:

| Outcome               | Exit          | What it means                                                                                                                                                       | What you do                                                                                          |
| --------------------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `claimed`             | 0             | The item is yours, and other machines can see it. The receipt says `published: true`.                                                                               | Start the work.                                                                                      |
| `claimed-unpublished` | 0             | The item is yours on this machine, but a sync failed, so other machines cannot see the claim yet. The receipt says `published: false`, and stderr gives the reason. | Start the work, and run `workbench beads sync` until it succeeds. Say that the claim is unpublished. |
| `held`                | 4, `CONFLICT` | Another claimant already holds the item. The error's `details.holder` names it.                                                                                     | The item is not available. Pick other work, or ask the human.                                        |
| `lost`                | 4, `CONFLICT` | You claimed it, but after syncing another claimant holds it: a concurrent claim from another machine won. `details.holder` names it.                                | Do not start the work.                                                                               |

An unknown item is `NOT_FOUND` (exit 3). A claim Beads refuses for another reason, such as a closed item, is `PRECONDITION_FAILED` (exit 3) with Beads' reason. When the replica's Beads remote is not this project's `origin`, the claim changes nothing and fails with `PRECONDITION_FAILED`.

A claim made on two machines before either syncs succeeds on both, and the later write silently replaces the earlier one. `lost` catches that only when the other claim arrives before your second sync, so a claimed item that appears in another claimant's name in `workbench workflow status` is no longer yours either.

A claim made before claimants named machines holds a Git user name, such as `Mark Malstrom`, so `workbench beads claim` reports it as `held` on every machine. When you know the claim is this session's, or the human confirms it is this machine's, move it to this machine's claimant and publish it: `bd update <id> --if-assignee "<old name>" --assignee <user>@<short host name>`, then `workbench beads sync`.

### Closing

Close only with `workbench beads close <id> --reason "<reason>"`. `bd close` refuses an item whose assignee is not the actor running it, and a plain `bd` command runs as the Git user name, not as this machine's claimant. The command closes the item as the claimant, then syncs:

| Outcome               | Exit                     | What it means                                                                                                                                                            |
| --------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `closed`              | 0                        | The item is closed, and other machines can see it. The receipt says `published: true`.                                                                                   |
| `closed-unpublished`  | 0                        | The item is closed on this machine, but the sync failed. The receipt says `published: false`, and stderr gives the reason. Run `workbench beads sync` until it succeeds. |
| Refused: held         | 4, `CONFLICT`            | Another claimant holds the item. The error's `details.holder` names it, and its message gives Beads' reason. Nothing was synced.                                         |
| Refused: unknown item | 3, `NOT_FOUND`           | No item has that id.                                                                                                                                                     |
| Refused: other        | 3, `PRECONDITION_FAILED` | Beads refused for another reason, which the message gives.                                                                                                               |

### Sync outcomes

`workbench workflow status` reports the outcome of its sync in `beads.sync` and exits 0 whatever it is; `--no-sync` skips the sync and reports `skipped`. `workbench beads sync` exits 0 only for `synced`. Otherwise it writes an error to stderr whose code says what happened, and whose `details.outcome` is the outcome below. The `sync` object in `workflow status` and in the receipts of `beads claim` and `beads close` carries the same `code`, and `retryable`, whenever it did not sync, so read the cause from the code rather than from Beads' wording: Dolt adds hints about credentials to every failed fetch.

| Outcome       | Error code                | Exit | Cause                                                                                                                                    |
| ------------- | ------------------------- | ---- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `failed`      | `UNAVAILABLE` (retryable) | 5    | The remote or the network was unreachable, push retries ran out, or Beads failed for another reason.                                     |
| `failed`      | `TIMEOUT` (retryable)     | 5    | The sync passed its time limit: 20 seconds, or `--timeout`.                                                                              |
| `failed`      | `UNAUTHENTICATED`         | 3    | The remote rejected Git's credentials.                                                                                                   |
| `failed`      | `PRECONDITION_FAILED`     | 3    | The replica's Beads remote is not this project's `origin`, or the checkout has no `origin`. Nothing was pulled or pushed.                |
| `conflict`    | `CONFLICT`                | 4    | Beads halted on a conflict. `details.tables` names the conflicted tables, and `details.differing` the items that differ from the remote. |
| `blocked`     | `PRECONDITION_FAILED`     | 3    | A dirty working set stops publishing.                                                                                                    |
| `unavailable` | `DEPENDENCY_MISSING`      | 3    | `bd` is not installed.                                                                                                                   |
| `unavailable` | `PRECONDITION_FAILED`     | 3    | There is no Beads workspace, or its database is not initialized.                                                                         |

What each outcome asks of you:

- **`synced`**: nothing to do.
- **`skipped`**: nothing was pulled or pushed; run `workbench workflow status` without `--no-sync`, or `workbench beads sync`, before relying on what another machine changed.
- **`failed`**: continue, and say that Beads may be stale. An unreachable remote never stops work: claims and closes still happen on this machine as `claimed-unpublished` or `closed-unpublished`. `code` says why; for `UNAVAILABLE` or `TIMEOUT` a later sync point retries, and `UNAUTHENTICATED` needs Git's credentials fixed. When the Beads remote is not this project's `origin` (`PRECONDITION_FAILED`), nothing was pulled or pushed and no later sync will be: stop and tell the human, as [Recovering a Beads sync](recovering-a-sync.md#failed) describes.
- **`conflict`**: Beads halted on a conflict it cannot settle, and nothing was pushed. Resolve it with your best judgement:
  1. Export a backup outside the repository: `bd export --all -o <path>`.
  2. Find the items. With embedded storage, which Workbench uses, the halted merge is aborted, so `bd conflicts list` shows nothing: compare `bd diff main origin/main` with `bd show <id>`. When `bd conflicts list` does show live conflicts, `bd conflicts show` presents them field by field.
  3. Settle an aborted merge with `bd vc merge origin/main --strategy ours` or `--strategy theirs`, and live conflicts with `bd conflicts resolve <id> --ours` or `--theirs`. When the right answer combines both sides, follow up with `bd update`.
  4. Finish with `workbench beads sync`.
  5. Tell the human what you resolved and why, as a report rather than a request for approval.
- **`blocked`**: a dirty working set stops publishing. Back up first, find what left it dirty and clear it, finish with `workbench beads sync`, and report afterwards the same way.
- **`unavailable`**: run `mise run setup`. When a read of the replica failed after the sync, the reason names the read and the sync's outcome; a locked or damaged database needs [Recovering a Beads sync](recovering-a-sync.md) rather than setup.

Whatever the resolution, never force a push, and never delete or re-bootstrap the replica to make a conflict go away. Either one discards another machine's work. [Recovering a Beads sync](recovering-a-sync.md) has worked resolutions and what to do when these steps are not enough.

### Never

- Run `bd dolt push`, `bd dolt pull`, or `bd sync` directly. The `workbench` commands run them with the environment that keeps Dolt from publishing a `__dolt_remote_info__` branch, and only against the project's own remote.
- Run `bd update --claim` outside `workbench beads claim`, or `bd close` outside `workbench beads close`.
- Run `bd init` in a project that has `.beads/config.yaml`.
- `bd import` the JSONL export.
- Pass `--force` to a Beads push.
- `git push --mirror`: it deletes `refs/dolt/data` on the remote.
- Enable `dolt.auto-push`.
- Create a Git branch or commit to carry Beads data.
- Run `bd prime`: its generic guidance contradicts these rules. `workbench workflow status` shows the memories.

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
