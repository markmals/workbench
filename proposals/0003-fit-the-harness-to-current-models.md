---
id: proposal.0003
title: Fit the Harness to Current Models
authors: [markmals, claude-opus-5-5]
status: implemented
pull-request: 7
issues: []
supersedes: []
---

# Fit the Harness to Current Models

## Summary

Shrink the agent contract and collapse the implementation phase's review ceremony. A controlled experiment on Trove showed full Workbench cost 2.7× a lean variant and 4.7× a process-free one on Opus 5.5 and GPT 6.1 Sol, with no gain in correctness or judged quality. Two runs timed out after widening scope.

## Motivation

Workbench's implementation phase was written for models that needed to be held to a procedure: a strategist that hands every slice to an implementer subagent, a five-status return protocol, an excellence pass, two sequential inline reviews, a cross-artifact review, and an adversarial review that repeats until it converges. `AGENTS.md` restates much of `PROCESS.md`, and the two together run to roughly 7,500 words before any skill is read.

The vision says ceremony must earn its cost and lists "measurement of … which stages earn their cost in practice" as a direction worth pursuing. Nobody had measured it. The Claude Code team's public advice for current models is that most instructions written for older models can be removed without loss, that subagents parallelize but do not make the work smarter, and that review should be done once, by fresh context, against the spec, ignoring style. If that holds for Workbench, every adopting project pays for steps that buy nothing, and the cost lands on the people Workbench is for: a single owner running an agent daily.

The experiment below measured it on a real Workbench project. The full process was the most expensive arm on every task and model, never the best on correctness, and on the larger task its review loop pushed both strong models past a four-hour cap while widening scope well beyond the proposal.

## Domain grounding

### Established context

- **The source of the hypothesis.** Addy Osmani, "You're probably overprompting Claude" (X post, late October 2026), reports that for Claude Opus 5 and Fable 5 over 80% of the Claude Code system prompt was removed with no measurable drop on coding evals; recommends one clear goal plus a way to verify it; says subagents "don't, by default, make things smarter" and that "a single agent with skills can outperform a subagent design on quality, often at lower cost and latency"; recommends reviewing with a fresh-context subagent given the spec and told to ignore stylistic issues; and recommends auditing instructions every few months by removing some and adding back only what the model misses. These are a practitioner's claims about Claude Code's own evals, not independent evidence about Workbench; they motivated the experiment, and the experiment is the evidence.
- **The experiment (direct observation).** Three contract variants ("arms") of the same Trove commit, two approved Trove proposals, three models, one unattended Oh My Pi run per cell (18 runs). Every role and subagent in a run used that run's model; cross-provider fallback was disabled. Each run received the same prompt: implement the proposal's remaining scope unattended, commit locally, skip PR/preview/human steps and put their content in the final message.
    - **baseline** — Trove's contract synchronized to Workbench at `cb2cd56` (4,910-word `AGENTS.md`, current `PROCESS.md` and skills, `check-before-stop` and `format-after-edit` extensions, Trove's `implementer`/`adversary`/`reframer` agents).
    - **lean** — the change this proposal specifies, applied to the same commit (1,874-word `AGENTS.md` of which about 1,200 words are Trove project facts).
    - **minimal** — Trove's project facts, conventions, and domain skills only; no Workbench process text, process skills, or role agents.
    - **Tasks.** Proposal 0007's missing piece (embed a same-name subtitle file beside a video; Go CLI, small) and proposal 0012's missing pieces (Torznab indexer client, settings contract and web form, failure reporting; Go daemon, OpenAPI, TypeScript, medium).
    - **Models.** Claude Opus 5.5 and GPT 6.1 Sol at `xhigh` reasoning effort; Kimi K3.1 Pro at `high` (its highest below `max`).
    - **Grading.** Hidden acceptance probes the agents never saw: four `ffprobe` scenarios for 0007; a black-box settings round-trip against the built daemon and nine behavioral Torznab scenarios for 0012, written as throwaway tests against each implementation by a model other than the one that wrote it. All gates re-run independently. Blinded diff review by all three models on a 20-point rubric (fidelity, tests, design, docs; style excluded). Cost, time, and dispatches read from session transcripts.

