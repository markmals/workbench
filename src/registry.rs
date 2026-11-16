//! The command registry: the one declaration of every operation `workbench` performs.
//!
//! The parser, `--help`, shell completions, `workbench schema`, the `workbench cli search` index,
//! and the CLI reference page are all generated from these entries, so a command's declared
//! arguments, defaults, effects, and errors are the ones it actually has.

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::response::Response;

/// One operation, declared once.
pub struct CommandSpec {
    /// Stable identifier, the command path joined with dots: `beads.claim`.
    pub id: &'static str,
    /// The words after `workbench`: `["beads", "claim"]`.
    pub path: &'static [&'static str],
    /// One line, shown in search results and `--help` listings.
    pub summary: &'static str,
    /// Everything an agent needs to use the command correctly, in prose.
    pub description: &'static str,
    pub arguments: &'static [ArgumentSpec],
    pub effect: Effect,
    pub requires: &'static [Requirement],
    /// Whether the command accepts `--dry-run`. Every modifying command does.
    pub dry_run: bool,
    /// Whether running it twice with the same input leaves the same state as running it once.
    pub idempotent: bool,
    /// Every error code the command can return, besides `INTERNAL`.
    pub errors: &'static [ErrorCode],
    /// At least one example. Each one runs against a fixture project in the test suite.
    pub examples: &'static [Example],
    /// Words an agent might use for this operation that its name and summary do not contain.
    pub synonyms: &'static [&'static str],
    /// JSON Schema for the result on stdout.
    pub output_schema: fn() -> schemars::Schema,
    pub run: fn(&Invocation) -> Result<Response, Failure>,
}

/// Whether an operation reads or modifies state, and what it modifies.
#[derive(Clone, Copy)]
pub enum Effect {
    Reads,
    Modifies(&'static str),
}

/// What an operation needs at run time beyond `workbench` itself.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    Git,
    Gh,
    GhAuthentication,
    Bd,
    Network,
    WorkbenchLink,
}

impl Requirement {
    pub fn as_str(self) -> &'static str {
        match self {
            Requirement::Git => "git",
            Requirement::Gh => "gh",
            Requirement::GhAuthentication => "gh-authentication",
            Requirement::Bd => "bd",
            Requirement::Network => "network",
            Requirement::WorkbenchLink => "workbench-link",
        }
    }
}

pub struct ArgumentSpec {
    /// Kebab-case. A flag or option is passed as `--<name>`; a positional is passed bare.
    pub name: &'static str,
    pub kind: ArgumentKind,
    pub value: ValueKind,
    pub required: bool,
    /// The value used when the argument is absent. Clap applies it, so it is the runtime default.
    pub default: Option<&'static str>,
    /// A positional or option that accepts several values.
    pub multiple: bool,
    pub summary: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArgumentKind {
    Positional,
    Option,
    Flag,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    String,
    Path,
    Integer,
    /// Free text, `@<path>` to read a file, or `-` to read stdin.
    Text,
    Choice(&'static [&'static str]),
    Boolean,
}

impl ValueKind {
    pub fn type_name(self) -> &'static str {
        match self {
            ValueKind::String => "string",
            ValueKind::Path => "path",
            ValueKind::Integer => "integer",
            ValueKind::Text => "text",
            ValueKind::Choice(_) => "choice",
            ValueKind::Boolean => "boolean",
        }
    }

    pub fn accepted_values(self) -> Option<&'static [&'static str]> {
        match self {
            ValueKind::Choice(values) => Some(values),
            _ => None,
        }
    }
}

pub struct Example {
    pub summary: &'static str,
    /// The arguments after `workbench`.
    pub args: &'static [&'static str],
}

/// Options every command accepts.
pub const GLOBAL_ARGUMENTS: &[ArgumentSpec] = &[ArgumentSpec {
    name: "output",
    kind: ArgumentKind::Option,
    value: ValueKind::Choice(&["json", "text"]),
    required: false,
    default: Some("json"),
    multiple: false,
    summary: "Result format: JSON (compact when piped, indented on a terminal) or text for humans",
}];

/// The `--dry-run` flag every modifying command declares.
pub const DRY_RUN: ArgumentSpec = ArgumentSpec {
    name: "dry-run",
    kind: ArgumentKind::Flag,
    value: ValueKind::Boolean,
    required: false,
    default: None,
    multiple: false,
    summary: "Validate input, resolve the target, and describe the changes without making them",
};

/// The resource groups, in the order `--help` lists them, with a summary of each.
pub const RESOURCES: &[(&str, &str)] = &[
    ("cli", "Discover workbench's own commands"),
    ("schema", "Inspect a command's full specification"),
    ("docs", "Search the Workbench package's documentation"),
    (
        "context",
        "Inspect the project, pin, link, harness, and Beads setup",
    ),
    ("workflow", "Where the work stands and what happens next"),
    (
        "records",
        "The project's proposals, policies, decisions, guides, and vision",
    ),
    ("project", "Set up and check a project's use of Workbench"),
    ("commits", "Commit subjects and their Scoped Commits scopes"),
    ("change-files", "Knope change files for release notes"),
    (
        "beads",
        "This machine's Beads replica: sync, claim, and close",
    ),
];

/// Every command, in the order discovery lists them.
pub fn all() -> Vec<&'static CommandSpec> {
    let mut commands: Vec<&'static CommandSpec> = Vec::new();
    for group in crate::commands::GROUPS {
        commands.extend(group.iter());
    }
    commands
}

