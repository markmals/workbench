---
name: completing-a-feature
description: Use after a human has reviewed a pull request and the accepted change needs integration, release, and cleanup. Do not use before human review.
---

# Completing a Feature

**Phase 5.** Complete work in this order: **process feedback → vision update → merge → release → dependent-repo updates → cleanup**.

## 1. Process feedback

Read the human's PR comments and any direct commits before acting. Classify each item:

| Feedback changes   | Action                                                                                                                    |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| The implementation | Fix it, then rerun the affected implementation loop and review evidence.                                                  |
| Intended design    | Update the proposal first with the human, then rerun tests → guides → code → quality gates → Review against the revision. |

This judgment matters most. Never encode a design change in code, tests, or guides while leaving the proposal stale.

## 2. Update the vision

Reread `VISION.md`. Update it when the accepted change alters the project's capabilities, MVP, or direction. Bump `updated` to the current date.

Consult the human before a substantial rewrite. Do not turn a narrow completion change into a new vision exercise.

## 3. Merge

Merge only after human acceptance. Preserve the record of what was proposed, what was implemented, why decisions were made, what review found, and how the final state differs from earlier revisions.

Choose a merge strategy that retains that context in the PR and its history. Do not squash the work into one opaque commit that discards the discussion. Set the accepted proposal's `status: implemented` as part of the final PR record.

## 4. Release

Release through the project's existing CI/CD process. Inspect `mise tasks` for the project's release path.

**Never publish without explicit human confirmation.** Approval to merge is not approval to release.

## 5. Update dependent repositories

For each draft PR consuming a preview version, move it to the production release. Mark it ready for review when appropriate after the preview dependency is gone.

## 6. Clean up

After release and dependent updates, remove temporary files, tear down preview configuration and published prerelease artifacts, and prune the feature branch locally and remotely.

> **Explicit human confirmation required:** Branch deletion and force-push are the most destructive operations in this process. Ask before either one, even when cleanup appears routine.

Cleanup runs when confidence is highest. Treat it as a separate destructive operation, not an automatic epilogue.

## Red flags

- You fixed a design-change comment without revising the proposal.
- You ignored a direct human commit.
- You merged without preserving why the final state differs.
- You released because the PR merged.
- A dependent PR still uses a preview version after release.
- A preview or prerelease artifact outlives its PR.
- You deleted a branch or force-pushed without explicit human confirmation.

Any of these means stop and return to the relevant ordered step.

## Related skills

- `producing-a-preview` — previews are torn down here.
- `reporting-readiness` — the record consumed during human review.
