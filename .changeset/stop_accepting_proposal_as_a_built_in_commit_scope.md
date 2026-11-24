---
default: minor
---

# Stop accepting `proposal` as a built-in commit scope

A scope names the feature area a change addresses, and `proposal` names a step of the process instead, so `workbench commits check` no longer accepts it without an entry in `.agents/commit-scopes`. Commits that draft, revise, or implement a proposal take the scope of the feature it changes, or the proposal's own `<NNNN>-<slug>` once the file exists: `0007-export-queue: propose queuing report exports`, not `proposal: add an export queue`. An open pull request with a `proposal:` commit fails the commit check once the project moves to this version; reword that commit, or list `proposal` in `.agents/commit-scopes` if proposals really are a feature of the project.
