# Worked Examples

These files teach the skill. They are not Workbench requirements.

## End-user workflow: placing holds

`.workbench/skills/specifying-behavior-with-gherkin/examples/placing-holds.feature` specifies how library members reserve titles.

- The description defines _active hold_, _queue_, and the pickup period, and names what another owner specifies. Scenarios do not execute it.
- The first rule uses a scenario outline for the two accepted counts at the edge of the limit, 0 and 4. The refusal at 5 is a separate scenario because its outcome differs in kind: it states the reason and what stays unchanged.
- The second rule has its own `Background` with a data table, so the queue exists only for that rule's scenarios.
- No step names a screen, control, or endpoint. The binding decides whether `When Ada places a hold on "Kindred"` calls a domain service, an HTTP API, or drives a browser, and the specification stays the same.

The first rule's scenarios illustrate the limit at three counts. They say nothing about one member placing two holds at once while holding four. If the limit must hold under concurrent requests, that claim needs its own evidence, such as a concurrency test.

## Developer workflow: applying migrations

`.workbench/skills/specifying-behavior-with-gherkin/examples/applying-migrations.feature` specifies a `migrate up` command. Developers are its users.

Gherkin fits because the contract is a sequence over stored state: which files exist, what the history records, what the command reports, and what the schema looks like after a failure. Before-and-after tables show that directly. A function signature would not. Maintainers of each database adapter review the same rules and bind the same feature.

- The feature-level `Background` gives every scenario the same three files. Each scenario states only the history that matters to it.
- The second rule is the failure case. It specifies what the developer can observe afterwards, not how the command recovers internally.
- `@transactional-ddl` marks the one rule that differs by platform, and the rule's description says where it applies. A MySQL run excludes that rule through the integration's tag filter and reports it as not applicable. It does not count as a pass.

Migration file parsing, ordering helpers, and SQL generation keep their native unit tests. Those tests need no Gherkin counterpart.

## Library API: Pitlane's `getStylesheets()`

Pitlane's `@pitlane/assets` declares the method on its resolver:

```ts
/** Stylesheets the modules need, from the client graph and the current server graph. */
getStylesheets(path: string | readonly string[]): Promise<string[]>;
```

Its README adds a guarantee the declaration cannot carry: "Looking up a module's stylesheets never compiles it for the browser, so `getStylesheets("app/entry.server.ts")` is safe for server code." Vitest tests check the returned URLs, the rejection for a module in neither graph, and the accepted parameter types with `expectTypeOf`.

The readers are developers and their coding agents, and they read the declaration, its doc comment, and the README. A change to `getStylesheets()` revises that contract text first, then the Vitest tests, then any affected usage guidance. There is no `.feature` counterpart. Restating the guarantee in Gherkin would look like this:

```gherkin
Scenario: Looking up stylesheets for a server module
  Given a manifest whose server graph includes "app/entry.server.ts"
  When getStylesheets is called with "app/entry.server.ts"
  Then the module is not compiled for the browser
```

Each step is a function call or an assertion renamed as prose. The scenario sits apart from the declaration it describes and adds bindings to maintain, and a reader learns nothing that the README and declaration do not already say.

Choose Gherkin when examples of state before and after an event explain a rule better than a signature and its documented guarantees.