Results for the two primary models (cost / active minutes / blinded score out of 20; the four-hour cap stopped a run):

| Task      | Model       | baseline             | lean                | minimal            |
| --------- | ----------- | -------------------- | ------------------- | ------------------ |
| 0007      | Opus 5.5    | $35.28 / 225 / 14.7  | $19.31 / 131 / 13.7 | $8.21 / 101 / 14.3 |
| 0007      | GPT 6.1 Sol | $4.04 / 99 / 15.7    | $3.25 / 90 / 16.7   | $1.17 / 61 / 17.0  |
| 0012      | Opus 5.5    | $124.07 / cap / 14.3 | $38.16 / 91 / 15.0  | $26.71 / 58 / 15.0 |
| 0012      | GPT 6.1 Sol | $24.56 / cap / 9.0   | $9.22 / 92 / 16.7   | $3.70 / 46 / 16.7  |
| **Total** |             | **$187.95**          | **$69.94**          | **$39.79**         |

- **Correctness.** Every completed run passed every hidden check: 4/4 subtitle scenarios on 0007, and on 0012 the settings round-trip plus 9/9 behavioral scenarios. Both baseline 0012 runs hit the cap with uncommitted work. Opus's working tree was functionally complete. GPT's did not compile, stopped mid-rename in unrelated acquisition code, and failed 4/9 scenarios.
- **Why baseline 0012 ran out of time.** The two runs made 11 and 17 subagent dispatches (implementers, compliance and quality reviewers, repeated adversaries, reframers). Fixing successive review findings carried both runs outside the proposal: a new database migration, a changed torrent-version identifier scheme, new acquisition columns with no migration, and a new duplicate-ranking policy. Judges flagged this scope growth as major defects. Lean made one dispatch (its adversary) in three of four runs; on 0012 GPT split the work into four implementer slices and ran four adversary passes. Minimal made zero to three dispatches, all scouts or general research tasks.
- **Tests first.** By first-edit timestamps, baseline and lean wrote a test before production code in 4 of 5 completed runs each; minimal did in 1 of 5. Blinded test scores did not differ across arms at this sample size.
- **Independent review earns something.** Lean's adversarial review reported confirmed defects in three of five runs (three on 0007 Opus, five on 0012 Opus, three and then two on 0012 GPT), which the agents fixed. It found nothing in the other two. Minimal ran no reviewer.
- **Context.** Instruction text before the first message (GPT tokenizer): baseline 18.1k tokens, lean 14.0k, minimal 12.0k. The direct token cost of prose is small; the cost is the behavior it prescribes.
- **Kimi K3.1 Pro.** On 0007 all three arms passed every check (baseline $5.30 / 110 min / 12.7; lean $2.18 / 87 / 13.0; minimal $2.01 / 94 / 15.7). Its 0012 runs hit the provider's five-hour usage limit minutes after starting and were stopped; they are excluded.

### Relevant constraints and principles

- **Vision: no single artifact defines a feature.** Proposal, tests, guides, code, and preview each check the others. Removing a representation removes a check; removing a redundant _review stage_ over the same representations does not.
- **Vision: ceremony must earn its cost.** A step that cannot be justified by a specific recurring failure gets deleted.
- **Vision: keep the human as the final authority; agents do not ratify their own work.** Independent review and human-owned status transitions stay.
- **Vision: enforce mechanically or not at all.** Prose rules a model already follows are the first candidates to cut.
- **Workbench rule: never expand scope.** The baseline's review loop broke this rule in both runs of the larger task. A process that reliably induces a hard-rule violation is worse than one that omits the step.

### Quality bar

