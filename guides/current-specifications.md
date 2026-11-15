---
id: guide.current-specifications
title: Keeping Current Specifications
---

# Keeping Current Specifications

## Where do I read what the project requires now?

Read the source that owns the surface. A library's contract lives in its public declarations and the documentation beside them, a protocol's in its schema, and an application workflow's in reference prose or a Gherkin feature file. Proposals explain how those requirements changed. A completed proposal is history: it never overrides the current source, and nobody should have to replay a chain of proposals to learn what applies today.

Each normative fact has one current owner. A change edits that owner directly, on its branch, before production code. A feature branch holds the proposed end state, the default branch holds the integrated contract, and a release tag identifies the contract that shipped. Git keeps every earlier state, and the proposal keeps the reasons.

## Ten roles, not ten documents

[The process](../PROCESS.md#artifacts) defines ten roles that together account for what a project needs to know about itself: behavioral specification, native contract, tests, guides, implementation, vision, domain references, policies, decisions, and proposals. The first five describe and realize the product. The next four give project context. A proposal records a change to whichever of the others it affects.

The roles are conditional. One file can fill several of them, and a feature needs an artifact only for the roles it actually touches. Public declarations with semantic documentation can be both the behavioral specification and the native contract. A Gherkin rule can carry a product policy.

| Role | How to write and keep it |
| --- | --- |
| Behavioral specification | State observable behavior, defaults, failures, and guarantees, organized by capability or interface. Reuse the source that already owns the behavior. |
| Native contract | Keep declarations, schemas, protocol definitions, and persisted formats in their native form rather than copying them into Markdown. |
| Tests | Verify the current contract and the implementation risks that matter, in the project's own frameworks. Name each test after the behavior it checks. |
| Guides | Explain how to accomplish a task for the intended reader, including developers consuming an API. Link to the reference that states the contract rather than restating it. |
| Implementation | Code and executable configuration: build definitions, tasks, environment schemas, design tokens. |
| Vision | Purpose, audience, goals, principles, boundaries, and direction. Leave out the feature catalogue. |
| Domain references | A short explanation of a shared term, identity, relationship, or lifecycle, placed near the specifications that use it. Write one only when existing feature or interface documentation leaves a shared meaning ambiguous. It needs no ID, status, or frontmatter schema. |
| Policies and decisions | Current files, revised in place and deleted when they no longer apply. [Keeping the Record](./keeping-the-record.md) covers when to write one. |
| Proposals | One change, its scope, approach, and reasons. Revise it while the change is designed and reviewed; afterward it is the record of that change. |

None of this requires separate narrative, use-case, flow, error-catalogue, view-model, toolchain, or architecture-overview documents, and none of it introduces a registry, metadata schema, or pointer from code back to a specification. A useful diagram can sit in a domain reference or a decision. An entity diagram can be generated from a schema.

## Choose the representation by surface

One project can use several representations without choosing a project-wide mode:

| Surface | Current specification |
| --- | --- |
| Library or SDK | Public declarations and the contract documentation beside them: defaults, errors, side effects, ordering, lifecycle, compatibility. |
| Protocol, wire format, or stored data | The native schema, with semantic requirements next to it. |
| Application workflow | Gherkin rules and scenarios, or current reference prose. |
| Process or tooling | Reference prose and the tool's own documentation. |

Contract text and type expectations are written or revised before implementation. Documentation colocated with source can state intended behavior independently of the function body, and generated API reference publishes that source; it does not infer requirements from the code. Look and feel stays out of every representation, as it stays out of proposals.

## Where Workbench's own requirements live

| Subject | Current owner |
| --- | --- |
| The workflow, its phases, roles, and approval points | [The process](../PROCESS.md) |
| `mise run validate` and `mise run status` | [Record Validation and Workflow Status](./record-validation.md) |
| What an agent must do in a session | `AGENTS.md` and the skills under `.agents/skills/` |
| The shape of each record | `.agents/templates/` |
| Tool and hook specifics | Source documentation and native tests in `tools/` and `.omp/tools/dynamic-workflows/` |
| Purpose and direction | [The vision](../VISION.md) |
| Why any of these changed | The proposals and their pull requests |

## How a change moves through the sources

1. In the proposal, identify the current sources the change affects and any missing information. Link those owners and explain the delta, scope, alternatives, approach, and compatibility work. Detailed design describes the change; it does not maintain a second complete contract.
2. After the human approves the proposal, revise the affected specifications and any agreed policies or decisions before production code. Reuse their representation, or add the smallest adequate source for a requirement with no home. A new substantive decision goes back to the human.
3. Write failing tests against the revised contract and watch them fail. Write or revise guides from the intended contract and compare them with the tests. Then implement.
4. Run the gate. Self-check the authorized delta against the specifications, tests, guides, and code, then give the independent reviewer the current sources and relevant unchanged constraints alongside the proposal.
5. Post the preview and readiness report. After human acceptance, finalize the affected records and the vision, record `implemented`, and merge the sources with their implementation.

When two current sources disagree, or a source disagrees with approved intent, raise it with the human. The proposal bounds what is authorized; the branch's specifications state the result. Documenting whatever the implementation happens to do does not approve it.

## Worked example: a library contract

An asset package exposes `getStylesheets()`. An approved proposal changes it to return stylesheets in dependency order and to reject an unknown module instead of returning an empty array. The contract lives in the declaration's documentation, which the generated API reference publishes, so that is the first edit:

```diff
 /**
- * Returns the URLs of the stylesheets a module needs. Returns an empty
- * array when `moduleId` is not in the module graph.
+ * Returns the URLs of the stylesheets a module needs, in dependency order:
+ * a stylesheet imported by a dependency comes before those of the modules
+ * that import it.
  *
  * Looking up stylesheets never compiles the module for the browser.
+ *
+ * @throws {UnknownModuleError} when `moduleId` is not in the module graph.
  */
 export function getStylesheets(moduleId: string): string[];
```

The unchanged guarantee about compilation stays where it was; the delta is the ordering and the error. If the package README restates the contract, revise it in the same edit or replace the restatement with a link to the reference, so the fact keeps one owner.

Next come failing tests named for the behavior: `returns a dependency's stylesheets before its importer's` and `throws UnknownModuleError for a module outside the graph`. The existing test that lookup never compiles the module stays as it is. Then the usage guide changes where a reader must now handle the error, and links to the reference instead of repeating it. Then the implementation.

There is no `.feature` file in this change. Given/When/Then would restate the same promise in another form without making it clearer or better verified.

## Worked example: an application workflow in Gherkin

An application keeps its report export workflow in `specs/features/report-export.feature`. An approved proposal adds spreadsheet export beside CSV. The feature file is the current owner, so it changes first, turning the single scenario into an outline over both formats:

```gherkin
Feature: Export a report
  Project members download a report's rows to use in other tools.

  Rule: A member can export any report they can view

    Scenario Outline: Export in a supported format
      Given Dana can view the "Q3 revenue" report
      When Dana exports "Q3 revenue" as <format>
      Then Dana receives a <format> file containing the report's rows

      Examples:
        | format |
        | CSV    |
        | XLSX   |

  Rule: A report with no rows is not exported

    Scenario: Export an empty report
      Given the "Q4 forecast" report has no rows
      When Dana exports "Q4 forecast" as CSV
      Then Dana is told the report has no rows to export
      And no file is downloaded
```

The scenarios describe what Dana can do and is told, not which button she presses. The project's existing Gherkin integration runs the new XLSX example through its bindings, and it fails until the implementation exists. The guide gains the new format, and the code follows.

A project uses Gherkin only where rules and examples communicate a requirement well, and only with a real integration wired into its test task before it claims executable coverage. For syntax, file layout, bindings, and adopting an integration, use the skill in `.agents/skills/specifying-behavior-with-gherkin/`. Workbench itself specifies its process in prose and its tools in source documentation and native tests.

## Worked example: revising and retiring a policy

`policies/0003-document-exported-functions.md` requires a runnable example in the doc comment of every exported function. The human agrees to narrow it to `packages/core`. Edit the same file: its rule, its scope, and the lint configuration that enforces it change in one commit. The ID stays `policy.0003` and the status stays `active`. No replacement file and no `supersedes` entry are written; Git and the proposal hold the earlier wording and why it changed.

`policies/0001-use-yarn.md` no longer applies because the project moved to pnpm. Retire it by deleting the file:

1. Find every link to it, for example with `rg 'policies/0001-use-yarn'`.
2. Repoint or remove current references. A guide that linked it now describes pnpm and links whatever owns that requirement today.
3. Where a historical link must survive, such as a completed proposal that cited the policy, replace it with an immutable Git revision URL. Before the deletion, `git log -1 --format=%H -- policies/0001-use-yarn.md` prints a commit that still contains the file:

    ```md
    [policy.0001](https://github.com/acme/app/blob/4f2c9e1d7a3b5c6e8f9a0b1c2d3e4f5a6b7c8d9e/policies/0001-use-yarn.md)
    ```

    A completed proposal receives no edit beyond that link repair.

4. Delete the file in the same change, and do not reuse `policy.0001` for a different rule.
5. Run `mise run check`. The documentation build reports any link the search missed.

## Adopting this in an existing project

1. Find where each capability's current requirements are owned today: contract documentation, schemas, tests, guides, and the proposals that changed them.
2. Reconcile approved intent with those sources. Reuse what is adequate, fill the gaps, and raise disagreements with the human instead of copying whichever source is easiest.
3. Migrate policy and decision supersession chains into current records, following [Keeping the Record](./keeping-the-record.md).
4. Replace each guide's `describes` field with ordinary links to the sources it explains. Historical provenance, when useful, can be a sentence of prose.
5. Verify with the existing gate.

A large archive can migrate in stages tracked as work items. Any area that still needs proposal archaeology to understand is disclosed as unmigrated rather than presented as specified.

## Evidence

Native tests, type checks, executable examples, consumer fixtures, and other checks all count as evidence. Claims about security, convergence, performance, or human judgment need evidence suited to the claim. A test that only confirms a document exists, or that a test carries an identifier, establishes nothing about behavior. Readiness reports say what was exercised and what remains unverified. Workbench adds no scenario registry, coverage format, or state machine for automated, manual, and excluded checks.
