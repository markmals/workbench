# Proposals

This directory holds the durable design record for one change per file.

Name each file `<NNNN>-<slug>.md`, using the next four-digit number and a kebab-case slug.
Required frontmatter is `id: proposal.<NNNN>`, `title`, `authors` as a non-empty list, `status`,
`pull-request`, and `supersedes`. Optional `issues` is a list of issue references or URLs.
The normal lifecycle is `draft` → `awaiting-implementation` → `active-review` → `accepted` →
`implemented`.

| Status                    | Meaning                                                                                        | Set by | Phase      |
| ------------------------- | ---------------------------------------------------------------------------------------------- | ------ | ---------- |
| `draft`                   | Being written with the human. May carry `[NEEDS CLARIFICATION:` markers.                       | agent  | 2          |
| `awaiting-implementation` | The human approved the proposal. Implementation is underway or not yet started.                | human  | 3          |
| `active-review`           | Readiness report posted, pull request marked ready. The human is evaluating the finished work. | agent  | 5          |
| `returned-for-revisions`  | The human requested changes. Back to implementation, or to the proposal if the design changed. | human  | 5 → 3 or 2 |
| `accepted`                | The human accepted the work. Vision update, merge and release are underway.                    | human  | 5          |
| `implemented`             | Merged and released.                                                                           | agent  | done       |
| `rejected`                | The human declined the change. Kept as record.                                                 | human  | terminal   |
| `withdrawn`               | The author pulled it. Kept as record.                                                          | either | terminal   |
| `superseded`              | A later proposal replaced it.                                                                  | either | terminal   |

The **Set by** column is the point. An agent may never set `awaiting-implementation` or `accepted`
— those record the human's judgement that work may begin and that finished work is good enough.
An agent that can set them is an agent that approves its own work.

History is preserved: never delete a superseded entry — mark it `superseded` and link its
replacement through `supersedes`.