- After the change, an agent with a current frontier model implements an approved proposal end to end, with tests written before production code, guides derived from the proposal, one independent review, and a readiness report, at substantially lower cost than today and with no loss of correctness or independent checks.
- Every rule that remains in `AGENTS.md` is either governance the human chose (consent, status ownership, destructive actions), an ordering a model does not follow unprompted (tests before code), or project knowledge the agent cannot infer.

### Remaining uncertainty

- **Sample size.** One run per cell; two tasks from one repository. Re-judging an identical diff moved its blinded score by 1.7 points, so score differences under about 2 points are noise. Cost and time differences of 2–5× and the timeouts are well outside that noise.
- **Load.** Up to ten runs shared one machine, which inflated wall time for long gates; active minutes exclude provider rate-limit stalls but not CPU contention.
- **Phase coverage.** The experiment measured implementation of an already-approved proposal. It says nothing about grounding, ontology review, or proposal writing, which this proposal leaves unchanged.
- **Author bias.** The lean arm was designed by the agent that wrote this proposal. The minimal arm bounds that bias: lean beat or matched baseline, and minimal matched lean on quality while costing less.

## Existing baseline

The implementation phase today, at `cb2cd56`:

- `AGENTS.md` (3,703 words) restates the reasoning order, agent roles, the five implementer statuses, the full feature loop, and the ordered tests → guides → code list (twice), then the hard rules. `PROCESS.md` (3,775 words) describes the same loop for humans.
- `implementing-a-proposal` makes delegation the default ("Do inline only when trivial"), with a handoff checklist and a status table, then requires an `excellence-pass`, then a proposal-compliance review, then a code-quality review.
- `reviewing-an-implementation` requires a cross-artifact behavior inventory, then an `adversarial-review` dispatch that repeats until `CONVERGED`, then a preview and readiness report.

What expresses the problem well and stays: the proposal as the record of intent; human consent before a proposal and human ownership of status; tests before code and guides from the proposal; an adversary with fresh context that reports but never edits; the preview and readiness report; and the refute-by-default stance of `adversarial-review`, which found real defects in the lean arm.

What is incidental: the delegation default and its status protocol, which exist to manage a handoff that a single agent does not need; three confirmatory passes (excellence, compliance, quality) and a cross-artifact inventory over the same four artifacts, which a single self-check covers; the convergence loop, which on a larger task turned each round of findings into new work; and the duplication between `AGENTS.md` and `PROCESS.md`.

## Proposed solution

Keep every artifact and every independent check; remove the stages that re-check the same artifacts and the delegation that a current model does not need.

```text
approved proposal → tests → guides (when user-facing) → code → gates
                  → self-check → one fresh-context adversarial review → preview → readiness report
```

The strategist implements inline by default and splits work only into independent slices worth running in parallel. One self-check compares the proposal, tests, guides, and code and asks whether this is the implementation that should remain. One adversarial review, told to ignore style, tries to falsify the work; confirmed in-scope defects are fixed, and anything outside the proposal is recorded for the human rather than built. `AGENTS.md` becomes a short operative contract; procedure lives in the skills, and the full narrative stays in `PROCESS.md` for humans.

## Detailed design

### `AGENTS.md`

- `AGENTS.md` contains, in this order: a two-sentence description of Workbench and `vp run status`; the "does this change decide something?" test and the proposal-consent rule; who owns each proposal status transition; a six-step summary of implementing an approved proposal (read Detailed design, failing tests, guide when user-facing, code, verify with shown output, one independent review and readiness report) that names `implementing-a-proposal` and `reviewing-an-implementation`; the delegation rule (below); a rules list; quality gates; harness setup; and pointers to the rules files and reasoning skills.
- The rules list keeps: stay inside the proposal and route out-of-scope defects to issues; raise ambiguity instead of choosing silently; root cause before fixes; vision, policies, and decisions constrain proposals and new records need the human; never delete a branch, force-push, publish a release, or deploy without explicit confirmation; never write production code before its failing test; never write a guide from finished code; never claim done without running the verifying command this turn; the adversarial reviewer never edits.
- `AGENTS.md` no longer contains: the reasoning-order essay, the strategist/implementer/reframer/adversary role essays, the implementer status list, the duplicated tests → guides → code list, the full feature-loop diagram, the phase-by-phase restatement of `PROCESS.md`, or the grounding sub-steps. Each survives where it is used: grounding and ontology in their skills and `writing-a-proposal`, reframing in `reframing-a-problem`, the loop in `PROCESS.md`.
- The generic part of `AGENTS.md` (everything a project does not customize) is at most 1,000 words.

