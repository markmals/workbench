---
name: reviewing-an-implementation
description: Use when an implemented proposal has fresh validation evidence and needs its self-check, independent refutational review, human-exercisable preview, and ready-for-review handoff.
---

# Reviewing an Implementation

**Phase 3 — Implementation.** Check the work against the approved change and the current specifications it revised, have an agent without your context try to break it, then hand it to the human. This procedure does not decide whether the human accepts the work.

Begin after applicable validation has fresh evidence under `.agents/rules/verification.md`. Proceed in this order:

1. Self-check.
2. Dispatch `adversarial-review`; consume its report.
3. Produce the preview.
4. Post the readiness report and move the proposal to `active-review`; the implementation PR should already be ready.

## 1. Self-check

Read the proposal, the revised current specifications, the changed tests, the affected guides, and the diff together. For each observable behavior the change adds or alters in those specifications — including boundary cases, errors, defaults, and side effects that could disagree — confirm that the specification states it, that it is tested, that it is described where a user would look when it is user-facing, and that it is implemented. Then look the other way: anything implemented that the specifications and guides do not describe is scope creep or a gap, and anything the specifications now state that the proposal did not authorize is scope creep in the specification. Look and feel the proposal leaves open is neither, and neither is a look-and-feel detail the proposal pinned that the human has since changed through polish.

Then ask whether this is the implementation that should remain. Does its structure fit the domain the specifications describe, or does it leak implementation conveniences into public behavior? Is there a simpler realization that expresses the contract better, or complexity inherited from the baseline that no longer earns its place?

Resolve each disagreement in exactly one way:

| Resolution | Use when | Action |
| --- | --- | --- |
| Fix the wrong artifact | Intent has not changed | Correct the specification, tests, guides, or code within the approved change, and rerun the affected gates. |
| Revise the proposal | Intent or scope changed | Discuss with the human first, then revise the affected specifications and derive the other artifacts again. |
| Record the divergence | It remains deliberate for human review | Name the behavior, sources, impact, and reason in the readiness report. |

You may edit specifications, tests, guides, and code within the approved change. You must not revise a proposal without the human.

## 2. Independent review

Dispatch one subagent with fresh context for `adversarial-review`, on a different model from yours where the harness allows. Give it the proposal, the current specifications the change revised and the relevant unchanged constraints, the commit range, and the commands that verify the work. Tell it to try to show the work is not done — behavior missing or wrong relative to the specifications and the proposal, edge cases the tests miss, tests that would still pass with a broken implementation, security or data-loss risks — to confirm findings by running code, to ignore style, naming, formatting, and look and feel, and to report without editing.

Do not replace this dispatch with your own review. Consume its report:

| Report item | Response |
| --- | --- |
| Confirmed defect inside the proposal's scope | Fix it and refresh the affected evidence. Run the review once more only when the fix was substantial. |
| Suspicion | Verify it or drop it; never fix it blind. |
| Proposal gap | Surface it to the human; do not silently choose an interpretation. |
| Finding outside the proposal's scope | Never widen the change to fix it. Decline it with reasoning, or file it through `tracking-work`. |
| Declined finding | Preserve it with reasoning for the readiness report. |

A disputed finding does not disappear because you decline it.

## 3. Produce the preview

A preview is **the cheapest realistic artifact through which the human can exercise the changed behavior.**

| Requirement | Meaning                                                                   |
| ----------- | ------------------------------------------------------------------------- |
| Cheapest    | Do not stand up production infrastructure merely for review.              |
| Realistic   | Do not substitute screenshots, transcripts, or descriptions for behavior. |

Reviewing prose is not reviewing behavior. Use the configured project task and CI mechanism; run `mise tasks` when the setup is unknown.

| Repository shape | Preview                                               |
| ---------------- | ----------------------------------------------------- |
| Library          | Prerelease package                                    |
| Web app          | Preview deployment                                    |
| API              | Preview endpoint                                      |
| Docs             | Deployed documentation site                           |
| CLI              | Installable executable                                |
| Desktop          | Installable build                                     |
| Mobile           | Installable build via ad-hoc or internal distribution |

For a mixed repository, preview the surface the proposal changed; do not produce every repository-shaped preview by habit. A pure refactor with no exercisable surface produces no preview. State that plainly in the readiness report rather than fabricating one.

