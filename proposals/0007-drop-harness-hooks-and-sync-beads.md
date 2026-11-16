---
id: proposal.0007
title: Drop Harness Hooks and Sync Beads Across Machines
authors: [markmals, anthropic/claude-opus-5-5]
status: draft
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
- **Measured hook costs.**
    - `mise run check` takes 4.7 s in Workbench, and the 4–8 minutes in Trove come from [proposal 0003](0003-fit-the-harness-to-current-models.md).
    - `bd prime` took 0.7 s and printed 8,444 characters. With `--memories-only` it printed 2,831, all of them memories recorded that day.
    - `mise run fmt` on one file took 0.26 s.

### Relevant constraints and principles

| Concept | Identity and lifetime |
| --- | --- |
| Beads replica | One machine's local Dolt database for a project: a full copy that works offline. Lives in that checkout's `.beads/`. |
| Beads remote | `refs/dolt/data` on the project's own GitHub repository. The meeting point of the replicas; never a branch. |
| Sync | One `bd sync` cycle: pull, conflict check, push with retries. |
| Sync point | A moment in the Workbench process at which a sync runs. Defined by this proposal. |

The ontology review is COHERENT. Beads already defines replicas, remotes, and sync. This proposal adds only the process concept of a sync point. A claim is atomic within one replica. Across replicas, how soon others see a claim is bounded by the sync points.

Other constraints:

- **Owner's priorities.** Agent-loop speed comes first, and Workbench adds no per-turn work. GitHub branch noise from syncing is not acceptable.
- **[Vision](../VISION.md).** Enforce mechanically, and make ceremony earn its cost. Beads holds the work graph; Workbench does not compete with it.
- **Error handling.** A failed sync is reported, never hidden.

### Quality bar

- After a session's `mise run status`, the agent sees every Beads change any machine had pushed at its own latest sync point.
- No turn pays for a check, a formatter run, or a prime injection it did not ask for.
- No sync creates or updates anything under `refs/heads/`.
- Concurrent syncs from two machines never silently lose a write that Beads does not settle by design. Push races retry, and unsafe conflicts halt and are reported.

### Remaining uncertainty

- **Claims are not atomic across machines.** Two machines can claim the same item between syncs. The pull settles issue fields last-write-wins, so one claim replaces the other; the losing machine learns at its next sync. Syncing immediately after claiming narrows that window but does not close it.
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
Machine B: mise run status → sync → sees A's claims and new items → claims t-42
           mise run beads:sync → the claim is published
