---
id: architecture
kind: architecture
---

# Architecture

> **This is a template.** Replace the placeholder content below with the architecture for this product. Sections are intentionally short — this document is for orientation, not exhaustive reference.
>
> This template ships as the **superset** of every platform the stack supports. Run `/setup` on a fresh copy to declare which targets your product actually uses and prune the rest. The full toolchain catalog lives in [STACK.md](../STACK.md).

## Product overview

<!-- One paragraph: what is this product, who uses it, what problem does it solve? -->

_[NEEDS CLARIFICATION: describe what this product is, who uses it, and what problem it solves. One paragraph is enough.]_

## Platforms

The web app is the **reference implementation** — build features there first; every other client mirrors its behavior idiomatically. Delete the rows your product doesn't ship.

| Platform      | Stack                                                                                                                            | Role                                                        |
| ------------- | -------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| Web app       | React + TanStack Start/Router/Query + Tailwind v4 + React Aria                                                                   | **Reference implementation. Built first.**                  |
| Website       | Astro + React islands + Tailwind v4 + content collections                                                                        | Marketing / content surface. Mostly static.                 |
| Apple         | Swift + UIKit (AppKit on macOS, SwiftUI on watchOS) + Observation + SwiftData (iOS · iPadOS · macOS · tvOS · watchOS · visionOS) | Native clients. Mirror web behavior.                        |
| Android       | Kotlin + Jetpack Compose + Material 3 + coroutines/Flow + Room                                                                   | Native client. Mirrors web behavior.                        |
| Windows       | C# + WinUI 3 + XAML + MVVM Toolkit + EF Core                                                                                     | Native client. Mirrors web behavior.                        |
| Linux         | Rust + GTK 4 + Adwaita + Relm4 + Diesel                                                                                          | Native client. Mirrors web behavior.                        |
| CLI           | One platform, one stack (chosen at `/setup`): Node (TS-Rest + Bombshell) · Rust (Clap + charmed_rust) · Go (Cobra/Fang + Charm) | Headless automation + TUI client. One CLI per app. Node/Go host the API in OpenAPI mode. |
| Backend       | One of — **Convex** · a **TS-Rest / OpenAPI** server · **none** (local-only). Clerk for identity.                                | Chosen at `/setup`. See "Backend modes".                    |

Desktop web apps (web app stack wrapped in **Electron**) are a packaging concern, not a separate platform — the same React/TanStack code ships to the browser and to the desktop shell.

## Layering

Every client follows the same conceptual layering, even though the language and idiom differ.

```
┌──────────────────────────────────────┐
│  View                                │  React component / UIKit · AppKit view / Compose composable / XAML / GTK widget
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
│  Backend: mode-dependent (see below) │  spec: protocol — canonical per mode
│  + Clerk identity, when there's a remote backend
└──────────────────────────────────────┘
```

- **View** is platform-native. Tests are minimal — just enough to catch wiring mistakes.
- **View Model** is the primary spec target. State, actions, transitions, derived values. Behavioral tests live here.
- **Domain** is the data shape and invariants. Pure types and validation, no I/O.
- **Client** is the layer that differs most by platform — see below. The wrapper exists only to expose idiomatic call sites, never to reimplement a protocol.

### The purity boundary

The layering above is also a **verifiability boundary**, and drawing it deliberately is the most consequential structural decision in this template.

- **Pure core** — the Domain layer and the decision logic of the View Models: data shapes, invariants, validation, state transitions, derived values. No I/O, no framework imports, no clock, no network, no persistence. Same inputs always produce the same outputs.
- **Effectful shell** — the View (rendering), the Client (transport / persistence), identity, background work, and anything else that touches the outside world.

Dependencies point **inward**: the shell depends on the core; the core depends on nothing but other pure code. Effects are pushed to the edges and injected, never reached for from inside the core. This is what `code-quality.md` means by "I/O at the edges; pure logic in the middle" and "the domain layer doesn't import the framework" — stated here as an architectural invariant, not just a style rule.

Why it earns its place:

- **Behavioral tests live in the core**, which is why the same scenarios port across platforms without standing up a runtime — a View Model test substitutes a client interface, not "the universe."
- **Invariants are property-tested in the core** (see the `test-driven-development` skill): a function that takes data in and returns a result is one a property runner can hammer with thousands of generated inputs; a function that also reads a database is not.
- If a behavior can't be tested without mocking half the system, the boundary was drawn in the wrong place. Fix the architecture, not the test.

### Backend modes — pick one at `/setup`

The backend is **not** assumed. A project chooses exactly one of three mutually-exclusive modes; the Client layer realizes the chosen mode idiomatically on each platform. These do **not** combine — it is Convex, _or_ OpenAPI, _or_ nothing.

