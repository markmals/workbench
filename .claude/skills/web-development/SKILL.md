---
name: web-development
description: Use when writing or modifying web app code under `apps/web/` or `services/convex/`. Covers TanStack Start + Convex + Tailwind v4 + React Aria idioms, and points at /llms.txt endpoints for first-party docs. Complementary to `implementing-a-spec` (process) and `web-verification` (visual verification loop).
---

# Web Development

This skill covers **how to write web code** in this repo. For the _workflow_ of implementing a spec, see `implementing-a-spec`. For the _verify-iterate loop_ in a browser, see `web-verification`. For _what to build_, see the spec.

The web app is the **reference implementation**. Build features here first; iOS and Android use the web realization as a worked example.

## Stack at a glance

| Concern            | Choice                                     | First-party docs                                                               |
| ------------------ | ------------------------------------------ | ------------------------------------------------------------------------------ |
| Framework          | TanStack Start                             | [llms.txt](https://tanstack.com/llms.txt)                                      |
| Backend            | Convex                                     | [llms.txt](https://docs.convex.dev/llms.txt)                                   |
| Components         | React Aria Components                      | [llms.txt](https://react-spectrum.adobe.com/llms.txt)                          |
| Build tool         | Vite                                       | [llms.txt](https://vitejs.dev/llms.txt)                                        |
| Tests              | Vitest                                     | [llms.txt](https://vitest.dev/llms.txt)                                        |
| Linter / formatter | Oxlint + Oxfmt                             | [llms.txt](https://oxc.rs/llms.txt)                                            |
| Styling            | Tailwind v4                                | [tailwindcss.com/docs](https://tailwindcss.com/docs) _(no /llms.txt yet)_      |
| Server state       | TanStack Query + `@convex-dev/react-query` | (covered by Convex docs above)                                                 |
| Production runtime | Cloudflare Workers                         | [developers.cloudflare.com/workers](https://developers.cloudflare.com/workers) |

When you need to look something up: fetch the relevant `/llms.txt` with WebFetch and let it route you to the specific page. The `/llms.txt` is the index, not the content — it tells you which URLs to fetch next.

## Idioms (read these before writing code)

### View models are query options + helpers, not classes

A view model in this stack is rarely a class. It's typically:

- A `queryOptions` factory exposing the read state
- A handful of selectors (pure functions of the query data)
- One or more mutation wrappers exposing user actions

```ts
// SPEC: vm.items.list
import { queryOptions } from "@tanstack/react-query";
import { convexQuery } from "@convex-dev/react-query";
import { api } from "convex/_generated/api";

export const itemsListQueryOptions = queryOptions(convexQuery(api.items.list, {}));

export function selectItemCount(items: Item[] | undefined): number {
    return items?.length ?? 0;
}
```

Tests target the queryOptions key and the selectors — not the rendered DOM. Behavior tests live at the view-model layer; component tests are usually limited to wiring smoke tests.

### Components stay dumb

Components consume view models and render. They do **not** call mutations directly, do **not** read from the Convex client directly, do **not** contain branching business logic.

```tsx
// SPEC: manual (this component renders vm.items.list)
import { useSuspenseQuery } from "@tanstack/react-query";
import { itemsListQueryOptions } from "./items.list.vm";

export function ItemsList() {
    const { data: items } = useSuspenseQuery(itemsListQueryOptions);
    if (items.length === 0) return <EmptyState />;
    return (
        <ul>
            {items.map((c) => (
                <ItemRow key={c._id} item={c} />
            ))}
        </ul>
    );
}
```

The reverse pointer lives on the **view model**, not on the component. Components carry `// SPEC: manual` because they have no cross-platform behavioral contract.

### Server functions are the call site for non-data work

Anything that isn't a Convex query/mutation but needs to run server-side goes in a TanStack Start server function (`createServerFn`). Sending an email, calling a third-party API, computing a sensitive value — server function. Don't put server logic in route components.

### No global state library

TanStack Query + Convex's reactive cache _is_ the global state. Don't reach for Redux, Zustand, Jotai, or Pinia. Local UI state (open/closed, hover, transient input) stays in components via `useState`.

### Tailwind v4 conventions

- Use the `@theme` directive to declare design tokens; reference them via `var(--color-...)` in components when you need to bind dynamically.
- Prefer utility classes over `@apply`. Reach for `@apply` only for repeated patterns that are genuinely component-level (focus rings, button bases).
- Tailwind v4 reads the config from CSS, not a JS config file. Live with it — don't shim a v3-style `tailwind.config.js`.

### React Aria over hand-rolled accessibility

Use React Aria Components for any interactive primitive (Button, Select, Menu, Dialog, Listbox, etc.). Don't reach for `<button>` + manual ARIA — Aria handles focus, keyboard, screen reader semantics for you.

### Oxlint and Oxfmt

These are the linter and formatter. Don't ship ESLint or Prettier config files. If a rule is too strict, change the rule in `.oxlintrc.json` — don't disable it inline unless you have a real reason.

## Convex specifics

- **Schema is the protocol.** Every entity lives in `services/convex/schema.ts` with explicit indexes. Add `// SPEC: domain.<entity>` to each table block.
- **Functions are thin.** Queries return the precise shape the client needs; mutations validate inputs and return the affected entity. Heavy logic lives in `services/convex/lib/`.
- **Use actions for I/O only.** Anything calling an external API goes in an action, not a mutation.
- **Run `mise run -C services/convex codegen` after schema changes.** Then `mise run -C apps/web typecheck` to surface breaks.

## TanStack Start specifics

- **File-based routing.** Route files under `apps/web/src/routes/` map to URL paths. The conventions are documented at the TanStack Start docs (see /llms.txt above).
- **Data loaders go in `loader`.** Route-scoped data fetching uses the loader API — not `useEffect`. The loader runs on the server during SSR.
- **Cloudflare Workers as the deploy target.** Avoid Node-only APIs (`fs`, `child_process`, etc.) in code that runs in the worker. Use Web Standard APIs (`fetch`, `Request`, `Response`, `crypto.subtle`).

## File layout (within apps/web/)

See `apps/web/CLAUDE.md` for the canonical layout. Summary:

```
apps/web/src/
├── routes/             ← file-based routes
├── features/<slug>/    ← feature-scoped: <view>.vm.ts, <view>.vm.test.ts, components
├── components/         ← shared dumb components
├── domain/             ← domain types (one file per domain.* spec)
├── client/             ← Convex client wrapper (rarely edited)
└── styles/             ← Tailwind layers and tokens
```

## When to invoke a more specific skill

- About to write tests? → `test-driven-development`
- About to claim work is done? → `verification-before-completion`
- Verifying visually in a browser? → `web-verification`
- Debugging something unexpected? → `systematic-debugging`
- Implementing a spec end-to-end? → `implementing-a-spec` (this skill supports that workflow with idiom knowledge)
