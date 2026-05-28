---
id: architecture
kind: architecture
---

# Architecture

> **This is a template.** Replace the placeholder content below with the architecture for this product. Sections are intentionally short — this document is for orientation, not exhaustive reference.
>
> This template ships as the **superset** of every platform the stack supports. Run `/setup` on a fresh copy to declare which targets your product actually uses and prune the rest. The full toolchain catalog lives in [STACK.md](./STACK.md).

## Product overview

<!-- One paragraph: what is this product, who uses it, what problem does it solve? -->

_[NEEDS CLARIFICATION: describe what this product is, who uses it, and what problem it solves. One paragraph is enough.]_

## Platforms

The web app is the **reference implementation** — build features there first; every other client mirrors its behavior idiomatically. Delete the rows your product doesn't ship.

| Platform        | Stack                                                                                  | Role                                          |
| --------------- | -------------------------------------------------------------------------------------- | --------------------------------------------- |
| Web app         | React + TanStack Start/Router/Query + Tailwind v4 + React Aria                          | **Reference implementation. Built first.**    |
| Website         | Astro + React islands + Tailwind v4 + content collections                              | Marketing / content surface. Mostly static.   |
| Apple           | Swift + SwiftUI + Observation + SwiftData (iOS · iPadOS · macOS · tvOS · watchOS · visionOS) | Native clients. Mirror web behavior.     |
| Android         | Kotlin + Jetpack Compose + Material 3 + coroutines/Flow + Room                          | Native client. Mirrors web behavior.          |
| Windows         | C# + WinUI 3 + XAML + MVVM Toolkit + EF Core                                            | Native client. Mirrors web behavior.          |
| Linux           | Rust + GTK 4 + Adwaita + Relm4 + Diesel                                                 | Native client. Mirrors web behavior.          |
| Server CLI      | Node single-file exe + TS-Rest + Bombshell (args/clack/tab) + Drizzle + plainjob        | Headless/automation client. Hosts the API.    |
| High-perf CLI   | Rust single-file exe + Clap + Ratatui + Diesel + Progenitor                             | TUI client. Generated API client.             |
| Backend         | Convex (database, file storage, cron, queues, realtime) + Clerk (auth)                  | Single backend serving every client.          |

Desktop web apps (web app stack wrapped in **Electron**) are a packaging concern, not a separate platform — the same React/TanStack code ships to the browser and to the desktop shell.

## Layering

Every client follows the same conceptual layering, even though the language and idiom differ.

```
┌──────────────────────────────────────┐
│  View                                │  React component / SwiftUI view / Compose composable / XAML / GTK widget
├──────────────────────────────────────┤
│  View Model                          │  spec: vm.<feature>.<view>
├──────────────────────────────────────┤
│  Domain                              │  spec: domain.<entity>
├──────────────────────────────────────┤
│  Client                              │  spec: implicit; talks to the backend
└──────────────────────────────────────┘
                  │
                  ▼
┌──────────────────────────────────────┐
│  Convex (queries, mutations, actions)│  spec: protocol — Convex schema is canonical
│  + Clerk (identity)                  │
└──────────────────────────────────────┘
```

- **View** is platform-native. Tests are minimal — just enough to catch wiring mistakes.
- **View Model** is the primary spec target. State, actions, transitions, derived values. Behavioral tests live here.
- **Domain** is the data shape and invariants. Pure types and validation, no I/O.
- **Client** is the layer that differs most by platform — see below. The wrapper exists only to expose idiomatic call sites, never to reimplement a protocol.

### The Client layer is contract-first, not hand-rolled

Two realizations, by platform family:

