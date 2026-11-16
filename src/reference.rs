//! The CLI reference page, generated from the registry and shipped as `reference/cli.md`.

use std::fmt::Write;

use crate::failure::ErrorCode;
use crate::registry::{self, ArgumentDescription, CommandDescription, GLOBAL_ARGUMENTS, RESOURCES};

const INTRODUCTION: &str = "\
`workbench` is the Workbench package's CLI. Agents run it directly; humans usually reach it \
through the project's `mise run status`, `mise run validate`, and `mise run beads:*` tasks. This \
page is generated from the command registry by `mise run cli:reference`; do not edit it by hand.

Every command follows one contract:

- **Results** are JSON on stdout: compact when stdout is not a terminal, indented when it is. \
`--output text` renders the same result for humans. Progress, warnings, and diagnostics go to stderr.
- **Collections** return `items`, plus `hasMore` and `nextCursor` whenever the command bounds the \
result; pass `nextCursor` back as `--cursor` for the next page.
- **Modifying commands** accept `--dry-run`, which validates the input, resolves the target, and \
describes the changes without making them. Their receipts carry a `status`, the target, and the \
identifiers they produced.
- **Failures** write `{\"error\": {\"code\", \"message\", \"retryable\", \"details\", \"suggestions\"}}` \
to stderr and exit with the code's status. A check that ran and found violations still writes its \
result to stdout.
- **No command prompts.** Missing input fails at once with `INVALID_ARGUMENT`.

Discover a command with `workbench cli search \"<task>\"`, inspect it with `workbench schema \
<resource> <operation>`, preview it with `--dry-run`, then run it and verify the receipt.";

/// The whole page.
pub fn render() -> String {
    let mut page = String::from("---\ntitle: CLI Reference\n---\n\n# CLI Reference\n\n");
    page.push_str(INTRODUCTION);
    page.push_str("\n\n## Options every command accepts\n\n");
    page.push_str(&argument_table(
        &GLOBAL_ARGUMENTS
            .iter()
            .map(|argument| argument.describe())
            .collect::<Vec<_>>(),
    ));
    page.push_str(
        "\n## Error codes\n\n| Code | Exit | Retryable | Meaning |\n| --- | --- | --- | --- |\n",
    );
    for code in ErrorCode::ALL {
        let _ = writeln!(
            page,
            "| `{}` | {} | {} | {} |",
            code.as_str(),
            code.exit_status(),
            if code.retryable() { "Yes" } else { "No" },
            code.meaning()
        );
    }
    page.push_str("\n## Commands\n");
    for (resource, summary) in RESOURCES {
        let commands: Vec<CommandDescription> = registry::all()
            .into_iter()
            .filter(|command| command.path[0] == *resource)
            .map(|command| command.describe())
            .collect();
        if commands.is_empty() {
            continue;
        }
        let _ = write!(page, "\n### `{resource}`\n\n{summary}.\n");
        for command in &commands {
            page.push_str(&command_section(command));
        }
    }
    page
}

fn command_section(command: &CommandDescription) -> String {
    let mut section = format!(
        "\n#### `{}`\n\n{}.\n\n{}\n\n",
        command.command, command.summary, command.description
    );
    let effect = match &command.effect.what {
        Some(what) => format!("modifies {what}"),
        None => "reads only".to_string(),
    };
    let requires = if command.requires.is_empty() {
        "nothing beyond `workbench`".to_string()
    } else {
        command
            .requires
            .iter()
            .map(|requirement| format!("`{requirement}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let _ = writeln!(
        section,
        "- **Effect:** {effect}\n- **Requires:** {requires}\n- **Dry run:** {}\n- **Idempotent:** {}\n- **Errors:** {}\n",
        if command.dry_run { "yes" } else { "no" },
        if command.idempotent { "yes" } else { "no" },
        command
            .errors
            .iter()
            .map(|error| format!("`{}`", error.code))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let own: Vec<&ArgumentDescription> = command
        .arguments
        .iter()
        .filter(|argument| {
            !GLOBAL_ARGUMENTS
                .iter()
                .any(|global| global.name == argument.name)
        })
        .collect();
    if !own.is_empty() {
        section.push_str(&argument_table(
            &own.into_iter().cloned().collect::<Vec<_>>(),
        ));
        section.push('\n');
    }
    section.push_str("Examples:\n\n```sh\n");
    for example in &command.examples {
        let _ = writeln!(section, "# {}\n{}", example.summary, example.command);
    }
    section.push_str("```\n\nResult schema:\n\n```json\n");
    section.push_str(&serde_json::to_string_pretty(&command.output).expect("schemas serialize"));
    section.push_str("\n```\n");
    section
}

fn argument_table(arguments: &[ArgumentDescription]) -> String {
    let mut table = String::from(
        "| Argument | Type | Required | Default | Description |\n| --- | --- | --- | --- | --- |\n",
    );
    for argument in arguments {
        let value_type = match &argument.accepted_values {
            Some(values) => values
                .iter()
                .map(|value| format!("`{value}`"))
                .collect::<Vec<_>>()
                .join(", "),
            None if argument.multiple => format!("{}, repeatable", argument.value_type),
            None => argument.value_type.clone(),
        };
        let _ = writeln!(
            table,
            "| `{}` | {} | {} | {} | {} |",
            argument.usage,
            value_type,
            if argument.required { "yes" } else { "no" },
            argument
                .default
                .as_deref()
                .map(|default| format!("`{default}`"))
                .unwrap_or_else(|| "—".into()),
            argument.summary.replace('|', "\\|")
        );
    }
    table
}
