---
id: proposal.0004
title: Make Room for UI Polish
authors: [markmals, claude-opus-5-5]
status: draft
pull-request: 8
issues: []
supersedes: []
---

# Make Room for UI Polish

## Summary

Proposals say what a feature does; the human decides its look and feel on the running UI. Look-and-feel changes never revise a proposal, are applied while work stays in review, and need no proposal between features. A short polish skill holds the method.

## Motivation

Polishing an interface is iterative and visual: the human looks at a running screen, asks for a change, looks again, and repeats until it is right. Current models are good at this loop. Workbench gives it no place.

**During implementation and review, every tweak becomes a proposal revision.** In Pico's Mac app ([proposal 0005](https://github.com/markmals/pico/blob/main/proposals/0005-mac-app-ui-workspace.md), [PR #41](https://github.com/markmals/pico/pull/41)), the approved proposal went through five rounds of revise → approve → implement → active review. Twenty of the pull request's 35 commits were proposal bookkeeping, fifteen of them from those five rounds, and each round posted a new readiness report. The proposal now records that the window has "a fixed width of 715 points" and "a fixed 215-point sidebar", then concedes "Human visual refinement may adjust these point values" (L138). It specifies Sign Out as "a red, destructive button standing on its own below a provider page's Account group" (L104). Some of those rounds did change what the feature does — a sample data mode, collapsing "set up" and "needs sign-in" into signed in or not, dropping the model search — and belonged in the proposal. Much of the rest was look and feel: a sheet instead of a pull-down, a button's color and label, a window's width, tile insets.

**Between proposals, there is no sanctioned way to polish.** Pico's proposal closes with "Further refinement of the panes needs its own proposal" (L330). The agent read the contract correctly. `AGENTS.md` offers two paths: a correction whose intent already exists, or the proposal question for anything that "decides something", with "If a small fix turns out to need a design decision, stop and ask the proposal question then." Polish is neither a correction nor a feature, so it always lands on the proposal question, and `mise run status` on a pull request without a proposal reports **Phase 2 — Proposal**.

The cost falls on the people Workbench is for — a single owner working with an agent daily — and grows as more of that work is interface work. Proposals bloat into screen specifications that drift from the product, review stalls in status churn, and the owner either fights the process for every spacing change or works around it.

## Domain grounding

### Established context

