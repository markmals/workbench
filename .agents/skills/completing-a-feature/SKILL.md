---
name: completing-a-feature
description: Use after a human has reviewed a pull request and the accepted change needs integration, release, and cleanup. Do not use before human review.
---

# Completing a Feature

**Phase 5.** Complete work in this order: **process feedback → finalize policies and decisions → update the vision → record `implemented` in the feature PR → merge → required release → dependent-repo updates → cleanup**.

## 1. Process feedback

Read the human's PR comments and direct commits before acting. Classify each item:

| Feedback changes | Action |
| --- | --- |
| The implementation | Fix it, then rerun the affected implementation loop and review evidence. |
| Intended design | Update the proposal first with the human, then rerun current specifications → tests → guides → code → quality gates → review against the revision. |

Never encode a design change in specifications, code, tests, or guides while leaving the proposal stale.

While revisions are outstanding, the proposal is `returned-for-revisions`, which the human sets when requesting changes. Return to Phase 2 for a design revision or Phase 3 for implementation. If reworking the design reopens a question, record a clarification marker — `returned-for-revisions` is exempt from the marker rule so honest rework remains possible. At the end of Phase 3, hand off to `reviewing-an-implementation`, which alone owns the status and readiness-report handoff. Do not set proposal status when reposting.

## 2. Finalize policies and decisions

After feedback establishes the feature's final shape, settle the records this change adds, revises, or retires: knowledge that outlives its proposal.

**The usual answer is that there is none.** Most features establish no policy and no decision, and recording nothing is the correct outcome rather than a step you skipped. Empty `policies/` and `decisions/` directories are healthy. A record written because the phase has a slot for it is one nobody follows and everybody has to read.

| Record   | Distinguishing test                                                              |
| -------- | -------------------------------------------------------------------------------- |
| Policy   | It constrains future work: “always or never do X.”                               |
| Decision | It explains a choice future work must understand: “we chose X over Y because Z.” |

If it does not outlive this proposal, it belongs in the proposal and nowhere else.

Before adding, revising, or deleting either:

1. Agree it with the human. Neither is an agent-only conclusion.
2. For a new record, copy the canonical template. Required frontmatter is `id`, `title`, and `status`; `established-by: proposal.<NNNN>` is optional provenance.
3. Give each agreed record its current status: `active` for a policy, `accepted` for a decision. A proposed record does not leave the change branch until it is agreed.

### Enforce policies first

A policy that can be enforced by a check **must** be. Before writing prose, try to express it as a validator rule, continuous-integration step, or template section, per `.agents/rules/enforcement-hierarchy.md`.

Keep prose for judgment a deterministic check cannot decide. Prose policies are the ones that rot.

### Revise or retire in place

A policy or decision file states what applies now, and Git with the proposal keeps what applied before.

| The agreed record | Do this |
| --- | --- |
| Changes meaning or scope | Edit the same file. Keep its filename and ID, revise its rationale and enforcement together, and update current references in the same change. |
| No longer applies | Delete the file. First repoint or remove current links to it. Replace each historical link that must keep working, such as one in a completed proposal, with an immutable Git revision URL; `git log -1 --format=%H -- <file>` gives a commit that contains it. Never reuse its ID. |
| Still binds a supported compatibility promise or an unfinished migration | It still applies. Keep it until that obligation ends. |

Write no replacement record and no `supersedes` or `superseded` metadata on a policy or decision; the validator rejects both. [Keeping Current Specifications](../../../guides/current-specifications.md#worked-example-revising-and-retiring-a-policy) works through one revision and one retirement.

## 3. Update the vision

Reread `VISION.md`. Update it when the accepted change alters the project's capabilities, MVP, or direction. Bump `updated` to the current date.

Consult the human before a substantial rewrite. Do not turn a narrow completion change into a new vision exercise.

## 4. Record the accepted implementation

Add the changeset if the work does not already carry one: `mise run changeset`, bump chosen deliberately, entry written for the person reading the changelog. A change with no user-visible effect still carries one, created with `--empty`. CI rejects a pull request without a changeset, so this is a gate, not a courtesy.

After human acceptance and the vision update, set the proposal to `implemented` in the same feature pull request that contains the implementation. This status records accepted implementation, not a merged pull request or completed release. Record acceptance and any outstanding delivery in the proposal or PR, then commit and verify the updated integration candidate before merging. Never open a follow-up PR solely to move this status.

Confirm the implemented record is present in the pull request revision before merging; a local file or unpushed commit is not proof. For an already merged pull request, inspect its merged revision before concluding a local `accepted` status means the record is missing. If absent, ask the human how to reconcile it and keep any required delivery visible. Do not try to amend a merged pull request or open a status-only PR.

## 5. Merge

Merge only after human acceptance and fresh checks on the pull request revision that contains the implementation record. The merge lands the revised current specifications together with their implementation. Preserve what was proposed, implemented, and learned in review, including why the final state differs from earlier revisions.

An accepted PR still in draft is an anomalous state. Verify its pushed implementation, mark it ready, and verify the updated revision before merging. Draft status never authorizes skipping readiness or review.

Choose a merge strategy that retains the proposal and PR discussion. Do not squash the work into one opaque commit that discards that context.

## 6. Release

When release is required and separately authorized, use the project's existing CI/CD process. Merging accumulates changesets on the default branch, and the release workflow opens a release pull request that runs `mise run version`: `changeset version` applies the pending changesets, bumping the version in the root `package.json` and writing `CHANGELOG.md`. Merging that release pull request is what publishes. Run `mise tasks` when the setup is unknown. Skip publication for approved merge-only delivery.

**Never publish without explicit human confirmation.** Approval to merge is not approval to release.

The implementation record does not complete a required release. Keep that obligation visible in the proposal or PR until the authorized release actually happens; do not call the feature delivered before then.

## 7. Update dependent repositories

After an authorized release, update any dependent draft PRs consuming a preview version to the production release, then mark them ready for review when appropriate. No dependent-release update is needed for merge-only delivery.

## 8. Clean up

After the required delivery, remove temporary files and preview configuration and prune the feature branch only with the required authorization.

> **Explicit human confirmation required:** Branch deletion, force-push, and tearing down published prerelease artifacts are the most destructive operations in this process. Ask before any of them, even when cleanup appears routine.

Cleanup runs when confidence is highest. Treat it as a separate destructive operation, not an automatic epilogue.

## Red flags

- You fixed a design-change comment without revising the proposal.
- You ignored a direct human commit.
- You recorded, revised, or deleted a policy or decision without the human.
- You wrote a prose policy before checking whether automation can enforce it.
- You wrote a replacement record or supersession metadata instead of revising or deleting the existing policy or decision.
- You deleted a record while a current link, or a historical link that must keep working, still pointed at it.
- You merged without preserving why the final state differs.
- You released because the PR merged.
- You merged a pull request that carries no changeset.
- A dependent PR still uses a preview version after release.
- A preview or prerelease artifact outlives its PR without a tracked, authorized teardown plan.
- You deleted a branch, force-pushed, or tore down a published prerelease artifact without explicit human confirmation.
- You waited until after merge to move `implemented` and opened a bookkeeping-only PR.
- You treated `implemented` as proof that a required release has already happened.

Any of these means stop and return to the relevant ordered step.

## Related skills

- `reviewing-an-implementation` — produces the preview and readiness record consumed during human review.