Machine A: next mise run status, git push, or git pull → sync → sees t-42 claimed
GitHub:    refs/dolt/data advances; no branch is created or updated; no banner
```

## Detailed design

### 1. Remove the harness hooks

Delete `.omp/extensions/prime-beads.ts`, `format-after-edit.ts`, and `check-before-stop.ts`, and `tools/format-after-edit.test.ts`. Workbench then wires no harness lifecycle event. `.omp/` keeps only the `workflow` tool, which [proposal 0006](https://github.com/markmals/workbench/pull/10) removes separately.

| Removed hook | Covered by |
| --- | --- |
| `bd prime` before the first turn | `mise run status`, already the contract's first instruction, now reports Beads ([section 2](#2-beads-in-mise-run-status)). |
| Format after every edit | hk's `pre-commit` hook, which already formats staged files and stages the result. Agents may run `mise run fmt -- <files>`. |
| Full check before every stop | The verification rule (run `mise run check` this turn before claiming work is done), hk's `pre-push` hook (see [Open questions](#open-questions)), and CI. |

### 2. Beads in `mise run status`

- **Order.** `mise run status` runs one sync ([section 3](#3-syncing-beads-across-machines)) before reporting. After the phase report, it adds a Beads section:
    - the sync outcome;
    - ready work, from `bd ready`;
    - the project's memories, from `bd prime --memories-only`.
- **Sync outcomes:**

    | Outcome | Reported as |
    | --- | --- |
    | `synced` | One line. |
    | `failed` | The reason (unreachable remote, timeout, authentication), and that the rest shows this machine's state. |
    | `conflict` | `bd sync` exit 2: the conflict needs a person to resolve it, with a link to the adoption guidance. |
    | `blocked` | `bd sync` exit 4: a dirty working set blocks publishing. |
    | `unavailable` | `bd` is missing or the workspace is not initialized, with a pointer to `mise run setup`. Shown instead of the whole section. |

    The sync is limited to 20 seconds; past that its outcome is `failed`.

- **Exit status.** Beads outcomes never change the exit status of `mise run status`.
- **JSON.** `mise run status -- --json` adds a `beads` object that holds either:
    - `sync` (`outcome` and `detail`), `ready` (a list of `id`, `title`, `priority`), and `memories` (text); or
    - `unavailable` (the reason).
- **Contract and skill updates.** [Record Validation and Workflow Status](../guides/record-validation.md) gains this section as part of `status`'s contract. The `tracking-work` skill's start sequence becomes `mise run status`, then `bd show <id>`.

### 3. Syncing Beads across machines

- **The remote** is the project's own GitHub repository, through `sync.remote`, with Beads history in `refs/dolt/data`. That is unchanged from today.
- **Every sync runs through mise.** A project's `mise.toml` sets:

    ```toml
    [env]
    DOLT_REMOTE_INFO_BRANCH = ""
    ```

    It defines a `beads:sync` task that runs `bd sync`. `mise run status`, the `beads:sync` task, and hk's mise-run steps all see that variable, so no sync publishes `__dolt_remote_info__`.

- **Sync points:**
    1. `mise run status` ([section 2](#2-beads-in-mise-run-status)).
    2. hk's `pre-push` hook runs `mise run beads:sync` before its other steps.
    3. hk's `post-merge` hook runs `mise run beads:sync` after a pull or merge.
    4. After claiming, closing, or creating a Beads item another machine should see, the agent runs `mise run beads:sync`. The contract states this rule.
- **Failures in hooks.** A Beads sync failure in a git hook prints its outcome and never blocks the git operation.
- **Never:**
    - `dolt.auto-push`, which `mise run setup` sets to `false`;
    - `bd dolt push`, `bd dolt pull`, or `bd sync` run outside mise;
    - `git push --mirror` against the project's repository.

    The contract states the last two.

- **Setting up a machine.** `mise run setup` runs `bd bootstrap --yes` in a project that already has `.beads/config.yaml`. That restores the replica from `refs/dolt/data` instead of creating a fresh database with an unrelated history. It runs `bd init` only in a project that has no Beads workspace yet.
- **Existing info branches.** A project whose repository has `__dolt_remote_info__` deletes it once; [beads#5266](https://github.com/gastownhall/beads/issues/5266) confirms that is safe. Workbench's repository has none. When Beads publishes beads#7945, the `[env]` entry becomes redundant and harmless.

### 4. Documents and configuration that change

- **`AGENTS.md`:**
    - The harness setup table drops the lifecycle rows.
    - Work tracking states the sync rules from [section 3](#3-syncing-beads-across-machines).
    - Work tracking states what each tracker holds. Beads holds the work agents discover and the project's memory. GitHub Issues and Projects hold the owner's planning. The sentence calling Beads a replacement for GitHub-issue defect tracking goes.
    - The quality-gates list adds `mise run beads:sync`.
- **`PROCESS.md`'s Work Tracking section** describes replicas, the remote, and the sync points, and states the same division between Beads and GitHub Issues and Projects.
- **The `writing-a-proposal` skill's red-flag table** files a newly discovered out-of-scope defect in Beads, not as a GitHub issue.
- **`guides/adapting-your-harness.md`** drops "Wire lifecycle events" and the extension rows. A harness needs only the contract and the skills.
- **`guides/getting-started.md` and `README.md`** stop describing lifecycle extensions.
- **`.agents/rules/enforcement-hierarchy.md`'s Tier 0** drops `.omp/extensions/` and names the hk steps.
- **[Record Validation and Workflow Status](../guides/record-validation.md)** and the **`tracking-work` skill** change as described in [section 2](#2-beads-in-mise-run-status).
- **Configuration:**
    - `mise.toml` gains the `[env]` entry, the `beads:sync` task, and the setup changes.
    - `hk.pkl` gains the `pre-push` and `post-merge` steps.
    - `tools/status.ts` and its tests gain the Beads section.

## Compatibility

- **Sessions lose their automatic work.** They no longer format after each edit, run the check before stopping, or receive `bd prime`. Agents run `mise run status` and the check themselves, as the existing rules already require.
- **Beads behavior changes:**
    - Workbench's own setup changes from `bd init --init-if-missing` to `bd bootstrap` for an existing workspace. A machine whose replica already exists keeps it.
    - A project that sets `dolt.auto-push: true`, such as Recon, switches to the sync points.
    - A project whose repository lacks `refs/dolt/data`, such as Bento, publishes it at its first sync.
- **Consumer projects.** Until [proposal 0006](https://github.com/markmals/workbench/pull/10) delivers Workbench as a package, each consumer takes this change by copying it, like any other Workbench change today.

## Implications on adoption

- **Requirements:**
    - bd 1.3.2 or newer, for `bd sync` and `bd bootstrap`;
    - Git credentials for the project's GitHub repository on every machine (pushing already needs them);
    - a Dolt commit identity on every machine.
- **Steps for an existing project:**
    1. Delete the three extensions.
    2. Add the `[env]` entry, the `beads:sync` task, and the hk steps.
    3. Turn off `dolt.auto-push`.
    4. Delete any `__dolt_remote_info__` branch.
    5. Run `mise run setup` on each machine.
- **Reversal.** Restore the files from Git history and re-enable whatever sync the project used before.

## Scope

- Delete the three extensions and their test, and update the documents in [section 4](#4-documents-and-configuration-that-change).
- Add the Beads section to `mise run status`, with tests that stub `bd` for each outcome in [section 2](#2-beads-in-mise-run-status).
- Add the `[env]` entry, the `beads:sync` task, the setup change, and the hk steps to Workbench.

### Out of scope

- **Packaging Workbench and porting `status` to Rust.** That is [proposal 0006](https://github.com/markmals/workbench/pull/10), which ports this behavior unchanged.
- **Migrating consumer projects,** including deleting Recon's info branch. Each is a change in that repository.
- **Changing Beads itself**, such as beads#7945.
- **A shared Dolt server, a DoltHub or bucket remote, or per-machine identities for claims.** See Alternatives and Future directions.

## Preview

- **Repository shape:** process tooling with CLI checks.
- **Artifact:** this branch, exercised against the real GitHub remote from two clones, each with its own replica. Two clones on one machine behave as two machines for Beads; a second physical machine is optional.
- **Exercise:**
    1. In clone A, run `mise run status` and read the Beads section.
    2. In clone B, claim an item and run `mise run beads:sync`.
    3. In A, run `mise run status` and see the claim.
    4. Push a branch from A and confirm hk synced.
    5. Run `git ls-remote origin` and confirm there is no `refs/heads/__dolt_remote_info__`.
    6. Run `mise run status` with the network off and see the `failed` outcome.
- **Cleanup:** close the probe items, sync, and delete the second clone.

## Policies and decisions checked

- **Policies and decisions.** None: `policies/` and `decisions/` hold only their READMEs.
- **[VISION.md](../VISION.md).** "Ceremony must earn its cost": the hooks are removed on measured cost. "Enforce mechanically": checks remain at commit, push, and CI. "Not an issue tracker": Beads keeps the work graph.
- **[Proposal 0003](0003-fit-the-harness-to-current-models.md)** recorded `check-before-stop`'s timeout in Trove and suggested a fast-subset stop hook. This proposal removes the hook instead; see Alternatives.
- **[Proposal 0005](0005-current-specifications.md)** put `status`'s contract in `guides/record-validation.md`, which this proposal extends.
- **[Proposal 0006](https://github.com/markmals/workbench/pull/10)** depends on this one: its package declares no harness hooks, and its Rust `status` ports this Beads section.
- **`.agents/rules/enforcement-hierarchy.md`.** Tier 0 loses the harness wiring and gains the hk Beads steps. Every remaining check runs whether or not the agent remembers.

## Future directions

- **Visible claimants.** A per-machine Beads actor, such as `BEADS_ACTOR=<user>@<host>`, would show which machine holds a claim. All machines currently claim as the same Git user.
- **A shared server.** A single Dolt server, for example on a home server over Tailscale, would make claims atomic across machines if double claims become a real problem.
- **Timed syncs.** A sync on a timer could serve long sessions that rarely orient, push, or pull.

## Alternatives considered

- **Cheaper hooks.** A fast subset of the check before stopping, or a check that the gate passed since the last edit, still forces work before any pause that follows an edit, including mid-work questions. Formatting at commit time already exists.
- **A DoltHub remote.** Hosted Dolt remotes are built for many writers and offer a web view of the database. They add an account, credentials on every machine, and a service outside GitHub. Suppressing the info branch removes the banner without them.
- **An S3 or GCS bucket.** No server to run, but it needs a cloud account and credentials on every machine.
- **A shared Dolt server** (Beads server mode, reached over Tailscale). One database means atomic claims and no merges. But every machine must reach the server to use Beads at all, and it adds a service to run and back up.
- **A dedicated repository for Beads data.** It moves the banner to a repository nobody watches, at the cost of one more repository per project.
- **`dolt.auto-push`.** It does network work on every write, and Beads documents it as unsafe for racing writers. Pushes from shells without the variable would also publish the info branch.
- **JSONL in Git.** Beads documents the export as unsuitable for sync: import is upsert-only and loses deletions.

## Open questions

- [NEEDS CLARIFICATION: Should hk's `pre-push` hook keep running the full `mise run check`, or should the gate run only in CI, with the agent waiting for CI (`gh pr checks --watch`) before marking a pull request ready? Dropping it removes a 4–8 minute wait from every push in Trove-sized projects, at the cost of failing commits reaching a draft pull request before CI flags them.]
