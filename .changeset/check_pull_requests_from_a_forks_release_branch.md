---
default: patch
---

# Check pull requests from a fork's release branch

The `.github/workflows/workbench.yml` that `workbench project init` generates skipped the commit and change-file checks for any pull request whose branch was named `release`, including one from a fork. It now skips them only for a `release` branch in the same repository, where Knope opens its release pull request. A project that already has the file should change the job's `if:` to `github.head_ref != 'release' || github.event.pull_request.head.repo.full_name != github.repository`.
