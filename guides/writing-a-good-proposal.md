---
id: guide.writing-a-good-proposal
title: Writing a Good Proposal
describes: [proposal.0001]
---

# Writing a Good Proposal

**The template gives me sections. What makes the content worth the effort?**

The `.agents/templates/PROPOSAL.md` teaches structure: what each
section is for. This guide is about judgement: what makes a proposal make the next three
days obvious instead of adding ceremony.

## Spend the hour where change is cheap

A proposal is the highest-leverage artifact in a feature. It is the one place where changing
your mind is cheap. Tests, guides, and code derive from it; an hour spent making intent clear
here can save a day spent reconciling them later.

It also outlives the pull-request conversation. Once the feature ships, the proposal is the
explanation of _why_ the project changed. Write for the person who finds it then, not for the
agent that heard the original request.

[Proposal 0001](../proposals/0001-workbench-2.md) does this in its Motivation: it names the
three failures of Workbench 1.0 and the workaround people use today. Its solution can be
reconsidered without changing the problem those facts establish.

## Describe the problem before the fix

Motivation explains who has a problem, what it costs, the evidence that it exists, and the
current workaround. It does not announce the feature you want to build.

Use this test: would the Motivation still read correctly if somebody proposed a completely
different solution? If not, it is probably a design pitch wearing a problem statement. Move the
pitch to the proposal's solution and design.

This distinction makes review useful. Readers can agree that a problem matters while disagreeing
about the answer, rather than having to reject both at once.

## Make decisions implementation-ready

Detailed design is the source from which tests derive. It must specify observable behavior,
interfaces, failures, boundaries, data, and operational constraints precisely enough that
someone who is not the author can implement it without asking questions.

That is the Swift Evolution bar the template borrows. A sentence such as “make validation
robust” does not meet it; it leaves a later person to invent what invalid input means, what the
user observes, and which behavior wins at a boundary.

Write the choices you expect the tests to prove. If two reasonable implementations would expose
different behavior and the proposal does not choose, the design is incomplete. Raise the question
with the human rather than allowing the agent to make an unreviewed decision.

## Do not create a second specification

Acceptance criteria are usually a mistake. Detailed design is already the specification and the
source of the tests. Repeating it as a checklist creates a second version that will drift.

The current [proposal 0001](../proposals/0001-workbench-2.md) deletes Acceptance criteria: after
Detailed design, it continues directly to Compatibility. That is the right default.

Keep the section only when the change has genuinely discrete outcomes that Detailed design cannot
express naturally — for example, a structural change with many independent deliverables or a
migration with separable steps. If every checkbox paraphrases a design sentence, delete it.

## Scope by intent, not by available time

Out of scope is load-bearing. Name work you are deliberately not doing and why it does not belong
in this change. A future reader should be able to see the boundary and decide whether it still
makes sense.

Do not use it to excuse work that the proposal requires but you have not done. An earlier
[revision of proposal 0001](https://github.com/markmals/workbench/blob/e5e3315b7321/proposals/0001-workbench-2.md#L129-L132)
excluded Workbench's own guides as “its own proposal.” That was an omission wearing a scope
decision, not a defensible boundary. The exclusion was removed, and the current proposal puts
writing the guides in Scope.

Use this test: would the exclusion still look correct to someone who does not know what you had
time for? If not, put the necessary work in scope or reduce the proposed change.

A proposal should be one coherent change. Two unrelated motivations, or a Scope section long
enough to need its own summary, means you should split it before detail disguises the problem.

## Answer the objection you expect

Alternatives considered is where you engage seriously with the approach a reader may already
prefer. State the strongest credible version of each alternative, then explain why it loses for
this change. You need not be neutral, but you must be fair.

Keep the section alive during review. Add alternatives that reviewers raise. If an alternative
wins, revise the proposal to use it and record why it won; do not preserve the original plan out
of attachment to the draft.

Proposal 0001 shows the standard: its Alternatives considered weighs retaining the multiplatform
layer, splitting repositories, and shipping several harness configurations against the chosen
process kit. Each rejection names a cost, not a slogan.

## Mark questions; never fill them with guesses

Use an inline marker for every unresolved decision:

```text
[NEEDS CLARIFICATION: <question>]
```

A guessed answer becomes a silent decision nobody reviewed. Ask the human, record the resolution,
and remove the marker. The validator rejects markers once a proposal leaves `draft`, except when
it is `returned-for-revisions`, `rejected`, or `withdrawn`.

## Publish when it can be challenged

Push the proposal to its draft pull request as soon as it is coherent enough to react to. The
human can then challenge the problem, scope, and design while revisions are still cheap.

Do not polish it privately into apparent certainty. A proposal held back until it looks finished
is a proposal the agent decided alone.

For the full path from proposal to release, see [Running a Feature](./running-a-feature.md).

## Red flags

| Symptom                                                   | What it means                                | What to do                                                                            |
| --------------------------------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------- |
| Motivation begins with “Add” or “Build”                   | The fix is masquerading as the problem.      | Restate the harm, affected people, evidence, and workaround without naming an answer. |
| A design sentence leaves two observable outcomes possible | A later implementer must invent a decision.  | Choose the behavior or add a clarification marker.                                    |
| Acceptance checkboxes repeat Detailed design              | There are two specifications that can drift. | Delete Acceptance criteria.                                                           |
| “Out of scope” means “not finished”                       | The proposal hides required work.            | Put the work in scope or reduce the change honestly.                                  |
| Two motivations need separate explanations                | The proposal is really two changes.          | Split it into coherent proposals.                                                     |
| An alternative is a weak caricature                       | The reader's real objection is unaddressed.  | State the strongest version and explain the trade-off.                                |
| The draft is being polished in private                    | The agent is deciding intent alone.          | Publish it for review now.                                                            |
