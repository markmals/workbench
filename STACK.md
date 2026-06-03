# Stack

The canonical toolchain catalog for this template — every tool, framework, and service it knows how to wire up, organized by layer. This is the **superset**: a given product uses a subset. Run `/setup` on a fresh copy to declare which platforms you ship and prune the rest.

Spec-driven-development multiplatform application toolkit powered by Claude.

## Specification

| Concern                | Choice                                                                                             |
| ---------------------- | -------------------------------------------------------------------------------------------------- |
| Product specs          | Markdown in `specs/` & `features/`                                                                 |
| Architecture decisions | [ADRs](https://adr.github.io/)                                                                     |
| API contracts          | [Convex](https://docs.convex.dev/functions) schema · [OpenAPI](https://www.openapis.org/) document |
| Runtime validation     | [Zod](https://zod.dev/)                                                                            |
| UI component specs     | [Storybook](https://storybook.js.org/)                                                             |
| Agent instructions     | `CLAUDE.md`                                                                                        |
| Acceptance criteria    | Gherkin-in-markdown                                                                                |

API contracts are mode-dependent and mutually exclusive — a project uses the Convex schema **or** an OpenAPI document **or** neither. See [ARCHITECTURE.md](specs/ARCHITECTURE.md) → "Backend modes".

## Quality

| Concern           | Choice                                                                                                                                                                                                                                                                                                                                 |
| ----------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Unit tests        | [Vitest](https://vitest.dev/), [Swift Testing](https://developer.apple.com/xcode/swift-testing/), [kotlin.test](https://kotlinlang.org/api/core/kotlin-test/), [Cargo Test](https://doc.rust-lang.org/cargo/commands/cargo-test.html), [MSTest](https://learn.microsoft.com/en-us/dotnet/core/testing/unit-testing-csharp-with-mstest) |
| Integration & E2E | [Playwright](https://playwright.dev/)                                                                                                                                                                                                                                                                                                  |
| Component tests   | [Testing Library](https://testing-library.com/)                                                                                                                                                                                                                                                                                        |
| Visual regression | [Playwright screenshots](https://playwright.dev/docs/test-snapshots)                                                                                                                                                                                                                                                                   |
| Contract tests    | Convex / OpenAPI validation                                                                                                                                                                                                                                                                                                            |
| Performance tests | [Lighthouse](https://developer.chrome.com/docs/lighthouse) + [Web Vitals](https://web.dev/articles/vitals)                                                                                                                                                                                                                             |

## Tooling

| Concern                          | Choice                                                |
| -------------------------------- | ----------------------------------------------------- |
| Agent                            | [Claude Code](https://claude.com/product/claude-code) |
| IDE                              | [Visual Studio Code](https://code.visualstudio.com/)  |
| Toolchain manager                | [Mise](https://mise.jdx.dev/)                         |
| Task runner                      | [Mise](https://mise.jdx.dev/tasks/)                   |
| Shell env manager                | [Mise](https://mise.jdx.dev/environments/)            |
| Environment variables            | [Varlock](https://varlock.dev/)                       |
| CI/CD                            | [GitHub Actions](https://github.com/features/actions) |
| Error tracking & crash reporting | [Sentry](https://sentry.io/)                          |
| Feature flags                    | [PostHog](https://posthog.com/)                       |
| Analytics                        | [PostHog](https://posthog.com/)                       |

### Web tooling

| Concern         | Choice                                                                                                                                                             |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Dev runtime     | [Node](https://nodejs.org/en)                                                                                                                                      |
| Package manager | [pnpm](https://pnpm.io/)                                                                                                                                           |
| Dev server      | [Vite](https://vite.dev/)                                                                                                                                          |
| Library bundler | [tsdown](https://tsdown.dev/)                                                                                                                                      |
| Test runner     | [Vitest](https://vitest.dev/)                                                                                                                                      |
| Formatter       | [Oxfmt](https://oxc.rs/docs/guide/usage/formatter.html), [Prettier](https://prettier.io/) + [@prettier/plugin-oxc](https://npmx.dev/package/@prettier/plugin-oxc)  |
| Linter          | [Oxlint](https://oxc.rs/docs/guide/usage/linter.html), [ESLint](https://eslint.org/) + [eslint-plugin-oxlint](https://github.com/oxc-project/eslint-plugin-oxlint) |
| Type checker    | [tsgo](https://npmx.dev/package/@typescript/native-preview)                                                                                                        |
| Dev tools       | [TanStack DevTools](https://tanstack.com/devtools/latest)                                                                                                          |

## Platform

The backend is one of three mutually-exclusive modes (**Convex** · **OpenAPI** · **none**) chosen at `/setup` — see [ARCHITECTURE.md](specs/ARCHITECTURE.md) → "Backend modes". The catalog below is the **Convex** backend; identity is **Clerk** in any remote mode.

| Concern              | Choice                                                                        |
| -------------------- | ----------------------------------------------------------------------------- |
| Serverless functions | [Convex](https://docs.convex.dev/functions)                                   |
| Database             | [Convex](https://docs.convex.dev/database)                                    |
| File storage         | [Convex](https://docs.convex.dev/file-storage)                                |
| Search               | [Convex](https://docs.convex.dev/search)                                      |
| Cron                 | [Convex Scheduling](https://docs.convex.dev/scheduling)                       |
| Queues               | [Convex Workpool](https://www.convex.dev/components/workpool)                 |
| Realtime multiplayer | [Convex ProseMirror sync](https://www.convex.dev/components/prosemirror-sync) |
| Authentication       | [Clerk](https://clerk.com/)                                                   |
| Email                | [Resend](https://resend.com/), [React Email](https://react.email/)            |
| Payments             | [Stripe](https://stripe.com/)                                                 |

## Web Apps

| Concern                | Choice                                                                                                                                                    |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Components             | [React](https://react.dev/learn)                                                                                                                          |
| Optimizer              | [React Compiler](https://react.dev/learn/react-compiler/introduction)                                                                                     |
| Router                 | [TanStack Router](https://tanstack.com/router/latest/docs/overview)                                                                                       |
| Framework              | [TanStack Start](https://tanstack.com/start/latest/docs/framework/react/overview)                                                                         |
| Async state management | [TanStack Query](https://tanstack.com/query/latest)                                                                                                       |
| Local-first storage    | [TanStack DB](https://tanstack.com/db/latest)                                                                                                             |
| Tables                 | [TanStack Table](https://tanstack.com/table/latest)                                                                                                       |
| Forms                  | [TanStack Form](https://tanstack.com/form/latest)                                                                                                         |
| Hotkeys                | [TanStack Hotkeys](https://tanstack.com/hotkeys/latest)                                                                                                   |
| Styles                 | [Tailwind CSS](https://tailwindcss.com/)                                                                                                                  |
| Component styles       | [Tailwind Plus](https://tailwindcss.com/plus/ui-kit)                                                                                                      |
| Unstyled components    | [React Aria](https://react-aria.adobe.com/)                                                                                                               |
| Animations             | [Motion](https://motion.dev/docs/react-quick-start)                                                                                                       |
| Validation             | [Zod](https://zod.dev/)                                                                                                                                   |
| Rich text editor       | [TipTap](https://tiptap.dev/)                                                                                                                             |
| Database               | [Drizzle](https://orm.drizzle.team/docs/get-started) + [`node:sqlite`](https://nodejs.org/api/sqlite.html) or [D1](https://developers.cloudflare.com/d1/) |
| Networking             | [fetch](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API)                                                                                       |
| Logging                | [Evlog](https://www.evlog.dev/)                                                                                                                           |
| Desktop web apps       | [Electron](https://www.electronjs.org/)                                                                                                                   |

## Websites

| Concern              | Choice                                                                                                                                                    |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Framework            | [Astro](https://docs.astro.build/en/concepts/why-astro/)                                                                                                  |
| Components           | [React](https://react.dev/learn)                                                                                                                          |
| Optimizer            | [React Compiler](https://react.dev/learn/react-compiler/introduction)                                                                                     |
| Styles               | [Tailwind CSS](https://tailwindcss.com/)                                                                                                                  |
| Component styles     | [Tailwind Plus](https://tailwindcss.com/plus/ui-kit)                                                                                                      |
| Unstyled components  | [React Aria](https://react-aria.adobe.com/)                                                                                                               |
| Animations           | [View Transitions](https://docs.astro.build/en/guides/view-transitions/)                                                                                  |
| Validation           | [Zod](https://zod.dev/)                                                                                                                                   |
| Internationalization | [Astro](https://docs.astro.build/en/recipes/i18n/)                                                                                                        |
| Database             | [Drizzle](https://orm.drizzle.team/docs/get-started) + [`node:sqlite`](https://nodejs.org/api/sqlite.html) or [D1](https://developers.cloudflare.com/d1/) |
| Markdown             | [Content collections](https://docs.astro.build/en/guides/content-collections/)                                                                            |
| Networking           | [fetch](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API)                                                                                       |
| Logging              | [Evlog](https://www.evlog.dev/)                                                                                                                           |

## Deployment

| Concern            | Choice                                                                      |
| ------------------ | --------------------------------------------------------------------------- |
| Domains            | [Cloudflare](https://www.cloudflare.com/products/registrar/)                |
| DNS                | [Cloudflare](https://www.cloudflare.com/application-services/products/dns/) |
| CDN                | [Cloudflare](https://www.cloudflare.com/application-services/products/cdn/) |
| Image optimization | [Cloudflare](https://developers.cloudflare.com/images/)                     |
| Observability      | [Cloudflare](https://developers.cloudflare.com/workers/observability/)      |
| Bot protection     | [Cloudflare Turnstile](https://www.cloudflare.com/products/turnstile/)      |
| Static hosting     | [Cloudflare](https://developers.cloudflare.com/workers/static-assets/)      |
| Edge hosting       | [Cloudflare](https://workers.cloudflare.com/)                               |
| VPS hosting        | [Railway](https://railway.com/deploy/bun-starter)                           |

## CLI

The CLI is **one platform with a choice of stack** — Node, Rust, or Go — chosen at `/setup` and mutually exclusive, the way a backend mode is. One CLI per app: a second CLI is a second app, not a second folder (see [CLAUDE.md](CLAUDE.md) → "One product, or several related ones"). The catalog documents all three stacks; a project keeps one and prunes the rest. Whichever you pick lives at `apps/cli`.

- **Node** — headless automation; in OpenAPI mode the TS-Rest server also **hosts** the backend.
- **Rust** — a high-performance binary with a rich TUI via [charmed_rust](https://github.com/dicklesworthstone/charmed_rust), a Rust port of the Charm ecosystem.
- **Go** — a single binary with a rich TUI via the [Charm](https://charm.sh/) ecosystem; in OpenAPI mode it can host the backend via [`oapi-codegen`](https://github.com/oapi-codegen/oapi-codegen).

### Node CLI

| Concern                 | Choice                                                                                                                                                                                          |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Single-file executables | [Node](https://nodejs.org/api/single-executable-applications.html)                                                                                                                            |
| Bundler                 | [tsdown](https://tsdown.dev/options/exe)                                                                                                                                                       |
| Argument parser         | [Bombshell Args](https://github.com/bombshell-dev/args)                                                                                                                                        |
| Prompts                 | [Bombshell Clack](https://github.com/bombshell-dev/clack)                                                                                                                                      |
| Completions             | [Bombshell Tab](https://github.com/bombshell-dev/tab)                                                                                                                                          |
| Server                  | [TS-Rest](https://ts-rest.com/server/serverless/fetch-runtimes)                                                                                                                                |
| RPC                     | [TS-Rest](https://ts-rest.com/client/fetch)                                                                                                                                                    |
| OpenAPI                 | [TS-Rest](https://ts-rest.com/openapi)                                                                                                                                                         |
| Database                | [Drizzle](https://orm.drizzle.team/docs/connect-node-sqlite) + [`node:sqlite`](https://nodejs.org/api/sqlite.html)                                                                            |
| Networking              | [fetch](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API)                                                                                                                           |
| Logging                 | [Evlog](https://www.evlog.dev/)                                                                                                                                                                |
| Background jobs         | [plainjob](https://github.com/justplainstuff/plainjob)                                                                                                                                         |
| Distribution            | [Homebrew](https://brew.sh/), [Mise](https://mise.jdx.dev/), [apt](<https://en.wikipedia.org/wiki/APT_(software)>), [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/) |

### Rust CLI

Full parity with the Go stack via [charmed_rust](https://github.com/dicklesworthstone/charmed_rust) — a Rust port of the Charm ecosystem (requires Rust 1.85+, edition 2024).

| Concern                 | Choice                                                                                                                                                                                          |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Single-file executables | [Rust](https://rust-lang.org/)                                                                                                                                                                 |
| Argument parser         | [Clap](https://github.com/clap-rs/clap)                                                                                                                                                        |
| State management        | [charmed_rust](https://github.com/dicklesworthstone/charmed_rust) (bubbletea)                                                                                                                  |
| Views                   | [charmed_rust](https://github.com/dicklesworthstone/charmed_rust) (bubbletea + bubbles)                                                                                                        |
| Styling                 | [charmed_rust](https://github.com/dicklesworthstone/charmed_rust) (lipgloss)                                                                                                                   |
| Prompts/forms           | [charmed_rust](https://github.com/dicklesworthstone/charmed_rust) (huh)                                                                                                                        |
| Markdown rendering      | [charmed_rust](https://github.com/dicklesworthstone/charmed_rust) (glamour)                                                                                                                    |
| Animations              | [charmed_rust](https://github.com/dicklesworthstone/charmed_rust) (harmonica)                                                                                                                  |
| SSH app framework       | [charmed_rust](https://github.com/dicklesworthstone/charmed_rust) (wish)                                                                                                                       |
| Networking              | [reqwest](https://github.com/seanmonstar/reqwest)                                                                                                                                              |
| Serialization           | [serde_json](https://github.com/serde-rs/json)                                                                                                                                                 |
| Logging                 | [charmed_rust](https://github.com/dicklesworthstone/charmed_rust) (charmed_log)                                                                                                                |
| Embedded assets         | [rust-embed](https://github.com/pyrossh/rust-embed)                                                                                                                                            |
| Database interface      | [Diesel](https://diesel.rs/) (SQLite)                                                                                                                                                          |
| OpenAPI client          | [Progenitor](https://github.com/oxidecomputer/progenitor)                                                                                                                                      |
| Tests                   | [cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html)                                                                                                                         |
| Snapshot tests          | [insta](https://insta.rs/)                                                                                                                                                                     |
| Formatter               | [rustfmt](https://github.com/rust-lang/rustfmt)                                                                                                                                                |
| Linter/static checks    | [Clippy](https://doc.rust-lang.org/stable/clippy/usage.html)                                                                                                                                   |
| Docs                    | [rustdoc](https://doc.rust-lang.org/rustdoc/)                                                                                                                                                  |
| Package manager         | [Cargo](https://crates.io/)                                                                                                                                                                     |
| Distribution            | [Homebrew](https://brew.sh/), [Mise](https://mise.jdx.dev/), [apt](<https://en.wikipedia.org/wiki/APT_(software)>), [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/) |

### Go CLI

| Concern                 | Choice                                                                                                                                                                                          |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Single-file executables | [Go](https://go.dev/)                                                                                                                                                                          |
| CLI starter kit         | [Fang](https://github.com/charmbracelet/fang)                                                                                                                                                  |
| Argument parser         | [Cobra](https://github.com/spf13/cobra)                                                                                                                                                        |
| State management        | [Bubble Tea](https://github.com/charmbracelet/bubbletea)                                                                                                                                       |
| Views                   | [Bubble Tea](https://github.com/charmbracelet/bubbletea) + [Bubbles](https://github.com/charmbracelet/bubbles)                                                                                 |
| Styling                 | [Lip Gloss](https://github.com/charmbracelet/lipgloss)                                                                                                                                         |
| Prompts/forms           | [Huh](https://github.com/charmbracelet/huh)                                                                                                                                                    |
| Markdown rendering      | [Glamour](https://github.com/charmbracelet/glamour)                                                                                                                                            |
| Animations              | [Harmonica](https://github.com/charmbracelet/harmonica)                                                                                                                                        |
| SSH directory           | [Wishlist](https://github.com/charmbracelet/wishlist)                                                                                                                                          |
| SSH app framework       | [Wish](https://github.com/charmbracelet/wish)                                                                                                                                                  |
| Networking              | [`net/http`](https://pkg.go.dev/net/http)                                                                                                                                                      |
| Serialization           | [`encoding/json`](https://pkg.go.dev/encoding/json)                                                                                                                                            |
| Logging                 | [`log/slog`](https://pkg.go.dev/log/slog)                                                                                                                                                      |
| Embedded assets         | [`embed`](https://pkg.go.dev/embed)                                                                                                                                                            |
| Database interface      | [`database/sql`](https://pkg.go.dev/database/sql)                                                                                                                                              |
| SQLite driver           | [glebarez/go-sqlite](https://github.com/glebarez/go-sqlite)                                                                                                                                    |
| OpenAPI server          | [oapi-codegen](https://github.com/oapi-codegen/oapi-codegen)                                                                                                                                   |
| OpenAPI client          | [oapi-codegen](https://github.com/oapi-codegen/oapi-codegen)                                                                                                                                   |
| Tests                   | [`testing`](https://pkg.go.dev/testing), [`go test`](https://pkg.go.dev/cmd/go#hdr-Test_packages)                                                                                              |
| Snapshot tests          | [`testing`](https://pkg.go.dev/testing) + files in `testdata/`                                                                                                                                 |
| Formatter               | [`go fmt`](https://pkg.go.dev/cmd/gofmt)                                                                                                                                                       |
| Linter/static checks    | [`go vet`](https://pkg.go.dev/cmd/vet)                                                                                                                                                          |
| Docs                    | [`go doc`](https://pkg.go.dev/cmd/doc)                                                                                                                                                         |
| Package manager         | [Go modules](https://go.dev/ref/mod)                                                                                                                                                           |
| Distribution            | [Homebrew](https://brew.sh/), [Mise](https://mise.jdx.dev/), [apt](<https://en.wikipedia.org/wiki/APT_(software)>), [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/) |

## Apple

Targets: iOS · iPadOS · macOS · tvOS · watchOS · visionOS.

| Concern                                | Choice                                                                                                                                  |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Language                               | [Swift](https://www.swift.org/)                                                                                                         |
| Concurrency                            | [Swift Concurrency](https://developer.apple.com/documentation/swift/concurrency)                                                        |
| State management                       | [Swift Observation](https://developer.apple.com/documentation/Observation)                                                              |
| Views (iOS · iPadOS · tvOS · visionOS) | [UIKit](https://developer.apple.com/documentation/uikit)                                                                                |
| Views (macOS)                          | [AppKit](https://developer.apple.com/documentation/appkit)                                                                              |
| Views (watchOS)                        | [SwiftUI](https://developer.apple.com/swiftui/)                                                                                         |
| Database                               | [SwiftData](https://developer.apple.com/documentation/swiftdata)                                                                        |
| Networking                             | [URLSession](https://developer.apple.com/documentation/foundation/urlsession)                                                           |
| OpenAPI client                         | [Swift OpenAPI Generator](https://github.com/apple/swift-openapi-generator)                                                             |
| Push notifications                     | [APNs](https://developer.apple.com/documentation/usernotifications/sending-notification-requests-to-apns)                               |
| Test runner                            | [Swift Testing](https://developer.apple.com/xcode/swift-testing/)                                                                       |
| Formatter                              | [swift-format](https://github.com/swiftlang/swift-format#formatting)                                                                    |
| Linter                                 | [swift-format](https://github.com/swiftlang/swift-format#linting)                                                                       |
| Package manager                        | [Swift Package Manager](https://developer.apple.com/documentation/xcode/swift-packages)                                                 |
| Project manager                        | [Tuist](https://tuist.dev)                                                                                                              |
| IDE MCP                                | [Xcode MCP](https://developer.apple.com/documentation/xcode/giving-external-agents-access-to-xcode)                                     |
| Distribution                           | [TestFlight](https://developer.apple.com/testflight/), [App Store](https://www.apple.com/app-store/), [Homebrew](https://brew.sh/), web |

## Android

| Concern            | Choice                                                                                                             |
| ------------------ | ------------------------------------------------------------------------------------------------------------------ |
| Language           | [Kotlin](https://kotlinlang.org/multiplatform/)                                                                    |
| Concurrency        | [Kotlin Coroutines](https://kotlinlang.org/docs/coroutines-overview.html)                                          |
| State management   | [Kotlin Flows](https://kotlinlang.org/docs/flow.html)                                                              |
| Views              | [Jetpack Compose](https://developer.android.com/compose)                                                           |
| Database           | [Room](https://developer.android.com/jetpack/androidx/releases/room)                                               |
| Networking         | [Ktor](https://ktor.io/docs/client-create-and-configure.html) + [OkHttp](https://ktor.io/docs/client-engines.html) |
| OpenAPI client     | [OpenAPI Generator](https://openapi-generator.tech/docs/generators/kotlin/)                                        |
| Push notifications | [Firebase Cloud Messaging](https://firebase.google.com/docs/cloud-messaging)                                       |
| Test runner        | [kotlin.test](https://kotlinlang.org/api/core/kotlin-test/)                                                        |
| Formatter          | [ktfmt](https://github.com/facebook/ktfmt)                                                                         |
| Linter             | [ktlint](https://github.com/pinterest/ktlint)                                                                      |
| Package manager    | [Gradle](https://gradle.org/)                                                                                      |
| IDE MCP            | [JetBrains MCP](https://www.jetbrains.com/help/idea/mcp-server.html)                                               |
| Distribution       | [Google Play Store](https://play.google.com/)                                                                      |

## Windows

| Concern            | Choice                                                                                                                                 |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------------- |
| Language           | [C#](https://learn.microsoft.com/en-us/dotnet/csharp/)                                                                                 |
| Concurrency        | [async/await](https://learn.microsoft.com/en-us/dotnet/csharp/asynchronous-programming/)                                               |
| State management   | [MVVM Toolkit](https://learn.microsoft.com/en-us/dotnet/communitytoolkit/mvvm/)                                                        |
| Views              | [WinUI](https://learn.microsoft.com/en-us/windows/apps/winui/winui3/) + [XAML](https://github.com/microsoft/microsoft-ui-xaml)         |
| Database           | [EF Core](https://learn.microsoft.com/en-us/ef/core/) (SQLite)                                                                         |
| Networking         | [HttpClient](https://learn.microsoft.com/en-us/dotnet/fundamentals/networking/http/httpclient)                                         |
| OpenAPI client     | [Kiota](https://learn.microsoft.com/en-us/openapi/kiota/quickstarts/dotnet)                                                            |
| Push notifications | [Windows App SDK](https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/app-notifications-quickstart) |
| Test runner        | [MSTest](https://learn.microsoft.com/en-us/dotnet/core/testing/unit-testing-csharp-with-mstest)                                        |
| Formatter          | [dotnet format](https://learn.microsoft.com/en-us/dotnet/core/tools/dotnet-format)                                                     |
| Linter             | [StyleCop](https://github.com/DotNetAnalyzers/StyleCopAnalyzers)                                                                       |
| Package manager    | [NuGet](https://www.nuget.org/)                                                                                                        |
| IDE MCP            | [RoslynMcpExtension](https://github.com/sailro/RoslynMcpExtension)                                                                     |
| Distribution       | [Microsoft Store](https://apps.microsoft.com/), [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/), web       |

## Linux

| Concern          | Choice                                                                                                            |
| ---------------- | ----------------------------------------------------------------------------------------------------------------- |
| Language         | [Rust](https://rust-lang.org/)                                                                                    |
| Concurrency      | [Tokio](https://tokio.rs/)                                                                                        |
| State management | [Relm4](https://github.com/Relm4/Relm4/blob/main/examples/simple_manual.rs)                                       |
| Views            | [GTK 4](https://relm4.org/book/stable/gtk_rs.html) + [Adwaita](https://relm4.org/docs/next/libadwaita/index.html) |
| Database         | [Diesel](https://diesel.rs/) (SQLite)                                                                             |
| Networking       | [reqwest](https://github.com/seanmonstar/reqwest)                                                                 |
| OpenAPI client   | [Progenitor](https://github.com/oxidecomputer/progenitor)                                                         |
| Test runner      | [Cargo Test](https://doc.rust-lang.org/cargo/commands/cargo-test.html)                                            |
| Formatter        | [rustfmt](https://github.com/rust-lang/rustfmt)                                                                   |
| Linter           | [Clippy](https://doc.rust-lang.org/stable/clippy/usage.html)                                                      |
| Package manager  | [Cargo](https://crates.io/)                                                                                       |
| Distribution     | [Flatpak](https://flatpak.org/)                                                                                   |
