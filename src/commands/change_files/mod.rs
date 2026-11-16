//! `workbench change-files`: Knope change files for release notes.

mod change_file;
mod check;
mod create;
mod knope;

use schemars::schema_for;

use crate::failure::ErrorCode;
use crate::registry::{
    ArgumentKind, ArgumentSpec, CommandSpec, DRY_RUN, Effect, Example, Requirement, ValueKind,
};
use check::ChangeFilesCheck;
use create::ChangeFileReceipt;

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        id: "change-files.create",
        path: &["change-files", "create"],
        summary: "Write a Knope change file describing a change for the release notes",
        description: "Writes `.changeset/<name>.md`, the change file Knope turns into a version \
            bump and a changelog entry. Write it for someone reading release notes, not for the \
            reviewer of the diff. `<name>` comes from the summary the way Knope names change \
            files: ASCII letters and digits lowercased, each run of spaces, `-`, and `_` one `_`, \
            everything else dropped. The file is the frontmatter `<package>: <type>`, the heading \
            `# <summary>`, then the body when given. `--type` must be `major`, `minor`, `patch`, or \
            a type the package's `extra_changelog_sections` add in knope.toml. A single \
            `[package]` is `default`; when knope.toml declares several `[packages.<name>]`, \
            `--package` must name one. Invalid input is INVALID_ARGUMENT, with the accepted values \
            in `details`; without knope.toml it is PRECONDITION_FAILED. It never overwrites: a \
            summary whose file exists is INVALID_ARGUMENT. The receipt gives `status` `created`, \
            the `project`, the file `path`, `type`, and `package`. `--dry-run` validates the same \
            way, writes nothing, and returns `status` `dry-run` with the `content` it would write. \
            Never prompts; use it instead of `knope document-change`.",
        arguments: &[
            ArgumentSpec {
                name: "type",
                kind: ArgumentKind::Option,
                value: ValueKind::String,
                required: true,
                default: None,
                multiple: false,
                summary: "The change type: major, minor, patch, or one the package's knope.toml sections add",
            },
            ArgumentSpec {
                name: "summary",
                kind: ArgumentKind::Option,
                value: ValueKind::String,
                required: true,
                default: None,
                multiple: false,
                summary: "One line for the changelog; it becomes the heading and names the file",
            },
            ArgumentSpec {
                name: "package",
                kind: ArgumentKind::Option,
                value: ValueKind::String,
                required: false,
                default: None,
                multiple: false,
                summary: "The package the change affects; required when knope.toml declares several",
            },
            ArgumentSpec {
                name: "body",
                kind: ArgumentKind::Option,
                value: ValueKind::Text,
                required: false,
                default: None,
                multiple: false,
                summary: "Detail below the heading: text, @<file>, or - for stdin",
            },
            DRY_RUN,
        ],
        effect: Effect::Modifies("one new file under .changeset/"),
        requires: &[Requirement::Git],
        dry_run: true,
        idempotent: false,
        errors: &[
            ErrorCode::InvalidArgument,
            ErrorCode::PreconditionFailed,
            ErrorCode::DependencyMissing,
        ],
        examples: &[
            Example {
                summary: "Describe a new feature",
                args: &[
                    "change-files",
                    "create",
                    "--type",
                    "minor",
                    "--summary",
                    "Export reports as CSV",
                ],
            },
            Example {
                summary: "Preview a fix with detail, writing nothing",
                args: &[
                    "change-files",
                    "create",
                    "--type",
                    "patch",
                    "--summary",
                    "Keep the last row when exporting",
                    "--body",
                    "Exports no longer drop the final row of a report.",
                    "--dry-run",
                ],
            },
        ],
        synonyms: &[
            "changeset",
            "release note",
            "changelog",
            "knope",
            "document change",
            "version bump",
            "add",
            "new",
        ],
        output_schema: || schema_for!(ChangeFileReceipt),
        run: create::create,
    },
    CommandSpec {
        id: "change-files.check",
        path: &["change-files", "check"],
        summary: "Check that a commit range adds a valid change file or carries Release-Note: none",
        description: "Passes when the range, the commits reachable from head and not from base, \
            either adds a Markdown file directly under `.changeset/` with valid change-file \
            frontmatter, or has a commit carrying the Git trailer `Release-Note: none` (the key in \
            any case, the value exactly `none`, parsed by Git the way `git interpret-trailers \
            --parse` parses it). Added files are the ones head adds since its merge base with \
            base, read as committed at head. Valid frontmatter is a `---` line, one `<package>: \
            <type>` line per affected package, and a closing `---` line; when the project has a \
            knope.toml, every package must be one it declares (`default` for a single \
            `[package]`) and every type `major`, `minor`, `patch`, or one that package's \
            `extra_changelog_sections` add. The result lists each added change file with its \
            problems and the commits carrying the trailer. Otherwise it fails with CHECK_FAILED \
            naming both ways to satisfy it. When the check passes but another added file is \
            invalid, stderr warns about it, because Knope fails on it. An unknown revision is \
            NOT_FOUND; base and head without shared history is INVALID_ARGUMENT; an unusable \
            knope.toml is PRECONDITION_FAILED. CI runs it on every pull request except Knope's \
            release pull request.",
        arguments: &[
            ArgumentSpec {
                name: "base",
                kind: ArgumentKind::Positional,
                value: ValueKind::String,
                required: true,
                default: None,
                multiple: false,
                summary: "The range's base, such as the pull request's target branch",
            },
            ArgumentSpec {
                name: "head",
                kind: ArgumentKind::Positional,
                value: ValueKind::String,
                required: true,
                default: None,
                multiple: false,
                summary: "The range's head, such as the pull request's last commit",
            },
        ],
        effect: Effect::Reads,
        requires: &[Requirement::Git],
        dry_run: false,
        idempotent: true,
        errors: &[
            ErrorCode::CheckFailed,
            ErrorCode::InvalidArgument,
            ErrorCode::NotFound,
            ErrorCode::PreconditionFailed,
            ErrorCode::DependencyMissing,
        ],
        examples: &[Example {
            summary: "Check that a branch describes its change for release notes",
            args: &["change-files", "check", "main", "feature"],
        }],
        synonyms: &[
            "changeset",
            "release note",
            "trailer",
            "pull request",
            "require",
            "verify",
            "validate",
            "ci",
        ],
        output_schema: || schema_for!(ChangeFilesCheck),
        run: check::check,
    },
];
