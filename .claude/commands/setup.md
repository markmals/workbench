---
description: First-run setup — choose which platforms this copy ships and prune everything else.
---

# /setup

This template ships as the **superset** of every platform the stack supports. This command turns it into _just this project_: it asks which platforms and backend you're actually shipping, then deletes the skills, hooks, permissions, and documentation rows for everything else. The file you're reading is the only thing that knows the full matrix — so run this **once, on a fresh copy**, before you scaffold anything.

There is no automation behind this command. You (the agent) drive it with `AskUserQuestion`, `rg`, `Edit`, and `rm`, exactly like the `/sdd-*` commands.

## Step 1 — Guard

Check whether the template looks already-customized: any directory under `apps/` or `services/` exists, or one of the platform skills below is already missing. If so, **stop and confirm** with the user before pruning — they may have already run this, and re-running would delete real work.

## Step 2 — Ask

The platform set is larger than one `AskUserQuestion` question allows (max 4 options each), so ask it in two multi-select questions plus one backend question:

1. **App / GUI platforms** (multiSelect): `Web app (reference)`, `Website (Astro)`, `Apple`, `Android`.
2. **Additional platforms** (multiSelect): `Windows`, `Linux`, `Server / Node CLI`, `High-performance Rust CLI`.
3. **Backend mode** (single — these are mutually exclusive; see `specs/ARCHITECTURE.md` → "Backend modes"):
    - `Convex` — a reactive backend; web/website use the Convex TS client, native clients and CLIs use Convex's first-party SDK. No OpenAPI layer.
    - `OpenAPI` — a TS-Rest server is the backend and owns the OpenAPI document; web/website use the TS-Rest client, native clients and the Rust CLI consume a generated OpenAPI client. No Convex.
    - `No API` — local-only; each client persists on-device (Drizzle / SwiftData / Room / EF Core / Diesel). No backend, no networking, no Convex.

Notes for interpreting answers:

