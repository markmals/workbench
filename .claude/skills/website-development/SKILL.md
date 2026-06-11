---
name: website-development
description: Use when writing or modifying marketing/content site code under `apps/website/`. Covers Astro + React islands + content collections + Tailwind v4 + React Aria + View Transitions + Zod idioms, and points at first-party docs. Complementary to `implementing-a-spec` (process) and `web-verification` (browser verification loop). Distinct from `web-development` (the TanStack Start app).
---

# Website Development

This skill covers **how to write marketing/content site code** in this repo. For the _workflow_ of implementing a spec, see `implementing-a-spec`. For the _verify-iterate loop_ in a browser, see `web-verification`. For _what to build_, see the spec.

The website is a **sibling of the app, not the app**. It's the marketing / content surface — landing pages, docs, blog, pricing — and it ships mostly static HTML. The interactive product lives under `apps/web/` and is the reference implementation; for that, see `web-development` (TanStack Start, not Astro).

## Stack at a glance

| Concern              | Choice                                   | First-party docs                                                                  |
| -------------------- | ---------------------------------------- | --------------------------------------------------------------------------------- |
| Framework            | Astro                                    | [docs.astro.build](https://docs.astro.build)                                      |
| Components / islands | React + React Compiler (optimizer)       | [react.dev/llms.txt](https://react.dev/llms.txt)                                  |
| Content              | Astro content collections                | [content-collections](https://docs.astro.build/en/guides/content-collections/)    |
| Internationalization | Astro i18n routing                       | [astro i18n](https://docs.astro.build/en/recipes/i18n/)                           |
| Styling              | Tailwind v4 (+ Tailwind Plus blocks)     | [tailwindcss.com/docs](https://tailwindcss.com/docs) _(no /llms.txt yet)_         |
| Unstyled UI          | React Aria Components                    | [llms.txt](https://react-spectrum.adobe.com/llms.txt)                             |
| Animation            | View Transitions                         | [astro view transitions](https://docs.astro.build/en/guides/view-transitions/)    |
| Validation           | Zod                                      | [zod.dev](https://zod.dev/)                                                       |
| Relational / edge DB | Drizzle (`node:sqlite` or Cloudflare D1) | [orm.drizzle.team](https://orm.drizzle.team/docs)                                 |
| Networking           | fetch                                    | [MDN Fetch API](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API)       |
| Logging              | Evlog                                    | [evlog.dev](https://www.evlog.dev/)                                               |
| Tests                | Vitest                                   | [llms.txt](https://vitest.dev/llms.txt)                                           |
| Linter / formatter   | Oxlint + Oxfmt                           | [llms.txt](https://oxc.rs/llms.txt)                                               |
| Type checker         | tsgo (`@typescript/native-preview`)      | —                                                                                 |
| Package manager      | pnpm                                     | [pnpm.io](https://pnpm.io/)                                                       |
| Production           | Cloudflare (static + CDN + image opt)    | [workers/static-assets](https://developers.cloudflare.com/workers/static-assets/) |

When you need to look something up: fetch the relevant `/llms.txt` with WebFetch and let it route you. For Astro and Cloudflare (no `/llms.txt`), WebFetch the canonical docs URL above.

## Idioms (read these before writing code)

### Content-first; ship HTML, hydrate sparingly

Astro renders static HTML by default — that's the point. A page is `.astro` markup that emits zero JavaScript unless you ask for some. Add a React island only where interactivity is genuinely needed (a search box, a tabbed pricing table, an animated hero), and pick the cheapest `client:*` directive that works:

- `client:visible` — hydrate when it scrolls into view. Default for below-the-fold interactivity.
- `client:idle` — hydrate when the main thread is free. For low-priority widgets.
- `client:load` — hydrate immediately. Reserve for above-the-fold interactivity the user touches right away.

```astro
---
import SearchBox from "../components/SearchBox.tsx";
---
<SearchBox client:visible />
```

If a section doesn't need state or event handlers, it's plain `.astro` and ships no JS. Keep it that way.

### Content collections are the content source

Markdown / MDX content lives in **typed content collections** under `src/content/`, validated by a schema at load time. Don't hand-parse markdown or glob files yourself — define the collection and let Astro give you typed, validated entries.

```ts
// src/content.config.ts
// SPEC: domain.post
import { defineCollection } from "astro:content";
import { glob } from "astro/loaders";
import { z } from "zod";

const post = defineCollection({
    loader: glob({ pattern: "**/*.md", base: "./src/content/posts" }),
    schema: z.object({
        title: z.string(),
        publishedAt: z.date(),
        draft: z.boolean().default(false),
    }),
});

export const collections = { post };
```

The reverse pointer goes on the schema (it realizes the content domain). Use **Zod** for the schema — `astro:content` supports it natively, and the same shape can be shared with app code so the site and the app can't disagree about what's valid.

### Islands stay dumb

React islands render the props they're handed. They do **not** fetch, do **not** branch on business logic, do **not** know where their data came from. Shared dumb components mirror the design system, same vocabulary as the app.

```tsx
// SPEC: vm.pricing.table
export function PricingTable({ tiers }: { tiers: Tier[] }) {
    return (
        <ul>
            {tiers.map((tier) => (
                <PricingCard key={tier.id} tier={tier} />
            ))}
        </ul>
    );
}
```

The reverse pointer lives on the **logic** (the view model / selector), not on island markup. `.astro` components carry `// SPEC: manual` — they're page composition with no cross-platform behavioral contract.

### The website is mostly read-only

It reads content collections and, where it needs structured data, queries the edge DB via **Drizzle** (`node:sqlite` locally or Cloudflare D1 in production) server-side during the build or render. It usually does **not** talk to Convex realtime — that's the app's job (`web-development`). If a marketing page genuinely needs live product data (a public stats counter, say), fetch it **server-side** in the frontmatter or a server endpoint with `fetch`, render the result into HTML, and don't drag a realtime client onto the page.

### Internationalization is Astro routing

Localized routes use **Astro's built-in i18n** — configure locales and the default locale in `astro.config`, keep translated content in per-locale content-collection entries, and let Astro generate the `/<locale>/…` routes. Don't hand-roll a locale router or a runtime translation layer for static content.

### Tailwind v4 + React Aria + View Transitions

Same conventions as the app, with one difference — animation:

- **Tailwind v4** reads config from CSS via `@theme`; reference tokens with `var(--color-...)`. Utility-first; reach for `@apply` only for genuine component-level patterns. No v3-style `tailwind.config.js`.
- **React Aria Components** for any interactive primitive inside an island (Dialog, Menu, Disclosure, Select). Don't hand-roll ARIA on raw elements.
- **View Transitions** are the site's animation primitive — Astro's `<ClientRouter />` plus `transition:*` directives for page and element transitions, no JS animation runtime on the page. Honor `prefers-reduced-motion`, and keep durations in line with the `motion.*` tokens in `DESIGN_SYSTEM.md`. For genuinely complex in-island motion the app's **Motion** convention applies, but reach for it last — most marketing motion is a View Transition or CSS.

## File layout (within apps/website/)

See `apps/website/CLAUDE.md` for the canonical layout. Summary:

```
apps/website/src/
├── pages/              ← file-based routes (.astro / .md / .mdx → URLs)
├── content/            ← content collections + content.config.ts (schema)
├── components/         ← React islands + shared dumb components
├── layouts/            ← shared page shells (.astro)
└── styles/             ← Tailwind layers and tokens
```

## Verifying

- **Visual / behavioral** — use the `web-verification` skill (the `chrome-devtools` CLI) against the Astro dev server (`mise run website:dev`). Screenshot, inspect console, exercise islands in a tight verify-iterate loop.
- **Content + types** — run `astro check` (via `mise run -C apps/website check`) to surface content-collection schema errors and type breaks. Then `mise run -C apps/website typecheck` (tsgo).
- **Performance + SEO** — Lighthouse matters more here than for the app; content sites live or die on perf and SEO. Run `chrome-devtools lighthouse_audit` against a production-like build (`mise run website:build` then preview) and read the LCP / CLS / SEO scores before claiming a page is done.

## When to invoke a more specific skill

- About to write tests? → `test-driven-development`
- About to claim work is done? → `verification-before-completion`
- Verifying visually or running Lighthouse in a browser? → `web-verification`
- Debugging something unexpected? → `systematic-debugging`
- Implementing a spec end-to-end? → `implementing-a-spec` (this skill supports that workflow with idiom knowledge)
- Working on the interactive product instead of the marketing site? → `web-development`

## Commit

Land focused, atomic commits as the work hits natural boundaries — typically per spec ID, per island + its view-model tests, or per cohesive page. See `.claude/rules/commit-discipline.md`.

Website-specific notes:

- **Content additions go alone.** A new collection entry (blog post, doc page) or a batch of images belongs in its own commit — `docs: add <post>` — separate from component or layout code.
- **Don't commit generated build outputs.** `apps/website/dist/` and `.astro/` are gitignored.
- **Dependency bumps are separate.** A Tailwind, Astro, or React bump is its own `chore:` commit, never bundled with content or feature work.
