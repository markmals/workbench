---
id: proposal.0002
title: Guides for Workbench
authors: [markmals, Claude]
status: draft
pull-request: https://github.com/markmals/workbench/pull/2
issues: []
supersedes: []
---

# Guides for Workbench

## Summary

Writes Workbench's own user-facing guides, which it currently ships none of despite treating
guides as a load-bearing artifact. Adds a check that every command and internal link a guide shows
actually resolves, so the documentation cannot rot silently.

## Motivation

Workbench's central claim is that a feature is defined by several independent representations, and
that guides are one of them. It ships zero guides. A kit that mandates a discipline it does not
practise teaches the discipline is optional.

The gap is not merely embarrassing. The three documents that exist serve different readers and
none of them is a guide:

| Document      | Answers                             | Reader                       |
| ------------- | ----------------------------------- | ---------------------------- |
| `README.md`   | Should I use this?                  | Someone evaluating the kit   |
| `PROCESS.md`  | What is the process?                | Someone who wants the theory |
| `ADOPTING.md` | How do I wire this into my harness? | Someone who just cloned it   |

Nothing answers _how do I actually run a feature through this?_ — which is the only question a
user has after the first hour. Today they would read `AGENTS.md`, a contract written for an agent,
and infer their own role from it.

There is a second reason, named in proposal 0001: this is the first change small enough to run end
to end through the process without the bootstrapping excuse. Proposal 0001 could not follow the
process it defined. This one can, and whether it is pleasant to do is information worth having.

## Proposed solution

Six task-oriented guides under `guides/`, each answering one question a user actually asks, plus a
check that keeps their worked examples honest.

Guides describe what the human does and what the agent does in response. They are not a second
copy of `AGENTS.md` written in the second person — where the agent's behavior matters to the
human, the guide says what to expect, not how the agent achieves it.

`ADOPTING.md` becomes `guides/adapting-your-harness.md`, since it is already a guide wearing a
root-level filename.

## Detailed design

**The six guides.**

| Guide                        | Answers                                                                  |
| ---------------------------- | ------------------------------------------------------------------------ |
| `getting-started.md`         | I just cloned this. What do I do?                                        |
| `running-a-feature.md`       | What actually happens, from idea to merged, and what is my part?         |
| `writing-a-good-proposal.md` | The template gives me sections. What makes the content worth the effort? |
| `choosing-a-preview.md`      | What do I wire up so I can exercise a change?                            |
| `keeping-the-record.md`      | When does something become a policy, a decision, or neither?             |
| `adapting-your-harness.md`   | How do I wire this into the agent I actually use?                        |

Each carries `id: guide.<slug>`, a `title`, and `describes` naming the proposals it explains. Each
opens with the question it answers, so a reader scanning the sidebar can route themselves.

**`running-a-feature.md` is the load-bearing one.** It walks a single small feature from request
to merge, showing what the human types, what `mise run status` reports at each step, who moves the
proposal's status and when, and what arrives for review. It is the guide that replaces reading
`AGENTS.md` to work out your own role.

**The guides check.** A new `tools/check-guides.mjs`, dependency-free Node, run by `mise run check`:

- Every internal markdown link in a guide resolves to a file that exists, and to an anchor that
  exists when one is given.
- Every `mise run <task>` shown in a fenced block names a task that exists in `mise.toml`.
- Every path referenced in prose using backticks and looking like a repository path
  (`.agents/...`, `tools/...`, `guides/...`) exists.

This makes Tier 0 what `writing-guides` currently only asserts in prose: that every worked example
actually runs. The enforcement hierarchy requires exactly this conversion where it is possible.

**Pure logic separated from I/O**, matching `tools/validate.mjs`, so the rules are testable against
in-memory guide content without a filesystem.

**`README.md` keeps its role** as the evaluation document and gains a short link list into the
guides. It does not become a table of contents.

## Compatibility

No behavior changes. `ADOPTING.md` moves, so its inbound links must move with it — `README.md`,
`AGENTS.md`, and `.agents/hooks/README.md` reference it today. Anyone who bookmarked the old path
loses it; the repository is a template consumed by copying, so there is no meaningful installed
base to break.

