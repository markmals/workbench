# Spec Conventions

This document defines the structure of specs in this repo. Every spec, every reverse pointer, every drift check assumes these rules. If you change anything here, audit every existing spec and pointer for consistency.

> **TL;DR:** Markdown files with YAML frontmatter, stable dotted IDs, one logical thing per file, and `// SPEC: <id>` comments in the code that implements them.

## Why specs at all

Native, idiomatic implementations on every platform mean the _behavior_ must converge even though the _code_ won't. Specs are the only artifact shared across platforms — they are the contract. Everything else (TanStack Start app, UIKit app, Compose app, Convex schema) is a regeneration target.

Specs describe **what** must hold. Tests prove it. Implementations satisfy it. None of those three is the source of truth on its own.

## File and directory layout

```
specs/                          ← cross-cutting (used by ≥ 2 features or platform-wide)
├── ARCHITECTURE.md             ← singular per-product
├── DESIGN_SYSTEM.md            ← singular per-product
├── CONVENTIONS.md              ← this file
├── models/<id>.md              ← cross-cutting domain models
└── view-models/<id>.md         ← cross-cutting view models (rare; usually feature-scoped)

features/<NNNN>-<slug>/         ← feature-scoped (only this feature uses it)
├── NARRATIVE.md                ← singular per feature
├── README.md                   ← singular per feature; describes the folder
├── stories/<id>.md             ← one user story per file
├── use-cases/<id>.md           ← one concrete use case per file
├── user-flow/<id>.md           ← one interaction sequence per file
├── models/<id>.md              ← one domain model per file
├── view-models/<id>.md         ← one view model per file
└── errors/<id>.md              ← one error catalog entry per file
```

### One logical thing per file

If a kind has multiple instances in a feature (multiple stories, multiple errors, multiple models), it gets a **directory** of `<id>.md` files. If a kind has exactly one instance per feature (the narrative), it stays a **file**.

The directory name is the kebab-case equivalent of the kind name (`view-models/`, not `view_models/` or `viewModels/`).

### Cross-cutting vs feature-scoped

A spec lives in `features/<n>/` until a _second_ feature depends on it. At that point, it gets **promoted**: the file moves to `specs/<kind>/<id>.md`, but its **ID does not change**. Reverse pointers in code stay valid through the move.

The only specs that start cross-cutting are `ARCHITECTURE.md`, `DESIGN_SYSTEM.md`, and this file.

## Frontmatter schema

Every spec file (in `specs/<kind>/` or `features/<n>/<kind>/`, plus the singular files like `NARRATIVE.md`) starts with YAML frontmatter:

```yaml
---
id: <stable-dotted-id> # required, must match filename stem
kind: <one of the kinds below> # required
depends-on: [<id>, <id>, ...] # optional; specs this one references
status: draft | accepted # optional; default = accepted
---
```

The top-level singular files (`ARCHITECTURE.md`, `DESIGN_SYSTEM.md`, `NARRATIVE.md` per feature, `CONVENTIONS.md`) use a special form:

```yaml
---
id: architecture # or design-system, conventions, narrative.<feature-slug>
kind: architecture # the kind matches the file's role
---
```

`depends-on` is a flat list of IDs. It is not transitive, not enforced by tooling yet, and exists primarily so a human or agent can grep for "what depends on `domain.item`".

## Kind taxonomy

Kinds are the closed set of allowed `kind:` values, paired with their directory and ID prefix.

| Kind            | Directory       | ID prefix                      | One per file? | Notes                                                                  |
| --------------- | --------------- | ------------------------------ | ------------- | ---------------------------------------------------------------------- |
| `narrative`     | (singular file) | `narrative.<feature-slug>`     | yes           | One per feature.                                                       |
| `story`         | `stories/`      | `story.<feature>.<capability>` | yes           | Authored using the `writing-user-stories` skill. Gherkin lives inline. |
| `use-case`      | `use-cases/`    | `usecase.<feature>.<scenario>` | yes           | Concrete walkthrough; complements stories.                             |
| `flow`          | `user-flow/`    | `flow.<feature>.<action>`      | yes           | Step-by-step interaction sequence.                                     |
| `domain`        | `models/`       | `domain.<entity>`              | yes           | Plain data shapes, invariants, validation rules.                       |
| `view-model`    | `view-models/`  | `vm.<feature>.<view>`          | yes           | State, actions, transitions, derived values.                           |
| `error`         | `errors/`       | `error.<domain>.<kind>`        | yes           | User-observable failure mode + recovery affordance.                    |
| `architecture`  | (singular file) | `architecture`                 | yes           | Cross-cutting; one per product.                                        |
| `design-system` | (singular file) | `design-system`                | yes           | Cross-cutting; one per product.                                        |
| `conventions`   | (this file)     | `conventions`                  | yes           | Cross-cutting; one per product.                                        |

