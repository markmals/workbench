---
name: writing-acceptance-criteria
description: Use when drafting or reviewing the acceptance criteria section of a proposal, especially when a change must be expressed through observable outcomes and its behavior may require scenarios.
---

# Writing Acceptance Criteria

Acceptance criteria describe one observable capability. They state what a person can do, see, receive, or experience — not how the project makes it happen.

**Core principle: imagine it is 1922.** If a criterion needs a UI element name, endpoint, data row, framework noun, or storage detail, it describes an implementation rather than the outcome.

This is a sub-skill of `writing-a-proposal`. Write only the proposal's **Acceptance criteria** section.

## Choose the form

| Behavior                                                       | Form                                      |
| -------------------------------------------------------------- | ----------------------------------------- |
| Has meaningful starting state, alternate outcomes, or branches | Gherkin scenarios                         |
| Is a simple, unbranched observable outcome                     | Numbered list                             |
| Contains unrelated capabilities                                | Split the proposal before choosing a form |

Do not force Gherkin onto a simple change. Do not use a numbered list to hide branching behavior.

## One capability

One set of criteria covers one user-observable capability. Split it when the criteria add unrelated verbs, such as sharing, revoking, and auditing.

More than roughly six criteria is a sizing signal. Revisit the proposal's scope rather than burying separate changes in a long list.

## Gherkin rules

| Part  | Must contain                                | Must not contain                                             |
| ----- | ------------------------------------------- | ------------------------------------------------------------ |
| Given | State that exists before the trigger        | An action or user intent                                     |
| When  | Exactly one user action or external trigger | A chain of actions, `And`, or a branch                       |
| Then  | Only outcomes a person can observe          | Internal work, implementation state, or a technical response |

Use `And` or `But` only to add state to `Given` or outcomes to `Then`. A second trigger is a second scenario.

### Worked Gherkin example

```md
## Scenario: A recipient opens a shared document

- Given Pat has shared a document with Rowan as a viewer
- When Rowan accesses the document
- Then Rowan can read the document
- But Rowan cannot change it

## Scenario: A person without access attempts to open the document

- Given Pat has shared a document with Rowan as a viewer
- When Alex attempts to open the document
- Then Alex is told they do not have access
```

The capability is controlled reading of a shared document. The state and alternative outcome make scenarios useful.

## Numbered-list rules

State the observable outcomes directly. Each item must be independently understandable and testable.

### Worked numbered-list example

```md
1. A person can access the project's support information.
2. The support information tells the person how to request help.
```

The behavior has no meaningful state or branch. A list is clearer than ceremonial scenarios.

## Keep the language observable

| Write this                                       | Not this                                 |
| ------------------------------------------------ | ---------------------------------------- |
| “A person requests a copy of their information.” | “The person clicks the export button.”   |
| “The person receives a copy they can keep.”      | “A background job writes a CSV file.”    |
| “The person is told they cannot continue.”       | “The request returns a technical error.” |
| “A recipient can read the document.”             | “A permission record is created.”        |

Name the intent, not its presentation or mechanism. A criterion may name a domain concept the person recognizes; it must not prescribe its implementation.

## Red flags — stop and rewrite

| Red flag                                                                 | Why it fails                   | Rewrite                                         |
| ------------------------------------------------------------------------ | ------------------------------ | ----------------------------------------------- |
| UI element names, layout, clicks, taps, or fields                        | Prescribes presentation        | Describe the person’s intent                    |
| Endpoints, technical responses, frameworks, data rows, queues, or caches | Describes internal machinery   | State the observed result                       |
| Action in `Given`                                                        | Setup has become the trigger   | Move it to one `When` or rewrite it as state    |
| More than one action in `When`                                           | The scenario hides a sequence  | Collapse to one intent or split scenarios       |
| `OR` or an alternate outcome in one step                                 | The scenario contains a branch | Write a scenario per outcome                    |
| A `Then` no person can perceive                                          | It cannot describe acceptance  | State what the person sees, receives, or can do |
| Several unrelated verbs in one section                                   | More than one capability       | Split the proposal                              |

## Anti-patterns

- **Criteria as a build plan.** Omit presentation, mechanisms, and technical choices.
- **Gherkin by habit.** Choose it for state or branches, not ceremony.
- **A technical assertion disguised as an outcome.** Replace it with the person-visible consequence.
- **Compound triggers.** One `When` has one trigger.