## Implications on adoption

Adopters gain a worked example of the guides discipline, which is the hardest part of the process
to convey. They also inherit six guides describing Workbench rather than their own project, so the
adoption step must say to delete them — the same instruction that already covers the record.

The guides check runs against whatever guides a project writes, and will fail on a broken link or
a `mise run` for a task the project has not defined. That is the intended behavior and it needs
saying, because the first failure will look like the kit being broken.

## Scope

- Six guides under `guides/`, with `ADOPTING.md` moved and its inbound links updated.
- `tools/check-guides.mjs` and its tests, wired into `mise run check`.
- A link list in `README.md`.
- An adoption instruction to delete Workbench's guides.

### Out of scope

- **Rewriting `PROCESS.md` or `README.md`.** They serve readers the guides do not. `PROCESS.md`
  in particular is the authored source of truth and is not a guide.
- **Guides for the record artifacts themselves.** `proposals/README.md` and its siblings already
  explain their directories to anyone standing in them.
- **A tutorial project.** A worked example on a real second repository would teach more than any
  guide, and is far larger than this.

## Preview

The deployed documentation site, which is already wired: `.github/workflows/preview.yml` publishes
it per pull request and comments the URL.

This is the right preview and an unusually direct one — the guides _are_ the deployed artifact, so
reviewing the preview is reviewing the product rather than a proxy for it. Read them in the
sidebar, in the order a new user would meet them.

## Policies and decisions checked

`policies/` and `decisions/` are both empty; both directories were read. Proposal 0001 declined to
promote anything into them, so there is still nothing to check against.

Two rules in `.agents/rules/` do constrain this work:

- `enforcement-hierarchy.md` — requires attempting a check before writing a rule as prose. The
  guides check is that attempt applied to `writing-guides`'s worked-example rule.
- `code-quality.md` — the check separates pure logic from I/O for the same reason the validator
  does.

This proposal may be the one that justifies the first real policy: _every worked example in a
guide must be executable and checked._ Recording it is deferred until the check exists and has
survived contact with actual guides.

## Future directions

- Extending the guides check to execute fenced commands in a sandbox rather than merely resolving
  their task names, which would close the gap between "the command exists" and "the command works".
- A guide covering the dependent-project workflow, once that workflow exists.
- Screenshots or a recorded session for `running-a-feature.md`, if prose turns out to be a poor
  medium for showing an interactive loop.

## Alternatives considered

**Fold the guides into proposal 0001.** Closes the gap one pull request sooner and avoids a
stacked branch. Rejected because 0001 is already a whole-repository rewrite and adding a
documentation set makes an over-large pull request harder to review — and because 0001 explicitly
predicted this would be its own proposal, so amending it would discard a decision the record
already made deliberately.

**Write one long guide instead of six.** Fewer files, and a linear read. Rejected because the
questions are asked at genuinely different moments: adapting a harness happens once, choosing a
preview happens per project, and running a feature happens constantly. A single document forces
every reader through all of it.

**Skip the guides check.** The guides are prose, and a link checker is not free. Rejected because
`writing-guides` already asserts that every worked example must run, and the enforcement hierarchy
says a rule that can be a check must be one. A documentation set with no check is the artifact
most likely to rot, since nothing else fails when it does.

**Leave `ADOPTING.md` at the repository root.** It is discoverable there and adopters are told to
read it before anything else. Rejected because it is already a guide by the kit's own definition,
and keeping it outside `guides/` would mean the one document that best demonstrates the form is
excluded from the set.

## Open questions

- [NEEDS CLARIFICATION: Should `ADOPTING.md` move into `guides/` as proposed, or stay at the root
  as the one document a fresh cloner is told to read before the site is even running? Moving it
  is more consistent; keeping it is more discoverable at the moment of first contact.]
- [NEEDS CLARIFICATION: Six guides is a judgement, not a derivation. Which of them do you actually
  want — and is `writing-a-good-proposal.md` redundant now that `.agents/templates/PROPOSAL.md`
  carries instructional prose in every section?]
