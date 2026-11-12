---
id: proposal.0005
title: Keep Current Specifications Alongside Change Proposals
authors: [markmals, openai-codex/gpt-6-astra]
status: draft
pull-request: https://github.com/markmals/workbench/pull/9
issues: [workbench-1uz]
supersedes: []
---

# Keep Current Specifications Alongside Change Proposals

## Summary

Keep current requirements in Gherkin features, native contracts, and domain references. Keep proposals as change records. Define ten artifact roles and when each is required, revise policies and decisions in place, and update Workbench's process, validation, documentation, and own specifications to use this model.

## Motivation

Workbench makes an individual change reviewable, but its proposals accumulate requirements whose current meaning must be reconstructed. Later work commonly changes one clause of an earlier proposal while preserving the rest. Whole-document supersession cannot express that distinction without either discarding requirements that still apply or keeping conflicting statements under the same apparent authority.

Workbench demonstrates the problem itself. [Proposal 0001](0001-workbench-2.md) specifies a separate cross-artifact review. [Proposal 0003](0003-fit-the-harness-to-current-models.md) replaces that stage with a self-check. Both remain `implemented`, with `supersedes: []`. Their statuses correctly describe completed changes. Neither status tells a reader which individual requirements apply now. `PROCESS.md` already provides the current workflow, but Workbench gives proposals the role of specification and has no general current-specification layer for adopting products.

The same pattern occurs in local adopting projects. Pico's `proposals/0008-background-bash-jobs.md` changes one rule from proposal 0007: reopening a session reads the settings file for auto-background. Recon's `decisions/0005-homebrew-ad-hoc-privacy-identity.md` supersedes decision 0002 while retaining its permission-request and typed-failure behavior by reference. Reading only the newest record is insufficient in both cases.

The owner wants to preserve proposal development, human approval, tests before code, independently written guides, and review. Maintaining a second prose specification compiled from proposals would add another representation to synchronize. The requested change instead gives each current requirement an authoritative home, makes changes there directly, and preserves earlier states through Git and the change record.

## Domain grounding

### Established context