pub fn find(path: &[&str]) -> Option<&'static CommandSpec> {
    all().into_iter().find(|command| command.path == path)
}

impl CommandSpec {
    pub fn command_line(&self) -> String {
        format!("workbench {}", self.path.join(" "))
    }

    pub fn effect_label(&self) -> String {
        match self.effect {
            Effect::Reads => "reads".to_string(),
            Effect::Modifies(what) => format!("modifies {what}"),
        }
    }

    /// The command's arguments followed by the global ones.
    pub fn all_arguments(&self) -> impl Iterator<Item = &'static ArgumentSpec> {
        self.arguments.iter().chain(GLOBAL_ARGUMENTS.iter())
    }

    /// The full entry `workbench schema <command>` returns.
    pub fn describe(&self) -> CommandDescription {
        let errors = self
            .errors
            .iter()
            .chain(std::iter::once(&ErrorCode::Internal))
            .map(|code| ErrorDescription {
                code: code.as_str().to_string(),
                exit: code.exit_status(),
                retryable: code.retryable(),
                meaning: code.meaning().to_string(),
            })
            .collect();
        CommandDescription {
            id: self.id.to_string(),
            command: self.command_line(),
            summary: self.summary.to_string(),
            description: self.description.to_string(),
            arguments: self.all_arguments().map(ArgumentSpec::describe).collect(),
            effect: match self.effect {
                Effect::Reads => EffectDescription {
                    kind: "reads".into(),
                    what: None,
                },
                Effect::Modifies(what) => EffectDescription {
                    kind: "modifies".into(),
                    what: Some(what.to_string()),
                },
            },
            requires: self
                .requires
                .iter()
                .map(|requirement| requirement.as_str().to_string())
                .collect(),
            dry_run: self.dry_run,
            idempotent: self.idempotent,
            errors,
            examples: self
                .examples
                .iter()
                .map(|example| ExampleDescription {
                    summary: example.summary.to_string(),
                    command: example_command_line(example),
                })
                .collect(),
            output: serde_json::to_value((self.output_schema)()).expect("schemas serialize"),
        }
    }
}

/// A command's full specification, as `workbench schema <command>` returns it.
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandDescription {
    /// Stable identifier: the command path joined with dots.
    pub id: String,
    /// The command to run, without arguments.
    pub command: String,
    pub summary: String,
    pub description: String,
    /// The command's own arguments, then the options every command accepts.
    pub arguments: Vec<ArgumentDescription>,
    pub effect: EffectDescription,
    /// What the command needs besides workbench: `git`, `gh`, `gh-authentication`, `bd`,
    /// `network`, or `workbench-link` (the project's `.workbench` package link).
    pub requires: Vec<String>,
    /// Whether the command accepts `--dry-run`.
    pub dry_run: bool,
    /// Whether running it twice with the same input leaves the same state as running it once.
    pub idempotent: bool,
    /// Every error the command can report on stderr.
    pub errors: Vec<ErrorDescription>,
    pub examples: Vec<ExampleDescription>,
    /// JSON Schema of the result the command writes to stdout.
    pub output: Value,
}

#[derive(Serialize, JsonSchema)]
pub struct EffectDescription {
    /// `reads` or `modifies`.
    pub kind: String,
    /// What a modifying command changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct ErrorDescription {
    pub code: String,
    /// The exit status the error sets.
    pub exit: i32,
    /// Whether retrying the same command can succeed.
    pub retryable: bool,
    pub meaning: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ExampleDescription {
    pub summary: String,
    pub command: String,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArgumentDescription {
    pub name: String,
    /// `positional`, `option`, or `flag`.
    pub kind: String,
    /// How to write it: `<query>`, `--limit <integer>`, `--dry-run`.
    pub usage: String,
    /// `string`, `path`, `integer`, `text` (text, `@<file>`, or `-` for stdin), `choice`, or `boolean`.
    #[serde(rename = "type")]
    pub value_type: String,
    pub required: bool,
    pub default: Option<String>,
    /// Whether it accepts several values.
    pub multiple: bool,
    /// The only values a `choice` accepts.
    pub accepted_values: Option<Vec<String>>,
    pub summary: String,
}

pub fn example_command_line(example: &Example) -> String {
    let words: Vec<String> = example.args.iter().map(|arg| shell_word(arg)).collect();
    format!("workbench {}", words.join(" "))
}

fn shell_word(word: &str) -> String {
    let plain = !word.is_empty()
        && word
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_./@=:".contains(character));
    if plain {
        word.to_string()
    } else {
        format!("'{}'", word.replace('\'', r"'\''"))
    }
}

impl ArgumentSpec {
    pub fn usage(&self) -> String {
        match self.kind {
            ArgumentKind::Positional => format!("<{}>", self.name),
            ArgumentKind::Option => format!("--{} <{}>", self.name, self.value.type_name()),
            ArgumentKind::Flag => format!("--{}", self.name),
        }
    }

    pub fn describe(&self) -> ArgumentDescription {
        ArgumentDescription {
            name: self.name.to_string(),
            kind: match self.kind {
                ArgumentKind::Positional => "positional",
                ArgumentKind::Option => "option",
                ArgumentKind::Flag => "flag",
            }
            .to_string(),
            usage: self.usage(),
            value_type: self.value.type_name().to_string(),
            required: self.required,
            default: self.default.map(str::to_string),
            multiple: self.multiple,
            accepted_values: self
                .value
                .accepted_values()
                .map(|values| values.iter().map(|value| value.to_string()).collect()),
            summary: self.summary.to_string(),
        }
    }
}
