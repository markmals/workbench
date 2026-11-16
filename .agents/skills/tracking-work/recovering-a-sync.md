# Recovering a Beads Sync

Detail for the sync outcomes in [Tracking Work](SKILL.md#sync-outcomes). Read the model there first. Every step here keeps another machine's work: back up before changing anything, never force a push, and never delete or re-bootstrap the replica.

## Back up first

```sh
bd export --all -o "$HOME/beads-backup-$(date +%Y%m%d-%H%M%S).jsonl"
```

`--all` includes memories. Write the backup outside the repository so it is never committed, and keep it until the human has read your report. It holds issues, not Dolt history, so it is a record of what you saw, not a restore point.

## `conflict`: an item deleted on one machine and edited on another

Beads settles most concurrent edits on its own, field by field, with the later write winning. It halts on what it cannot settle, such as an item one machine deleted while another edited it. Embedded storage, which Workbench uses, aborts the halted merge and restores the working set, so the replica keeps its own state and `bd conflicts list` shows nothing.

`mise run beads:sync` reports the halt:

```text
Beads: conflict. Beads halted on a conflict in issues; nothing was pushed.
  Differs from the remote: lab-7ku (Probe one renamed)
```

1. Back up.
2. See what the remote has. `bd diff main origin/main` lists every item that differs between this replica (`main`) and the remote as last fetched (`origin/main`); `--json` gives both sides' values:

    ```text
    - Removed (1):
      - lab-7ku: Probe one renamed
    ```

    The remote deleted `lab-7ku`; this replica renamed it. `bd show lab-7ku` shows the local side, and `bd history lab-7ku` how it got there.

3. Decide which side should stand. Here: did the other machine delete the item on purpose, for example as a duplicate, or is the rename work that should survive?
4. Settle the merge. Take the remote's side of every conflict with `bd vc merge origin/main --strategy theirs`, or this replica's with `--strategy ours`. The strategy applies only to the conflicting rows; everything else merges normally.
5. Combine both sides when neither alone is right. After `--strategy theirs` deleted an item whose local edits matter, recreate it from the backup's values with `bd create` and link it to whatever replaced it. After `--strategy ours`, apply the remote's changes you want with `bd update`.
6. Publish with `mise run beads:sync`. It should now report `synced`.
7. Report to the human: which items conflicted, which side you kept and why, and anything you recreated or changed. This is a report, not a request for approval.

### When `bd conflicts list` shows live conflicts

Server storage can leave a halted merge live in the working set. Then `bd conflicts list` names the conflicted issues, `bd conflicts show <id>` shows base, ours, and theirs for each field, and `bd conflicts resolve <id> --ours` or `--theirs` settles one issue at a time; `--all` settles every one. Follow up with `bd update` where the answer combines both sides, then `mise run beads:sync`.

To abandon an open merge rather than finish it, Dolt's [`DOLT_MERGE('--abort')`](https://docs.dolthub.com/sql-reference/version-control/dolt-sql-procedures#dolt_merge) restores the pre-merge state: `bd sql "CALL DOLT_MERGE('--abort')"`. `bd sql` works only with server storage; embedded storage never leaves a merge open.

## `conflict` on the `events` table

A halt naming `events`, or "local changes would be stomped by merge: events", usually means the machines run different `bd` versions across a schema migration. `bd` is pinned in `mise.toml`: run `mise install` on the machine that is behind, then `mise run beads:sync`. Beads' [sync-failures guide](https://github.com/gastownhall/beads/blob/main/docs/recovery/sync-failures.md#merge-blocked-on-the-events-table-migration-0062) covers the cases that remain.

## `blocked`: a dirty working set

`bd sync` exits 4 when the same uncommitted changes to issues or dependencies have blocked several consecutive syncs, or when they violate a constraint. No later sync can publish until they are cleared.

1. Back up.
2. Make sure no other `bd` process is writing to this replica: another agent's session in a worktree of the same checkout shares it.
3. `bd vc status` shows the uncommitted changes, and `bd doctor` diagnoses the database.
4. If the changes are legitimate work that was never committed, commit them with `bd dolt commit`. If `bd doctor` reports a constraint violation, fix the item it names with `bd update`, or follow `bd doctor --dry-run` and then `bd doctor --fix`.
5. Publish with `mise run beads:sync`, and report what you found and changed.

## `failed`

The rest of the report is this machine's state. Continue, say that Beads may be stale, and let a later sync point retry.

- **Unreachable remote or timeout**: the network, or GitHub, is down or slow. Nothing to repair.
- **Authentication**: Beads uses Git's credentials for `origin`. If `git fetch origin` also fails, fix the credentials; that is the human's machine setup.
- **Push retries exhausted**: another machine kept winning the push race. Run `mise run beads:sync` again.
- **The Beads remote is not this project's `origin`**: `.beads/config.yaml` points at another repository, usually because the project was created from the Workbench template and still carries Workbench's tracker. The guard refused before anything was pulled or pushed. Tell the human; replacing the tracker, as Workbench's getting-started guide describes, deletes `.beads/` and is their decision.
- **The checkout has no `origin`**: the guard cannot tell whose Beads remote this is, so it refused. Ask the human to add the project's `origin`.

## `unavailable` after a sync: a replica that cannot be read

`mise run status` reports `unavailable` when `bd ready`, `bd list`, or `bd prime` fails after the sync, and names the read that failed and how the sync ended.

- **A lock**: another `bd` process is using this replica, often another agent's session in a worktree of the same checkout. Wait for it and run `mise run status` again.
- **Anything else**: run `bd doctor` and read what it reports. A damaged database is covered by Beads' [database-corruption guide](https://github.com/gastownhall/beads/blob/main/docs/recovery/database-corruption.md); back up first, and ask the human before any step that deletes or rebuilds `.beads/`.

## When these steps are not enough

Beads' [recovery guide](https://github.com/gastownhall/beads/tree/main/docs/recovery) covers sync failures, merge conflicts, and database corruption. Its steps run `bd dolt push` and `bd dolt pull` directly; here they run as `mise run beads:sync`. Stop and ask the human before any step that reseeds or rewrites the remote, deletes `.beads/`, or re-runs `bd init`.