A kind can grow over time (e.g., `migration` for schema changes), but adding a kind is a deliberate change to this document, not an ad-hoc choice.

## Stable IDs

IDs are dotted, lowercase, hierarchical, and stable. The first segment is the kind prefix; the rest narrow to a specific instance.

**Good:** `domain.item`, `vm.items.list`, `story.item.create`, `flow.item.edit`, `error.item.duplicate`

**Bad:** `Item`, `items/list`, `vm-items-list`, `viewmodel.items.list` (use `vm.`)

### Stability rules

- IDs are immutable once an implementation references them. Renaming requires a deliberate migration: update the spec ID, every `// SPEC:` reference, and every test tag in one commit.
- IDs do not change when a spec is promoted from `features/` to `specs/`.
- IDs do not encode platform — they describe the abstract behavior. Platform divergence is captured in the implementation, not the ID.

### Filename = ID stem

Filename matches the trailing segment of the ID, with dots → hyphens:

- `domain.item` → `models/item.md`
- `story.item.create` → `stories/item.create.md` (dots preserved within the stem)
- `vm.items.list` → `view-models/items.list.md`

Dots are legal in macOS/Linux filenames and survive grep, git, and most editors. Keep them.

## Reverse pointers

Every implementation file, class, or function that realizes a spec carries the spec ID in a comment.

### Per-language form

```ts
// SPEC: vm.items.list
export const itemsListQueryOptions = queryOptions({
    /* ... */
});
```

```swift
// SPEC: vm.items.list
@Observable
final class ItemsListViewModel { /* ... */ }
```

```kt
// SPEC: vm.items.list
class ItemsListViewModel : ViewModel() { /* ... */ }
```

```cs
// SPEC: vm.items.list
public sealed partial class ItemsListViewModel : ObservableObject { /* ... */ }
```

```rust
// SPEC: vm.items.list
pub struct ItemsListViewModel { /* ... */ }
```

For Convex functions:

```ts
// SPEC: protocol.items.create
export const create = mutation({
    /* ... */
});
```

### Granularity

- One reverse pointer per spec realization, attached to the smallest unit that fully realizes the spec (usually a class or top-level function, sometimes a module).
- Multiple files may reference the same ID if the implementation is split across them.
- Do not annotate every helper function — only the unit that fulfills the contract.

### Tests carry the same IDs

Every behavioral test is tagged with the spec IDs it verifies. The mechanism is per-platform but the discipline is uniform:

- **Vitest:** test name prefix `[spec.id]` and a `describe` block per spec ID.
    ```ts
    describe("vm.items.list", () => {
        it("[scenario.items.list.empty] shows empty state when no items exist", () => {
            /* ... */
        });
    });
    ```
