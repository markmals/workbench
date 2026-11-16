---
name: specifying-behavior-with-gherkin
description: Use when authoring or maintaining Gherkin `.feature` files or their step bindings, when the human chooses Gherkin (Given/When/Then, Cucumber) for a surface, or when deciding whether behavior belongs in Gherkin scenarios or in API contract documentation.
---

# Specifying Behavior with Gherkin

Gherkin is one optional representation for a behavioral specification. A feature file states a capability's rules, its scenarios illustrate those rules with concrete examples, and a test integration executes the examples. Choose it per surface.

## Choose the representation

| Surface                                 | Owner of the current requirement                                                                              |
| --------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| Library function or type                | Declarations plus contract documentation: defaults, errors, side effects, ordering, lifecycle, compatibility. |
| Workflow across actors, steps, or state | Gherkin, when the human chooses it or rules and examples help reviewers; otherwise current reference prose.   |
| Protocol or persisted format            | The native schema plus its semantic requirements.                                                             |

Revise whichever source already owns a requirement rather than copying it into a `.feature` file. `.workbench/skills/specifying-behavior-with-gherkin/examples.md` contrasts Pitlane's `getStylesheets()` contract with two workflows where Gherkin helps.

## Write the feature

Start from the feature template: `.agents/templates/FEATURE.feature` when the project overrides it, otherwise `.workbench/templates/FEATURE.feature`. A new project puts features at `specs/features/<capability>.feature`; an existing coherent location takes precedence.

- One capability per file, named for that capability.
- The description holds purpose, vocabulary, and scope. Nothing executes it.
- One `Rule:` per business rule. Its scenarios illustrate outcomes and boundaries, including refusals and failures.
- Steps state domain events and observable outcomes. Incidental selectors and helper calls belong in bindings. Keep exact commands, inputs, or outputs when they are part of the public contract.
- Identity follows the selected integration's conventions. Workbench requires no tags or scenario IDs.

Standard semantics, from the [Gherkin reference](https://cucumber.io/docs/gherkin/reference):

| Construct                        | Meaning                                                                                                                           |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `Background:`                    | Given steps run before each scenario of its Feature or Rule. One per level.                                                       |
| `Scenario Outline:`, `Examples:` | One scenario per row. `<column>` placeholders are replaced before steps are matched.                                              |
| Data table, doc string           | Attached to the step above; its binding receives it.                                                                              |
| Tags                             | Inherited by the Rules, scenarios, and Examples beneath them.                                                                     |
| `And`, `But`, `*`                | `And` and `But` continue the preceding step kind; `*` is a wildcard step keyword. The selected integration defines step matching. |

Examples illustrate a rule at chosen points. They do not prove it for every input. Security, concurrency, and performance claims need evidence suited to the claim.

## Change behavior

1. Revise the owning rule and scenarios before production code. Take any gap the approved change leaves open to the human instead of settling it in a scenario.
2. Update the bindings and watch the changed scenarios fail.
3. Implement until they pass. Keep native tests for what the feature does not show, such as types, properties, and internal edge cases.
4. When implementations share a contract, each binds the same feature. State platform differences as their own rules, with a description and tag saying where they apply.

## Report what ran

A scenario is automated evidence only when a real integration ran it in the project's test task and its `Then` bindings assert the stated outcome. Matching every step establishes binding completeness. After a step changes, a tag or test name that still matches proves nothing about the new requirement. Unknown step text must fail; recognized parameter changes must reach the test inputs and assertions. List excluded and manually checked scenarios as such in the review evidence.

Before claiming executable coverage in a project, follow `.workbench/skills/specifying-behavior-with-gherkin/integration-adoption.md`.
