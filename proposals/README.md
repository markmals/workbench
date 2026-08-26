# Proposals

This directory holds the durable design record for one change per file.

Name each file `<NNNN>-<slug>.md`, using the next four-digit number and a kebab-case slug.
Required frontmatter is `id: proposal.<NNNN>`, `title`, `status`, `pull-request`, and `supersedes`.
The lifecycle is `draft` → `accepted` → `implemented`; a replaced proposal is `superseded`.
History is preserved: never delete a superseded entry — mark it `superseded` and link its replacement through `supersedes`.
