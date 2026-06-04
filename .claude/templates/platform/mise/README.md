# Per-platform `mise.toml` templates

Starting points for the `mise.toml` you create when you scaffold a platform. Copy
the matching file to its destination, drop the `<PLACEHOLDERS>`, and prune the
tasks you don't use.

| Template        | Copy to                     | Stack                            |
| --------------- | --------------------------- | -------------------------------- |
| `web.toml`      | `apps/web/mise.toml`        | React + TanStack Start + Convex  |
| `website.toml`  | `apps/website/mise.toml`    | Astro + React islands            |
| `convex.toml`   | `services/convex/mise.toml` | Convex backend                   |
| `ios.toml`      | `apps/ios/mise.toml`        | Swift + UIKit + Tuist            |
| `android.toml`  | `apps/android/mise.toml`    | Kotlin + Compose + Gradle        |
| `windows.toml`  | `apps/windows/mise.toml`    | C# + WinUI 3 + EF Core           |
| `linux.toml`    | `apps/linux/mise.toml`      | Rust + GTK 4 + Adwaita           |
| `cli-node.toml` | `apps/cli/mise.toml`        | Node CLI (TS-Rest + Bombshell)   |
| `cli-rust.toml` | `apps/cli/mise.toml`        | Rust CLI (Clap + charmed_rust)   |
| `cli-go.toml`   | `apps/cli/mise.toml`        | Go CLI (Cobra/Fang + Bubble Tea) |

The CLI is **one platform, one stack** — pick a single `cli-*.toml`.

## The task-name contract

Task **names** are not free choices. The hooks and `sdd-*` commands dispatch to
them by name, so a platform that renames `fmt` silently loses formatting. Keep
these names (or their documented aliases) and you inherit the whole harness:

| Task                  | Who calls it                                                             | Notes                                                                       |
| --------------------- | ------------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| `fmt`                 | `format-on-edit.sh` → `mise run -C <dir> fmt -- <file>`                  | **Must accept optional file paths** — narrow to those, else whole platform. |
| `lint` (ios: `l`)     | `stop-lint.sh` → `mise run -C <dir> lint`                                | iOS is called as `l` (alias). Android/Windows lint is run manually (slow).  |
| `test` (ios: `t`)     | `/sdd-verify` → `mise run -C <dir> test`                                 | The spec-bound suite. This is the verification surface.                     |
| `codegen`             | `convex-codegen.sh` (`mise run codegen`) · `openapi-codegen.sh` reminder | Convex types; or the per-platform OpenAPI client (OpenAPI mode only).       |
| `generate` (ios: `g`) | `tuist-regen.sh` → `mise run g`                                          | iOS only — regenerates the Xcode project from `Project.swift`.              |

The `fmt`-takes-paths contract is satisfied two ways, both verified against this
mise version (2026.6):

- **trailing append** — `run = "oxfmt"`; `mise run fmt -- a.ts` runs `oxfmt a.ts`.
  Works for a **single-command** task when the tool formats the cwd with no args
  and a path with one (oxfmt). mise appends the args to that one command.
- **`usage` arg + `set --`** — when "no args" and "some paths" need _different_
  invocations (e.g. `cargo fmt` for the whole crate vs `rustfmt <file>` per file):

    ```toml
    usage = 'arg "[paths]" var=#true'   # optional, variadic; no default
    run = '''
    set -- $usage_paths
    if [ "$#" -gt 0 ]; then rustfmt --edition 2024 "$@"; else cargo fmt; fi
    '''
    ```

    mise exposes the parsed arg as `$usage_paths`; `set --` re-seats it as the
    shell's positional params so `$#` branches cleanly (empty when bare). Do **not**
    use the older `{{arg(...)}}` / `{{flag(...)}}` template functions in `run` — mise
    deprecated them (removed in 2027.5) and they quote a multi-token default as one
    bogus argument.

## Root orchestration

Per-platform tasks are local (`mise run -C apps/web dev`). The **cross-platform**
names — `web:dev`, `ios:test`, `website:build` — are thin wrappers you add to the
repo-root `mise.toml` so they work from anywhere. When you scaffold a platform,
add its wrappers there too:

```toml
# in the repo-root mise.toml
[tasks."web:dev"]   { run = "mise run -C apps/web dev" }
[tasks."web:test"]  { run = "mise run -C apps/web test" }
```

## Tools

`[tools]` pins runtimes mise can install directly (node, pnpm, rust, go, dotnet,
java, tuist). Auxiliary tools — formatters, codegen CLIs — are either project
dependencies resolved through the build system (Gradle, SPM, cargo, the app's
own `package.json`) or commented with the backend to add. After copying a
template, run `mise install` in its directory and fix anything that won't resolve.