- The [Gherkin reference](https://cucumber.io/docs/gherkin/reference) distinguishes feature descriptions, general business rules, and scenarios that illustrate those rules. Descriptions can hold context and vocabulary; executing scenarios does not verify every statement in their descriptions.
- [Writing better Gherkin](https://cucumber.io/docs/bdd/better-gherkin) recommends describing observable behavior rather than interaction mechanics. That agrees with Workbench's existing separation of behavioral intent from look and feel.
- The [Gherkin parser and compiler](https://github.com/cucumber/gherkin) expose a document model and executable scenarios independently of Cucumber's runner. A project can keep Gherkin while using idiomatic test-framework bindings. Scenario outlines, backgrounds, and inherited tags have defined semantics that integrations must preserve.
- [OpenSpec's concepts](https://github.com/Fission-AI/OpenSpec/blob/main/docs/concepts.md) distinguish current specifications from proposed changes. Workbench can use that distinction with ordinary branch diffs rather than adopting a second delta-specification language.
- [Michael Nygard's architecture decision records](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions) preserve the context and consequences of significant choices. This proposal retains that information while choosing mutable current decision files and Git history for previous versions.

### Repository observations

The project inspection on 2026-11-12 read local checkouts, not a verified inventory of deployed behavior. The examples above establish documentation relationships only.

The local SpecKit's `internal/coreassets/templates/rules/spec-conventions.md` already requires editing existing specifications before changing behavior. Trove's `specs/protocol/troved.search.md` incorporates TV and movie search into one current wire contract. Pico's guides consolidate behavior that spans several proposals. These are existing examples of direct maintenance rather than compilation from proposal history.

Mutable files still require coherent edits. Flexscope's `features/0017-flexmac/models/walker.md` contains changes explicitly described as "flagged here, not yet propagated" to the shared node-identity model. Appending an amendment without changing its owning definition leaves the same reconstruction problem inside the specification library.

SpecKit's `docs/design/feature-loop.md` previously considered retaining contracts while consuming other specifications into tests and guides. That document is marked proposal / direction-setting. Its historical measurements and conclusions are context for this discussion, not validation of the design proposed here.

### Relevant constraints and principles

The conceptual distinctions are:

| Concept | Identity and lifetime |
| --- | --- |
| Capability | A stable area of product behavior that can be changed by many proposals. |
| Current specification | The required behavior or contract for a particular repository revision. It changes with the capability. |
| Proposal | One proposed change, its reasons, scope, and implementation approach. It remains evidence of that change after completion. |
| Policy | One continuing constraint, with applicability and exceptions. Its file changes while that logical policy remains in force. |
| Decision | One current significant choice, with the context and consequences needed to understand it. Its file changes when that choice changes. |
| Verification | Evidence about a specification and implementation at a revision. A passing example is evidence for that example, not proof of every possible execution. |

The ontology assessment is REVISE. The material conflation is a change record also acting as an indefinitely current definition of a capability. The missing distinction is between authority over current requirements and provenance explaining how those requirements changed. No new work-item or approval concept is needed.

A format is separate from an artifact's role. A native schema can be both source code and a contract. A Gherkin rule can carry the authoritative requirement for a product policy. The ten roles below do not require ten independent files or ten copies of a fact.

### Quality bar

A reader can find a capability's current requirements without replaying proposals or following supersession chains. Each normative fact has one current owner. Tests and guides remain independent interpretations that can expose defects in that owner or in the implementation. Historical rationale remains recoverable. A change updates its affected definitions before production code and lands those definitions with the implementation.

### Remaining uncertainty

The reusable Gherkin integrations will be designed separately, as the owner explicitly requested. Workbench currently runs its record-tool tests with `node:test` and its workflow-tool tests with Vitest. The integration and any necessary runner migration must be selected before implementation is approved; the open question below records that dependency. No claim about an available Swift Testing or Vitest integration is made here.

## Existing baseline

Preserve the proposal lifecycle and its human-owned approval points, the correction and look-and-feel paths, the shared `mise run check` gate, one independent adversarial review, and the coupled readiness handoff. Preserve native tests where Gherkin would add no useful behavioral specification. Preserve the current public documentation-preview workflow.

The relevant implementation is concentrated in:

- `AGENTS.md`, `PROCESS.md`, and the proposal, implementation, review, completion, and polish skills. They currently derive tests and guides from proposal Detailed design. Completion records policies and decisions after review and requires supersession for substantive changes.
- `.agents/templates/`, which has templates for vision, proposal, policy, decision, guide, and readiness report. There is no feature or domain-reference template. The guide template's `describes` field names historical proposals.
- `tools/validate.ts`, which recognizes proposals, policies, decisions, guides, and the vision. Policies and decisions require `established-by` and `supersedes`; supersession requires a corresponding status and an acyclic graph. Record directories are flat.
- `tools/status.ts`, which reports the proposal phase and next actor. Its implementation guidance starts at failing tests, and its proposal-history handling includes supersession.
- `docs/.vitepress/sidebar.ts`, `config.ts`, and `theme/components/RecordHeader.vue`, which publish Markdown records and their relationships. Superseded records are grouped separately. They do not discover or render Gherkin specifications or domain references.
- `tools/*.test.ts` and `.omp/tools/dynamic-workflows/*.test.ts`, which provide existing behavioral and lower-level coverage that the migration must preserve.

Workbench has three existing implemented proposals, six guides, and no substantive policy or decision files. The proposal archive is small enough to reconcile during this change. Adopting repositories with larger archives need an explicit migration procedure rather than an automatic claim that their specifications are complete.

## Proposed solution

Maintain the current contract directly, in the form suited to each fact. Use Gherkin for behavior, native formats for structural interfaces and data contracts, and short domain references for shared meanings that scenarios or schemas would otherwise leave implicit. Keep policies and decisions as current records. Keep proposals as historical explanations of changes.

A feature branch contains the proposed end state of those artifacts. The proposal identifies the intended delta and its reasons. The main branch contains the integrated contract, while a release tag identifies the contract shipped with that release. Tests exercise the implementation against that contract. The documentation site renders the current sources without committing a generated prose specification.

## Detailed design

### 1. The complete artifact inventory

`PROCESS.md` defines the following ten durable roles, including their trigger and update responsibility. The contract, templates, skills, and guides use the same inventory. There is no requirement to create an empty artifact merely because the inventory has a row for it.

| Role | Current authority and expected home | Creation and maintenance trigger |
| --- | --- | --- |
| Behavioral specification | `specs/features/<slug>.feature`, organized by capability. Owns observable behavior, rules, failures, and scenarios. | Add or revise when required behavior changes. Extend the existing owner rather than making a file per proposal. |
| Native contract | The owning subsystem's native schema, exported interface, protocol definition, or persisted-format definition. | Add or revise when that boundary changes. Keep a single source in its native format. |
| Tests | The project's native test locations and frameworks. Behavioral tests exercise the feature scenarios; other tests provide additional evidence. | Change with behavior, contracts, implementation risks, and regressions. Preserve useful non-Gherkin tests. |
| Guides | `guides/<slug>.md`, written for the feature's intended user, including consuming developers for libraries. | Add or revise when users must do or understand something differently. |
| Implementation | Source code and executable configuration, including build definitions, task graphs, environment schemas, and design tokens. | Change to realize the approved behavior or technical change. |
| Vision | `VISION.md`. Owns purpose, audience, goals, principles, boundaries, and direction. | Revise when these change. Avoid turning it into a complete feature catalogue. |
| Domain references | `specs/domain/<slug>.md`. Own shared terminology, identities, relationships, and lifecycle meanings. | Add when feature-local explanations are insufficient or multiple features depend on a nontrivial shared meaning. Revise that owner when the meaning changes. |
| Policies | `policies/<NNNN>-<slug>.md`. One logical continuing constraint, its applicability, rationale, and enforcement. | Add only after human agreement; revise in place or delete when retired. |
| Decisions | `decisions/<NNNN>-<slug>.md`. One current significant choice, its context, alternatives, and consequences. | Add only after human agreement; revise in place or delete when the choice no longer applies. |
| Proposals | `proposals/<NNNN>-<slug>.md`. One change and the reasons for it. | Create through the existing proposal process. Revise while the change is being designed or reviewed; retain the completed record as history. |

The first five roles may be feature-local or shared. The four project-context roles are reused by the features that depend on them. A proposal changes whichever current artifacts it affects; it does not create a new version of each as an independently maintained document.

Work items, changesets and changelogs, review reports, previews, and Workbench's own process machinery retain their existing purposes. They are part of the complete process accounting, but they do not acquire authority over current product requirements.

Separate narrative, use-case, user-flow, error-catalogue, view-model, toolchain-prose, and architecture-overview documents are not required. Their useful information has an owner in the table. A diagram can explain a domain reference or decision. An ERD can be generated from a schema. A navigation overview can link to current definitions without independently restating them. Look and feel continues to be judged on the running interface; design assets and tokens remain implementation or design material.

### 2. Current authority, scope, and approval

Each normative fact has one current owner. A feature states behavior, a native contract defines its exact structural boundary, and a domain reference defines shared meaning. They link where necessary. A policy can state its rule in its own file or point to the authoritative Gherkin rule or native contract; it does not maintain a second independent version of the same rule. Its scope, rationale, and enforcement information remain useful even when the rule is owned elsewhere.

Guides explain the current behavior independently and tests verify it. Agreement is assessed during review; neither a passing test nor the existing implementation silently changes intended requirements. When two current sources disagree, the agent raises the disagreement with the human rather than selecting one by filename, date, or implementation convenience.

During a change, the approved proposal bounds the authorized delta. The branch's current specifications express the resulting contract. If elaborating a specification introduces a new product or architectural decision, the agent resolves it with the human before implementing it. Completed proposals explain historical intent and do not override current specifications.

The proposal lifecycle, the consent question, and the owners of `awaiting-implementation`, `active-review`, and `accepted` remain unchanged. Approval of this proposal authorizes its implementation only after the existing approval conditions and the integration prerequisite below are satisfied. No agent may approve its own feature specification by deriving it from finished code.

### 3. Behavioral specifications and native contracts

Gherkin files use the standard language. Feature descriptions explain purpose and local vocabulary. Rules state general requirements. Scenarios provide concrete observable examples, including relevant failures and boundaries. A scenario outline supplies examples of a rule; it does not establish a universal claim by enumeration alone. Additional property, model-based, fuzz, security, performance, or architectural tests remain appropriate where the requirement calls for them.

A `.feature` file has one `Feature` and at least one scenario. It carries `@feature.<slug>` above the feature, matching its filename. Every scenario or scenario outline carries a globally unique `@scenario.<feature-slug>.<scenario-slug>` tag. Example rows are executions of their owning scenario outline, rather than independently maintained requirements. Scenario identities stay stable while their wording or test implementation changes; removing a scenario removes its bindings in the same change. Names describe the capability and behavior, rather than a proposal number.

For example, a policy-change scenario belongs to the current record-validation feature:

```gherkin
@feature.record-validation
Feature: Current record validation
  Authors can revise a current policy without creating a replacement record.

  Rule: A policy keeps its identity when its rule changes

    @scenario.record-validation.revise-policy
    Scenario: Revise an active policy in place
      Given an active policy with a valid identity
      When its rule is changed without adding a supersession record
      Then record validation accepts the revised policy
```

The test must exercise actual validation behavior. Checking that the scenario exists or that a test carries its tag is insufficient.

Native contracts stay with their owners. Workbench does not require a second Markdown copy of an OpenAPI schema, a new registry for every exported type, or a manually synchronized ERD. The relevant project-native checks run through its `mise run check` task. Exact request shapes, enum members, and validation constraints are defined by the authoritative contract; scenarios describe behavior using that contract.

Human judgment and behavior requiring a special environment remain visible in verification. Each applicable scenario is accounted for as automated, explicitly manual, or excluded from a particular target with a reason. A manual scenario has a named review action and recorded evidence at acceptance; it is never reported as an automated pass. Workbench's own programmatically testable tool behavior must have executable bindings rather than manual exemptions.

### 4. Domain references and mutable records

A domain reference is a short current definition of a coherent topic. It explains the concepts, identity rules, relationships, or lifecycle meanings needed to read the specifications correctly. It links to behavioral and native-contract owners instead of duplicating their scenario lists or field catalogues. There is no file-per-class or file-per-table requirement. Its frontmatter contains `id: domain.<slug>` and `title`, with the ID matching the filename. It has no independent workflow status; proposal review controls changes.

Policies and decisions retain their current numbered filenames and IDs. A meaning change updates the existing logical record. A record is deleted when it no longer applies, with affected current references updated in the same change. A supported compatibility obligation or incomplete migration is still an applicable constraint and cannot be deleted merely because the preferred design has changed.

The required policy and decision frontmatter becomes `id`, `title`, and `status`. Policy statuses are `draft` and `active`; decision statuses are `proposed` and `accepted`. Proposed records remain on the change branch until approved with the change. The existing `established-by` field becomes optional historical provenance; when present, it still resolves to a proposal. Git and the proposal record the reasons for later revisions. There is no new revision-history field or mandatory in-document change log.

`supersedes` and the `superseded` status are removed from current policy and decision records and their templates. Validation rejects them with migration guidance. Proposal supersession remains available for relationships between historical change records, with the existing validation rules. Changing an implemented feature does not require changing the old proposal's status or declaring the entire old proposal superseded.

The policy template retains the rule or a link to its owner, rationale, applicability and exceptions, and how compliance is checked. The decision template retains the current choice, context, rationale, alternatives, and consequences. Both lose the mandatory append-only revision-history procedure. Moving a rule into `.agents/rules` is appropriate only when its subject is how development work is performed; product guarantees remain product requirements.

### 5. Discovery, references, and checks

Add `FEATURE.feature` and `DOMAIN.md` to `.agents/templates/`, with instructions for the roles above and their destination paths. Update the template inventory. Native contracts use their native tools and do not receive a generic Workbench wrapper template. No record or domain topic is created simply to populate an empty directory.

`mise run validate` discovers the current feature and domain directories as well as the existing records. It parses Gherkin with a standard parser, validates the feature and scenario identities above, and rejects malformed files, absent or duplicate identities, empty features, and mismatched feature IDs. It validates domain frontmatter and the revised policy and decision schemas. Existing proposal clarification-marker and frontmatter checks remain in force.

Guide `describes` lists current owners: feature, domain, policy, or decision IDs. It is a non-empty list of resolvable current IDs. A native contract is linked directly from the guide and its behavioral owner rather than being assigned a second Workbench identity. Historical proposal IDs cease to be guide `describes` targets. Optional provenance links can remain in prose without determining what the guide currently describes.

A deleted current target cannot leave a dangling current reference. Validation continues to fail on unresolved metadata references; the documentation build checks local document links. Historical links to a file being deleted are changed to an immutable Git revision URL when they are needed to preserve the old record. This repairs the reference without rewriting historical intent or keeping a retired policy active solely to satisfy a link.

The test integration checks executable bindings in the test gate. Every applicable scenario and every expanded examples row must be accounted for on each applicable target. Missing, ambiguous, stale, or silently skipped bindings fail the relevant gate. Renaming or changing steps cannot leave an unrelated test green solely because it still has the old scenario tag. Structural binding checks cannot establish semantic fidelity; the self-check and adversarial review still examine the assertions and the implementation they exercise.

There is no requirement for a `SPEC` comment on every implementation unit, a lock file for every specification, or a proposal-to-code dependency graph. Stable scenario identities support verification and reporting without restoring SpecKit's reverse-pointer machinery.

### 6. The change process

The updated skills and contract prescribe this order when the stages apply:

1. During proposal development, identify the existing current owners and any missing specification or record. The proposal describes the problem, authorized delta, alternatives, technical approach, and compatibility or migration work. Detailed design links to current definitions and specifies the change sufficiently for elaboration; it does not become a second permanently current specification. Proposed paths can be named before they exist.
2. After human approval, update the affected feature files, native contracts, domain references, and agreed policies or decisions before production code. Required behavior absent from the approved intent is raised with the human. Specification diffs remain part of the reviewed change.
3. Write the failing behavioral bindings and other appropriate tests, and observe the relevant failure before production code. Preserve existing coverage that still applies. Test names and links follow current behavior and scenario identities rather than historical proposal IDs.
4. Write or revise user-facing guides from the intended specifications, compare them with the tests, and then implement. A guide's control names may still follow the running interface under the existing polish rule.
5. Run the shared gate. Self-check the proposal's authorized delta against the affected current specifications, tests, guides, implementation, and records. The independent reviewer receives those sources and the relevant unchanged constraints, not only the proposal.
6. Provide the preview and readiness report. The report identifies the changed current specifications and contract checks, automated verification, any manual verification still requiring the human, and any deliberate divergences. The existing coupled `active-review` transition is preserved.
7. Resolve human review, finalize current records and the vision where affected, and record `implemented` before merge. Merge the specifications and implementation together. Required release and cleanup retain their existing authorization rules.

A correction implements an already current requirement without inventing a proposal or changing the requirement to fit a bug. An underspecified behavior that requires a decision returns to the proposal path. Look-and-feel polish remains outside behavioral specification churn unless it changes a capability, outcome, or guarantee.

`tools/status.ts` keeps its phase derivation and human ownership rules. Its implementation and review guidance includes current-specification updates and review. `implemented` remains a record of accepted implementation, separate from merge and release. A separate approval state machine for feature files is not introduced.

### 7. Workbench's own adoption and publication

The implementation updates `AGENTS.md`, `PROCESS.md`, `VISION.md`, the relevant skills and rules, all affected templates and directory READMEs, and the guides that describe proposal writing, implementation, record maintenance, and adoption. The inventory in this proposal must be explained as conditional roles, not a ten-document checklist.

Workbench adopts the current specifications itself. Reconcile its exposed behavior across record validation, workflow status, the dynamic-workflow tool, and specification/documentation discovery into capability-scoped Gherkin files. Keep implementation-level tests in their native form where appropriate; bind behavioral scenarios to real tests using the integration selected below. Write the domain reference for record identity, current authority, and proposal lifecycle where those meanings are shared. Existing behavior is evidence during reconciliation; a discrepancy with approved intent is raised rather than automatically copied into the specification.

The current process documents retain their role as developer instructions and user guides. Their human-judgment requirements are classified explicitly as policies, current domain meanings, or procedure; they are not replaced with a claim that all process judgment is executable. Workbench's native contracts and configuration remain in their actual source locations. No speculative API schema or policy is added to fill the inventory.

The documentation site exposes current feature specifications and domain references alongside guides and current policies and decisions. Proposal history is identifiable as change history, and `implemented` is not presented as proof that every historical sentence is current. Current topics are discoverable by subject. Empty optional directories do not fail validation or the site build.

Gherkin pages are rendered from the `.feature` source during the documentation build, including descriptions, rules, scenarios, and examples with usable links. They are generated output, never a committed parallel Markdown specification. Record links and headers resolve current IDs and retain historical proposal navigation. Deleted current records disappear from current navigation. Superseded policy/decision groups and their current-record relationship display are removed; historical proposal relationships remain available.

The work includes behavioral regression coverage for the changed validator and status guidance, Gherkin discovery and reference handling, and documentation generation. The migration must demonstrate that changing a feature scenario affects its actual test execution, a missing binding fails, a policy can be revised without supersession, and deleting a still-referenced current record is rejected. These are consumer-visible behaviors; tests that only pin prose wording or file layout are not substitutes.

### 8. Integration prerequisite and completion boundary

The reusable Vitest and Swift Testing integrations are separate work by the owner's explicit choice. This proposal defines what Workbench needs from an integration, but does not design those libraries' public APIs or introduce a temporary Workbench-only runner as a substitute.

Before this proposal is approved for implementation, select a concrete integration release or reviewed source revision and establish how it runs Workbench's behavioral suites. If adopting it requires migrating the relevant `node:test` suites, that migration is part of Workbench's adoption and must preserve their coverage. Unrelated test suites keep their existing runners. Swift integration delivery is not a prerequisite for Workbench's own TypeScript tools, but an adopting Swift project needs a working integration before it can claim executable Gherkin coverage.

Implementation is complete only when Workbench's own feature specifications are exercised by real bindings through `mise run check` and rendered by its documentation build. Installing a parser, attaching scenario IDs to unrelated tests, or publishing process prose while the required integration is unavailable does not satisfy this proposal. The remaining selection is explicit under Open questions.

## Compatibility

This changes the record contract for policies, decisions, and guide references. Adopters migrate those records and the corresponding validator together. Old `supersedes` fields and `superseded` policy/decision statuses receive an actionable validation error instead of silently selecting a current interpretation. Proposal files retain their historical lifecycle and relationship fields.

For each retired policy or decision chain, the human and agent determine the complete rule or choice that still applies, incorporate retained clauses into the surviving logical record, remove supersession metadata, and delete retired current files. Keep the surviving ID where possible. Update current references and replace necessary historical links with immutable Git links before deletion. The process does not assume that the newest record already contains every retained clause.

For behavioral specifications, reconcile approved intent with the current implementation, tests, guides, and relevant proposals. Write each current requirement once, bind the applicable scenarios, and identify unresolved disagreements for the human. Guides replace their historical `describes` lists with current owners. The agent may not infer user approval from whichever artifact happens to be easiest to copy.

Workbench performs its own reconciliation in this change. Its existing proposal bodies remain historical records; only references that must survive a deletion need mechanical repair. `supersedes: []` on this proposal does not claim that the earlier Workbench proposals remain the current specification.

The new parser and integration dependencies are provisioned through the existing project setup and task graph. Projects with other languages retain their native toolchains and select compatible bindings. This proposal introduces no requirement to run TypeScript tests for a Swift product.

## Implications on adoption

A new project starts with a vision and the process machinery. It creates current specifications, contracts, guides, domain references, policies, and decisions only when their subject exists. The Gherkin integration is configured before the project relies on executable feature specifications.

An existing project first identifies its capability owners and current shared constraints. It then migrates the records and guides, supplies bindings, and runs its gate before switching to the new contract. A staged migration may be organized in work items, but an area still requiring proposal archaeology must be reported as unmigrated. Workbench does not claim complete current specifications while unaccounted areas remain.

The adoption guide explains the ten roles, supplies one complete worked change, shows a policy revision and retirement, and describes the binding prerequisite and historical-link handling. It contains no mandatory feature-folder taxonomy or registry of duplicated native contracts.

Previous states remain recoverable from Git. Reverting an adopted process change requires restoring compatible tooling and records together; deleting current records is safe only after the relevant historical state is committed and any required historical links are preserved.

## Scope

- Define and document the agreed artifact ownership and change process across Workbench's contract, skills, templates, guides, and vision.
- Add current Gherkin and domain-reference discovery, identities, guide relationships, and validation; support mutable current policies and decisions while retaining proposal history.
- Publish the current sources in the documentation site without committed generated specification copies.
- Adopt the model for Workbench's own exposed behavior, consume the separately selected Gherkin integration, and preserve existing test protection.
- Supply the record migration and adoption guidance, exercise the changed tooling and site, and include the implementation's appropriate changeset.

### Out of scope

- Designing or implementing the reusable Vitest and Swift Testing libraries. The owner chose separate proposals for them; Workbench's required integration remains a delivery prerequisite.
- Automatically migrating other repositories. Their current intent must be reconciled with their owners, and this proposal delivers the procedure rather than asserting knowledge of their intended behavior.
- Reintroducing SpecKit's full taxonomy, per-implementation reverse pointers, specification lock shards, or a new requirements-delta language.
- Replacing Beads, changing release authorization, adding another approval state machine, or changing the existing look-and-feel exception.
- Requiring a new architecture overview, diagram, domain file, policy, or native-contract wrapper for every feature.

## Preview

- Repository shape: documentation and process tooling, with CLI checks and a test-framework integration.
- Artifact: the existing pull-request documentation deployment plus the checkout's runnable validation and behavioral-test commands.
- Access: the existing public preview workflow publishes under `https://markmals.github.io/workbench/pr-preview/pr-9/` for this pull request. This repository already deliberately publishes its guides and records. The implementer verifies the actual deployment before handing it off.
- Exercise: open a current feature, its guide, and its domain reference without following proposal history; open the historical proposal separately; use a disposable fixture to revise a policy in place, reject deletion while a current reference remains, and then retire it with references repaired. Change a scenario and remove a binding in disposable fixtures to observe test-gate failures. Confirm the unchanged fixture passes.
- Cleanup: remove disposable fixtures after verification. The existing workflow manages preview lifetime when the pull request closes; any manual teardown, branch deletion, or release still needs the existing authorization.

The draft proposal can be read before those implementation previews exist. Its publication does not claim the proposed workflow is implemented.

## Policies and decisions checked

- Policies: None. `policies/` currently contains only its README.
- Decisions: None. `decisions/` currently contains only its README.
- [VISION.md](../VISION.md) requires explicit intent, independently checkable representations, preserved rationale, human authority, and ceremony that earns its cost. This proposal preserves those goals while giving current requirements explicit owners. Its artifact inventory must be updated when the implementation is accepted.
- [Proposal 0001](0001-workbench-2.md) established the proposal-centric record and supersession machinery. This proposal changes the current process and tooling without pretending the historical change never occurred.
- [Proposal 0003](0003-fit-the-harness-to-current-models.md) established inline implementation, self-check, and one independent review. Those remain.
- [Proposal 0004](0004-make-room-for-ui-polish.md) leaves look and feel to the running product. Gherkin specifications follow that boundary rather than recreating screen specifications.
- `.agents/rules/enforcement-hierarchy.md` prefers deterministic checks where possible. Identity, syntax, references, and binding completeness belong in checks; semantic fidelity and meaningful scope remain review judgments. Rationale and applicability remain available even where a check enforces the underlying constraint.

## Future directions

Generated contract diagrams or a richer specification browser could improve discovery if real projects need them. They would derive from current sources rather than create a second normative document. Additional language integrations can follow the same behavioral requirements after their own API proposals.

## Alternatives considered

### Keep proposal history and add finer-grained supersession

Requirement-level replacement links could represent partial amendments, but current-state discovery would still require interpreting a history graph. Every edit would also require maintaining that graph. Direct current specifications make the common read and edit simpler.

### Compile and commit a current specification from proposals

This retains the existing authoring model, but semantic reconciliation of prose is not a deterministic compilation step. It gives the compiled view and the proposal archive competing claims to authority. This proposal instead authors the current source directly and generates only presentation output.

### Put every specification in Gherkin

Gherkin is suitable for behavior and examples. Exact wire shapes, shared identity definitions, architectural rationale, and executable build configuration have other useful representations. Encoding them all as scenarios or feature descriptions would preserve the information only by making those files carry several unrelated responsibilities.

### Make tests and guides the only current requirements

This reduces explicit specification files, but a test implementation can assert less than the intended rule and a guide can omit an important failure or cross-cutting constraint. The human-approved behavioral and structural definitions remain independently readable sources against which those interpretations can disagree.

### Restore the full SpecKit taxonomy

Separate narratives, use cases, flows, models, errors, and view models provide places for many kinds of information. Requiring all those places per feature multiplies maintenance and can specify implementation choices that should remain local. The conditional roles in this proposal retain necessary information without requiring a document for every category.

### Remove policies and decisions or move all of them into agent rules

The owner wants to keep both concepts and revise their current files. Product guarantees and architectural reasoning have readers beyond the coding agent and lifetimes beyond one invocation. Development instructions remain in `.agents/`; current product constraints and choices retain their own records.

## Open questions

- [NEEDS CLARIFICATION: Which separately reviewed Gherkin integration release or source revision will Workbench consume, and will its record-tool behavioral tests keep node:test or migrate to the chosen framework? Resolve the integration dependency and runner choice before approving implementation; the reusable library APIs remain outside this proposal.]
