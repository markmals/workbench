---
id: proposal.0002
title: Make Preview Teardown Final
authors: [markmals, Claude]
status: draft
pull-request:
issues: [https://github.com/markmals/workbench/issues/3]
supersedes: []
---

# Make Preview Teardown Final

## Summary

A preview deployment can survive its pull request. The deploy job now confirms the pull request is
still open immediately before publishing, so a late-arriving run cannot resurrect a preview that
was already torn down.

## Motivation

Closing a pull request is supposed to remove its preview. On pull request 1 it did not, and the
preview stayed live until it was removed by hand.

Both fixes suggested when the defect was filed would have failed, which is the reason this is
worth a proposal rather than a patch. The observed timings show why:

| Run           | Created  | Finished | Build steps | Effect              |
| ------------- | -------- | -------- | ----------- | ------------------- |
| `33020579883` | 22:43:31 | 22:43:47 | skipped     | removed the preview |
| `33020601519` | 22:43:53 | 22:44:28 | ran         | re-deployed it      |

The deploy run was **created six seconds after the removal run had already finished**. The two
never coexisted. Concurrency controls only order runs that overlap: `cancel-in-progress` cancels
what is currently running, and a queued run is cancelled only by a newer queued run in the same
group. Neither applies to a run that does not exist yet.

So the guess in the issue — that the concurrency group failed to serialize them — is wrong. Nothing
about concurrency configuration can fix this, because the race is between a webhook delivery and a
completed workflow.

The underlying error is a category one: the deploy path trusts an event payload describing the
world at the moment the event fired, and acts on it later without rechecking. `mise run status`
already refuses to guess about state it cannot confirm; this workflow does not.

## Proposed solution

Split the workflow into a deploy job and a remove job, then have the deploy job ask GitHub whether
the pull request is still open at the moment it is about to publish, and stop if it is not.

Removal becomes final: once a pull request closes, no later run can publish for it, regardless of
when the run was created or how long it queued.

## Detailed design

`.github/workflows/preview.yml` becomes two jobs with mutually exclusive conditions, replacing the
current single job whose steps carry per-step `if` guards.

**`remove`** — runs when `github.event.action == 'closed'`. Checks out, then invokes
`rossjrw/pr-preview-action` to remove. No build, so it stays fast.

**`deploy`** — runs when `github.event.action != 'closed'`. Before publishing, and after the build,
it queries the live state of the pull request:

```sh
gh pr view "$PR_NUMBER" --json state --jq .state
```

The publish step runs only when that returns `OPEN`. The query happens **after** the build rather
than before it, so the window between the check and the publish is as small as possible — checking
first and building for a minute would reopen the same race in miniature.

When the state is not `OPEN`, the job logs that it is skipping publication for a closed pull
request and succeeds. A skipped publication is correct behavior, not a failure, and failing would
put a red mark on a merged pull request for something nobody needs to act on.

The query needs `GITHUB_TOKEN` with `pull-requests: read`, which the workflow already grants.

Concurrency is unchanged. `cancel-in-progress` remains useful for superseded deploys; it is simply
not what was broken.

## Compatibility

No behavior change for any pull request that is still open. The only altered outcome is the one
that was wrong.

## Implications on adoption

None. Adopting projects that wired a different preview mechanism from the recipe menu should apply
the same guard if their mechanism publishes on `synchronize`, which the comments in the workflow
will note.

## Scope

- Split `preview.yml` into `deploy` and `remove` jobs.
- Add the open-state guard to the deploy path.
- Note the hazard in the recipe menu so adapted recipes inherit the fix.

### Out of scope

- **Sweeping already-stale previews.** A scheduled job reconciling `gh-pages` against open pull
  requests would catch previews orphaned by other means. There are none — the one that existed was
  removed by hand — so this would be built for a problem that is not currently observed.
- **Changing the concurrency configuration.** It is not the cause. Changing it to look responsive
  would leave the actual defect in place.

## Preview

None. This change has no user-exercisable surface: it alters when a workflow publishes, and the
observable result is the _absence_ of a stale deployment.

The honest verification is behavioral rather than a preview — open a pull request, let it deploy,
merge it, and confirm the preview is gone and stays gone. That is recorded in the readiness report.

## Policies and decisions checked

`policies/` and `decisions/` are both empty; both directories were read. `VISION.md` was read and
this change is consistent with it.

One principle in `VISION.md` bears directly on the fix: _previews expose behavior, not descriptions
of behavior._ A preview that outlives its pull request inverts that — it shows behavior from a
revision nobody is reviewing.

This is a candidate for the first real decision entry: **act on live state, not on a stale event
payload.** Whether one defect justifies a record is a judgement to make at completion, not now.

## Future directions

- A scheduled reconciliation between `gh-pages` and open pull requests, if previews are ever
  orphaned by a route this guard does not cover.
- Applying the same live-state discipline to any other workflow that acts on an event payload
  after a delay. There are currently none.

## Alternatives considered

**Gate the build and deploy steps at the job level instead of per-step.** Suggested in the issue.
Rejected because it changes nothing about the outcome — the deploy run's steps were meant to run;
it was a genuine deploy event. Job-level conditions are still worth adopting for readability, and
this proposal does adopt them, but as tidying rather than as the fix.

**Drop `cancel-in-progress` for the close path.** Also suggested in the issue. Rejected on the
evidence: the two runs never overlapped, so nothing was there to cancel. This would have looked
like a fix while leaving the defect intact — the worst outcome available, because the issue would
have been closed.

**Have the remove job re-run on a delay to catch late deploys.** Rejected as a race against a race.
It narrows the window without closing it, and it makes teardown timing nondeterministic.

**Check the pull request state before building rather than after.** Rejected because it leaves a
minute-wide window between the check and the publish, which is the same defect with a smaller
constant.

## Open questions

None.
