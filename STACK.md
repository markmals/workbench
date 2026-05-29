# Stack

The canonical toolchain catalog for this template — every tool, framework, and service it knows how to wire up, organized by layer. This is the **superset**: a given product uses a subset. Run `/setup` on a fresh copy to declare which platforms you ship and prune the rest.

Spec-driven-development multiplatform application toolkit powered by Claude.

## Tooling

| Concern               | Choice                                                |
| --------------------- | ----------------------------------------------------- |
| Agent                 | [Claude Code](https://claude.com/product/claude-code) |
| IDE                   | [Visual Studio Code](https://code.visualstudio.com/)  |
| Toolchain manager     | [Mise](https://mise.jdx.dev/)                         |
| Task runner           | [Mise](https://mise.jdx.dev/tasks/)                   |
| Environment manager   | [Mise](https://mise.jdx.dev/environments/)            |
| Environment variables | [Varlock](https://varlock.dev/)                       |
| CI/CD                 | [GitHub Actions](https://github.com/features/actions) |

### Web tooling

| Concern         | Choice                                                                                                                             |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Dev runtime     | [Node](https://nodejs.org/en)                                                                                                      |
| Package manager | [pnpm](https://pnpm.io/)                                                                                                           |
| Dev server      | [Vite](https://vite.dev/)                                                                                                          |
| Library bundler | [tsdown](https://tsdown.dev/)                                                                                                      |
| Test runner     | [Vitest](https://vitest.dev/)                                                                                                      |
| Formatter       | [Oxfmt](https://oxc.rs/docs/guide/usage/formatter.html), [@prettier/plugin-oxc](https://npmx.dev/package/@prettier/plugin-oxc)     |
| Linter          | [Oxlint](https://oxc.rs/docs/guide/usage/linter.html), [eslint-plugin-oxlint](https://github.com/oxc-project/eslint-plugin-oxlint) |
| Type checker    | [tsgo](https://npmx.dev/package/@typescript/native-preview)                                                                        |
| Dev tools       | [TanStack DevTools](https://tanstack.com/devtools/latest)                                                                          |

## Platform

| Concern              | Choice                                                                        |
| -------------------- | ----------------------------------------------------------------------------- |
| Database             | [Convex](https://docs.convex.dev/database)                                    |
| File storage         | [Convex](https://docs.convex.dev/file-storage)                                |
| Cron                 | [Convex](https://docs.convex.dev/scheduling)                                  |
| Queues               | [Convex Workpool](https://www.convex.dev/components/workpool)                 |
| Realtime multiplayer | [Convex ProseMirror sync](https://www.convex.dev/components/prosemirror-sync) |
| Authentication       | [Clerk](https://clerk.com/)                                                   |

## Web Apps

| Concern             | Choice                                                                            |
| ------------------- | --------------------------------------------------------------------------------- |
| Components          | [React](https://react.dev/learn)                                                  |
| Optimizer           | [React Compiler](https://react.dev/learn/react-compiler/introduction)             |
| Router              | [TanStack Router](https://tanstack.com/router/latest/docs/overview)               |
| Framework           | [TanStack Start](https://tanstack.com/start/latest/docs/framework/react/overview) |
| Async state         | [TanStack Query](https://tanstack.com/query/latest)                               |
| Local-first storage | [TanStack DB](https://tanstack.com/db/latest)                                     |
| Tables              | [TanStack Table](https://tanstack.com/table/latest)                               |
| Forms               | [TanStack Form](https://tanstack.com/form/latest)                                 |
| Hotkeys             | [TanStack Hotkeys](https://tanstack.com/hotkeys/latest)                           |
| Styles              | [Tailwind CSS](https://tailwindcss.com/)                                          |
| Component styles    | [Tailwind Plus](https://tailwindcss.com/plus/ui-kit)                              |
| Unstyled components | [React Aria](https://react-aria.adobe.com/)                                       |
| Animations          | [Motion](https://motion.dev/docs/react-quick-start)                               |
| Validation          | [Valibot](https://valibot.dev/)                                                   |
| Database            | [Drizzle](https://orm.drizzle.team/docs/connect-cloudflare-d1)                    |
| Networking          | [fetch](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API)               |
| Logging             | [Evlog](https://www.evlog.dev/)                                                   |
| Desktop web apps    | [Electron](https://www.electronjs.org/)                                           |

## Websites

| Concern             | Choice                                                                         |
| ------------------- | ------------------------------------------------------------------------------ |
| Framework           | [Astro](https://docs.astro.build/en/concepts/why-astro/)                       |
| Components          | [React](https://react.dev/learn)                                               |
| Optimizer           | [React Compiler](https://react.dev/learn/react-compiler/introduction)          |
| Styles              | [Tailwind CSS](https://tailwindcss.com/)                                       |
| Component styles    | [Tailwind Plus](https://tailwindcss.com/plus/ui-kit)                           |
| Unstyled components | [React Aria](https://react-aria.adobe.com/)                                    |
| Animations          | [Motion](https://motion.dev/docs/react-quick-start)                            |
| Validation          | [Valibot](https://valibot.dev/)                                                |
| Database            | [Drizzle](https://orm.drizzle.team/docs/connect-cloudflare-d1)                 |
| Markdown            | [Content collections](https://docs.astro.build/en/guides/content-collections/) |
| Networking          | [fetch](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API)            |
| Logging             | [Evlog](https://www.evlog.dev/)                                                |

## Deployment

| Concern            | Choice                                                                      |
| ------------------ | --------------------------------------------------------------------------- |
| Domains            | [Cloudflare](https://www.cloudflare.com/products/registrar/)                |
| DNS                | [Cloudflare](https://www.cloudflare.com/application-services/products/dns/) |
| CDN                | [Cloudflare](https://www.cloudflare.com/application-services/products/cdn/) |
| Image optimization | [Cloudflare](https://developers.cloudflare.com/images/)                     |
| Static hosting     | [Cloudflare](https://developers.cloudflare.com/workers/static-assets/)      |
| Edge hosting       | [Cloudflare](https://workers.cloudflare.com/)                               |
| VPS hosting        | [Railway](https://railway.com/deploy/bun-starter)                           |

## Server CLI

| Concern                 | Choice                                                              |
| ----------------------- | ------------------------------------------------------------------- |
| Single-file executables | [Node](https://nodejs.org/api/single-executable-applications.html)  |
| Bundler                 | [tsdown](https://tsdown.dev/options/exe)                            |
| Argument parser         | [Bombshell Args](https://github.com/bombshell-dev/args)             |
| Prompts                 | [Bombshell Clack](https://github.com/bombshell-dev/clack)           |
| Completions             | [Bombshell Tab](https://github.com/bombshell-dev/tab)               |
| Server                  | [TS-Rest](https://ts-rest.com/server/serverless/fetch-runtimes)     |
| RPC                     | [TS-Rest](https://ts-rest.com/client/fetch)                         |
| OpenAPI                 | [TS-Rest](https://ts-rest.com/openapi)                              |
| Database                | [Drizzle](https://orm.drizzle.team/docs/connect-node-sqlite)        |
| Networking              | [fetch](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API) |
| Logging                 | [Evlog](https://www.evlog.dev/)                                     |
| Background jobs         | [plainjob](https://github.com/justplainstuff/plainjob)              |

## High-Performance CLI

| Concern                 | Choice                                                                 |
| ----------------------- | ---------------------------------------------------------------------- |
| Single-file executables | [Rust](https://rust-lang.org/)                                         |
| Argument parser         | [Clap](https://github.com/clap-rs/clap)                                |
| Renderer                | [Ratatui](https://github.com/ratatui/ratatui)                          |
| Database                | [Diesel](https://diesel.rs/) (SQLite)                                  |
| Networking              | [reqwest](https://github.com/seanmonstar/reqwest)                      |
| OpenAPI                 | [Progenitor](https://github.com/oxidecomputer/progenitor)              |
| Test runner             | [Cargo Test](https://doc.rust-lang.org/cargo/commands/cargo-test.html) |
| Formatter               | [rustfmt](https://github.com/rust-lang/rustfmt)                        |
| Linter                  | [Clippy](https://doc.rust-lang.org/stable/clippy/usage.html)           |
| Package manager         | [Cargo](https://crates.io/)                                            |

## Apple

Targets: iOS · iPadOS · macOS · tvOS · watchOS · visionOS.

| Concern         | Choice                                                                                                                |
| --------------- | --------------------------------------------------------------------------------------------------------------------- |
| Language        | [Swift](https://www.swift.org/)                                                                                       |
| Concurrency     | [Swift Concurrency](https://developer.apple.com/documentation/swift/concurrency)                                      |
| Reactivity      | [Swift Observation](https://developer.apple.com/documentation/Observation)                                            |
| Views           | [SwiftUI](https://developer.apple.com/swiftui/)                                                                       |
| Database        | [SwiftData](https://developer.apple.com/documentation/swiftdata)                                                      |
| Networking      | [URLSession](https://developer.apple.com/documentation/foundation/urlsession)                                         |
| OpenAPI         | [Swift OpenAPI Generator](https://github.com/apple/swift-openapi-generator)                                           |
| Test runner     | [Swift Testing](https://developer.apple.com/xcode/swift-testing/)                                                     |
| Formatter       | [swift-format](https://github.com/swiftlang/swift-format#formatting)                                                  |
| Linter          | [swift-format](https://github.com/swiftlang/swift-format#linting)                                                     |
| Package manager | [Swift Package Manager](https://developer.apple.com/documentation/xcode/swift-packages)                               |
| Agent ↔ IDE     | [Xcode external agent access](https://developer.apple.com/documentation/xcode/giving-external-agents-access-to-xcode) |

## Android

| Concern         | Choice                                                                                                             |
| --------------- | ------------------------------------------------------------------------------------------------------------------ |
| Language        | [Kotlin](https://kotlinlang.org/multiplatform/)                                                                    |
| Concurrency     | [Kotlin Coroutines](https://kotlinlang.org/docs/coroutines-overview.html)                                          |
| Reactivity      | [Kotlin Flows](https://kotlinlang.org/docs/flow.html)                                                              |
| Views           | [Jetpack Compose](https://developer.android.com/compose)                                                           |
| Database        | [Room](https://developer.android.com/jetpack/androidx/releases/room)                                               |
| Networking      | [Ktor](https://ktor.io/docs/client-create-and-configure.html)                                                      |
| OpenAPI         | [OpenAPI Generator](https://openapi-generator.tech/docs/generators/kotlin/)                                        |
| Test runner     | [kotlin.test](https://kotlinlang.org/api/core/kotlin-test/)                                                        |
| Formatter       | [ktfmt](https://github.com/facebook/ktfmt)                                                                         |
| Linter          | [ktlint](https://github.com/pinterest/ktlint)                                                                      |
| Package manager | [Gradle](https://gradle.org/)                                                                                      |
| Agent ↔ IDE     | [Android Studio / JetBrains MCP server](https://www.jetbrains.com/help/idea/mcp-server.html#external-client-setup) |

## Windows

| Concern         | Choice                                                                                                                         |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Language        | [C#](https://learn.microsoft.com/en-us/dotnet/csharp/)                                                                         |
| Concurrency     | [async/await](https://learn.microsoft.com/en-us/dotnet/csharp/asynchronous-programming/)                                       |
| Reactivity      | [MVVM Toolkit](https://learn.microsoft.com/en-us/dotnet/communitytoolkit/mvvm/)                                                |
| Views           | [WinUI](https://learn.microsoft.com/en-us/windows/apps/winui/winui3/) + [XAML](https://github.com/microsoft/microsoft-ui-xaml) |
| Database        | [EF Core](https://learn.microsoft.com/en-us/ef/core/) (SQLite)                                                                 |
| Networking      | [HttpClient](https://learn.microsoft.com/en-us/dotnet/fundamentals/networking/http/httpclient)                                 |
| OpenAPI         | [Kiota](https://learn.microsoft.com/en-us/openapi/kiota/quickstarts/dotnet)                                                    |
| Test runner     | [MSTest](https://learn.microsoft.com/en-us/dotnet/core/testing/unit-testing-csharp-with-mstest)                                |
| Formatter       | [dotnet format](https://learn.microsoft.com/en-us/dotnet/core/tools/dotnet-format)                                             |
| Linter          | [StyleCop](https://github.com/DotNetAnalyzers/StyleCopAnalyzers)                                                               |
| Package manager | [NuGet](https://www.nuget.org/)                                                                                                |
| Agent ↔ IDE     | [RoslynMcpExtension](https://github.com/sailro/RoslynMcpExtension)                                                             |

## Linux

| Concern         | Choice                                                                                      |
| --------------- | ------------------------------------------------------------------------------------------- |
| Language        | [Rust](https://rust-lang.org/)                                                              |
| Concurrency     | [Tokio](https://tokio.rs/)                                                                  |
| Reactivity      | [Relm4](https://github.com/Relm4/Relm4/blob/main/examples/simple_manual.rs)                 |
| Views           | [GTK 4](https://gtk-rs.org/) + [Adwaita](https://relm4.org/docs/next/libadwaita/index.html) |
| Database        | [Diesel](https://diesel.rs/) (SQLite)                                                       |
| Networking      | [reqwest](https://github.com/seanmonstar/reqwest)                                           |
| OpenAPI         | [Progenitor](https://github.com/oxidecomputer/progenitor)                                   |
| Test runner     | [Cargo Test](https://doc.rust-lang.org/cargo/commands/cargo-test.html)                      |
| Formatter       | [rustfmt](https://github.com/rust-lang/rustfmt)                                             |
| Linter          | [Clippy](https://doc.rust-lang.org/stable/clippy/usage.html)                                |
| Package manager | [Cargo](https://crates.io/)                                                                 |