- **Requirements and interface details are separate concerns.** Mike Cohn's _[User Stories Applied](https://www.informit.com/store/user-stories-applied-for-agile-software-development-9780321205681)_ advises keeping the user interface out of stories "as long as possible", because mixing requirements with solution specification, "most commonly with aspects of the user interface", has "plagued every approach to software requirements".
- **Shaped work is concrete about what, rough about how it looks.** Basecamp's _[Shape Up](https://basecamp.com/shapeup/1.1-chapter-02)_ says "Wireframes are too concrete": defining "too much detail too early" leaves designers "no room", and fixed details stop a team from reconsidering "a design decision that is turning out to cost more than it's worth". Words alone are "too abstract". Shaped work "defines what the feature does, how it works, and where it fits into existing flows" at the level of places, affordances, and connections.
- **Look and feel is settled on the running product.** _[Shape Up, "Get One Piece Done"](https://basecamp.com/shapeup/3.2-chapter-11)_: "Questions about font, color, spacing, and layout can be resolved after the raw affordances are in place." Its case study's designer, once a toggle was wired, "could continue to experiment with copy, placement, color, mobile view rendering, and more", judging it by clicking it "with live data on a staging server"; a demo "led to a few more tweaks and then they were able to call the toggle 'done.'" "First make it work, then make it beautiful."
- **Behavior specifications describe what, not how.** Cucumber's _[Writing better Gherkin](https://cucumber.io/docs/bdd/better-gherkin/)_ separates functional requirements from "procedural reference", observes that imperative scenarios are "so closely tied to the mechanics of the current UI" that they need updating whenever it changes, and offers a test: "Will this wording need to change if the implementation does?"
- **The established name for the other side of the line.** [Look and feel](https://en.wikipedia.org/wiki/Look_and_feel) "comprises aspects of its design, including elements such as colors, shapes, layout, and typefaces (the 'look'), as well as the behavior of dynamic elements such as buttons, boxes, and menus (the 'feel')" — "parts … not related to its functional properties".
- **The owner's user-story guidance already draws this line.** Mark's `writing-user-stories` skill (`~/Developer/Templates/agent-skills/writing-user-stories/SKILL.md`): stories describe "what and why, never how"; "No specific UI elements (buttons, dialogs, dropdowns) by name"; "Express intent, not mechanics. UI choices belong in design, not in acceptance criteria"; outcomes are "what the user can see, receive, or experience"; and the test "Imagine it's 1922."
- **Teams give polish its own small path.** GitLab runs a UX Paper Cuts effort for "small yet impactful usability issues", which starts from a merge request rather than a feature process ([epic](https://gitlab.com/groups/gitlab-org/-/epics/10026), as summarized in search results; not read directly).

### Relevant constraints and principles

- **Ontology: one word doing two jobs.** Workbench's "design" and "decides something" name both _what the feature does_ — capability, information, states, outcomes, errors, guarantees — and its _look and feel_. The two change independently (Pico's window width changed with no behavioral consequence), are decided through different media (a reviewed document versus the running UI), are verified differently (tests versus the human's eye), and keep different records (the proposal versus the product and its commits). Every rule written to protect the first also fires on the second. Polish is an activity, not an artifact, phase, or status; it differs from a correction, whose intent already exists, in that the human decides look and feel in the moment by seeing it.
- **Vision: keep the human as the final authority.** The human still decides look and feel — by looking at it rather than by approving prose. The agent's look-and-feel choices during implementation are provisional until the human has seen them.
- **Vision: previews expose behavior, not descriptions of behavior.** A paragraph naming a point width is a description; the running window is the artifact. Deciding look and feel on the running UI applies this principle.
- **Vision: no single artifact defines a feature.** Look and feel ends up with one representation, the running UI. That removes no check: tests cannot judge taste, a guide need not describe it, and a proposal that pins it only creates a second description that drifts. The independent check on look and feel is the human looking at it.
- **Vision: ceremony must earn its cost.** Pico's revision rounds for look-and-feel changes produced proposal edits, status changes, and readiness reports that no later reader needs.
- **Vision: enforce mechanically or not at all.** Telling look and feel from what the feature does is a judgement; a mechanical check such as flagging point sizes or colors in Detailed design would misfire on proposals where they are the requirement. The line therefore lives in the contract, the skills, and the template.
- **Workbench history: proposal 0002 was withdrawn** in [PR #6](https://github.com/markmals/workbench/pull/6) as "Over-engineered — the ask was to soften absolutist ceremony language and let the agent judge, not to add a tier framework with its own vocabulary and enforcement." This change adds no tier, mode, status, label, or detection.

### Quality bar

- An agent writing a proposal for a UI feature specifies what the user can do, find out, and is told, and no layout, size, color, type, control, or label unless the human made that detail a requirement.
- During implementation and review, look-and-feel feedback produces commits on the feature branch and nothing else: no proposal edit, no status change, no readiness report, no adversarial review.
- Asked to polish between proposals, an agent starts without the proposal question, keeps the UI running where the human can see it, and stops to ask when a request would change what the feature does.
- The line fits in one sentence, and classifies each of Pico's revision-round changes without debate.

### Remaining uncertainty

- **Edge cases between feel and flow.** Whether removing a sidebar toggle or moving an action to another screen changes what a user can do is a judgement. The design resolves doubt by asking the human in one line; whether that produces too many questions is untested.
- **Wording.** Treating wording as look and feel except where the exact words are a requirement may prove too wide for projects whose copy carries product decisions.
- **One project's evidence.** The churn is observed in Pico. Trove's web proposals stayed at capability level without this guidance; its native prototype polish happened in commits outside any proposal.

## Existing baseline

Workbench at `2d9e7b9`:

- `AGENTS.md` L9–13 asks "does this change decide something?" and offers two paths, with "If a small fix turns out to need a design decision, stop and ask the proposal question then." L34: "Behavior it does not describe does not get built; if it should exist, revise the proposal with the human."
- `writing-a-proposal` L12 makes proposals "for changes that decide something"; L31 requires Detailed design precise enough that "someone who is not the author could implement it without asking questions". `guides/writing-a-good-proposal.md` L35 adds: "If two reasonable implementations would expose different behavior and the proposal does not choose, the design is incomplete." For an interface, two reasonable implementations always look and operate differently, so read literally these demand the whole look and feel.
- `implementing-a-proposal` L48: "Add no behavior the proposal does not describe." `reviewing-an-implementation` L19 calls anything implemented that the proposal does not describe "scope creep or a gap in the proposal". `adversarial-review` L53 routes such behavior "back to the proposal". Its L24 excludes "taste" — meaning code style.
- `PROCESS.md` L231, `README.md` L72, `guides/running-a-feature.md` L134 and L142, and `tools/status.ts` L151 route review feedback that "changes the design" through a proposal update. `tools/status.ts` L121 tells the agent on a pull request without a proposal to ask the proposal question.

What expresses the problem well and stays: the proposal as the reviewed record of intent; the consent question for anything that decides what the software does or how it is built; human ownership of status; the coupled handoff into review; the independent review of behavior.

What is incidental: the word "design" standing for both intent and look and feel; the absence of any path for work that is neither a correction nor a feature.

Nearby baselines: Trove's `prototyping-apple-app-design` skill sanctions proposal-free design iteration, but only for exploration before implementation ("Not for: … adding screens to an existing shipping app"). The `writing-user-stories` skill holds the UI guidance this proposal adapts. Proposal 0002 is the closest prior attempt at a lighter path and shows what not to build.

## Proposed solution

Draw one line and apply it everywhere the contract currently says "design":

> A proposal specifies **what the feature does**. **Look and feel** — how the interface looks and how it is operated — is decided by the human on the running UI.

```text
"Sign Out should be a red button on its own"       → look and feel: commit it, no proposal edit
"Signing out should keep the provider's models"    → what it does: proposal revision, as today
"Tighten the sidebar spacing" (between features)   → polish: no proposal question
```

Writing a proposal for an interface adapts the user-story guidance: say what the user does and finds out, not which control they press. Implementation chooses look and feel to fit the project and platform; the human refines it in the preview. Look-and-feel feedback during review is applied while the proposal stays in `active-review`. Between proposals, polish is a direct change. A new `polishing-a-ui` skill holds the loop and its boundaries.

## Detailed design

### The line

- **What the feature does** is what a user can do, what they can find out, the states and outcomes they observe, the errors they are told about, and any guarantee the project makes — about their data, privacy, accessibility commitments, or platform requirements the human has chosen. Proposals specify it, tests encode it, and changing it needs the proposal path.
- **Look and feel** is how the interface looks and how it is operated: layout, sizes, spacing, typography, color, iconography, motion, grouping and placement, which control presents a choice (a sheet, a menu, a toggle), what activating an element opens, and the wording of labels and messages. The human decides it on the running UI. It is never recorded in a proposal, and changing it never needs the proposal path.
- **Wording.** What a message must convey is what the feature does ("the user is told the file exceeds the size limit and what the limit is"). Its words are look and feel, unless the exact words are themselves a requirement: legal or policy text, text a user must type or match, or a term the project's vocabulary fixes.
- **Accessibility.** A commitment ("every action is reachable from the keyboard", "status is never conveyed by color alone") is what the feature does. How a screen meets it is look and feel. Polish never breaks a commitment.
- **Non-graphical interfaces.** The same line applies to the human-facing side of a command-line or terminal interface: colors, alignment, progress display, and help-text wording are look and feel. Commands, flags, exit codes, machine-readable output, and a library's API are what the feature does.
- **The test.** "Would this sentence need to change if the screen were redesigned while the feature did the same thing?" If yes, it is look and feel.
- **Doubt.** When a change could be either, the agent asks the human which it is in one line before acting. A change that removes something a user could do or find out is what the feature does.

### Writing proposals

- `writing-a-proposal` gains a section, "Describing a user interface", adapted from `writing-user-stories`: state the user's intent, not the mechanics ("the user shares the document with a teammate", not "clicks Share and types an email"); outcomes are what the user sees, receives, or is told; no layout, sizes, colors, typefaces, control types, element names, or click-and-tap steps unless the human has made that detail a requirement; the redesign test above. It states that two implementations differing only in look and feel do not make Detailed design incomplete.
- `writing-a-proposal`'s opening section says that look and feel is decided by the human on the running UI, not in a proposal, and that polish is not a reason to ask the proposal question.
- The Detailed design comment in `.agents/templates/PROPOSAL.md` gains one sentence: for a user interface, specify what the user can do, find out, and is told, and leave look and feel to the running UI.
- `guides/writing-a-good-proposal.md` gains a section, "Leave look and feel to the running UI", with one before-and-after example drawn from a Pico-style settings pane, and a red-flags row: Detailed design names sizes, colors, controls, or labels → look and feel in the specification → state what the user can do and find out, and decide the rest on the running UI. Its "two reasonable implementations" paragraph is qualified the same way as the skill.

### During implementation

- `AGENTS.md`'s "Stay inside the proposal" rule applies to what the feature does. It adds that look and feel is not behavior the proposal must describe: the agent chooses it to fit the project's existing interface and platform conventions, and the human refines it on the running UI.
- `implementing-a-proposal`: tests for an interface assert what the user can do and observe, not its look and feel, so polish does not break them. A guide takes what the user does from the proposal; the names and placement of on-screen elements it mentions follow the running UI and are updated when polish changes them — this is not writing a guide from finished code. "Add no behavior the proposal does not describe" is qualified: look and feel the proposal leaves open is the agent's to choose and the human's to refine.
- `reviewing-an-implementation`'s self-check excludes look and feel from "anything implemented that the proposal and guides do not describe is scope creep or a gap in the proposal".
- `adversarial-review` lists look and feel beside style, naming, and formatting as out of scope, and lens 5 states that look-and-feel choices are not proposal gaps. Broken accessibility commitments and look-and-feel changes that alter what the feature does remain findings.

### During human review

- While a proposal is `active-review`, feedback that touches only look and feel is applied directly: the agent changes the code on the feature branch, runs `mise run check`, pushes, refreshes whatever the human is looking at (the pull-request preview, or a running build), and says what changed. The status stays `active-review`. The agent does not edit the proposal, post a readiness report, or run an adversarial review for it.
- Feedback that changes what the feature does follows the existing path: the human sets `returned-for-revisions`, the agent updates the proposal first, and the affected tests, guides, code, gates, self-check, and review are repeated.
- Feedback mixing both is split: the look-and-feel part is applied in review; the rest takes the existing path.
- `PROCESS.md` (Human Review), `README.md`, and `guides/running-a-feature.md` (Phase 4) describe this, replacing "changes the design" with "changes what the feature does".

### Between proposals

- `AGENTS.md`'s "Does this change need a proposal?" keeps its question, adds that look and feel is the exception, and gains a case: when the human is shaping look and feel, the agent makes the changes directly with `.agents/skills/polishing-a-ui/`, saying so in one line before starting. "If a small fix turns out to need a design decision" becomes "If a small fix or polish turns out to change what the feature does". `PROCESS.md`'s paragraphs on when the full loop applies and on the proposal question say the same.
- Polish lands the way the project lands other changes made without a proposal: a branch and pull request with a changeset where the project works through pull requests, otherwise commits. The agent chooses and states which in one line.
- Outside an approved proposal, the agent suggests look-and-feel changes but makes them only at the human's direction.

### The `polishing-a-ui` skill

`.agents/skills/polishing-a-ui/SKILL.md`, at most about 80 lines, triggers on refining an interface's look and feel during implementation, review, or between proposals, and not on changing what a feature does. It contains:

- **The loop.** Get the interface running where the human can see it — a development server, a simulator or installed build, or the pull-request preview — and keep it current after every change. Make one change, or one small coherent set, at a time and say what changed and where to look. When the human is weighing options, show them rather than describe them. The human keeps or reverts each change; continue until they are satisfied. Commit kept changes at natural points; the message names what changed and, when it is not obvious, why.
- **Boundaries.** A request that would change what the feature does stops the loop: in review it takes the `returned-for-revisions` path; between proposals it gets the proposal question. In doubt, ask which it is. Never record look and feel in a proposal.
- **Keeping the other representations true.** `mise run check` passes before every push. A test that breaks because it asserted look and feel is rewritten to assert behavior instead, never re-pinned to the new look; a behavior assertion is never weakened. Any guide that names, shows, or places an on-screen element that changed is updated. Accessibility commitments and platform requirements are never broken.
- **Conventions.** When the human applies the same look-and-feel rule repeatedly — a spacing scale, a type ramp, sentence-case labels — suggest recording it as a policy; record it only after discussing it with the human.
- **Landing.** Inside a proposal's pull request: commits on its branch with no status change, as in _During human review_. Between proposals: as in _Between proposals_.
- A short red-flags table covering: editing a proposal for look and feel; posting a readiness report or changing status for a polish round; asking the proposal question for polish; polishing without a running UI the human can see; re-pinning a test to new look and feel; changing what the feature does under the name of polish.

`AGENTS.md` names the skill in the case above; `PROCESS.md` names it where it describes polish.

### `tools/status.ts`

Wording only; phase derivation is unchanged.

- Default branch (L107): "…otherwise handle the correction or polish directly."
- Branch without a pull request (L114): adds that corrections and look-and-feel polish need no proposal.
- Pull request without a proposal (L121): "Corrections and look-and-feel polish continue without a proposal. Otherwise ask the human whether they want a proposal for this change before writing one, unless they already answered for this change. If yes, draft and publish it."
- `active-review` (L144): "Review the diff and the preview. Ask for look-and-feel changes as you go, comment on the pull request, or accept the work."
- `returned-for-revisions` (L151): "Process the feedback; update the proposal first if what the feature does changed."
- `guides/running-a-feature.md`'s description of these messages matches.

### Supporting changes

- `PROCESS.md` gains a short "Look and feel" subsection under Feature Development that defines the line and describes polish in implementation, in review, and between proposals, naming `polishing-a-ui`. Its Artifacts description of proposals says they describe what a feature does, not its look and feel. Its self-check paragraph excludes look and feel from "code that implements behavior the proposal does not describe".
- `guides/running-a-feature.md`'s "Not every change needs all of this" section describes polish between proposals for the human reader.
- `.github/pull_request_template.md`'s Proposal section says what to write when there is no proposal — a correction or look-and-feel polish — and that the phase checklist is checked only where it applies.
- The guides changed here add `proposal.0004` to their `describes` frontmatter.
- A changeset describes the change for adopters.
- `mise run check` passes.

## Compatibility

No artifact, frontmatter field, validator rule, proposal status, or phase changes. Existing proposals keep their text. Look-and-feel details already written into a proposal describe what was decided at the time: an agent implementing such a proposal builds them as written, the human may change them afterward through polish without editing the proposal, and neither the self-check nor the adversarial review treats the resulting divergence as a defect. Proposals in flight need no edits.

## Implications on adoption

Adopters update `AGENTS.md`, `PROCESS.md`, `README.md`, four skills, the proposal template, the pull-request template, `tools/status.ts`, and two guides, and add `polishing-a-ui`. Project-specific sections are untouched. A project with no user interface can still adopt it; the skill simply never triggers. Reversible by restoring the previous files and deleting the skill.

## Scope

- Make the changes in _Detailed design_ to `AGENTS.md`, `PROCESS.md`, `README.md`, `writing-a-proposal`, `implementing-a-proposal`, `reviewing-an-implementation`, `adversarial-review`, `.agents/templates/PROPOSAL.md`, `.github/pull_request_template.md`, `tools/status.ts`, `guides/writing-a-good-proposal.md`, and `guides/running-a-feature.md`.
- Add `.agents/skills/polishing-a-ui/SKILL.md`.
- Add a changeset.
- Exercise the revised contract in a Pico polish session, as described under _Preview_.

### Out of scope

- **Small changes to what a feature does.** "Remember the last selected pane" changes behavior, however small. The proposal question already handles it, and a "no" already lets the agent proceed; widening polish to cover behavior would erase the line this proposal draws.
- **Visual regression or screenshot testing, design systems, and design tokens.** These are project and stack choices, which Workbench does not make. Polish may surface conventions worth a policy; recording them stays the human's choice.
- **A polish tier, mode, label, status, or detection in `tools/status.ts`.** The lesson of proposal 0002.
- **Revising other projects' proposals.** Pico's proposal 0005 and Trove's proposals are history and keep their text.
- **The global `writing-user-stories` skill.** This proposal adapts its UI guidance into Workbench; it does not change or vendor the skill.
- **The vision.** Whether accepted work should add a revision-history line is decided at completion, as for every feature.

## Preview

- Artifact: the pull request's documentation-site preview under `pr-preview/`, which publishes the revised `PROCESS.md` and guides, plus a Pico branch carrying the revised generic contract, skills, template, and status tool, on which the human runs one polish session on the settings panes that proposal 0005 left needing "its own proposal".
- Reason: the change alters agent behavior, and reading prose is not exercising it. Asking an agent with the revised contract to refine a real pane, giving it look-and-feel feedback, and slipping in one request that changes what the feature does is the cheapest realistic exercise. Expected: no proposal question for the polish, commits only, and a one-line stop on the behavioral request.
- Cleanup: delete the Pico branch after acceptance, with explicit confirmation.

## Policies and decisions checked

- None — `policies/` and `decisions/` contain only their READMEs. `VISION.md` was read; its principles are addressed under _Relevant constraints and principles_.

## Future directions

- A lint that warns when Detailed design contains sizes or color values, if over-specification persists after this guidance. It was not chosen because such values are sometimes the requirement.
- A shared starting policy for common look-and-feel conventions, such as following the platform's interface guidelines, for projects to adopt or reject.
- Repeating the Pico comparison on a web project once one has proposal-driven UI work.

## Alternatives considered

- **Wording only, no skill.** It is the smallest change, but the method would live as a paragraph in `PROCESS.md` and a line in `AGENTS.md`. The agent would have no procedure loaded when polish starts, and `AGENTS.md` would grow. Chosen against by the human.
- **A record of look and feel** — a design-notes file per feature, or a Look and feel section the agent may update freely. It keeps a written history, but it is a second description of what the running UI already shows, and it drifts exactly as Pico's proposal did.
- **What it does versus appearance only.** Only the look would leave the proposal, and interaction mechanics would stay. Pico's sheet-versus-pull-down and Set Up changes would still have revised the proposal, and the user-story guidance treats mechanics as interface design.
- **Leave it to each proposal.** It is flexible, but agents would get no default, which is today's problem.
- **Return look-and-feel feedback through `returned-for-revisions`, more lightly.** It keeps every round visible in the status, but it charges a status change and a report for each tweak — the churn this proposal removes.
- **Accept first, polish afterward.** Review stays about behavior, but the human would accept an interface they have not finished shaping.
- **Always a pull request with a preview for polish between proposals.** It is uniform, but it costs a pull request for a single spacing fix, where the project's normal direct-change path already applies.
- **A polish tier with status detection or a pull-request label.** It is enforceable, but it is the tier framework with its own vocabulary and enforcement that proposal 0002 was withdrawn for.

## Open questions

- None.

## Acknowledgments

- Mark's `writing-user-stories` skill, whose UI guidance this adapts.
- Pico's proposal 0005 and PR #41, which showed the problem end to end.
- Basecamp's _Shape Up_ and Cucumber's Gherkin guidance, for the level of abstraction.