### Delegation

- The agent that owns the proposal implements it. It dispatches subagents only for slices that are independent, touch separate files, and save real time by running in parallel, and for the fresh-context adversarial review.
- A delegated slice's brief contains the behavior, the relevant Detailed design, constraints, scope, and verification commands. The delegating agent integrates and commits the work.
- The five-status implementer protocol (`DONE` … `BLOCKED`) is removed from `AGENTS.md` and `implementing-a-proposal`. `reframing-a-problem` keeps its triggers, with "an implementer returns `NEEDS_STRATEGY`" replaced by "a delegated slice reports that the evidence contradicts the proposal's model".

### `implementing-a-proposal`

- Keeps: preconditions; tests → guides → code; watching each test fail for the missing behavior; property-based tests for universal claims; guides from the proposal, compared with the tests; minimum code; `systematic-debugging` for unexplained failures; gates and verification evidence; marking the PR ready once the complete verified implementation is pushed.
- Replaces the slice-and-dispatch section, the status table, "do inline only when trivial", the inline-review gate table, and the dispatch/model-selection rules with the delegation rule above.
- Ends by handing off to `reviewing-an-implementation`.

### `reviewing-an-implementation`

- **Self-check.** The implementing agent reads the proposal, tests, guides, and diff together. For each behavior in Detailed design it confirms the behavior is tested, documented where a user would look, and implemented, and it looks for implemented behavior the proposal does not describe. It then asks whether the structure fits the domain and whether a simpler realization expresses the proposal better. It fixes what it finds and reruns the affected gates. This replaces the excellence pass, both inline reviews, and the separate cross-artifact review.
- **Independent review.** It dispatches one fresh-context adversary, on a different model where the harness allows, with the proposal, the commit range, and the verification commands, and instructs it to try to show the work is not done, to confirm findings by running code, to ignore style, naming, and formatting, and to report without editing.
- **Consuming findings.** A confirmed defect inside the proposal's scope is fixed and verified; the review runs once more only when the fix was substantial. A suspicion is verified or dropped. A proposal gap goes to the human. A finding outside the proposal's scope is never fixed by widening the change: it is recorded as declined or filed through `tracking-defects`. Every declined finding appears in the readiness report.
- Preview and readiness report are unchanged.

### `adversarial-review`

- Its "when to use" names the implementer's self-check, not cross-artifact review, as the preceding stage, and adds style, naming, and formatting to what it does not review.
- "Loop until VERDICT is CONVERGED" is replaced by the consumption rules above. The stance, lenses, verification-before-reporting, and output format are unchanged.

### `excellence-pass`

- The skill is deleted. Its question — is this the implementation that should remain? — is part of the self-check.

### `PROCESS.md`, `tools/status.ts`, guides, and README

- `PROCESS.md`'s Implementation section describes the flow above: the agent implements unless independent slices can run in parallel; tests; guides; implementation; self-check and independent review; preview; readiness report. Its feature-loop summaries and revision paragraph name "self-check → independent review" where they now list the excellence pass, inline reviews, cross-artifact review, and adversarial review.
- `tools/status.ts`'s `awaiting-implementation` guidance reads: "Write failing tests, then guides, then code, and pass the gates. Self-check against the proposal, get one independent review, and post the readiness report on the ready pull request."
- `guides/running-a-feature.md`, `guides/writing-a-good-proposal.md`, and `README.md` describe the same flow wherever they name the removed stages.
- `vp run validate` and `vp run check` pass.

### Measurement

