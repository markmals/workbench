---
id: proposal.0006
title: Drop Harness Hooks and Sync Beads Across Machines
authors: [markmals, anthropic/claude-opus-5-5]
status: implemented
pull-request: https://github.com/markmals/workbench/pull/11
issues: [workbench-lx8]
supersedes: []
---

# Drop Harness Hooks and Sync Beads Across Machines

## Summary

Remove the OMP extensions that run on every turn, deliver Beads ready work and memories through `mise run status`, and sync each project's Beads database across machines through its own GitHub repository without publishing a Git branch.

## Motivation

This proposal fixes two problems in how agent sessions run.

**The harness hooks slow the agent loop.** Workbench wires three OMP extensions into every session:

- `check-before-stop` runs the full `mise run check` at the end of every turn, including turns that changed nothing. That costs 4.7 s per turn in Workbench. In Trove, whose check takes 4–8 minutes, the hook always times out and sends the agent back once ([proposal 0003](0003-fit-the-harness-to-current-models.md)). Recon replaced the hook with a no-op because it kept blocking stops.
- `format-after-edit` runs the formatter after every edit. That costs 0.26 s per edit, plus a model round trip to re-read the file whenever it reformats.
- `prime-beads` injects `bd prime` before the first turn: 8,444 characters, sent with every request. Before 2026-11-15 Workbench had no Beads memories, so all of it was generic Beads guidance. That guidance includes `git add . && git commit`, which contradicts the project's commit rules.

The owner's top performance priority is the speed of the agent loop. Everything these hooks enforce is also enforced, or can be, at points outside it.

**Agents on different machines do not share their work graph.** The owner now runs agents on the same project from several machines. Each machine keeps its own Beads (Dolt) database.

Every Workbench project in `~/Developer` already points `sync.remote` at its own GitHub repository, and most have published Beads history there. But nothing in the process says when to pull or push. A machine therefore sees another's claims and new items only after someone remembers to run `bd dolt pull`.

Recon took a different route and turned on `dolt.auto-push`. Beads documents that setting as safe only for a single writer. Recon's repository also now carries a `__dolt_remote_info__` branch. Dolt publishes that branch so the remote appears in GitHub's interface, and GitHub then shows its banner announcing a recently pushed branch.

The owner wants every machine to have current Beads, including what other agents have claimed, without that noise.

The owner also reports that their agents did not understand how Beads works across machines. Workbench gives them little to go on, and some of what it does say is wrong:

- The `tracking-work` skill calls `bd update <id> --claim` atomic. It is atomic only within one machine's database.
- Every machine claims as the same Git user. Beads treats a claim "already claimed by you" as a no-op success, so a second machine can take an item the first is working on without any error.
- The generic `bd prime` guidance that the current hook injects tells agents to run `bd dolt push` directly.

**The contract misstates what Beads is for.** The owner's stack (`~/Developer/Notes/sprinkles-stack.md`) divides tracking: GitHub Issues and Projects hold the owner's planning, and Beads holds agents' work and the project's memory. `AGENTS.md` instead says Beads "replaces GitHub-issue defect tracking". The `writing-a-proposal` skill contradicts both: its red-flag table says to file a newly discovered defect as a GitHub issue (Beads `workbench-lx8`).

## Domain grounding

### Established context

Beads (bd 1.3.2, documentation in `gastownhall/beads`):

- **The Dolt database is the source of truth.** `.beads/issues.jsonl` is an export for viewers and backup, not a cross-machine sync channel. Importing JSONL is upsert-only and cannot see deletions (`docs/core-concepts/sync-concepts.md`).
- **Sync runs over Dolt remotes.** A project's own `origin` can serve as its Dolt remote. Dolt stores the history under `refs/dolt/data`, separate from source branches. `bd bootstrap` clones that history onto a fresh machine and never deletes existing data (`docs/getting-started/sync-setup.md`, `bd bootstrap --help`).
- **`bd sync` runs one full cycle** (`bd sync --help`):
  1. Pull from the remote.
  2. Detect conflicts from the merge itself rather than from the pull's exit status.
  3. Recompute blocked flags.
  4. Push, retrying a bounded number of times when another replica wins the push race.

  The pull settles some conflicts on its own, including last-write-wins on individual issue fields. A conflict it cannot resolve safely halts the cycle with exit status 2. A working set left dirty by another writer exits 4.

