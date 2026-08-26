# Proposals

This directory holds the durable design record for one change per file.

Name each file `<NNNN>-<slug>.md`, using the next four-digit number and a kebab-case slug.
Required frontmatter is `id: proposal.<NNNN>`, `title`, `authors` as a non-empty list, `status`,
`pull-request`, and `supersedes`. Optional `issues` is a list of issue references or URLs.
The lifecycle is `draft` → `accepted` → `implemented`. A proposal may instead be `rejected` or
`withdrawn`; a replaced proposal is `superseded`.
History is preserved: never delete a superseded entry — mark it `superseded` and link its replacement through `supersedes`.