- The experiment's arms, prompts, hidden probes, scenario brief, judge rubric, and per-run results are attached to this proposal's pull request so a later revision can rerun the comparison when a new model ships.

## Compatibility

Adopting projects that have copied Workbench's `.agents/` and `AGENTS.md` keep working: no artifact, template, frontmatter field, validator rule, or proposal status changes. Proposals already `awaiting-implementation` are implemented under the new flow without edits. A project that wants the old ceremony can keep its copy of the removed skills; nothing in the validator depends on them.

## Implications on adoption

Adopters update `AGENTS.md`, `PROCESS.md`, and four skills, and delete `excellence-pass`. Project-specific sections of `AGENTS.md` (repository map, commands, previews) are untouched. Trove's synchronization in the experiment shows the merge is mechanical. The change is reversible by restoring the previous files.

## Scope

- Rewrite `AGENTS.md` as specified.
- Revise `implementing-a-proposal`, `reviewing-an-implementation`, `adversarial-review`, and `reframing-a-problem`; delete `excellence-pass`.
- Revise `PROCESS.md`, `tools/status.ts` and its tests, `guides/running-a-feature.md`, `guides/writing-a-good-proposal.md`, and `README.md` where they name removed stages.
- Attach the experiment harness and results to the pull request.

### Out of scope

- **Proposal-phase skills** (`grounding-a-problem`, `ontology-review`, `writing-a-proposal`). The experiment did not exercise them; trimming them needs its own measurement.
- **Templates, validator, previews, and completion.** Unmeasured and not implicated.
- **Global user guidance.** The experiment environment also loaded about 2,400 words of user-level guidance that duplicates Workbench's code-quality and commit rules. That is the user's configuration, not Workbench's.
- **The `check-before-stop` time bound.** Its 25-second limit is fine for Workbench's own 3-second gate, but Trove's gate takes 4–8 minutes, so in Trove the hook always refuses once with "did not finish". That is an adoption limitation of the hook, recorded under Future directions.

## Preview

- Artifact: the pull request's documentation-site preview under `pr-preview/`, which publishes the revised `PROCESS.md` and guides, plus the attached experiment harness, which a human can rerun against a Trove branch carrying the revised contract.
- Reason: the change alters agent instructions and their human-facing description. Reading the published process and rerunning one experiment cell is the cheapest realistic way to exercise it.

## Policies and decisions checked

- None — `policies/` and `decisions/` contain only their READMEs.

## Future directions

- A repeatable ablation harness shipped with Workbench, so adopters can run the same baseline/lean/minimal comparison on their own repository when a new model ships, as the source post recommends.
- A measured trim of the proposal-phase skills, using an experiment that starts from a request instead of an approved proposal.
- A `check-before-stop` that runs a fast subset of the gate, or reads the result of a gate the agent already ran, for repositories whose full gate exceeds the harness's handler timeout.
- Making the independent review conditional on the change's size. In this experiment lean cost 1.75× minimal for no measurable quality difference, but its reviewer found confirmed defects in three of five runs; more runs are needed before removing it.

## Alternatives considered

- **Adopt the minimal arm** (no process text in the implementation phase). It was cheapest and scored as well as lean, but it lost tests-before-code in four of five runs and has no independent review. Both are core to the vision, and the review found real defects. Not chosen.
- **Keep the process and fix only the convergence loop.** This would likely prevent the timeouts, but leaves 0007's 1.2–1.8× overhead from mandatory delegation and three confirmatory passes, which bought nothing measurable.
- **Trim `AGENTS.md` prose only and keep the stages.** Prose was the smaller cost: lean's context saving was about 4k tokens, while the spending came from the dispatches the stages require.
- **Wait for a larger experiment.** The cost and timeout effects are large relative to the noise, and the change is reversible.

## Open questions

- None.

## Acknowledgments

- Addy Osmani's post on overprompting, which framed the hypothesis.
- Trove, whose approved proposals and hidden-testable behavior made the experiment possible.
