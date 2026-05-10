---
id: architecture
kind: architecture
---

# Architecture

> **This is a template.** Replace the placeholder content below with the architecture for this product. Sections are intentionally short — this document is for orientation, not exhaustive reference.

## Product overview

<!-- One paragraph: what is this product, who uses it, what problem does it solve? -->

_[NEEDS CLARIFICATION: describe what this product is, who uses it, and what problem it solves. One paragraph is enough.]_

## Platforms

| Platform | Stack                                                                                 | Role                                      |
| -------- | ------------------------------------------------------------------------------------- | ----------------------------------------- |
| Web      | TanStack Start + React + Tailwind v4 + React Aria Components                          | Reference implementation. Built first.    |
| iOS      | Swift + SwiftUI + `@Observable`                                                       | Native client. Mirrors web behavior.      |
| Android  | Kotlin + Jetpack Compose + `androidx.lifecycle.ViewModel` + `kotlinx.coroutines.flow` | Native client. Mirrors web behavior.      |
| Backend  | Convex                                                                                | Single backend serving all three clients. |

## Layering

Each client application follows the same conceptual layering, even though the language and idiom differ.

```
┌──────────────────────────────────────┐
│  View                                │  SwiftUI view / Compose composable / React component
├──────────────────────────────────────┤
│  View Model                          │  spec: vm.<feature>.<view>
├──────────────────────────────────────┤
│  Domain                              │  spec: domain.<entity>
├──────────────────────────────────────┤
│  Client (Convex wrapper)             │  spec: implicit; mirrors Convex schema
└──────────────────────────────────────┘
                  │
                  ▼
┌──────────────────────────────────────┐
│  Convex (queries, mutations, actions)│  spec: protocol — Convex schema is canonical
└──────────────────────────────────────┘
```

- **View** is platform-native. Tests are minimal — just enough to catch wiring mistakes.
- **View Model** is the primary spec target. State, actions, transitions, derived values. Behavioral tests live here.
- **Domain** is the data shape and invariants. Pure types and validation, no I/O.
- **Client** wraps Convex. Web uses Convex's generated TS client directly. iOS uses a thin Swift wrapper around Convex's HTTP/WebSocket protocol or the community Swift client. Android uses the official Kotlin client where available, or a thin wrapper.

## Data flow

- Reads are **reactive subscriptions** on every platform: `useQuery` on web (via TanStack Query + Convex integration), `@Observable` queries on iOS, `Flow`-backed queries on Android.
- Writes go through Convex mutations. The client wrapper exposes idiomatic call sites.
- Auth and identity are handled by Convex's built-in auth.

## Deployment

| Environment | Web                     | iOS                          | Android                       | Backend                      |
| ----------- | ----------------------- | ---------------------------- | ----------------------------- | ---------------------------- |
| Development | Local Node + Convex dev | Xcode simulator + Convex dev | Android emulator + Convex dev | `pnpx convex dev`            |
| Production  | Cloudflare Workers      | TestFlight → App Store       | Internal track → Play Store   | Convex production deployment |

## Cross-platform parity rules

- Behavior described in a spec must hold on every applicable platform. Tests prove it.
- Idiom differs by design: SwiftUI does not look like Compose does not look like React. Behavior converges; code does not.
- Platform-specific affordances (pull-to-refresh, share sheets, system back button) may exist without a spec but should be tagged `// SPEC: manual` so drift detection ignores them.
- Genuine deviations carry `(deviates: <reason>)` on their reverse pointer.

## Out of scope (this product)

<!-- Things this product explicitly does not do. List concrete capabilities you're choosing not to build, so the boundary is visible. -->

- _[NEEDS CLARIFICATION: list out-of-scope capabilities here.]_

## Open architectural questions

<!-- Things deliberately deferred. Tag with the date so they can be revisited. -->

- _(empty)_
