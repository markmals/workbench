//! `workbench cli`: discovering workbench's own commands.

use clap_complete::Shell;
use schemars::{JsonSchema, schema_for};
use serde::Serialize;

use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::parser;
use crate::registry::{self, ArgumentKind, ArgumentSpec, CommandSpec, Effect, Example, ValueKind};
use crate::response::{RenderText, Response};
use crate::search::{self, Page};

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        id: "cli.search",
        path: &["cli", "search"],
        summary: "Find the workbench command for a task described in plain words",
        description: "Ranks every workbench command against the query by its name, summary, \
            description, options, and declared synonyms, and returns the best matches with the exact \
            command to run. Runs locally, needs no credentials, and never changes anything. Inspect a \
            match with `workbench schema <command>` before running it.",
        arguments: &[
            ArgumentSpec {
                name: "query",
                kind: ArgumentKind::Positional,
                value: ValueKind::String,
                required: true,
                default: None,
                multiple: true,
                summary: "What you want to do, in plain words; several words need no quotes",
            },
            LIMIT,
            CURSOR,
        ],
        effect: Effect::Reads,
        requires: &[],
        dry_run: false,
        idempotent: true,
        errors: &[ErrorCode::InvalidArgument],
        examples: &[
            Example {
                summary: "Find how to claim a Beads item",
                args: &["cli", "search", "claim a work item"],
            },
            Example {
                summary: "Find how to add a release note",
                args: &["cli", "search", "add a change file", "--limit", "3"],
            },
        ],
        synonyms: &["find", "discover", "lookup", "help", "commands", "which"],
        output_schema: || schema_for!(Page<CommandMatch>),
        run: search_commands,
    },
    CommandSpec {
        id: "cli.completions",
        path: &["cli", "completions"],
        summary: "Print a shell completion script for workbench",
        description: "Generates the completion script for the named shell from the command registry. \
            The JSON result holds the script in `script`; `--output text` prints the script alone, \
            ready to redirect into the shell's completion directory.",
        arguments: &[ArgumentSpec {
            name: "shell",
            kind: ArgumentKind::Positional,
            value: ValueKind::Choice(&["bash", "zsh", "fish", "elvish", "powershell"]),
            required: true,
            default: None,
            multiple: false,
            summary: "The shell to generate completions for",
        }],
        effect: Effect::Reads,
        requires: &[],
        dry_run: false,
        idempotent: true,
        errors: &[ErrorCode::InvalidArgument],
        examples: &[Example {
            summary: "Completions for zsh",
            args: &["cli", "completions", "zsh"],
        }],
        synonyms: &["autocomplete", "completion", "tab", "shell"],
        output_schema: || schema_for!(Completions),
        run: completions,
    },
];

pub const LIMIT: ArgumentSpec = ArgumentSpec {
    name: "limit",
    kind: ArgumentKind::Option,
    value: ValueKind::Integer,
    required: false,
    default: Some("5"),
    multiple: false,
    summary: "The most matches to return",
};

pub const CURSOR: ArgumentSpec = ArgumentSpec {
    name: "cursor",
    kind: ArgumentKind::Option,
    value: ValueKind::String,
    required: false,
    default: None,
    multiple: false,
    summary: "Resume after a previous page: the nextCursor it returned",
};

/// One command that fits the query.
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandMatch {
    /// The exact command to run, without its arguments.
    pub command: String,
    /// The registry identifier, such as `beads.claim`.
    pub id: String,
    pub summary: String,
    /// `reads`, or `modifies <what>`.
    pub effect: String,
    /// Relevance from 0 to 1.
    pub score: f64,
}

impl RenderText for Page<CommandMatch> {
    fn render_text(&self) -> String {
        if self.items.is_empty() {
            return "No command matches. Try other words, or `workbench schema` for the full list."
                .into();
        }
        let width = self
            .items
            .iter()
            .map(|item| item.command.len())
            .max()
            .unwrap_or(0);
        let mut text = String::new();
        for item in &self.items {
            text.push_str(&format!("{:width$}  {}\n", item.command, item.summary));
        }
        if self.has_more {
            text.push_str("More matches follow; pass --cursor with nextCursor or raise --limit.\n");
        }
        text
    }
}

fn search_commands(invocation: &Invocation) -> Result<Response, Failure> {
    let query = invocation.strings("query").join(" ");
    let ranked = rank(&query);
    let limit = invocation.integer("limit").expect("limit has a default");
    let page = search::page(ranked, invocation.string("cursor").as_deref(), limit)?;
    Ok(Response::new(&page))
}

/// Every command with a positive score for `query`, best first, ties in registry order.
pub fn rank(query: &str) -> Vec<CommandMatch> {
    let terms = search::query_words(query);
    let mut scored: Vec<(f64, &CommandSpec)> = registry::all()
        .into_iter()
        .map(|command| (score(command, &terms), command))
        .filter(|(score, _)| *score > 0.0)
        .collect();
    scored.sort_by(|left, right| right.0.total_cmp(&left.0));
    scored
        .into_iter()
        .map(|(score, command)| CommandMatch {
            command: command.command_line(),
            id: command.id.to_string(),
            summary: command.summary.to_string(),
            effect: command.effect_label(),
            score: (score * 1000.0).round() / 1000.0,
        })
        .collect()
}

/// The share of query words the command matches, each weighted by where it matched.
fn score(command: &CommandSpec, terms: &[String]) -> f64 {
    if terms.is_empty() {
        return 0.0;
    }
    let path: Vec<String> = command
        .path
        .iter()
        .flat_map(|word| search::words(word))
        .collect();
    let synonyms: Vec<String> = command
        .synonyms
        .iter()
        .flat_map(|word| search::words(word))
        .collect();
    let summary = search::words(command.summary);
    let arguments: Vec<String> = command
        .arguments
        .iter()
        .flat_map(|argument| search::words(&format!("{} {}", argument.name, argument.summary)))
        .collect();
    let description = search::words(command.description);

    let fields: [(&[String], f64); 5] = [
        (&path, 1.0),
        (&synonyms, 0.8),
        (&summary, 0.7),
        (&arguments, 0.4),
        (&description, 0.3),
    ];
    let total: f64 = terms
        .iter()
        .map(|term| {
            fields
                .iter()
                .map(|(words, weight)| search::best_match(term, words) * weight)
                .fold(0.0, f64::max)
        })
        .sum();
    total / terms.len() as f64
}

#[derive(Serialize, JsonSchema)]
pub struct Completions {
    pub shell: String,
    /// The completion script.
    pub script: String,
}

impl RenderText for Completions {
    fn render_text(&self) -> String {
        self.script.clone()
    }
}

fn completions(invocation: &Invocation) -> Result<Response, Failure> {
    let name = invocation.string("shell").expect("shell is required");
    let shell: Shell = name
        .parse()
        .map_err(|_| Failure::internal(format!("clap accepted an unknown shell `{name}`")))?;
    let mut command = parser::build();
    let mut script = Vec::new();
    clap_complete::generate(shell, &mut command, "workbench", &mut script);
    Ok(Response::new(&Completions {
        shell: name,
        script: String::from_utf8_lossy(&script).into_owned(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn top(query: &str) -> String {
        rank(query)
            .first()
            .map(|item| item.id.clone())
            .unwrap_or_default()
    }

    #[test]
    fn ranks_the_named_command_first() {
        assert_eq!(top("shell completions"), "cli.completions");
        assert_eq!(top("find a command"), "cli.search");
    }

    #[test]
    fn returns_nothing_for_unrelated_words() {
        assert!(rank("zebra xylophone").is_empty());
    }
}
