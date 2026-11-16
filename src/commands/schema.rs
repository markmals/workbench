//! `workbench schema`: a command's full specification, straight from the registry.

use schemars::{JsonSchema, schema_for};
use serde::Serialize;
use serde_json::json;

use crate::commands::cli::rank;
use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::registry::{
    self, ArgumentKind, ArgumentSpec, CommandDescription, CommandSpec, Effect, Example, ValueKind,
};
use crate::response::{RenderText, Response};

pub static COMMANDS: &[CommandSpec] = &[CommandSpec {
    id: "schema",
    path: &["schema"],
    summary: "Show a command's arguments, defaults, effects, errors, examples, and output schema",
    description: "With no command, lists every command with its summary and effect. With a command \
        path such as `beads claim` (or its identifier, `beads.claim`), returns that command's full \
        registry entry: each argument with its type, whether it is required, its default, and its \
        accepted values; whether it reads or modifies state; what it requires; whether it supports \
        --dry-run and is idempotent; the error codes it can return; examples; and the JSON Schema of \
        its result. Never changes anything.",
    arguments: &[ArgumentSpec {
        name: "command",
        kind: ArgumentKind::Positional,
        value: ValueKind::String,
        required: false,
        default: None,
        multiple: true,
        summary: "The command path, such as `beads claim`; omit to list every command",
    }],
    effect: Effect::Reads,
    requires: &[],
    dry_run: false,
    idempotent: true,
    errors: &[ErrorCode::NotFound],
    examples: &[
        Example {
            summary: "List every command",
            args: &["schema"],
        },
        Example {
            summary: "Inspect the command search",
            args: &["schema", "cli", "search"],
        },
    ],
    synonyms: &[
        "inspect",
        "usage",
        "arguments",
        "options",
        "flags",
        "reference",
        "spec",
        "describe",
    ],
    output_schema: || schema_for!(SchemaResult),
    run: show_schema,
}];

#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum SchemaResult {
    /// With no command: every command.
    List(CommandList),
    /// With a command: its full specification.
    Command(Box<CommandDescription>),
}

#[derive(Serialize, JsonSchema)]
pub struct CommandList {
    pub items: Vec<CommandSummary>,
}

#[derive(Serialize, JsonSchema)]
pub struct CommandSummary {
    pub command: String,
    pub id: String,
    pub summary: String,
    /// `reads`, or `modifies <what>`.
    pub effect: String,
}

impl RenderText for SchemaResult {
    fn render_text(&self) -> String {
        match self {
            SchemaResult::List(list) => {
                let width = list
                    .items
                    .iter()
                    .map(|item| item.command.len())
                    .max()
                    .unwrap_or(0);
                list.items
                    .iter()
                    .map(|item| format!("{:width$}  {}\n", item.command, item.summary))
                    .collect()
            }
            SchemaResult::Command(description) => render_description(description),
        }
    }
}

fn render_description(description: &CommandDescription) -> String {
    let mut text = format!(
        "{}\n  {}\n\n{}\n\nArguments:\n",
        description.command, description.summary, description.description
    );
    for argument in &description.arguments {
        let mut facts = vec![argument.value_type.clone()];
        if argument.required {
            facts.push("required".into());
        }
        if let Some(default) = &argument.default {
            facts.push(format!("default {default}"));
        }
        if let Some(values) = &argument.accepted_values {
            facts.push(format!("one of {}", values.join(", ")));
        }
        text.push_str(&format!(
            "  {}  ({})\n      {}\n",
            argument.usage,
            facts.join("; "),
            argument.summary
        ));
    }
    let effect = match &description.effect.what {
        Some(what) => format!("modifies {what}"),
        None => "reads".into(),
    };
    text.push_str(&format!(
        "\nEffect: {effect}\nDry run: {}\nIdempotent: {}\nRequires: {}\n\nErrors:\n",
        if description.dry_run { "yes" } else { "no" },
        if description.idempotent { "yes" } else { "no" },
        if description.requires.is_empty() {
            "nothing beyond workbench".to_string()
        } else {
            description.requires.join(", ")
        },
    ));
    for error in &description.errors {
        text.push_str(&format!(
            "  {} (exit {}): {}\n",
            error.code, error.exit, error.meaning
        ));
    }
    text.push_str("\nExamples:\n");
    for example in &description.examples {
        text.push_str(&format!(
            "  {}\n      {}\n",
            example.command, example.summary
        ));
    }
    text
}

fn show_schema(invocation: &Invocation) -> Result<Response, Failure> {
    let words = invocation.strings("command");
    if words.is_empty() {
        let items = registry::all()
            .into_iter()
            .map(|command| CommandSummary {
                command: command.command_line(),
                id: command.id.to_string(),
                summary: command.summary.to_string(),
                effect: command.effect_label(),
            })
            .collect();
        return Ok(Response::new(&SchemaResult::List(CommandList { items })));
    }

    let path = command_path(&words);
    let path: Vec<&str> = path.iter().map(String::as_str).collect();
    let command = registry::find(&path).ok_or_else(|| unknown_command(&words))?;
    Ok(Response::new(&SchemaResult::Command(Box::new(
        command.describe(),
    ))))
}

/// `beads claim`, `workbench beads claim`, and `beads.claim` all name the same command.
fn command_path(words: &[String]) -> Vec<String> {
    let words = match words.first().map(String::as_str) {
        Some("workbench") => &words[1..],
        _ => words,
    };
    words
        .iter()
        .flat_map(|word| word.split('.'))
        .filter(|word| !word.is_empty())
        .map(str::to_string)
        .collect()
}

fn unknown_command(words: &[String]) -> Failure {
    let asked = words.join(" ");
    let mut failure = Failure::not_found(format!("no workbench command `{asked}`"))
        .with_details(json!({ "command": asked }));
    for candidate in rank(&asked).into_iter().take(3) {
        failure = failure.suggest(format!(
            "workbench schema {}",
            candidate.command.trim_start_matches("workbench ")
        ));
    }
    failure.suggest("workbench schema")
}
