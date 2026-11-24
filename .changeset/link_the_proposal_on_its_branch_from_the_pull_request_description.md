---
default: minor
---

# Link the proposal on its branch from the pull request description

Once a proposal is pushed, the agent puts a link to it at the top of the pull request description, at `https://github.com/<owner>/<repo>/blob/<branch>/proposals/<NNNN>-<slug>.md`. The link opens the rendered proposal on the feature branch rather than at one commit, so refreshing it shows each revision the agent pushes. The readiness report links the proposal the same way, and after the merge the agent repoints the description's link to the default branch.
