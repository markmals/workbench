---
title: A Vision for Workbench
updated: 2026-08-25
---

# A Vision for Workbench

## Introduction

Workbench is the structure that keeps agentic development honest.

An agent that goes straight from a request to an implementation produces code nobody specified,
tests that agree with whatever got written, and documentation reverse-engineered from the result.
Everything corroborates everything else, and none of it is evidence.

Workbench breaks that loop by insisting the same intent be written down more than once, in forms
that can disagree: a proposal that records what was meant, tests that encode it executably, guides
that explain it in prose, code that implements it, and a preview that lets a human exercise it.
Disagreement between those representations is the signal. The process exists to surface it before
release rather than after.

It is for someone who works with a coding agent daily, on projects they will still be maintaining
in a year, and who has noticed that the failure mode is not bad code — it is code nobody decided
on. Workbench assumes a single human owner reviewing an agent's work, not a large team with an
existing request-for-comments process, and it assumes that human intends to remain the authority
over product intent while delegating implementation.

## Goals

**Make intent explicit before implementation begins.** The proposal exists so that what was meant
is recoverable independently of the conversation that produced it.

**Keep several independent representations of that intent.** Proposal, tests, guides, code, and
preview each describe the same behavior differently, so that a defect in one is visible against
the others.

**Make disagreement between them visible before release.** Cross-artifact review and adversarial
review exist to find contradictions, not to confirm that work looks finished.

**Preserve the reasoning, not just the result.** Proposals, decisions, policies, and review
discussion outlive the change. A future reader needs to know why, and the diff will not tell them.

**Keep the human as the final authority.** Agents implement, review, and refute. They do not
ratify their own work.

## Minimum viable product

A repository you can clone that gives a project, on day one:

- the artifact structure — vision, policies, decisions, proposals, guides — with templates that
  make the right shape the easy shape;
- skills covering each phase of the process, written so any competent coding agent can execute
  them;
- a validator that mechanically enforces the parts of the process a machine can check, wired into
  continuous integration and a git hook;
- a documentation site that publishes the vision, the record, and the guides;
- a preview mechanism appropriate to what the project is.

It must be adoptable by an agent in a single session, and it must not assume a language, a
framework, a platform, or a particular coding harness.

## Design principles

**No single artifact defines a feature.** Every representation is a check on the others. Removing
one does not simplify the process; it removes a check.

**Enforce mechanically or not at all.** A rule that lives only in prose is a rule that will be
missed. Anything expressible as a validator rule, a continuous-integration step, or a template
section belongs there instead.

**Ceremony must earn its cost.** Every step exists because skipping it produced a specific,
recurring failure. A step that cannot be justified that way gets deleted.

**Templates teach.** A template that only names its sections produces documents that only fill
them in. Each one carries enough instruction that a good document can be written from it alone.

**Previews expose behavior, not descriptions of behavior.** Review of prose is not review of
behavior, so every change is exercised through the cheapest realistic artifact that runs.

## Beyond the minimum

Directions worth considering once the core is proven:

- richer mechanical checks — proposals with no corresponding tests, guides describing behavior no
  test covers, policies no proposal has cited in a long time;
- a dependent-project workflow, where a preview published by one repository is consumed by draft
  pull requests in others and moved onto the production release automatically;
- worked examples of the process running end to end on projects of different shapes, since the
  hardest thing to convey about a process is what good output looks like;
- measurement of whether adversarial review catches what confirmatory review misses, and which
  stages earn their cost in practice.

## Non-goals

**Not a project scaffold.** Workbench does not choose your language, framework, or architecture.
Version 1 did, and the process became the small part of a large repository about stacks.

**Not tied to one coding harness.** Infrastructure ships harness-neutral in `.agents/` because
shipping configuration for harnesses nobody here verifies is worse than a documented adaptation
step.

**Not an issue tracker.** The proposal queue is the work queue, and defects that outlive a pull
request go to the project's issue tracker. Tools that track work better than a directory of
markdown files already exist, and competing with them would cost more than it returns.

**Not a substitute for judgement.** Every gate here can be satisfied superficially by an agent
determined to look done. The process makes that harder and more visible; it does not make it
impossible, and treating it as though it does would be the most expensive mistake available.

## Revision history

- **2026-08-25** — Initial vision, written alongside
  [proposal.0001](proposals/0001-workbench-2.md).
