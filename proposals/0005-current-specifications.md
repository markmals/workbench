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

Keep current requirements in project-appropriate specifications, preserve proposals as change records, and revise policies and decisions in place. Document ten conditional artifact roles and supply reusable Gherkin guidance for projects that choose it. Update Workbench's process and existing tooling without introducing a specification framework.

## Motivation

Workbench makes an individual change reviewable, but its proposals accumulate requirements whose current meaning must be reconstructed. Later work commonly changes one clause of an earlier proposal while preserving the rest. Whole-document supersession cannot express that distinction without either discarding requirements that still apply or keeping conflicting statements under the same apparent authority.

Workbench demonstrates the problem itself. [Proposal 0001](0001-workbench-2.md) specifies a separate cross-artifact review. [Proposal 0003](0003-fit-the-harness-to-current-models.md) replaces that stage with a self-check. Both remain `implemented`, with `supersedes: []`. Their statuses correctly describe completed changes. Neither status tells a reader which individual requirements apply now. `PROCESS.md` already provides the current workflow, but Workbench gives proposals the role of specification and has no general current-specification layer for adopting products.

The same pattern occurs in local adopting projects. Pico's `proposals/0008-background-bash-jobs.md` changes one rule from proposal 0007: reopening a session reads the settings file for auto-background. Recon's `decisions/0005-homebrew-ad-hoc-privacy-identity.md` supersedes decision 0002 while retaining its permission-request and typed-failure behavior by reference. Reading only the newest record is insufficient in both cases.

The owner wants to preserve proposal development, human approval, tests before code, independently written guides, and review. Maintaining a second prose specification compiled from proposals would add another representation to synchronize. The requested change instead gives each current requirement an authoritative home, makes changes there directly, and preserves earlier states through Git and the change record.

Review of the first draft exposed a second problem: mandatory Gherkin fits some application workflows better than library interfaces. Pitlane needs precise public declarations and semantic API contracts for developer and agent consumers. Requiring a Gherkin counterpart for every API promise would add a translation layer without establishing that it improves understanding or verification. The owner also wants reusable Gherkin guidance available to projects that benefit from it.

## Domain grounding

### Established context