- **Swift Testing:** `@Test(.tags(.spec("vm.items.list"), .scenario("items.list.empty")))`.
- **kotlin.test (JUnit5):** `@Tag("spec:vm.items.list")` and `@DisplayName("[scenario.items.list.empty] ...")`.
- **MSTest (C#):** `[TestCategory("spec:vm.items.list")]` on the class and `[Description("[scenario.items.list.empty] ...")]` on the method.
- **Cargo test (Rust):** a `#[cfg(test)] mod` carrying `// SPEC: vm.items.list`, with a `// [scenario.items.list.empty]` comment above each `#[test]` fn (Rust test names can't hold dots or brackets, so the scenario sub-ID lives in the comment that drift tooling greps).

The `[scenario.<id>]` prefix is mandatory because Gherkin scenarios in story files have their own sub-IDs (see "Stories and scenarios" below) and tests must trace to a specific scenario, not just a story.

## Stories and scenarios

Stories follow the `writing-user-stories` skill. Each story file contains:

1. Frontmatter with `id: story.<feature>.<capability>`
2. The `As a / I want / So that` block
3. An `# Acceptance Criteria` section with Gherkin scenarios

Each scenario has a stable sub-ID derived from its position and intent:

```md
## Scenario 1: Creating an item with valid information

<!-- id: scenario.item.create.happy-path -->

- Given a signed-in user
- When the user creates an item with valid information
- Then the item appears in the user's list
```

Sub-IDs follow the pattern `scenario.<feature>.<capability>.<short-name>`. Tests reference them in the `[scenario.id]` prefix described above.

## Marking unspecified or ambiguous content

When authoring a spec, do **not** silently guess at unspecified details. Mark them inline with a `[NEEDS CLARIFICATION: <question>]` token:

```md
**FR**: The system requires authentication via [NEEDS CLARIFICATION: which provider — Convex Auth, Clerk, Auth0?].
```

```md
- Given a signed-in user
- When the user creates an item with [NEEDS CLARIFICATION: how is uniqueness defined for an item?]
- Then ...
```

Why: an LLM that fills in plausible-but-unverified details produces specs that _look_ complete but contain hidden assumptions. A spec sprinkled with `[NEEDS CLARIFICATION]` markers is more honest, easier to review, and forces a deliberate resolution step before implementation.

**Resolution:** the `/sdd-clarify <feature-or-spec-id>` slash command scans for these markers, surfaces the highest-priority questions to the user, and edits the answers back into the spec. A spec cannot be considered ready for `/sdd-apply` while `[NEEDS CLARIFICATION]` markers remain.

**When to use:**

- The user prompt didn't specify a behavior, constraint, or value.
- Two interpretations are equally plausible and you can't pick without input.
- A non-functional requirement (auth method, retention period, performance target) is implied but not stated.

**When NOT to use:**

- For known-unknowns about implementation details (those belong in `// SPEC: <id> (deviates: <reason>)` comments or in the platform CLAUDE.md, not in the spec).
- For "we'll figure this out later" placeholders for features outside the current scope (just don't write the spec yet).

## Deviation marker

Platforms diverge. When a platform's implementation must differ from the spec — because of platform constraints, idiom, or a deliberate UX choice — annotate the deviation:

```swift
// SPEC: vm.items.list (deviates: iOS uses pull-to-refresh; web uses a refresh button)
```

```ts
// SPEC: manual
// This component has no spec — platform-specific code.
```

`(deviates: <reason>)` keeps the pointer live so drift detection still flags spec changes; the agent then decides whether the deviation still makes sense. `// SPEC: manual` opts out entirely, and is used sparingly for genuinely platform-specific code (e.g., iOS-only widget extensions).

## Drift detection

A spec and its implementation are **in sync** when:

1. Every spec has at least one reverse pointer per applicable platform (or is intentionally not yet implemented).
2. The spec's mtime ≤ the most recent mtime of files containing reverse pointers to its ID.
3. Tests tagged with the spec's ID exist on every applicable platform and pass.

Drift is detected by `/sdd-drift <platform>` (scaffolded; implementation deferred). The slash command outputs the IDs that fail any of the above.

## Reconciliation

When a single platform's implementation diverges from the spec — usually because it was edited directly to fix a bug or change behavior — the spec and the other platforms must be updated to match. This is what `/sdd-reconcile <platform>` does:

1. Read the platform's current implementation and tests.
2. Diff against the spec.
3. Propose updates to the spec.
4. Propose updates to the other platforms' implementations and tests.
5. The human reviews each diff before it lands.

Reconciliation is **not automatic**. The agent proposes; a human approves.

## Adding a new feature

1. Pick the next number: `features/<NNNN>-<slug>/`. Slug is kebab-case.
2. Copy `.claude/templates/feature/` into the new feature directory.
3. Author `NARRATIVE.md` first (use the brainstorming-style narrative from interviews or product input).
4. Author stories from the narrative.
5. Derive use-cases, flows, models, view-models, errors as needed. Not every feature uses every kind.
6. Build the **web reference implementation** first (see `apps/web/CLAUDE.md`).
7. Use `/sdd-apply <spec-id> <platform>` to bring iOS and Android in line.

## Adding a new spec kind

1. Add a row to the kind taxonomy table above.
2. Decide ID prefix and directory name.
3. Add a template file at `.claude/templates/feature/<dir>/<KIND>.md`.
4. Update `.claude/templates/feature/README.md`.
5. Update the file/directory layout diagram at the top of this document.
6. Commit the convention change before authoring any specs of the new kind.

## What is NOT a spec

These are reference material an agent may read, but not the spec layer. Do not put them under `specs/` or in a feature folder's spec subdirectories.

- Wireframes, Figma URLs, visual designs (link from `DESIGN_SYSTEM.md` or `NARRATIVE.md` if needed)
- Prototype code or sandbox repos
- Meeting notes, RFCs, decision logs (use a `docs/` directory if you need one)
- Analytics events, telemetry, observability — these are implementation concerns
- Platform-local cosmetic defects (clipping, jank, missing haptic, polish issues) — these go in `apps/<platform>/DEFECTS.md`, not in specs. See the `triaging-defects` skill for the classifier that decides which side of the line an observation falls on.

If you find yourself writing implementation details into a spec, stop and ask: **could a different platform realize this differently and still be correct?** If no, it is implementation, not spec. The same test decides whether an observation belongs in `DEFECTS.md` (yes — platform idioms can differ) or in a spec amendment (no — every platform must converge).