- **Convex** — a reactive backend (database, file storage, cron, queues, realtime). Web and website use Convex's TypeScript client (TanStack Query + `@convex-dev/react-query`); native clients and CLIs use Convex's first-party client SDK for their platform. Reactive subscriptions wherever the platform supports them. No OpenAPI layer.
- **OpenAPI** — a server (the CLI in server mode — Node/TS-Rest or Go/oapi-codegen — or a dedicated `services/` server) is the backend and owns the OpenAPI document. Web and website use the TS-Rest typed fetch client; native clients and the Rust/Go CLI consume a **generated** OpenAPI client — Swift OpenAPI Generator, Kotlin OpenAPI Generator, Kiota (C#), Progenitor (Rust), oapi-codegen (Go) — over the platform's HTTP stack (URLSession / Ktor / HttpClient / reqwest / net/http). No Convex.
- **No API** — no backend at all. Each client is **local-first**, persisting on-device: Drizzle (web / Electron, local SQLite), SwiftData (Apple), Room (Android), EF Core (Windows), Diesel (Linux / Rust CLI), go-sqlite (Go CLI). No networking, no Convex, no OpenAPI.

Across all modes:

- The on-device databases (SwiftData / Room / EF Core / Diesel) and Drizzle are a **cache** in Convex/OpenAPI modes and the **source of truth** in no-API mode.
- The Client wrapper exposes idiomatic call sites; it never hand-rolls a transport or re-implements the protocol. The contract — the Convex schema, the OpenAPI document, or nothing — is the only thing that crosses the wire.
- **Clerk** provides identity in the Convex and OpenAPI modes; a no-API app has local or no identity.

## Data flow

The exact mechanics depend on the chosen backend mode (above):

- **Reads** — _Convex mode_: reactive subscriptions where supported (`useQuery` on web/website, `@Observable` on Apple, `Flow` on Android, observable view models on Windows/Linux). _OpenAPI mode_: request/response through the generated client, cached on-device. _No-API mode_: straight from the local store.
- **Writes** — Convex mutations (Convex mode), the generated client's typed operations (OpenAPI mode), or a local write (no-API mode). The client wrapper exposes idiomatic call sites.
- **Identity** — **Clerk** wherever there's a remote backend; Convex or the TS-Rest server validates the Clerk-issued token. No-API apps have local or no identity.
- **Server-side, non-data work** (email, third-party calls, sensitive computation) — Convex actions or TanStack Start server functions (Convex mode), or TS-Rest server handlers (OpenAPI mode). Never in view code.
- **Background work** — Convex cron + the Workpool component (Convex mode); `plainjob` in the Node CLI stack (OpenAPI mode).

## Deployment

| Platform      | Development                                           | Production                                              |
| ------------- | ----------------------------------------------------- | ------------------------------------------------------- |
| Web app       | Vite dev + backend dev                                | Cloudflare Workers (static assets + edge)               |
| Website       | Astro dev + backend dev                               | Cloudflare (static hosting + CDN + image opt.)          |
| Apple         | Xcode simulators + backend dev                        | TestFlight → App Store                                  |
| Android       | Android emulator + backend dev                        | Internal track → Play Store                             |
| Windows       | Local debug + backend dev                             | MSIX / Microsoft Store                                  |
| Linux         | Local debug + backend dev                             | Flatpak / distribution package                          |
| CLI           | Local `node` / `cargo run` / `go run` + backend dev   | Single-file executable (per stack); Node/Go can host the Railway API |
| Backend       | per mode (`convex dev` / local TS-Rest server / none) | per mode (Convex deploy / Railway / —)                  |

The **backend dev** step depends on your mode: `convex dev` (Convex), the local TS-Rest server (OpenAPI), or nothing (no-API). Domains, DNS, CDN, and image optimization are all **Cloudflare**. VPS workloads (the hosted TS-Rest API) run on **Railway**.

## Cross-platform parity rules

- Behavior described in a spec must hold on every applicable platform. Tests prove it.
- Idiom differs by design: UIKit does not look like Compose does not look like WinUI does not look like GTK does not look like React. Behavior converges; code does not.
- Platform-specific affordances (pull-to-refresh, share sheets, system back button, GNOME header bars, Windows jump lists) may exist without a spec but should be tagged `// SPEC: manual` so drift detection ignores them.
- Genuine deviations carry `(deviates: <reason>)` on their reverse pointer.

## Out of scope (this product)

<!-- Things this product explicitly does not do. List concrete capabilities you're choosing not to build, so the boundary is visible. -->

- _[NEEDS CLARIFICATION: list out-of-scope capabilities here.]_

## Open architectural questions

<!-- Things deliberately deferred. Tag with the date so they can be revisited. -->

- _(empty)_ — the backend choice (Convex / OpenAPI / none) is a deliberate `/setup` decision, not an open question. See "Backend modes".
