---
id: guide.record-validation
title: Record Validation and Workflow Status
---

# Record Validation and Workflow Status

`mise run validate` checks the current record metadata. `mise run status` identifies the active proposal and the next actor. This page is their current public contract; the [process](../PROCESS.md) defines the surrounding workflow.

## Validate the record

Run `mise run validate` from the repository root. Success prints `Validation passed: 0 violations` and exits zero. A failure identifies the file and problem, prints the violation count, and exits nonzero. `mise run validate -- --json` returns a JSON object with `violations` (file paths and messages) and `count`.

The validator reads `VISION.md` and Markdown files directly inside `proposals/`, `policies/`, `decisions/`, and `guides/`. Directory READMEs are indexes, not records. Empty or absent optional record directories are allowed. Nested Markdown records are rejected. Symlinked regular files are checked; symlinked directories are not traversed.

| Record | Required frontmatter | Allowed status |
| --- | --- | --- |
| Vision | `title`, `updated` as `YYYY-MM-DD` | None |
| Proposal | `id`, `title`, non-empty `authors` list, `status`, `pull-request`, `supersedes` list | `draft`, `awaiting-implementation`, `active-review`, `returned-for-revisions`, `accepted`, `implemented`, `rejected`, `withdrawn`, `superseded` |
| Policy | `id`, `title`, `status` | `draft`, `active` |
| Decision | `id`, `title`, `status` | `proposed`, `accepted` |
| Guide | `id`, `title` | None |

Proposal, policy, and decision filenames use `<NNNN>-<slug>.md`, with IDs such as `policy.0001`. Guides use kebab-case filenames and IDs such as `guide.record-validation`. IDs must match their filenames and be unique. Frontmatter supports scalar values and inline lists; duplicate keys and malformed lists are errors. Proposal `issues`, when present, is an inline list.

Policies and decisions may carry `established-by: proposal.<NNNN>` as historical provenance. It must be a scalar reference to an existing proposal. It remains optional when a record changes in place. A current policy or decision cannot contain `supersedes` or use `status: superseded`; the error explains how to revise or retire it instead. These fields still describe historical relationships between proposals. A replacement proposal names its predecessor in `supersedes`, that predecessor is marked `superseded`, and the relationship must not form a cycle.

Guides use ordinary links to their current sources. The obsolete `describes` field is rejected with migration guidance. The validator checks metadata references; the documentation build checks local Markdown links. Run the full `mise run check` before publishing a retirement, because metadata validation alone cannot detect a broken prose link.

An unresolved clarification marker in proposal prose blocks `awaiting-implementation`, `active-review`, `accepted`, `implemented`, and `superseded`. Draft, returned, rejected, and withdrawn proposals may contain unresolved questions. Inline code and fenced examples of the marker are ignored.

## Revise and retire a policy

A current policy can start with:

```yaml
---
id: policy.0001
title: Preserve User Focus
status: active
---
```

Revise its rule and applicable checks in the same change, keeping its logical identity. No replacement policy or supersession entry is needed. When the policy ceases to apply, repair its current links and remove the file. Preserve any historical link that still matters by pointing it at the Git revision that contains the old policy. See [Keeping the Record](./keeping-the-record.md) for the human decision and migration procedure.

## Find the next actor

Run `mise run status` with no arguments. It reads the branch, pull request, and active proposal, then reports the phase, situation, next action, and whether the agent or human acts next. `mise run status -- --json` provides the same facts and derived guidance as JSON.

An approved proposal routes to updating current specifications, then failing tests, guides, implementation, and review. The human still owns approval to implement and acceptance of the result. `implemented` records accepted implementation before merge; it does not prove integration or release. A closed, unmerged PR with accepted work requires human reconciliation rather than being treated as delivered.

Missing or unavailable GitHub state is reported as unknown rather than assumed to mean that no pull request exists. Ambiguous proposal selection, unresolved questions, unpushed commits, and disagreement between PR draft state and proposal review state appear as notices. See [Running a Feature](./running-a-feature.md) for the phase-by-phase process.

## What these commands do not establish

The checks cannot decide whether a specification states the right behavior, a test asserts enough, or a policy is still needed. Review those claims against the current requirements and human-approved intent. Gherkin bindings, native schemas, API examples, and platform checks use the project's own tools. Workbench introduces no universal specification registry or mandatory Gherkin runner.