- The [Gherkin reference](https://cucumber.io/docs/gherkin/reference) distinguishes feature descriptions, general business rules, and scenarios that illustrate those rules. Descriptions can hold context and vocabulary; executing scenarios does not verify every statement in their descriptions.
- [Writing better Gherkin](https://cucumber.io/docs/bdd/better-gherkin) recommends describing observable behavior rather than interaction mechanics. That agrees with Workbench's existing separation of behavioral intent from look and feel.
- The [Gherkin parser and compiler](https://github.com/cucumber/gherkin) expose a document model and executable scenarios independently of Cucumber's runner. A project can keep Gherkin while using idiomatic test-framework bindings. Scenario outlines, backgrounds, and inherited tags have defined semantics that integrations must preserve.
- [OpenSpec's concepts](https://github.com/Fission-AI/OpenSpec/blob/main/docs/concepts.md) distinguish current specifications from proposed changes. Workbench can use that distinction with ordinary branch diffs rather than adopting a second delta-specification language.
- [Michael Nygard's architecture decision records](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions) preserve the context and consequences of significant choices. This proposal retains that information while choosing mutable current decision files and Git history for previous versions.
- Addy Osmani's _You're probably overprompting Claude_, supplied by the owner as four screenshots, recommends a clear goal, executable verification, removing instructions and adding back what is missed, and fresh-context review against the specification. It also discusses a single agent using skills. This is practitioner guidance for keeping the core process small and making specialized instructions available when relevant; the reported model evaluations were not independently verified here.

### Repository observations

The project inspection on 2026-11-12 read local checkouts, not a verified inventory of deployed behavior. The examples above establish documentation relationships only.

The local SpecKit's `internal/coreassets/templates/rules/spec-conventions.md` already requires editing existing specifications before changing behavior. Trove's `specs/protocol/troved.search.md` incorporates TV and movie search into one current wire contract. Pico's guides consolidate behavior that spans several proposals. These are existing examples of direct maintenance rather than compilation from proposal history.

Mutable files still require coherent edits. Flexscope's `features/0017-flexmac/models/walker.md` contains changes explicitly described as "flagged here, not yet propagated" to the shared node-identity model. Appending an amendment without changing its owning definition leaves the same reconstruction problem inside the specification library.

SpecKit's `docs/design/feature-loop.md` previously considered retaining contracts while consuming other specifications into tests and guides. That document is marked proposal / direction-setting. Its historical measurements and conclusions are context for this discussion, not validation of the design proposed here.

Pitlane's `packages/content/README.md` specifies that `createContent()` returns synchronously without loading entries and that `getEntry()` returns `undefined` for a missing ID. Its `packages/assets/README.md` states that looking up a module's stylesheets never compiles that module for the browser. Those are behavioral requirements expressed directly as API contracts. The declarations alone cannot communicate every such guarantee, and these guarantees do not require a Given/When/Then representation.

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

A format is separate from an artifact's role. Public declarations with semantic contract documentation can fulfill both behavioral and structural specification roles. A Gherkin rule can carry a product policy. The ten roles below account for information and responsibilities; they do not require ten independent files or a schema for every role.

### Quality bar

A reader can find a capability's current requirements without replaying proposals or following supersession chains. Each normative fact has one current owner. Tests and guides remain independent interpretations that can expose defects in that owner or in the implementation. Historical rationale remains recoverable. A change updates its affected definitions before production code and lands those definitions with the implementation.

### Remaining uncertainty

Whether a specification is clear and discoverable remains a review judgment. The implementation must demonstrate the approach with a library contract and an application workflow, using the existing representations where they suffice. Reusable Vitest and Swift Testing integration APIs will be designed separately. Their availability is not a prerequisite for this process change, and this proposal does not claim that those integrations already exist.

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

Maintain current requirements directly in the representation suited to the surface: public API contracts and declarations, native protocol schemas, current reference prose, or Gherkin rules and scenarios. Reuse an adequate existing source. Add a document only when necessary information has no adequate home. Keep policies and decisions current, and keep proposals as explanations of changes.

A feature branch contains the proposed end state. The proposal bounds the authorized change and explains its reasons. Main contains the integrated contract; a release tag identifies the contract shipped with that release. Tests, guides, and implementation change with the affected requirements. Workbench supplies optional Gherkin guidance once, while projects retain their own documentation and testing tools.

## Detailed design

### 1. Ten roles as a complete accounting

`PROCESS.md` defines the following inventory and links to concise authoring guidance. These are conditional roles. A file can fulfill more than one role, and a feature need not produce an artifact for every row. The short agent contract points to that guidance instead of repeating the inventory or loading every specialized procedure.

| Role | Current authority and expected home | Creation and maintenance trigger |
| --- | --- | --- |
| Behavioral specification | Public API contract documentation, current reference prose, or Gherkin files, organized by capability or interface. Owns observable behavior, defaults, failures, and guarantees. | Revise when required behavior changes. Reuse the source that already owns it. |
| Native contract | The subsystem's declarations, schema, protocol definition, or persisted-format definition. | Revise when its structural boundary changes. Keep the native source rather than a Markdown copy. |
| Tests | Native test locations and frameworks, including behavioral, type, property, integration, and other appropriate tests. | Verify the current contract and relevant implementation risks. Use Gherkin bindings when that representation is chosen. |
| Guides | `guides/` or the project's existing user-documentation home, written for the intended user, including consuming developers. | Revise when users must do or understand something differently. |
| Implementation | Code and executable configuration, including build definitions, tasks, environment schemas, and design tokens. | Change to realize the approved behavior or technical change. |
| Vision | `VISION.md`: purpose, audience, goals, principles, boundaries, and direction. | Revise when these change. Avoid an exhaustive feature catalogue. |
| Domain references | Short explanations near the owning specifications of shared terminology, identity, relationships, and lifecycle meanings. | Add only when existing feature or interface documentation cannot adequately explain a shared meaning. |
| Policies | `policies/<NNNN>-<slug>.md`: one continuing constraint, its scope, rationale, and enforcement. | Add after human agreement, revise in place, and delete when retired. |
| Decisions | `decisions/<NNNN>-<slug>.md`: one current significant choice, context, alternatives, and consequences. | Add after human agreement, revise in place, and delete when no longer applicable. |
| Proposals | `proposals/<NNNN>-<slug>.md`: one change, its scope, approach, and reasons. | Revise while designing and reviewing the change; preserve the completed record as history. |

The first five roles describe and realize the product. The next four provide project context. The proposal records a change to whichever current sources it affects.

Work items, changesets and changelogs, review evidence, previews, and process machinery retain their existing purposes. Separate narrative, use-case, flow, error-catalogue, view-model, toolchain-prose, and architecture-overview documents are not required. A useful diagram can belong in a domain explanation or decision; an ERD can be generated from a schema. Look and feel remains subject to the existing running-product review.

### 2. Choose the representation by surface

Each normative fact has one current owner. A library can specify its public surface through declarations and contract documentation covering defaults, errors, side effects, ordering, lifecycle, and compatibility. A protocol can use a native schema with semantic requirements alongside it. An application workflow can use Gherkin or a current behavioral reference. One project may use all of these without selecting a project-wide mode.

Contract text and type expectations are written or revised before implementation. Documentation colocated with source code can state intended behavior independently of the function body. Generated API reference publishes that source; it does not infer intended requirements from the implementation. A guide explains how to accomplish a task, while a reference states the contract on which that task depends.

For Pitlane, changing `getStylesheets()` means revising its authoritative API contract and the relevant native tests, then updating usage guidance where affected. There is no required `.feature` counterpart. For a project already using Gherkin, the same process updates its existing feature and bindings. Tests that only confirm a document exists, or that a test carries an identifier, do not establish behavioral conformance.

Native tests, type checks, executable examples, consumer fixtures, and other checks can all provide evidence. Requirements involving security, convergence, performance, or human judgment require evidence appropriate to the claim. Reports state what was exercised and any limitations. Workbench introduces no universal scenario registry, coverage-report format, or automated/manual/excluded state machine.

When current sources disagree, raise the conflict with the human. The approved proposal bounds the authorized delta, and the branch's specifications state the resulting contract. A new substantive decision discovered while elaborating them requires human input. Completed proposals do not override current definitions. Existing proposal approval ownership and the correction and polish paths remain unchanged.

### 3. Reusable, optional Gherkin guidance

Ship `.agents/skills/specifying-behavior-with-gherkin/` with a focused skill, linked examples, and an integration-adoption procedure. Add `.agents/templates/FEATURE.feature` and document its destination and use in the template inventory. The skill is discovered through the existing skill mechanism and triggered when authoring or maintaining `.feature` files, or when the human chooses Gherkin for a surface. The core process points to it without loading the full procedure for unrelated work.

The skill teaches:

- Selecting Gherkin where rules and examples help, and selecting API contract documentation where that communicates the requirement more directly.
- Organizing one coherent capability per feature file, stating general rules, and using scenarios to illustrate outcomes and boundaries rather than incidental UI mechanics.
- Using feature descriptions, backgrounds, scenario outlines, examples, and data tables according to standard Gherkin semantics. Examples include success and failure behavior without claiming exhaustive verification.
- Revising the owning requirement before implementation, then connecting it to real tests and preserving useful tests that need no Gherkin counterpart.
- Sharing specifications across implementations where the same contract applies, making platform differences explicit, and reporting unexecuted behavior honestly.
- Distinguishing binding completeness from assertions that actually establish the promised behavior.

Provide a worked end-user workflow with a failure case and scenario outline, and a developer workflow example that explains why Gherkin is useful for that surface. Include the Pitlane API-contract contrast so the examples do not imply that all developer-facing behavior needs Gherkin. These examples are teaching resources within the skill and guides; they do not introduce production requirements or a second specification library.

For a new project choosing Gherkin, recommend `specs/features/<slug>.feature` with capability-oriented names. Respect an existing coherent location. Use standard Gherkin and the selected integration's naming and identity conventions. Workbench does not require `@feature` or `@scenario` tags, a new metadata schema, or implementation reverse pointers.

The shared adoption procedure covers choosing and pinning a real compatible integration, using its documented binding API, wiring its check into the project's existing test task, and verifying failures as well as successful execution. For automated Gherkin coverage, missing or ambiguous bindings fail, outlines execute their applicable examples, and failures identify the relevant specification. Changed steps must not leave unrelated tests green solely because an old identifier still matches. Deliberate exclusions and manual verification remain visible in the existing review evidence rather than being reported as automated passes.

Framework-specific instructions belong with the corresponding integration and are linked from the shared guidance when available. This proposal neither invents APIs for the separate Vitest and Swift Testing libraries nor requires projects to install an unused runner. A project must use an actual working integration before claiming executable Gherkin coverage. Workbench's own process and record changes can be delivered using its current `node:test` and Vitest suites.

### 4. Current records, ordinary links, and existing checks

Keep a domain explanation only when it resolves meaning that would otherwise be ambiguous across requirements. It is ordinary documentation, with no required Workbench ID, frontmatter schema, status, or file-per-type rule. An existing explanation can be extended rather than duplicated into a new domain directory.

Policies and decisions keep their current numbered filenames and IDs. Revise the same logical record when its meaning changes; delete it when it no longer applies. Update affected current references in the same change, and do not recycle a retired identity for a different record. Supported compatibility obligations and incomplete migrations remain applicable constraints.

Required policy and decision frontmatter becomes `id`, `title`, and `status`. Policy statuses are `draft` and `active`; decision statuses are `proposed` and `accepted`. Proposed records stay on the change branch until agreed. `established-by` becomes optional historical provenance and, when present, retains its existing reference check. Remove `supersedes` and `superseded` from policy and decision schemas and templates; reject those obsolete forms with migration guidance. Preserve proposal supersession and its existing validation as historical change relationships.

The policy template keeps its rule or link to the authoritative owner, scope and exceptions, rationale, and enforcement. The decision template keeps the current arrangement and choice, context, rationale, alternatives, and consequences. Remove mandatory in-document revision logs and the prohibition on revising or deleting current records. Product guarantees retain their product ownership; `.agents/rules` continues to hold development instructions.

Use ordinary links to make current requirements discoverable from the relevant guides and developer entry points. Remove the required `describes` field from the guide template and schema, migrate Workbench's guides to links to their actual current sources, and remove that relationship from the record header. Reject legacy guide `describes` metadata with guidance to use ordinary links; optional historical provenance can be written as prose. Guides keep their existing `id` and `title`. No replacement ID graph is introduced.

Retain existing frontmatter, identity, historical-proposal-reference, clarification-marker, and documentation-link checks where applicable. `mise run validate` changes only the affected record contracts; it does not become a multi-format specification scanner. Gherkin and native-contract checks belong to the chosen project tools. When deleting a current record would break a useful historical link, replace that link with an immutable Git revision URL rather than keeping the retired rule in the current record.

### 5. The change process

Update the existing skills and contract without adding phases or approval states:

1. During proposal development, identify the current sources affected by the intended change and any missing information. Explain the delta, scope, alternatives, approach, and compatibility work. Link the current owners rather than maintain a second complete contract in Detailed design.
2. After human approval, revise the affected specifications and agreed records before production code. Reuse their existing representation, or add the smallest adequate source for missing requirements. Resolve new substantive decisions with the human.
3. Write and observe the relevant failing tests, then write or revise guides from the intended contract and compare them with the tests. Implement against those sources. Preserve applicable native tests and the existing look-and-feel boundary.
4. Run the shared gate. Self-check the authorized delta, affected requirements, tests, guides, and implementation. Give the independent reviewer the current sources and relevant unchanged constraints alongside the proposal. Verification reports identify actual evidence and remaining limitations.
5. Provide the preview and readiness report through the existing coupled handoff. After human acceptance, finalize affected records and the vision, record `implemented`, and merge the current sources with their implementation. Release and cleanup keep their existing authorization rules.

`tools/status.ts` retains its phase derivation and ownership rules. Its implementation and review guidance names current-specification maintenance and verification in appropriate project-native forms. A correction implements the current requirement; an underspecified behavior requiring a decision returns to proposal development. The agent cannot approve a requirement by documenting whatever the implementation happens to do.

### 6. Workbench adoption and completion

Update `AGENTS.md`, `PROCESS.md`, `VISION.md`, affected skills, templates, directory READMEs, and guides to agree on the current-record model. Keep the core instructions concise and place optional Gherkin detail in its skill and resources. The adoption guidance covers the ten conditional roles and demonstrates a library contract change, a Gherkin workflow change, and a policy revision or retirement.

Reconcile Workbench's current requirements using its existing process documents, tool documentation, tests, guides, and relevant proposals. Establish where current workflow and public-tool contracts are read, repair stale guidance and fill material gaps, and raise disagreements with the human. Preserve adequate sources and tests. Neither converting every suite to Gherkin nor writing a separate domain model for concepts already explained in the process is required.

Use the existing documentation site to publish the revised process and guidance and link to current requirement owners. Make proposal history identifiable as history, keep deleted current records out of current navigation, and remove obsolete policy/decision supersession presentation and guide `describes` display. Historical proposal navigation remains. Gherkin examples can use ordinary code blocks or source links; this change requires no custom renderer, format plug-in system, or committed generated specification.

Completion requires the revised process, reusable Gherkin skill and teaching resources, migrated current records and guide references, and the working changes to existing tooling and navigation. Verify the changed record contracts and status behavior with native tests and exercise the documentation preview. A valid policy using the new schema must pass without supersession metadata, obsolete policy/decision metadata must produce migration errors, and removing a still-linked current document must surface the broken reference through the applicable existing check. Preserve test protection; do not substitute tests of prose wording for observable behavior.

The reusable Vitest and Swift Testing libraries remain separate work. They add no blocking prerequisite, runner migration, or installation dependency to this proposal.

## Compatibility

This changes policy and decision metadata and removes the required historical `describes` relationship from guides. Adopters migrate those records and their validator together. Obsolete metadata receives an actionable migration error. Proposal files retain their historical lifecycle and relationship fields, and their bodies continue to explain the changes made at the time.

For each retired policy or decision chain, the human and agent determine the complete rule or choice that still applies, incorporate retained clauses into the surviving logical record, remove supersession metadata, and delete retired current files. Keep the surviving ID where possible. Update current references and replace necessary historical links with immutable Git links before deletion. The process does not assume that the newest record already contains every retained clause.

For current requirements, reconcile approved intent with existing contract documentation, tests, guides, and relevant proposals. Reuse adequate sources and fill identified gaps rather than restating every historical sentence. A guide links to the actual current owner; it need not acquire a new specification ID or repeat that owner's contract. The agent raises disagreements with the human rather than choosing whichever source is easiest to copy.

Workbench performs its own reconciliation in this change. Its existing proposal bodies remain historical records; only references that must survive a deletion need mechanical repair. `supersedes: []` on this proposal does not claim that the earlier Workbench proposals remain the current specification.

There is no new mandatory Gherkin parser, runner, or general specification registry. Projects retain their native toolchains, test frameworks, and adequate specification formats. A project choosing Gherkin follows the supplied guidance and installs a real integration appropriate to that surface.

## Implications on adoption

A new project starts with its vision and process machinery, then adds current specifications and supporting records as their subjects arise. It receives the reusable Gherkin skill and template with Workbench. Using that guidance is conditional on the surface being specified; ordinary library API contracts require no Gherkin setup.

An existing project identifies where its current requirements are owned, repairs gaps and conflicts, migrates policy/decision metadata and guide links, and verifies the changes through its existing gate. It preserves useful native tests and documentation. Work items can track a staged migration, but any area still requiring proposal archaeology is disclosed as unmigrated.

The adoption guide gives the complete accounting without requiring ten documents per feature. Its worked examples show the same change process applied to Pitlane-like API semantics, a Gherkin workflow, and a mutable policy. Specialized Gherkin authoring and integration instructions are maintained once and linked from that guide.

Previous states remain recoverable from Git. Reverting an adopted process change requires restoring compatible tooling and records together; deleting current records is safe only after the relevant historical state is committed and any required historical links are preserved.

## Scope

- Define current-specification ownership and the conditional ten-role inventory across Workbench's process, skills, templates, guides, and vision.
- Supply the reusable optional Gherkin skill, feature template, worked examples, and integration-adoption procedure.
- Support mutable policies and decisions, migrate guide relationships to ordinary links, and update the affected validator, status guidance, and record presentation.
- Reconcile Workbench's current requirements and document adoption using existing sources and testing tools; verify the changed tooling and documentation.

### Out of scope

- Designing or implementing the reusable Vitest and Swift Testing libraries. They remain separate proposals and are not prerequisites for this change.
- Automatically migrating other repositories. Their current intent must be reconciled with their owners, and this proposal delivers the procedure rather than asserting knowledge of their intended behavior.
- A universal specification schema, cross-artifact ID registry, format plug-in framework, Gherkin renderer, reverse-pointer system, or specification lock files.
- Replacing Beads, changing release authorization, adding another approval state machine, or changing the existing look-and-feel exception.
- Requiring a new architecture overview, diagram, domain file, policy, or native-contract wrapper for every feature.
- Mandatory conversion of existing contracts or tests to Gherkin, or a new project-wide application/library mode.

## Preview

- Repository shape: documentation and process tooling with existing CLI checks and native test suites.
- Artifact: the existing pull-request documentation deployment and runnable record-validation checks.
- Access: the existing public preview workflow publishes under `https://markmals.github.io/workbench/pr-preview/pr-9/` for this pull request. This repository already deliberately publishes its guides and records. The implementer verifies the actual deployment before handing it off.
- Exercise: find the current contract for a Workbench workflow or tool without replaying proposal history. Follow the library, Gherkin, and policy worked changes. Find the reusable Gherkin skill and examples without configuring it for unrelated work. In a disposable fixture, validate an active policy with the new schema, observe migration errors for obsolete metadata, and retire a record with its current links repaired. Check that historical proposal navigation remains available.
- Cleanup: remove disposable fixtures after verification. The existing workflow manages preview lifetime when the pull request closes; any manual teardown, branch deletion, or release still needs the existing authorization.

The draft proposal can be read before those implementation previews exist. Its publication does not claim the proposed workflow is implemented.

## Policies and decisions checked

- Policies: None. `policies/` currently contains only its README.
- Decisions: None. `decisions/` currently contains only its README.
- [VISION.md](../VISION.md) requires explicit intent, independently checkable representations, preserved rationale, human authority, and ceremony that earns its cost. This proposal preserves those goals while giving current requirements explicit owners. Its artifact inventory must be updated when the implementation is accepted.
- [Proposal 0001](0001-workbench-2.md) established the proposal-centric record and supersession machinery. This proposal changes the current process and tooling without pretending the historical change never occurred.
- [Proposal 0003](0003-fit-the-harness-to-current-models.md) established inline implementation, self-check, and one independent review. Those remain.
- [Proposal 0004](0004-make-room-for-ui-polish.md) leaves look and feel to the running product. Current specifications, including optional Gherkin, follow that boundary.
- `.agents/rules/enforcement-hierarchy.md` prefers deterministic checks where they enforce a concrete requirement. Preserve the applicable record and documentation checks, and use project-native verification for current contracts. Meaning, adequate coverage, and the usefulness of a new artifact remain review judgments.

## Future directions

Reusable Vitest and Swift Testing integrations can supply their own binding APIs and framework-specific documentation. Workbench's Gherkin guidance can link to them when available. Further checks or publishing support should respond to an observed failure or a project's actual documentation needs.

## Alternatives considered

### Keep proposal history and add finer-grained supersession

Requirement-level replacement links could represent partial amendments, but current-state discovery would still require interpreting a history graph. Every edit would also require maintaining that graph. Direct current specifications make the common read and edit simpler.

### Compile and commit a current specification from proposals

This retains the existing authoring model, but semantic reconciliation of prose is not a deterministic compilation step. It gives the compiled view and the proposal archive competing claims to authority. This proposal instead authors the current source directly and generates only presentation output.

### Put every specification in Gherkin

Gherkin is useful for rules and examples where that is an effective review format. Pitlane's public API semantics are already expressed directly in contract documentation, declarations, and native tests. Requiring another representation for every surface would add maintenance without demonstrated benefit. This proposal retains Gherkin guidance while allowing the project's current sources to carry the contract.

### Make tests and guides the only current requirements

This reduces explicit specification files, but a test implementation can assert less than the intended rule and a guide can omit an important failure or cross-cutting constraint. The human-approved behavioral and structural definitions remain independently readable sources against which those interpretations can disagree.

### Restore the full SpecKit taxonomy

Separate narratives, use cases, flows, models, errors, and view models provide places for many kinds of information. Requiring all those places per feature multiplies maintenance and can specify implementation choices that should remain local. The conditional roles in this proposal retain necessary information without requiring a document for every category.

### Remove policies and decisions or move all of them into agent rules

The owner wants to keep both concepts and revise their current files. Product guarantees and architectural reasoning have readers beyond the coding agent and lifetimes beyond one invocation. Development instructions remain in `.agents/`; current product constraints and choices retain their own records.

### Remove Gherkin guidance to simplify the core

Removing reusable guidance would make each adopting project reconstruct its own procedure. A focused skill, template, examples, and adoption notes preserve that knowledge without requiring unrelated work to load it or install an integration.

### Build a general specification framework

A common registry, metadata schema, renderer, and binding-report protocol could coordinate many formats, but the observed problem is finding and maintaining current requirements. Ordinary source ownership, links, native checks, and the existing review process address that problem with fewer conventions. New machinery needs evidence that these mechanisms fail.

## Open questions

None. The owner selected format-neutral current specifications with reusable optional Gherkin guidance. The former integration-selection prerequisite has been removed; the reusable libraries remain separate work. This proposal stays `draft` until the human approves implementation.
