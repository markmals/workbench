# Proposals

This directory holds one change record per file: what the change was, why it was made, and how it was reviewed. A completed proposal is history. Current requirements live in the specifications, policies, and decisions it changed.

Name each file `<NNNN>-<slug>.md`, using the next four-digit number and a kebab-case slug. Required frontmatter is `id: proposal.<NNNN>`, `title`, `authors` as a non-empty list, `status`, `pull-request`, and `supersedes`. Optional `issues` is a list of issue references or URLs. The normal lifecycle is `draft` → `awaiting-implementation` → `active-review` → `accepted` → `implemented`.

| Status                    | Meaning                                                                                                                                                                        | Decided by | Phase      |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------- | ---------- |
| `draft`                   | Being written and iterated on in the draft pull request. May carry `[NEEDS CLARIFICATION:`.                                                                                    | agent      | 2          |
| `awaiting-implementation` | The human settled the proposal. Implementation is underway or not yet started; the feature pull request becomes ready once the complete implementation is verified and pushed. | human      | 3          |
| `active-review`           | The readiness report is posted after Phase 3 reviews and preview; the pull request was made ready earlier.                                                                     | agent      | 4          |
| `returned-for-revisions`  | The human requested changes. Back to implementation, or to the proposal if the design changed.                                                                                 | human      | 4 → 3 or 2 |
| `accepted`                | The human accepted the work. Vision update, merge and release are underway.                                                                                                    | human      | 5          |
| `implemented`             | The accepted implementation and final records are committed in the feature pull request before merge. Merge and any required release remain separate.                          | agent      | 5          |
| `rejected`                | The human declined the change. Kept as record.                                                                                                                                 | human      | terminal   |
| `withdrawn`               | The author pulled it. Kept as record.                                                                                                                                          | either     | terminal   |
| `superseded`              | A later proposal replaced it.                                                                                                                                                  | either     | terminal   |

**Decided by** is about judgement, not keystrokes. The human may ask the agent to make the edit. What an agent must never do is decide on its own that a proposal is settled enough to implement, or that finished work is good enough to accept.

Proposals are kept after they close. When a later proposal replaces an earlier one as a change, it names it in `supersedes` and the earlier one becomes `superseded`. That relationship records how the changes were made; it does not say which requirements apply now.
