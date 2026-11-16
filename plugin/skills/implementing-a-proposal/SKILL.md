---
name: implementing-a-proposal
description: Use when an approved proposal is ready for Phase 3 implementation, including a substantive bug fix whose root cause is known. Do not use for proposal authoring or trivial inline edits.
---

# Implementing a Proposal

Phase 3 writes an approved change into the project's current specifications, then turns them into tests, guides, and code that agree with each other.

**Hard order when stages apply:** current specifications → failing tests → guides → code → quality gates.

## Preconditions

- The proposal has `status: awaiting-implementation`, or the human returned it with `status: returned-for-revisions`; neither has an unresolved clarification marker.
- Phase 1 — preparation — created the branch, pushed it to `origin`, and opened a draft pull request for the proposal. It stays draft until the complete implementation has passed applicable checks and reached the pushed branch.
- Read the proposal's **Detailed design**, the current sources it links, the code it touches, and the vision, policies, and decisions it names. Completed proposals are history; read current requirements from the sources that own them.
- When the design is ambiguous, or current sources disagree with each other or with the proposal in a way that changes behavior, raise it. Do not choose silently or expand scope.

## Who does the work

Implement the proposal yourself by default. A single agent holding the whole change makes fewer handoff errors than a strategist briefing implementers.

Split the work only into slices that are independent, touch separate files, and save real time by running in parallel. Give each subagent the behavior, the current sources it touches, the relevant Detailed design, constraints, scope, and verification commands. Integrate and commit its work yourself. When a subagent reports that the evidence contradicts the proposal's model, stop that slice and use `reframing-a-problem`; settle intent with the human before resuming.

## 1. Current specifications

Before any test or production code, revise each current source the proposal changes so it states the approved end state. The source that owns a surface today stays its owner:

| Surface                   | What you revise                                                                                                                                                                                                               |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Library API               | The declaration and its contract documentation (doc comment, or the reference the package publishes): defaults, errors, side effects, ordering, lifecycle, compatibility. Edit the doc comment now, before the function body. |
| Protocol or stored data   | The native schema and the semantic requirements beside it.                                                                                                                                                                    |
| Application workflow      | Its feature file or behavioral reference. For `.feature` files, use the `specifying-behavior-with-gherkin` skill.                                                                                                             |
| Process or tooling        | The reference prose that owns it.                                                                                                                                                                                             |
| Agreed policy or decision | The same file, keeping its ID, or its deletion when the change retires it (`completing-a-feature` skill, §2).                                                                                                                 |

Reuse each source's representation. Add a source only for a requirement with no adequate home, and keep it as small as the requirement allows. When another document restates the contract, such as a README repeating the API reference, revise it in the same edit or replace the restatement with a link, so each fact keeps one owner. Write only what the proposal authorizes; a new substantive decision goes to the human. Never write a specification from what the code happens to do.

## 2. Failing tests

```text
NO TEST-WORTHY PRODUCTION BEHAVIOR WITHOUT A FAILING TEST FIRST
```

- Write one focused test per observable behavior the revised specification requires, named after that behavior. Tests verify the current contract; the proposal explains why the contract changed, so its id does not belong in test names.
- Keep existing native tests that still apply. Change one only where the contract it checks changed.
- When the surface uses Gherkin, run the changed scenarios through the project's real integration; the Gherkin skill covers bindings. A test that only confirms a document exists, or that a test carries an identifier, does not test behavior.
- Run each new test. Confirm it fails because the behavior is missing, not because of setup, an import, or a typo. If it passes, correct the test.
- If you wrote test-worthy production code first, delete it and start from the test.
- A claim that something holds "always", "never", "for any", or "for all" gets a property-based test that fails first, alongside example tests for named boundaries. Do not force properties onto a single mapping or constant.
- Test real code. Isolate only what you cannot control, such as the network, time, or randomness. When a test is hard to write, simplify the interface rather than adding mocks.
- For an interface, assert what the user can do and observe — reaching an action, seeing a state, being told about an error — not its look and feel, so polish does not break the tests.
- When nothing can be meaningfully tested, say so and why before writing code.

## 3. Guides

Write a guide only when the change alters what a user does, sees, or must know. Write it from the intended contract before production code exists, for a reader who will never read the proposal: inputs, prerequisites, actions, outcomes, errors, defaults, and durable effects, not internals.

Extend the guide in `guides/` that already owns the workflow; add one from the guide template only when none does: `.agents/templates/GUIDE.md` when the project overrides it, otherwise `.workbench/templates/GUIDE.md`. Its frontmatter is `id`, matching the path, and an accurate `title`. Link the current source that states the precise contract with an ordinary Markdown link rather than restating it. Every example must run as printed.

What the user does comes from the specification. The names and placement of on-screen elements a guide mentions follow the running UI, and you update them when polish changes them; that is not writing a guide from finished code.

Then compare the guide with the failing tests and the specification. Anything one states and another does not is a defect in one of them; resolve it by correcting the guide, test, or specification within the approved change, or by revising the proposal with the human.

## 4. Code

Write the minimum production code that makes the tests pass and matches the specification and guide. Add no behavior the specification does not state. Look and feel the proposal leaves open is yours to choose, fitting the project's existing interface and its platform's conventions, and the human's to refine with the `polishing-a-ui` skill. When an unexplained failure appears, find its cause with the `systematic-debugging` skill before changing code.

## 5. Gates and commits

Run `mise run check`, then any project-specific gate `mise tasks` lists that it does not already cover, and read the output; `.workbench/rules/verification.md` says what counts as evidence. Never commit while red. Commit at green boundaries following `.workbench/rules/commit-discipline.md`, and push.

After the complete implementation passes the applicable checks and is pushed, wait for CI to pass on the pushed revision (`gh pr checks <pr-number> --watch`); no git hook runs the gate, so a local run does not cover the pushed revision. When CI fails, fix the failure, push again, and wait again. Then mark the feature pull request ready for maintainer review. Human review can begin while the self-check, independent review, and readiness report remain in progress. Keep the proposal at `awaiting-implementation` until the handoff. If the push, CI, or PR state change cannot be completed, report the blocker without claiming the implementation is reviewable in the PR.

## Then

Continue with the `reviewing-an-implementation` skill. It owns the self-check, the independent review, the preview, and the handoff to the human. Set no proposal status before then. The pull request should already be ready.