- **Web is the default reference platform.** If the user deselects the web app, ask which selected platform becomes the reference and update `specs/ARCHITECTURE.md` + `CLAUDE.md` accordingly.
- The backend mode is independent of the platform set, but cross-check for nonsense: `No API` with a `Server / Node CLI` selected is contradictory (the Node CLI's reason to exist is hosting the API) — confirm with the user. `OpenAPI` mode wants something to host the contract (typically the Server CLI or a `services/` server).

## Step 3 — Prune

For every platform **not** selected, remove its artifacts using the map below. Watch the **shared-tooling cautions** — some permissions and the Chrome DevTools MCP are shared, so only remove them when _all_ sharing platforms are gone.

### Pruning map

| Platform        | Delete skills                                     | settings.json permissions to drop                             | Hooks to trim                                                       | Docs rows to remove (ARCHITECTURE · DESIGN_SYSTEM · STACK · CLAUDE · README) |
| --------------- | ------------------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Web app         | `web-development`, `web-verification`             | `pnpm`,`node`,`vite`,`vitest` ⚠️shared-TS                     | `stop-lint` `web` line                                              | Web rows/cols; the `apps/web` layout line; Web Apps STACK section            |
| Website         | `website-development`                             | (TS toolchain ⚠️shared)                                       | `stop-lint` `website` line                                          | Website rows; `apps/website` line; Websites STACK section                    |
| Apple           | `ios-development`, `ios-simulator-control`        | `xcrun simctl …`, `xcodebuild`, `swift-format`, `idb`         | `tuist-regen`; `stop-lint` `ios`; `block-generated` Tuist/xcodeproj | Apple row; iOS column; Apple STACK section; Xcode MCP bullet                 |
| Android         | `android-development`, `android-emulator-control` | `adb …`, `emulator`, `./gradlew`, `gradle`, `ktlint`, `ktfmt` | `stop-lint` Android note                                            | Android row; Android column; Android STACK section; JetBrains MCP bullet     |
| Windows         | `windows-development`                             | `dotnet`                                                      | `block-generated` `.NET obj/bin`; `stop-lint` Windows note          | Windows row; Windows column; Windows STACK section; Roslyn MCP bullet        |
| Linux           | `linux-development`                               | `cargo`,`rustfmt` ⚠️shared-Rust                               | `stop-lint` `linux`; `block-generated` `target/` ⚠️shared           | Linux row; Linux column; Linux STACK section                                 |
| Server/Node CLI | `server-cli-development`                          | (TS toolchain ⚠️shared)                                       | `stop-lint` `cli` line                                              | Server CLI rows; `apps/cli` line; Server CLI STACK section                   |
| Rust CLI        | `rust-cli-development`                            | `cargo`,`rustfmt` ⚠️shared-Rust                               | `stop-lint` `tui`; `block-generated` `target/` ⚠️shared             | High-Performance CLI STACK section; `apps/tui` line                          |

**Shared-tooling cautions:**

- **TS toolchain** (`pnpm`, `node`, `vite`, `vitest`, `oxfmt`, `oxlint`, `tsgo`) is shared by **web app, website, and server CLI**. Keep it while _any_ of those three survive.
- **Rust toolchain** (`cargo`, `rustfmt`, the `block-generated` `target/` rule) is shared by **Rust CLI and Linux**. Keep it while _either_ survives.
- **`format-on-edit` is platform-agnostic.** It dispatches the touched file to that platform's `fmt` task, so pruning a platform needs no edit to the hook — deleting `apps/<platform>/` and its skill is enough.
- **Chrome DevTools MCP** (`.mcp.json`) is used by **web app and website** via `web-verification`. Remove the `.mcp.json` server entry only if both are gone.
- **OpenAPI machinery** (`openapi-codegen.sh` hook + its settings registration, the OpenAPI mode in ARCHITECTURE, the generated-client framing in the native/CLI skills) belongs to **OpenAPI mode** only. Drop it in `Convex` and `No API` modes.
- **Convex machinery** (`convex-codegen.sh` hook + its registration, `services/convex`, the Convex idioms in `web-development`) belongs to **Convex mode** only. Drop it in `OpenAPI` and `No API` modes.

### Backend pruning (by mode)

Edit the **Backend modes** section of `specs/ARCHITECTURE.md` down to the single chosen mode, then:

- **Convex** — keep `services/convex`, `convex-codegen.sh`, and Clerk. Drop `openapi-codegen.sh` + its settings registration. In the native/CLI skills, the Client layer uses **Convex's SDK**, not a generated OpenAPI client — adjust their "Client layer" note accordingly.
- **OpenAPI** — keep the TS-Rest server surface (the Server CLI's `contract/` + `server/`), `openapi-codegen.sh`, and Clerk. Drop `convex-codegen.sh` + its registration and the `services/convex` references; `web-development` uses the **TS-Rest client** in place of Convex.
- **No API** — local-only. Drop both `convex-codegen.sh` and `openapi-codegen.sh` (+ registrations), `services/convex`, the networking/OpenAPI rows in the skills, and Clerk (unless you keep local identity). Each client keeps only its on-device database.

## Step 4 — Rewrite the surviving docs

After deleting, the kept docs must read as if the template were always this shape — no dangling references:

- `CLAUDE.md` — trim the layout tree, the Workflow-skills table, the MCP-bridge bullets, and the "What lives where" rows to the surviving platforms.
- `README.md` — trim the skill/command/hook catalogs, the repo-layout block, and the "Native everywhere" bullets.
- `specs/ARCHITECTURE.md`, `specs/DESIGN_SYSTEM.md`, `STACK.md` — remove the dropped rows/columns/sections.
- `.claude/settings.json` — confirm it still parses (`jq -e . .claude/settings.json`).
- Run `rg -n "<dropped-platform>"` across `CLAUDE.md README.md specs/ STACK.md .claude/` to catch stragglers.

## Step 5 — Self-remove

This command is one-time. Offer to delete `.claude/commands/setup.md` and remove the `/setup` row from the slash-command tables in `CLAUDE.md` and `README.md`. If the user prefers to keep it (to re-run later), leave it.

## Step 6 — Commit

Land the pruning as a **single commit** so it's easy to inspect or revert:

```
chore: scope template to <platform list> via /setup
```

Then point the user at the next step: scaffold the reference platform under `apps/<platform>/` and author their first feature with the `brainstorming-feature` skill.