- **Resolving conflicts.** `bd conflicts list` and `bd conflicts show` present the conflicts left in the working set issue by issue and field by field (base, ours, theirs). `bd conflicts resolve <id> --ours|--theirs` settles one issue, and `--all` settles all of them (`bd conflicts --help`). Beads' recovery guide warns never to force past a conflict and never to delete another machine's work (`docs/recovery/sync-failures.md`).

- **`dolt.auto-push` is opt-in.** It pushes after writes, debounced to one push per interval. Beads warns that "concurrent pushes to git-protocol Dolt remotes can corrupt or strand remote history when multiple writers race" (`docs/reference/configuration.md`).
- **Other remote types exist.** Beads supports DoltHub, S3, GCS, Git, and file remotes. It names DoltHub the zero-configuration default for syncing across machines and documents bucket-based syncing as an alternative (`docs/architecture/dolt.md`, `docs/multi-agent/bucket-federation.md`).

Dolt:

- **Data location.** A Git-hosted Dolt remote keeps its data in `refs/dolt/data`, outside `refs/heads/`, so it never collides with branches ([remotes](https://docs.dolthub.com/sql-reference/version-control/remotes), [Git remotes](https://www.dolthub.com/blog/2026-02-19-supporting-git-remotes-as-dolt-remotes/)).
- **The info branch.** `refs/heads/__dolt_remote_info__` is an ordinary branch holding one `DOLT_REMOTE.md` file. It exists so the remote appears in GitHub's interface, and nothing reads it back. An empty `DOLT_REMOTE_INFO_BRANCH` environment variable disables it ([dolt#11648](https://github.com/dolthub/dolt/issues/11648), [beads#5266](https://github.com/gastownhall/beads/issues/5266), [beads#8106](https://github.com/gastownhall/beads/pull/8106)). [beads#7945](https://github.com/gastownhall/beads/pull/7945), which would stop bd publishing the branch by default, is still open.
- **A mirror push destroys the data.** `git push --mirror` from an ordinary clone deletes `refs/dolt/data` on the remote, because ordinary clones never fetch it ([beads#5266](https://github.com/gastownhall/beads/issues/5266)).

### Repository observations

Checked on 2026-11-15:

- **Remote inventory.**
  - Workbench, Pico, Trove, and Recon set `sync.remote` to their own GitHub repository, and each has `refs/dolt/data`.
  - Bento sets `sync.remote`, but its repository has no `refs/dolt/data` yet.
  - Recon also sets `dolt.auto-push: true`. Its repository has `refs/heads/__dolt_remote_info__`, with one commit, "dolt remote info", from 2026-11-14.
- **The info branch, reproduced against local bare repositories:**
  - The Dolt 2.7.3 CLI's push created `refs/heads/__dolt_remote_info__`.
  - The same push with `DOLT_REMOTE_INFO_BRANCH=` created only `refs/dolt/data`.
  - A later push left the existing info branch unchanged.
  - bd 1.3.2's own push to a `git+file` remote created no info branch.
- **Two replicas, simulated with two clones of one bare repository:**
  - Clone B ran `bd bootstrap`, claimed an item, and ran `bd sync`.
  - Clone A created an item and ran `bd sync`. A then showed B's claim, in progress with its assignee.
  - B's next `bd sync` received A's item.
  - The remote held only `main` and `refs/dolt/data`.
- **Latency.** `bd dolt pull` against Workbench's real GitHub remote took 1.28 s.
- **mise environment.** mise exports an `[env]` value of `""` as a set, empty variable, both to `mise exec` and to tasks.
- **Claims across two clones, with distinct `BEADS_ACTOR` values** (`mark@alpha` and `mark@beta`):
  - Sequential: A claimed an item and synced; B synced and tried to claim it. Beads refused: "issue already claimed by mark@alpha".
  - Concurrent: both claimed another item before syncing, and both claims succeeded locally. After both synced, both replicas showed `mark@beta`. A's claim was gone, with no error.
- **First publish.** `bd sync` fails while the remote has no `refs/dolt/data` yet ("no branches found in remote"). The first publish needs `bd dolt push`.
- **Conflicts in embedded storage**, which Workbench uses. One replica deleted an item and the other renamed it.
  - `bd sync` exited 2 and named only the table (`"conflicts": ["issues"]`). It aborted the merge and restored the working set, so `bd conflicts list` and `bd conflicts show` reported nothing. `bd sql` is unsupported in embedded mode.
  - `bd diff main origin/main` listed the item. `bd vc merge origin/main --strategy theirs`, then `bd sync`, settled the conflict and published the result.
  - On 2026-11-15 the owner chose the conflict report and resolution procedure in sections 2 and 5 from these results.
- **Worktrees.** All worktrees of a checkout share one database: `bd where` in a Worktrunk worktree resolves to the main checkout's `.beads/embeddeddolt`.
- **Measured hook costs.**
  - `mise run check` takes 4.7 s in Workbench, and the 4–8 minutes in Trove come from [proposal 0003](0003-fit-the-harness-to-current-models.md).
  - `bd prime` took 0.7 s and printed 8,444 characters. With `--memories-only` it printed 2,831, all of them memories recorded that day.
  - `mise run fmt` on one file took 0.26 s.

### Relevant constraints and principles

| Concept       | Identity and lifetime                                                                                                                                                   |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Beads replica | One machine's local Dolt database for a project: a full copy that works offline. Lives in the main checkout's `.beads/`, and every worktree of that checkout shares it. |
| Beads remote  | `refs/dolt/data` on the project's own GitHub repository. The meeting point of the replicas; never a branch.                                                             |
| Sync          | One `bd sync` cycle: pull, conflict check, push with retries.                                                                                                           |
| Sync point    | A moment in the Workbench process at which a sync runs. Defined by this proposal.                                                                                       |
| Claimant      | Who holds a claim: `BEADS_ACTOR`, which Workbench sets to `<user>@<host>`. Identifies the machine holding an item.                                                      |

The ontology review is COHERENT. Beads already defines replicas, remotes, sync, and actors. This proposal adds the process concept of a sync point, and fixes the claimant as one identity per machine. A claim is atomic within one replica. Across replicas, how soon others see a claim is bounded by the sync points.

Other constraints:

- **Owner's priorities.** Agent-loop speed comes first, and Workbench adds no per-turn work. GitHub branch noise from syncing is not acceptable.
- **[Vision](../VISION.md).** Enforce mechanically, and make ceremony earn its cost. Beads holds the work graph; Workbench does not compete with it.
- **Error handling.** A failed sync is reported, never hidden.

### Quality bar

- After a session's `mise run status`, the agent sees every Beads change any machine had pushed at its own latest sync point.
- No turn pays for a check, a formatter run, or a prime injection it did not ask for.
- No sync creates or updates anything under `refs/heads/`.
- Concurrent syncs from two machines never silently lose a write that Beads does not settle by design. Push races retry. A conflict Beads cannot settle halts the sync until the agent resolves it, and the agent reports how it did so.

### Remaining uncertainty

- **Claims are not atomic across machines.** Two machines that claim the same item before syncing both succeed locally. After syncing, the later sync's claim stands and the other disappears without an error. `beads:claim` detects a lost claim by re-reading the item after its sync ([section 4](#4-claiming-work-across-machines)), but it cannot prevent one.
- **Network speed.** Sync latency on slower networks or larger databases is unmeasured. The only measurement is a 1.28 s pull from this machine.
- **New machines.** A machine without a Dolt commit identity may fail its first merge. Beads documents this as a known failure when setting up a new machine.

## Existing baseline

The relevant current behavior:

- **The extensions.** `.omp/extensions/prime-beads.ts`, `format-after-edit.ts`, and `check-before-stop.ts`, plus `tools/format-after-edit.test.ts`.
- **Where they are documented:**
  - `AGENTS.md`, in its harness setup table;
  - `guides/adapting-your-harness.md`, in its OMP table and its "Wire lifecycle events" section;
  - `guides/getting-started.md` and `README.md`;
  - `.agents/rules/enforcement-hierarchy.md`, whose Tier 0 includes "harness lifecycle wiring in `.omp/extensions/`".
- **The `tracking-work` skill** opens with `bd prime`.
- **`mise run status`** (`tools/status.ts`) reports phase, situation, next action, and next actor. Its contract is in [Record Validation and Workflow Status](../guides/record-validation.md). It says nothing about Beads.
- **`mise run setup`** runs `bd init --init-if-missing …`, `bd config set export.auto true`, and `hk install --mise`.
- **`hk.pkl`** runs `bd hooks run <hook>` in `pre-commit`, `prepare-commit-msg`, `pre-push`, `post-checkout`, and `post-merge`. Those keep the JSONL export current. They neither push nor pull Dolt history.
- **`.beads/config.yaml`** sets `sync.remote: "git+https://github.com/markmals/workbench.git"` and `export.auto: true`.

Preserve:

- Beads as the work tracker.
- hk's ownership of every git hook, including the Beads steps.
- The committed JSONL export.
- The verification rule and the gate in CI.

The per-turn hooks and the prime injection are incidental.

## Proposed solution

Remove the hooks. Orientation and syncing then happen at points the process already has:

- when an agent orients itself (`mise run status`);
- when work leaves the machine (`git push`);
- when work arrives (`git pull`);
- after Beads writes that another machine should see, through one explicit task.

```text
Machine B: mise run status → sync → sees A's claims and new items
           mise run beads:claim -- t-42 → sync, claim as mark@beta, sync, confirm
Machine A: next mise run status, git push, or git pull → sync → sees t-42 claimed
GitHub:    refs/dolt/data advances; no branch is created or updated; no banner
```

## Detailed design

### 1. Remove the harness hooks

Delete `.omp/extensions/prime-beads.ts`, `format-after-edit.ts`, and `check-before-stop.ts`, and `tools/format-after-edit.test.ts`. Workbench then wires no harness lifecycle event. `.omp/` keeps only the `workflow` tool, which [proposal 0007](https://github.com/markmals/workbench/pull/10) removes separately.

| Removed hook                     | Covered by                                                                                                                  |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `bd prime` before the first turn | `mise run status`, already the contract's first instruction, now reports Beads ([section 2](#2-beads-in-mise-run-status)).  |
| Format after every edit          | hk's `pre-commit` hook, which already formats staged files and stages the result. Agents may run `mise run fmt -- <files>`. |
| Full check before every stop     | The verification rule (run `mise run check` this turn before claiming work is done), and CI.                                |

**The pre-push hook stops running the check.**

- `mise run check` runs in CI only. hk's `pre-push` hook keeps only its Beads steps.
- Before marking a pull request ready or merging it, the agent waits for CI to pass on the pushed revision (`gh pr checks --watch`). When CI fails, the agent fixes the failure and pushes again.
- The verification rule is unchanged: the agent still runs `mise run check` itself before claiming work is done.
- The owner chose this on 2026-11-15. Every push no longer waits for the full check, which takes 4.7 s in Workbench and 4–8 minutes in Trove.

### 2. Beads in `mise run status`

- **Order.** `mise run status` runs one sync ([section 3](#3-syncing-beads-across-machines)) before reporting. After the phase report, it adds a Beads section:
  - the sync outcome;
  - ready work, from `bd ready`;
  - items in progress and who holds them, from `bd list --status in_progress`;
  - the project's memories, from `bd prime --memories-only`.
- **Sync outcomes:**

  | Outcome       | Reported as                                                                                                                                                                                                             |
  | ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
  | `synced`      | One line.                                                                                                                                                                                                               |
  | `failed`      | The reason (unreachable remote, timeout, authentication), and that the rest shows this machine's state.                                                                                                                 |
  | `conflict`    | `bd sync` exit 2: names the conflicted tables Beads reports, and the items that differ between this replica and the remote (`bd diff main origin/main`). The agent resolves them ([section 5](#5-agent-documentation)). |
  | `blocked`     | `bd sync` exit 4: a dirty working set blocks publishing. The agent investigates and clears it ([section 5](#5-agent-documentation)).                                                                                    |
  | `unavailable` | `bd` is missing or the workspace is not initialized, with a pointer to `mise run setup`. Shown instead of the whole section.                                                                                            |

  The sync is limited to 20 seconds; past that its outcome is `failed`.

- **Exit status.** Beads outcomes never change the exit status of `mise run status`.
- **JSON.** `mise run status -- --json` adds a `beads` object that holds either:
  - `sync` (`outcome` and `detail`), `ready` (a list of `id`, `title`, `priority`), `in_progress` (a list of `id`, `title`, `assignee`), and `memories` (text); or
  - `unavailable` (the reason).
- **Contract and skill updates.** [Record Validation and Workflow Status](../guides/record-validation.md) gains this section as part of `status`'s contract. The `tracking-work` skill's start sequence becomes `mise run status`, then `bd show <id>`.

### 3. Syncing Beads across machines

- **The remote** is the project's own GitHub repository, through `sync.remote`, with Beads history in `refs/dolt/data`. That is unchanged from today.
- **Only the project's own remote.** A sync whose Dolt remote is not the same repository as the checkout's `origin` is `failed`, without pulling or pushing. Workbench commits `.beads/config.yaml`, so a project created from the Workbench template would otherwise sync with Workbench's own Beads. Its getting-started steps replace Workbench's `.beads/` before the first setup. The owner added this on 2026-11-15 after review found the hazard; proposal 0007's `workbench init` is its long-term home.
- **Every sync runs through mise.** A project's `mise.toml` sets:

  ```toml
  [env]
  DOLT_REMOTE_INFO_BRANCH = ""
  ```

  It defines a `beads:sync` task. That task runs `bd sync`, or `bd dolt push` when the remote has no `refs/dolt/data` yet, so the project's first sync publishes its history. `mise run status`, `beads:sync`, `beads:claim`, and hk's mise-run steps all see the variable, so no sync publishes `__dolt_remote_info__`. Both tasks and `mise run status` share one implementation, `tools/beads.ts`.

- **Sync points:**
  1. `mise run status` ([section 2](#2-beads-in-mise-run-status)).
  2. hk's `pre-push` hook runs `mise run beads:sync` before its other steps.
  3. hk's `post-merge` hook runs `mise run beads:sync` after a pull or merge, and its `post-rewrite` hook runs it after a rebase, which is what `git pull --rebase` runs instead of `post-merge`. The owner added the rebase step on 2026-11-15 after review found the gap; an amend does not sync.
  4. After creating, closing, or updating a Beads item another machine should see, the agent runs `mise run beads:sync`. Claims sync themselves ([section 4](#4-claiming-work-across-machines)). The contract states this rule.
- **Failures in hooks.** A Beads sync failure in a git hook prints its outcome and never blocks the git operation.
- **Never:**
  - `dolt.auto-push`, which `mise run setup` sets to `false`;
  - `bd dolt push`, `bd dolt pull`, or `bd sync` run outside mise;
  - `git push --mirror` against the project's repository.

  The contract states the last two.

- **Setting up a machine.** `mise run setup` runs `bd bootstrap --yes` in a project that already has `.beads/config.yaml`. That restores the replica from `refs/dolt/data` instead of creating a fresh database with an unrelated history. It runs `bd init` only in a project that has no Beads workspace yet.
- **Existing info branches.** A project whose repository has `__dolt_remote_info__` deletes it once; [beads#5266](https://github.com/gastownhall/beads/issues/5266) confirms that is safe. Workbench's repository has none. When Beads publishes beads#7945, the `[env]` entry becomes redundant and harmless.

### 4. Claiming work across machines

Agents claim through `mise run beads:claim -- <id>`, never with `bd update --claim` directly. The task:

1. syncs;
2. claims the item as `BEADS_ACTOR`, which it sets to `<login user>@<short host name>` unless the environment already sets it;
3. syncs again;
4. re-reads the item and checks who holds it.

| Outcome               | Exit status | Reported as                                                                                                                                                  |
| --------------------- | ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `claimed`             | 0           | The item is claimed and published.                                                                                                                           |
| `claimed-unpublished` | 0           | The item is claimed on this machine, but a sync failed, so other machines cannot see the claim until a later sync succeeds. Gives the sync failure's reason. |
| `held`                | nonzero     | Beads refused because another claimant holds the item. Names that claimant.                                                                                  |
| `lost`                | nonzero     | After syncing, another claimant holds the item, meaning a concurrent claim won. Names that claimant. The agent must not start the work.                      |

**Closing.** `bd close` refuses an item claimed by a different actor, and an agent's plain `bd` runs as the Git user name rather than the claimant. Agents therefore close through `mise run beads:close -- <id> "<reason>"`, which closes the item as `BEADS_ACTOR` (set the same way) and then syncs. It reports `closed` (exit 0), `closed-unpublished` when the sync failed (exit 0, with the reason), or Beads' refusal (nonzero). The owner added this on 2026-11-15 after the preview found the gap.

### 5. Agent documentation

The `tracking-work` skill gains a section on Beads across machines. The owner reported that agents misread this, so the section must stand on its own. It teaches:

- **The model.**
  - Each machine has one replica per project, shared by all of that checkout's worktrees.
  - `bd` commands read and write only that replica, and other machines' changes arrive only through a sync.
  - The meeting point is `refs/dolt/data` on the project's `origin`. It is not a branch, does not appear in branch lists, and `git clone` does not fetch it.
  - `git pull` brings Beads changes only because hk's `post-merge` or `post-rewrite` hook syncs; `git fetch` brings none.
  - `.beads/issues.jsonl` is an export for diffs and viewers, never a sync channel.
- **When syncs happen.** `mise run status`, `git push`, and `git pull` sync on their own. The agent runs `mise run beads:sync` after creating, closing, or updating items another machine should see, and before ending a session that changed Beads.
- **Claiming.** Claim only through `mise run beads:claim`. The skill explains what each outcome in section 4 means, and that an item held by another claimant is not available.
- **Sync outcomes**, from `mise run status` and the tasks:
  - `failed`: continue, and say that Beads may be stale.
  - `conflict`: resolve it, using best judgement.
    1. Export a backup first (`bd export --all`).
    2. Find the items. When the merge was left live, `bd conflicts list` and `bd conflicts show` present them. When it was aborted, as embedded storage always does, `bd diff main origin/main` and `bd show` do.
    3. Settle live conflicts with `bd conflicts resolve <id> --ours` or `--theirs`, and an aborted merge with `bd vc merge origin/main --strategy ours` or `theirs`. When the right answer combines both sides, follow up with `bd update`.
    4. Finish with `mise run beads:sync`.
    5. Afterwards, tell the human what was resolved and why, as a report rather than a request for approval.
  - `blocked`: find what left the working set dirty and clear it, with the same backup first and the same report afterwards.
  - Whatever the resolution, never force a push, and never delete or re-bootstrap the replica to make a conflict go away. Either one discards another machine's work.
  - `unavailable`: run `mise run setup`.
- **Never:**
  - run `bd dolt push`, `bd dolt pull`, or `bd sync` directly;
  - run `bd update --claim` outside the task;
  - run `bd init` in a project that has `.beads/config.yaml`;
  - `bd import` the JSONL export;
  - pass `--force` to a Beads push;
  - `git push --mirror`;
  - enable `dolt.auto-push`;
  - create a Git branch or commit to carry Beads data;
  - run `bd prime`, whose generic guidance contradicts these rules.
- **A new machine.** `mise run setup` restores the replica.

The skill's claim example and the word "atomic" are replaced. Detail that only matters when something goes wrong lives in a supporting file in the skill's directory, linked from the section. That includes worked conflict resolutions, aborting an open merge, and pointers into Beads' recovery guide. The contract carries the short rules and points to the skill.

### 6. Documents and configuration that change

- **`AGENTS.md`:**
  - The harness setup table drops the lifecycle rows.
  - Work tracking states the sync rules from [section 3](#3-syncing-beads-across-machines).
  - Work tracking states what each tracker holds. Beads holds the work agents discover and the project's memory. GitHub Issues and Projects hold the owner's planning. The sentence calling Beads a replacement for GitHub-issue defect tracking goes.
  - The quality-gates list adds `mise run beads:sync`.
- **`PROCESS.md`'s Work Tracking section** describes replicas, the remote, and the sync points, and states the same division between Beads and GitHub Issues and Projects.
- **The `writing-a-proposal` skill's red-flag table** files a newly discovered out-of-scope defect in Beads, not as a GitHub issue.
- **`guides/adapting-your-harness.md`** drops "Wire lifecycle events" and the extension rows. A harness needs only the contract and the skills.
- **`guides/getting-started.md` and `README.md`** stop describing lifecycle extensions.
- **`.agents/rules/enforcement-hierarchy.md`'s Tier 0** drops `.omp/extensions/` and the `pre-push` check, and names the hk Beads steps.
- **Every mention of the pre-push check** says CI alone runs `mise run check`: `AGENTS.md`'s quality-gates list and its GitHub Actions paragraph, `README.md`'s `hk.pkl` row, and the comment in `.github/workflows/ci.yml`.
- **The skills that mark a pull request ready or merge it** (`implementing-a-proposal`, `reviewing-an-implementation`, `completing-a-feature`) wait for CI on that revision first.
- **[Record Validation and Workflow Status](../guides/record-validation.md)** changes as described in [section 2](#2-beads-in-mise-run-status), and the **`tracking-work` skill** as described in sections 2 and 5.
- **Configuration:**
  - `mise.toml` gains the `[env]` entry, the `beads:sync` and `beads:claim` tasks, and the setup changes.
  - `hk.pkl` gains the `pre-push`, `post-merge`, and `post-rewrite` steps.
  - `tools/status.ts` gains the Beads section. The new `tools/beads.ts` implements sync and claim. Both get tests.

## Compatibility

- **Sessions lose their automatic work.** They no longer format after each edit, run the check before stopping, or receive `bd prime`. Agents run `mise run status` and the check themselves, as the existing rules already require.
- **Beads behavior changes:**
  - Workbench's own setup changes from `bd init --init-if-missing` to `bd bootstrap` for an existing workspace. A machine whose replica already exists keeps it.
  - A project that sets `dolt.auto-push: true`, such as Recon, switches to the sync points.
  - A project whose repository lacks `refs/dolt/data`, such as Bento, publishes it at its first sync.
- **Consumer projects.** Until [proposal 0007](https://github.com/markmals/workbench/pull/10) delivers Workbench as a package, each consumer takes this change by copying it, like any other Workbench change today.
- **Pushes no longer run the check.** A commit that fails it can reach a pull request's branch. CI flags it before the pull request can be marked ready or merged.

## Implications on adoption

- **Requirements:**
  - bd 1.3.2 or newer, for `bd sync` and `bd bootstrap`;
  - Git credentials for the project's GitHub repository on every machine (pushing already needs them);
  - a Dolt commit identity on every machine.
- **Steps for an existing project:**
  1. Delete the three extensions.
  2. Add the `[env]` entry, the `beads:sync` and `beads:claim` tasks, and the hk steps.
  3. Turn off `dolt.auto-push`.
  4. Delete any `__dolt_remote_info__` branch.
  5. Run `mise run setup` on each machine.
- **Reversal.** Restore the files from Git history and re-enable whatever sync the project used before.

## Scope

- Delete the three extensions and their test, and update the documents in [section 6](#6-documents-and-configuration-that-change).
- Add the Beads section to `mise run status`, with tests that stub `bd` for each outcome in [section 2](#2-beads-in-mise-run-status).
- Add `tools/beads.ts` with its sync and claim behavior, with tests that stub `bd` for each outcome in [section 4](#4-claiming-work-across-machines).
- Write the agent documentation in [section 5](#5-agent-documentation).
- Add the `[env]` entry, the `beads:sync` and `beads:claim` tasks, the setup change, and the hk steps to Workbench.

### Out of scope

- **Packaging Workbench and porting `status` to Rust.** That is [proposal 0007](https://github.com/markmals/workbench/pull/10). It carries this behavior over under its agent-first CLI contract, which changes the output format and the error codes, not what is detected.
- **Migrating consumer projects,** including deleting Recon's info branch. Each is a change in that repository.
- **Changing Beads itself**, such as beads#7945.
- **A shared Dolt server, or a DoltHub or bucket remote.** See Alternatives and Future directions.

## Preview

- **Repository shape:** process tooling with CLI checks.
- **Artifact:** this branch, exercised against the real GitHub remote from two clones, each with its own replica. Two clones on one machine behave as two machines for Beads; a second physical machine is optional.
- **Exercise:**
  1. In clone A, run `mise run status` and read the Beads section.
  2. In clone B, run `mise run beads:claim -- <id>`.
  3. In A, run `mise run status` and see the item in progress, held by B's claimant.
  4. In A, run `mise run beads:claim` on B's item and see `held`, naming B's claimant. Tests with a stubbed `bd` cover `lost`, which needs a race between two machines.
  5. Push a branch from A and confirm hk synced.
  6. Run `git ls-remote origin` and confirm there is no `refs/heads/__dolt_remote_info__`.
  7. Run `mise run status` with the network off and see the `failed` outcome.
- **Cleanup:** close the probe items, sync, and delete the second clone.

## Policies and decisions checked

- **Policies and decisions.** None: `policies/` and `decisions/` hold only their READMEs.
- **[VISION.md](../VISION.md).** "Ceremony must earn its cost": the hooks and the pre-push check are removed on measured cost. "Enforce mechanically": checks remain at commit time and in CI. "Not an issue tracker": Beads keeps the work graph.
- **[Proposal 0003](0003-fit-the-harness-to-current-models.md)** recorded `check-before-stop`'s timeout in Trove and suggested a fast-subset stop hook. This proposal removes the hook instead; see Alternatives.
- **[Proposal 0005](0005-current-specifications.md)** put `status`'s contract in `guides/record-validation.md`, which this proposal extends.
- **[Proposal 0007](https://github.com/markmals/workbench/pull/10)** depends on this one: its package declares no harness hooks, and its `workbench workflow status` and `beads` commands carry this behavior over.
- **`.agents/rules/enforcement-hierarchy.md`.** Tier 0 loses the harness wiring and gains the hk Beads steps. Every remaining check runs whether or not the agent remembers.

## Future directions

- **Per-session claimants.** A claimant that names the session as well as the machine would tell apart two agents working on one machine.
- **A shared server.** A single Dolt server, for example on a home server over Tailscale, would make claims atomic across machines if double claims become a real problem.
- **Timed syncs.** A sync on a timer could serve long sessions that rarely orient, push, or pull.

## Alternatives considered

- **Cheaper hooks.** A fast subset of the check before stopping, or a check that the gate passed since the last edit, still forces work before any pause that follows an edit, including mid-work questions. Formatting at commit time already exists.
- **Keeping the pre-push check.** Nothing that fails the check ever reaches GitHub, but every push waits for the full check, and CI already blocks merging.
- **A DoltHub remote.** Hosted Dolt remotes are built for many writers and offer a web view of the database. They add an account, credentials on every machine, and a service outside GitHub. Suppressing the info branch removes the banner without them.
- **An S3 or GCS bucket.** No server to run, but it needs a cloud account and credentials on every machine.
- **A shared Dolt server** (Beads server mode, reached over Tailscale). One database means atomic claims and no merges. But every machine must reach the server to use Beads at all, and it adds a service to run and back up.
- **A dedicated repository for Beads data.** It moves the banner to a repository nobody watches, at the cost of one more repository per project.
- **`dolt.auto-push`.** It does network work on every write, and Beads documents it as unsafe for racing writers. Pushes from shells without the variable would also publish the info branch.
- **JSONL in Git.** Beads documents the export as unsuitable for sync: import is upsert-only and loses deletions.

## Open questions

None. On 2026-11-15 the owner chose to run `mise run check` in CI only.