- **Web app and website** talk to **Convex directly** through its generated TypeScript client (via TanStack Query + `@convex-dev/react-query`). Reactive subscriptions, mutations, and codegen-driven types flow through the official client. Relational or edge-local data that isn't a fit for Convex uses **Drizzle** (e.g. Cloudflare D1).
- **Native clients (Apple, Android, Windows, Linux) and the CLIs** consume the backend through a **generated OpenAPI client** — Swift OpenAPI Generator, Kotlin OpenAPI Generator, Kiota (C#), Progenitor (Rust). The platform's standard HTTP stack (URLSession / Ktor / HttpClient / reqwest) is the transport; the typed surface is generated, never hand-written. Each carries a local on-device database (SwiftData / Room / EF Core / Diesel) as a **local-first cache**, not a second source of truth.

No platform hand-rolls a transport or "mirrors" the protocol by hand. The contract — Convex's TypeScript client on web, the OpenAPI document everywhere else — is the only thing that crosses the wire boundary.

## Data flow

- **Reads are reactive where the platform supports it:** `useQuery` on web/website (TanStack Query + Convex), `@Observable`-backed queries on Apple, `Flow`-backed queries on Android, observable view models on Windows (MVVM Toolkit) and Linux (Relm4). Native clients fall back to fetch-and-cache against the OpenAPI surface where live subscriptions aren't available.
- **Writes** go through Convex mutations (web) or the generated client's typed operations (everywhere else). The client wrapper exposes idiomatic call sites.
- **Identity** is handled by **Clerk** on every platform — Clerk's web SDK on web/website, its native SDKs or token flows on the other clients. Convex validates the Clerk-issued identity.
- **Server-side, non-data work** (sending email, third-party calls, computing sensitive values) runs in Convex actions or in TanStack Start server functions — never in view code.
- **Background work**: Convex cron + the Convex Workpool component for queued jobs; the standalone Server CLI uses `plainjob` for its own background jobs.

## Deployment

| Platform     | Development                       | Production                                         |
| ------------ | --------------------------------- | -------------------------------------------------- |
| Web app      | Vite dev + `convex dev`           | Cloudflare Workers (static assets + edge)          |
| Website      | Astro dev + `convex dev`          | Cloudflare (static hosting + CDN + image opt.)     |
| Apple        | Xcode simulators + `convex dev`   | TestFlight → App Store                             |
| Android      | Android emulator + `convex dev`   | Internal track → Play Store                        |
| Windows      | Local debug + `convex dev`        | MSIX / Microsoft Store                             |
| Linux        | Local debug + `convex dev`        | Flatpak / distribution package                     |
| Server CLI   | Local Node + `convex dev`         | Single-file executable; Railway VPS for hosted API |
| High-perf CLI| Local `cargo run` + `convex dev`  | Single-file binary release (`cargo` → tsdown-style exe) |
| Backend      | `convex dev`                      | Convex production deployment                       |

Domains, DNS, CDN, and image optimization are all **Cloudflare**. VPS workloads (the hosted Server CLI / API) run on **Railway**.

## Cross-platform parity rules

- Behavior described in a spec must hold on every applicable platform. Tests prove it.
- Idiom differs by design: SwiftUI does not look like Compose does not look like WinUI does not look like GTK does not look like React. Behavior converges; code does not.
- Platform-specific affordances (pull-to-refresh, share sheets, system back button, GNOME header bars, Windows jump lists) may exist without a spec but should be tagged `// SPEC: manual` so drift detection ignores them.
- Genuine deviations carry `(deviates: <reason>)` on their reverse pointer.

## Out of scope (this product)

<!-- Things this product explicitly does not do. List concrete capabilities you're choosing not to build, so the boundary is visible. -->

- _[NEEDS CLARIFICATION: list out-of-scope capabilities here.]_

## Open architectural questions

<!-- Things deliberately deferred. Tag with the date so they can be revisited. -->

- **Who produces the OpenAPI contract that non-web clients generate against?** Two viable shapes: (a) Convex HTTP actions expose an OpenAPI document directly; (b) a thin TS-Rest gateway (the Server CLI's API surface) fronts Convex + Drizzle and owns the OpenAPI document. Pick one per project before scaffolding native clients. _[NEEDS CLARIFICATION: choose (a) or (b).]_