**The human must be able to open or install it directly.** A downloadable build artifact is not a preview — reviewing it would mean unzipping a file and starting a local server, which is enough friction that the change gets reviewed as prose instead. The one exception is a command-line tool, where installing the binary is the real act. If your preview step ends in "upload artifact" for anything else, it is not finished.

A site deployed to a subpath needs its base path set to that subpath at build time, or every asset resolves against the domain root and the preview loads blank. Verify the deployed preview actually renders before linking it; a green workflow is not evidence, per `.agents/rules/verification.md`.

Build from the pull-request revision. Give repository-accessible humans a stable link or installation location; they must not build the artifact themselves. The readiness report links it and states required setup, what to try, and the expected result. A bare URL is insufficient. Keep the preview's lifetime tied to the pull request.

Previews are private by default. Do not publish one publicly unless the project deliberately opts in. A prerelease package published to a public registry is public forever; use a prerelease tag and state that public status in the readiness report. Teardown belongs to Phase 5 cleanup through `completing-a-feature`.

## 4. Report readiness

Compose the pull-request comment from `.agents/templates/READINESS_REPORT.md`. Fill every template section and add the self-check's recorded divergences and their resolutions.

| Report section | Required content |
| --- | --- |
| Proposal | State `FULLY` implemented only when every promised behavior exists. For `PARTIALLY`, name completed and omitted behavior, limitations, and reason. Silent partial implementation is not allowed. |
| What was built | Delivered behavior, not internals, with links to the current specifications and records the change revised. |
| How to exercise it | Preview link or honest absence, setup, actions, and expected results. |
| Validation performed | Every command actually run in this session and its observed result, per `.agents/rules/verification.md`. Never report an unrun gate as passing. |
| Adversarial findings adopted | Each accepted or fixed finding, or `- None.` |
| Adversarial findings declined, with reasoning | Every declined finding and why it was declined, or `- None.` This section is always present. |
| Defects discovered and deferred | Each deliberate, out-of-scope defect as a linked Beads item from `tracking-work`, or `- None.` |
| Remaining limitations | Known behavior gaps, including specified behavior no check exercised, or `- None.` |
| What to look at first | The first behavior, preview action, or diff the human should inspect. |

Declined adversarial findings keep the independent review useful. Do not quietly omit a narrow, uncomfortable, or disputed finding.

### Post as a service account

Prefer the repository's agent identity for the readiness comment, rather than the human's PR identity. Where a service-account token is configured:

```sh
GH_TOKEN="$SERVICE_ACCOUNT_TOKEN" gh pr comment <pr-number> --body-file readiness-report.md
```

Confirm the authenticated identity before posting. Without a service account, identify the available account in the report unless repository identity rules forbid that fallback. PR readiness uses the ordinary `gh pr ready` under the human's identity during implementation; the service-account token is only for the agent-authored report.

### Make the Phase 4 proposal transition

The feature pull request should already be ready because the complete, verified implementation was pushed during Phase 3. If it is unexpectedly draft, verify that its pushed revision contains the complete implementation and mark it ready with `gh pr ready <pr-number>` before the handoff.

At the end of Phase 3, move the proposal to `status: active-review` and post the readiness report on that PR. These actions form the handoff to human review; do not claim the report was posted if the bot comment failed. PR readiness and proposal status are separate facts: the PR may be ready while the proposal is still `awaiting-implementation`.

### Look-and-feel feedback in review

While the proposal is `active-review`, apply feedback that touches only look and feel with [`.agents/skills/polishing-a-ui/`](../polishing-a-ui/SKILL.md): change the code, run the gates, push, refresh the preview, and say what changed. Do not edit the proposal, change its status, post another readiness report, or rerun the adversarial review. Feedback that changes what the feature does, or a design decision the proposal records, goes through the proposal first, after the human returns the work with `returned-for-revisions`. Split feedback that mixes both.

## Red flags

- Your self-check reviewed style instead of comparing the proposal, specifications, tests, guides, and code.
- A disagreement between them has no recorded resolution.
- You widened the change to fix a finding outside the proposal.
- You changed proposal intent without the human.
- You skipped independent adversarial review or dropped a declined finding.
- The preview is fabricated, public by accident, detached from the pull request, or a bare URL.
- The report claims a validation gate was not run.
- You left a verified, pushed implementation in a draft PR merely because its readiness report is pending.

Correct the record before handoff.
