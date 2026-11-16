# Adopting a Gherkin Test Integration

Follow this procedure before claiming that a `.feature` file runs as tests. Workbench supplies no Gherkin runner or binding API. The project selects an existing integration and writes its bindings from that integration's documentation.

## 1. Choose where steps act

For each feature, decide the layer its bindings drive: a domain service, a public API, a CLI process, an HTTP endpoint, or a browser. Use the lowest layer at which the feature's user could observe the outcomes. That layer's mechanics live in the bindings, never in the steps.

## 2. Select an integration

Prefer one that runs inside the project's existing test framework. [Cucumber's implementation list](https://cucumber.io/docs/installation/) covers official runners and community integrations, and each test framework's ecosystem lists more. Check that a candidate:

- supports every Gherkin construct your features use: `Rule`, Rule-level `Background`, outlines with several `Examples`, data tables, doc strings, and tags;
- implements standard Gherkin semantics, whether through Cucumber's [Gherkin parser](https://github.com/cucumber/gherkin) or another compatible parser;
- is maintained and supports the project's language, runtime, and test framework versions;
- can fail the run on undefined and ambiguous steps, if necessary through a strict option.

When two candidates fit, run step 5 against both before choosing.

## 3. Pin and document it

Install it as a development dependency at the exact version the lockfile resolves. Link that version's documentation from wherever the project documents its tests. Write bindings from that documentation, not from memory of another version or a similar library.

## 4. Wire it into the existing test task

Run features through the task the gate already runs, such as `mise run check`, `pnpm test`, or `swift test`. If the integration generates test files from features, make generation part of that task and handle the output as its documentation directs. A feature that runs only under a separate manual command is outside the gate.

## 5. Prove that it fails

Make each change below in a scratch copy, run the project's test task, record what happened, and revert.

| Change | Required result |
| --- | --- |
| Add a step that no binding matches | The task fails and names the feature file and step. |
| Add a second binding that matches an existing step | The task fails and reports the ambiguity. |
| Replace a bound step with text that no binding recognizes, preserving any scenario identifier | The task fails rather than running only the test formerly linked to the identifier. |
| Change the expected value in one `Then` step | That scenario fails at that step, and the report names its feature file and scenario. |
| Change the expected value in one outline row | Only that row fails, and the report shows one result per row. |
| Remove the binding for a `Background` step | The task fails, reports the missing step, and does not count affected scenarios as passed. |
| Exclude one scenario with the integration's filter or skip mechanism | The report shows it as skipped or excluded, not passed. |

If a result differs from the table, enable the integration's strict option, correct its configuration, or choose another integration. Until the check behaves as required, report that gap and make no automated-coverage claim that depends on it.

## 6. Write bindings that assert

Each `Then` binding compares the observable outcome with the value its step states. Review bindings as tests: would the scenario still pass if the behavior were wrong? An empty body, a log statement, or an assertion on a value the binding itself set establishes nothing, even though every step is bound.

## 7. Keep changed steps honest

A step's text is part of the requirement. When behavior changes, revise the scenario and the assertions that exercise it, and watch them fail before changing production code. A wording change may still match an existing parameterized binding. Confirm that its parameters reach the actual inputs and assertions; an unchanged tag or test name alone establishes nothing about the changed requirement. Use the selected integration's change-detection and binding mechanism rather than inventing a second Workbench registry.

## 8. Record exclusions and manual checks

Exclude a scenario only through the integration's documented filter or skip mechanism: for example, a platform-specific rule or a scenario that needs hardware the gate lacks. List each exclusion in the review evidence with its reason. For a manual check, also record what was done and what was observed. Neither counts as an automated pass.

When several implementations bind one feature, each follows this procedure with its own integration and reports its own results.

## 9. Report the evidence

State the integration and version, the task that ran it, how many scenarios executed (each outline row counts as one), and which scenarios were excluded, checked manually, or not covered:

> `specs/features/placing-holds.feature`: 5 scenarios ran through `<integration>@<version>` in `pnpm test`, 2 of them from outline rows. None were excluded. Concurrent hold placement is not covered.
