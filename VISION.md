---
title: Workbench Vision
updated: 2026-08-25
---

# Workbench Vision

## Purpose

Workbench is the structure that keeps agentic development honest.

An agent that goes straight from a request to an implementation produces code nobody specified,
tests that agree with whatever got written, and documentation reverse-engineered from the result.
Everything corroborates everything else, and none of it is evidence.

Workbench breaks that loop by insisting the same intent be written down more than once, in forms
that can disagree: a proposal that records what was meant, tests that encode it executably, guides
that explain it in prose, code that implements it, and a preview that lets a human exercise it.
Disagreement between those representations is the signal. The process exists to surface it before
release rather than after.

## Who this is for

Someone who works with a coding agent daily, on projects they will still be maintaining in a year,
and who has noticed that the failure mode is not bad code — it is code nobody decided on.

Workbench assumes a single human owner reviewing an agent's work, not a large team with an
existing RFC process. It assumes that human intends to remain the authority over product intent
while delegating implementation.

## Minimum viable product

A repository you can clone that gives a project, on day one:

- the artifact structure — vision, policies, decisions, proposals, guides — with templates that
  make the right shape the easy shape;
- skills covering each phase of the process, written so any competent coding agent can execute
  them;
- a validator that mechanically enforces the parts of the process a machine can check, wired into
  continuous integration and a git hook;
- a documentation site that publishes the guides;
- a preview mechanism appropriate to what the project is.

It must be adoptable by an agent in a single session, and it must not assume a language, a
framework, a platform, or a particular coding harness.

## Design principles

**No single artifact defines a feature.** Every representation is a check on the others. Removing
one does not simplify the process; it removes a check.

**Enforce mechanically or not at all.** A rule that lives only in prose is a rule that will be
missed. Anything expressible as a validator rule, a continuous-integration step, or a template
section belongs there instead.

**Preserve the reasoning, not just the result.** Proposals, decisions, and review discussion are
kept after a feature ships. A future reader needs to know why, and the diff will not tell them.

**Ceremony must earn its cost.** Every step exists because skipping it produced a specific,
recurring failure. A step that cannot be justified that way gets deleted.

**The human decides what is correct.** Agents implement, review, and refute. They do not ratify
their own work.

## Beyond the minimum

Directions worth exploring once the core is proven:

- richer mechanical checks — proposals with no corresponding tests, guides describing behavior no
  test covers, policies no proposal has cited in a long time;
- a real dependent-project workflow, where a preview published by one repository is consumed by
  draft pull requests in others and moved onto the production release automatically;
- worked examples of the process running end to end on projects of different shapes, since the
  hardest thing to convey about a process is what good output looks like;
- measurement — whether adversarial review actually catches what confirmatory review misses, and
  which stages earn their cost in practice.

## Non-goals

**Not a project scaffold.** Workbench does not choose your language, framework, or architecture.
Version 1 did, and the process got buried under the stack.

**Not tied to one coding harness.** Infrastructure ships harness-neutral in `.agents/`. Adapting
it is a documented one-time step, not a fork.

**Not an issue tracker.** The proposal queue is the work queue, and defects that outlive a pull
request go to the project's issue tracker. Tools that track work better than a directory of
markdown files already exist.

**Not a substitute for judgement.** Every gate here can be satisfied superficially by an agent
determined to look done. The process makes that harder and more visible. It does not make it
impossible.
