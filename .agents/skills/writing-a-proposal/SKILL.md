---
name: writing-a-proposal
description: Use when a new or substantial change needs definition before tests, guides, or code — especially when intent is unclear, designs compete, existing records may constrain the change, or human approval is required.
---

# Writing a Proposal

A proposal is the one durable record of one coherent change. It makes intent concrete enough to derive tests, guides, and code without returning to the original conversation.

## Open only after Preparation

Confirm that the change has a branch pushed to `origin` and an open draft pull request. The pull request is the durable workspace.

If either is absent, stop. Complete **Preparation** before asking discovery questions or creating a proposal. An urgent request, existing code, or a request to skip pull-request work does not create an exception.

## Output

Create exactly one `proposals/<NNNN>-<slug>.md`. Copy [`.agents/templates/PROPOSAL.md`](../../templates/PROPOSAL.md); do not replace it with a folder or auxiliary planning files.

- Use the next unused four-digit number and a kebab-case slug.
- Complete the template frontmatter: `id: proposal.<NNNN>`, Title Case `title`, `status: draft`, the draft PR URL or number, and `supersedes`.
- Complete every template section: Motivation, Intended behavior, Acceptance criteria, Scope, Out of scope, Design decisions and rejected alternatives, Preview, Policies and decisions checked, and Open questions.
- Write the Acceptance criteria section with `writing-acceptance-criteria`.

## Size before detail

One proposal covers one coherent change. Split before authoring when either is true:

- The motivation solves two unrelated problems.
- The change needs more than roughly six acceptance criteria.

Choose the smallest independently useful slice. A proposal with a broad motivation is not made coherent by a long scope section.

## Process

1. **Read the record.** Read `VISION.md`, `policies/`, and `decisions/` before proposing. In **Policies and decisions checked**, list each relevant record and its constraint; write `None` when none apply.

    If the change conflicts with a policy or decision, surface the conflict in that section. The human chooses either to revise the proposal or to supersede the record. Never quietly violate it.

2. **Discover one answer at a time.** Ask one question, wait for its answer, then ask the next. Use multiple-choice questions when the answer space is finite. Cover:
    - who needs the change and the outcome they seek;
    - the trigger, success condition, and boundaries;
    - dependencies, constraints, and affected records; and
    - what is deliberately out of scope.

    Do not guess. Ask, or record `[NEEDS CLARIFICATION: <question>]` in **Open questions** and continue.

3. **Compare approaches.** Present two or three viable approaches with tradeoffs. Recommend one and explain why. Ask the human to select an approach or direct a revision before treating the design as settled.

4. **Choose the preview now.** Select exactly one artifact through which the human will exercise the changed behavior. Record the repository shape, artifact, private access path, and completion cleanup in **Preview**.

    | Repository shape | Preview artifact                            |
    | ---------------- | ------------------------------------------- |
    | Library          | prerelease package                          |
    | Web app          | preview deployment                          |
    | API              | preview endpoint deployment                 |
    | Docs             | rendered documentation build                |
    | CLI              | installable executable                      |
    | Desktop          | installable build                           |
    | Mobile           | installable build (ad-hoc / internal track) |

    Keep the preview private unless the project deliberately opts into publication. Do not configure unused preview mechanisms.

5. **Author the proposal.** Fill the template from the answers and selected approach. State behavior and boundaries plainly. Record rejected approaches and why they lost. Keep unresolved questions as markers rather than inventing answers.

6. **File unrelated defects.** If discovery reveals a defect outside this proposal's scope, file it as a GitHub issue. Discovering a defect is never a reason to expand the proposal or forget it. Use [`.agents/skills/tracking-defects/`](../tracking-defects/) for the judgment and filing process.

7. **Self-review.** Before showing the human, check:
    - every template section and frontmatter field is complete;
    - the motivation, behavior, criteria, scope, preview, and record check agree;
    - no unstated assumption remains outside an explicit clarification marker;
    - no rejected approach or out-of-scope item reappears as promised behavior; and
    - the proposal remains one coherent change.

    Fix clear defects. Count remaining clarification markers and present them to the human.

8. **Get human approval.** Iterate on the proposal until the human explicitly approves it by setting `status: accepted`. Do not set that status yourself.

## Hard gate

Never write tests, guides, or code until all of these are true:

1. The proposal file exists.
2. It contains zero `[NEEDS CLARIFICATION:` markers.
3. The human has set `status: accepted`.

After the gate, enter **Implementation** with `implementing-a-proposal`: failing tests → guides → code → the project's quality gates from `mise tasks`.

## Red flags — stop and correct

| Symptom                                                     | Required response                                                   |
| ----------------------------------------------------------- | ------------------------------------------------------------------- |
| No pushed branch or draft pull request                      | Return to **Preparation**.                                          |
| Two unrelated motivations or more than roughly six criteria | Split the proposal.                                                 |
| A record conflicts with the change                          | Surface it; the human chooses revision or supersession.             |
| An unknown detail has no clarification marker               | Ask or add the marker.                                              |
| Preview names no exerciseable artifact                      | Choose and record one before approval.                              |
| A newly discovered defect expands scope                     | File a GitHub issue; keep the proposal bounded.                     |
| Existing code suggests the design                           | Treat it as evidence, not authority; settle intent in the proposal. |

## Anti-patterns

- **Proposal as a conversation summary.** Record decisions and observable behavior, not chat history.
- **Folder-shaped planning.** The proposal is one document. Do not distribute its sections across files.
- **Approval by silence.** `draft` is not accepted. Only the human changes the status.
- **Preview at review time.** Decide it during **Proposal**, then make it in **Review**.
- **Implementation-led intent.** Existing code does not resolve open questions.
