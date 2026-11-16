//! `workbench project`: a project's use of Workbench.

mod init;
mod omp_config;
mod package_templates;
pub mod setup;
mod validate;
mod vendored;

use schemars::schema_for;

use crate::failure::ErrorCode;
use crate::registry::{
    ArgumentKind, ArgumentSpec, CommandSpec, DRY_RUN, Effect, Example, Requirement, ValueKind,
};

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        id: "project.validate",
        path: &["project", "validate"],
        summary: "Check the project's Workbench setup: pin, link, harness declarations, names, vendored files",
        description: "Checks how the project uses Workbench and reports each problem as a violation \
            with a `path` relative to the project root and a `message`: `mise.toml` pinning no \
            `github:markmals/workbench` version (fix: `mise use github:markmals/workbench@<version>`); \
            a `.workbench` link that is missing, dangling, or holds a different version than the pin \
            (the link's version is the `version` in its `.claude-plugin/plugin.json`; fix: `mise \
            install`); a missing or unreadable harness declaration (the `workbench` marketplace and the \
            `workbench@workbench` plugin in `.claude/settings.json`, the `./.workbench` extension in \
            `.omp/config.yml`); a project skill `.agents/skills/<name>/` or rule \
            `.agents/rules/<name>.md` that reuses the name of a skill or rule this release ships; and \
            vendored Workbench files the package now supplies: templates in `.agents/templates/` \
            identical to the package's (a different one is an override and passes), the Oh My Pi \
            extensions `check-before-stop`, `format-after-edit`, and `prime-beads`, \
            `.omp/tools/dynamic-workflows/`, the TypeScript tools this CLI replaces and their tests, \
            `.changeset/config.json`, `docs/.vitepress/`, and `fnox.toml`. Templates are compared \
            with `.workbench/templates/`, or with the copies built into workbench from the same \
            release when the link is missing. Workbench's own repository, whose `.workbench` links \
            to its working tree's `plugin/`, has no pin and skips the pin and link check. Any \
            violation fails with CHECK_FAILED, and the result still goes to stdout. Never changes \
            anything.",
        arguments: &[],
        effect: Effect::Reads,
        requires: &[Requirement::Git],
        dry_run: false,
        idempotent: true,
        errors: &[
            ErrorCode::CheckFailed,
            ErrorCode::InvalidArgument,
            ErrorCode::PreconditionFailed,
            ErrorCode::DependencyMissing,
        ],
        examples: &[Example {
            summary: "Check the project's Workbench setup",
            args: &["project", "validate"],
        }],
        synonyms: &[
            "check",
            "verify",
            "lint",
            "setup",
            "collision",
            "vendored",
            "link",
            "pin",
            "harness",
            "doctor",
        ],
        output_schema: || schema_for!(validate::Validation),
        run: validate::run,
    },
    CommandSpec {
        id: "project.init",
        path: &["project", "init"],
        summary: "Add Workbench to this Git repository without overwriting anything",
        description: "Adds what Workbench owns to the Git repository, creating each missing item and \
            never overwriting or deleting a file. Runs only where `mise.toml` pins \
            `github:markmals/workbench` (otherwise PRECONDITION_FAILED; `mise use \
            github:markmals/workbench@<version>` writes the pin). To existing structured files it \
            adds only missing keys and entries: `mise.toml` gets the `postinstall` hook that links \
            `.workbench`, pins for hk, gh, and Beads, and the `status`, `validate`, `beads:sync`, \
            `beads:claim`, and `beads:close` tasks; `.gitignore` gets `.workbench`; \
            `.claude/settings.json` gets the `workbench` marketplace and the `workbench@workbench` \
            plugin; `.omp/config.yml` gets the `./.workbench` extension. It links `.claude/skills` \
            and `.claude/rules` to `../.agents/skills` and `../.agents/rules` when those directories \
            exist. It creates, when absent: `hk.pkl` (the Scoped Commits check on `commit-msg` and \
            the Beads hook steps and syncs), `.github/workflows/workbench.yml` (commit and \
            change-file checks on every pull request but Knope's release pull request), an \
            `AGENTS.md` stub with the `.workbench` line, `VISION.md` from the template \
            (`.agents/templates/VISION.md` when the project overrides it) with today's date in \
            place of `<YYYY-MM-DD>`, so record validation accepts it, the `proposals/`, \
            `policies/`, `decisions/`, and `guides/` READMEs, `.agents/commit-scopes`, and \
            `docs/hugo.toml`. `--site github-pages` or `--site cloudflare` also adds the `hugo` and \
            `github:CloudCannon/pagefind` pins, the `docs:build` and `docs:dev` tasks, \
            `docs/.gitignore` for the build output in `docs/public`, and that host's publishing and \
            preview workflows, `.github/workflows/docs.yml` and `preview.yml`, which pass the base \
            URL through `HUGO_BASEURL`; Cloudflare also gets `docs/wrangler.jsonc`, the Worker that \
            serves the site. The \
            receipt lists what it created, what it added to with the entries added, what it left \
            unchanged, what needs a person's attention (such as an `AGENTS.md` without the \
            `.workbench` line), every Workbench file the project still vendors (templates marked \
            `identical`, removable, or `different`, an override), and the project-owned pieces \
            Workbench relies on but does not create: the `setup`, `check`, and `fmt` tasks and a \
            release flow that consumes change files with Knope. `--dry-run` returns the same receipt \
            with status `dry-run` and writes nothing. Exits 0 unless a file could not be written, \
            which fails with PRECONDITION_FAILED while the receipt still reports what was written. \
            Afterwards, `mise install` links `.workbench`, and `workbench project validate` and \
            `workbench context show` read the setup back.",
        arguments: &[SITE, DRY_RUN],
        effect: Effect::Modifies(
            "project files: Workbench's mise, Git, and harness declarations, hooks, CI workflow, and record scaffolding",
        ),
        requires: &[Requirement::Git],
        dry_run: true,
        idempotent: true,
        errors: &[
            ErrorCode::InvalidArgument,
            ErrorCode::PreconditionFailed,
            ErrorCode::DependencyMissing,
        ],
        examples: &[
            Example {
                summary: "Preview what adopting Workbench would create and add",
                args: &["project", "init", "--dry-run"],
            },
            Example {
                summary: "Preview adoption with a documentation site on GitHub Pages",
                args: &["project", "init", "--site", "github-pages", "--dry-run"],
            },
        ],
        synonyms: &[
            "adopt",
            "setup",
            "install",
            "bootstrap",
            "scaffold",
            "create",
            "new",
            "migrate",
            "start",
        ],
        output_schema: || schema_for!(init::Receipt),
        run: init::run,
    },
];

const SITE: ArgumentSpec = ArgumentSpec {
    name: "site",
    kind: ArgumentKind::Option,
    value: ValueKind::Choice(&["github-pages", "cloudflare"]),
    required: false,
    default: None,
    multiple: false,
    summary: "Also set up the documentation site and its publishing and preview workflows on this host",
};
