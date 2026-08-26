---
name: producing-a-preview
description: Use when a change has reached Review and a human needs to exercise its changed behavior. Do not use to publish a release.
---

# Producing a Preview

**Phase 4, stage 3.** A preview is **the cheapest realistic artifact through which the human can exercise the changed behavior.**

Both words bind:

- **Cheapest** rules out production-grade infrastructure merely for review.
- **Realistic** rules out a screenshot, transcript, or description of what would happen.

Reviewing prose is not reviewing behavior.

## Choose one preview

A repository wires exactly the row matching its shape.

| Repository | Preview                                     |
| ---------- | ------------------------------------------- |
| Library    | prerelease package                          |
| Web app    | preview deployment                          |
| API        | preview endpoint deployment                 |
| Docs       | rendered documentation build                |
| CLI        | installable executable                      |
| Desktop    | installable build                           |
| Mobile     | installable build (ad-hoc / internal track) |

Use the project's configured task and CI mechanism; inspect `mise tasks` when that setup is not known.

### Mixed and non-exercisable changes

- A repository that is both a library and a CLI previews the surface the proposal changed. Do not produce both by habit.
- A pure refactor with no exercisable surface previews nothing. Say that plainly in the readiness report; do not fabricate an artifact.

## Make it reviewable

1. Build the preview from the PR revision, not a different checkout or release candidate.
2. Give repository-accessible humans a stable link or installation location.
3. In the readiness report, link it and state what to try, expected result, and any required setup. A bare URL is insufficient.
4. Keep its lifetime tied to the PR.

## Privacy and publication

Previews are private by default, reachable only by people with repository access. They MUST NOT become public publication unless the project deliberately opts in.

A prerelease package published to a public registry is public forever. Do not treat it as private review infrastructure. When deliberate publication is appropriate, use a prerelease tag and say in the readiness report that the package is public.

## Teardown

Preview configuration is Phase 5 cleanup. Use `completing-a-feature` to tear it down after the PR completes. A preview that outlives its PR is stale infrastructure nobody owns.

## Red flags

- The preview is a screenshot.
- The human must build it themselves.
- It exercises a different build than the PR.
- You skipped it because "the tests pass."
- It is publicly indexable and nobody deliberately decided that.

Any of these means the preview is wrong or absent. Report an absent preview honestly; do not substitute evidence of a different behavior.

## Related skills

- `reporting-readiness` — links the preview with review instructions.
- `completing-a-feature` — tears down the preview in Phase 5.
