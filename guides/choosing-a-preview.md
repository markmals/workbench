---
id: guide.choosing-a-preview
title: Choosing a Preview
describes: [proposal.0001]
---

# Choosing a Preview

## What do I wire up so I can exercise a change?

Wire the cheapest realistic artifact through which you can exercise the changed
behavior. This is the preview stage of [running a feature](./running-a-feature.md).

Both words carry weight:

- **Cheapest** rules out production-grade infrastructure merely to review a pull
  request.
- **Realistic** rules out screenshots, transcripts, and descriptions of what would
  happen.

Reviewing prose is not reviewing behavior. The preview exists so you can use the
change before deciding whether to accept it.

## Match the repository shape

Choose the surface the proposal changes. The agent wires the matching preview; you
exercise it from the pull request before review.

| Repository kind       | Preview                     | What you exercise                                         |
| --------------------- | --------------------------- | --------------------------------------------------------- |
| Library               | Prerelease package          | Install it in a dependent project.                        |
| Web app               | Preview deployment          | Click through the changed flow.                           |
| API                   | Preview endpoint            | Make real requests.                                       |
| Documentation         | Deployed documentation site | Read the changed documentation.                           |
| CLI                   | Installable executable      | Install it and run the changed command.                   |
| Desktop or mobile app | Installable build           | Install it on a real device and use the changed behavior. |

A mixed repository previews the surface the proposal changed, not every repository
shape by habit. A proposal that changes an API does not need a documentation preview
as well.

A pure refactor with no exercisable surface has no preview. Say that in the readiness
report rather than fabricating one.

## A build artifact is not a preview

**A downloadable build artifact is not a preview.** Downloading a zip, unzipping it,
and starting a local server is not an act a reviewer will routinely take. The change
then gets reviewed as prose instead of behavior.

This repository made exactly that mistake. Workbench first shipped a downloadable
documentation artifact; it now deploys the documentation site to a URL for each pull
request.

A CLI is the exception. Installing its binary is the real act of exercising the
program, so an installable executable is a preview for a CLI.

## Wire one recipe

`.github/workflows/preview.yml` ships a
working documentation deployment because Workbench is documentation and process.
It also contains commented recipes for the other repository shapes.

1. Choose the row that matches your project and the changed surface.
2. Wire that recipe in the workflow.
3. Delete the recipes that do not apply.
4. Open the preview URL from the pull request and exercise the change.

For the shipped documentation recipe, enable GitHub Pages with `gh-pages` as its
source. The workflow builds every open pull request, deploys beneath
`pr-preview/pr-<number>/`, comments the preview URL on the pull request, and removes
the deployment and comment when the pull request closes.

Do not keep a menu of inactive preview mechanisms as configuration. The comments are
examples for adoption; your workflow should express one maintained choice.

## The published site is separate from previews

Previews answer "does this change work." A published documentation site answers "where do I read
the guides." They are different jobs, and Workbench wires them in different workflows —
`preview.yml` on pull requests, `docs.yml` on the default branch.

Both write to `gh-pages`: the published site at the root, previews beneath `pr-preview/`. The
production deploy excludes that directory when it cleans, so publishing does not delete the
previews of open pull requests.

**`docs.yml` skips itself on a private repository.** Publishing guides is the one place the
private-by-default rule can be broken by accident, so the job is gated rather than trusting
whoever adopts the kit to remember. If your repository is public and your guides should not be,
delete that workflow.

## Keep previews private by default

A preview is private unless you deliberately decide otherwise. A public preview can
expose unfinished behavior, test data, or guides that the repository does not expose.

GitHub Pages cannot serve privately outside GitHub Enterprise. If your guides must
stay private, do not use this repository's GitHub Pages default. Use the Cloudflare
Pages recipe in the workflow instead and gate it behind Cloudflare Access. It requires
these repository secrets:

| Secret                  | Used for                                             |
| ----------------------- | ---------------------------------------------------- |
| `CLOUDFLARE_API_TOKEN`  | Authorizing the Cloudflare Pages deployment.         |
| `CLOUDFLARE_ACCOUNT_ID` | Selecting the Cloudflare account for the deployment. |

A prerelease published to a public package registry is public forever. Treat that as
a publication decision, not as temporary review infrastructure.

## Verify a subpath deployment

A site served beneath a subpath must receive that base path at build time. Otherwise
its assets resolve against the domain root and the page loads blank.

The shipped workflow sets `DOCS_BASE` to the preview path, and
`docs/.vitepress/config.ts` reads it at build time.
Workbench encountered this bug twice: once for assets and once for homepage links.

Open the deployed preview. Confirm that the page renders and that the homepage links
work. A green workflow is not evidence that a subpath deployment is usable.

## Remove it when the work ends

Teardown belongs to completion. Remove a preview when its pull request closes or the
feature is otherwise complete. A preview that outlives its pull request is stale
infrastructure nobody owns.

## Red flags

| Red flag                                   | Correction                                                                 |
| ------------------------------------------ | -------------------------------------------------------------------------- |
| “The artifact uploaded successfully.”      | Deploy or install the artifact in the way a reviewer will actually use it. |
| “The workflow is green.”                   | Open the preview and exercise the changed behavior.                        |
| “We can reuse the production environment.” | Choose the cheapest isolated review surface instead.                       |
| “Every repository needs every preview.”    | Preview only the surface changed by the proposal.                          |
| “We will leave it up for later.”           | Teardown the preview at completion.                                        |
