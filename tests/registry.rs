//! The registry is the one declaration of every command; these tests fail when the parser,
//! the runtime, or the examples drift from it.

mod support;

use std::collections::BTreeSet;

use clap::Command;
use workbench::parser;
use workbench::registry::{self, ArgumentKind, CommandSpec, ValueKind};

use support::example_project;

fn leaves(command: &Command, prefix: Vec<String>, found: &mut Vec<(Vec<String>, Command)>) {
    if !command.has_subcommands() {
        found.push((prefix, command.clone()));
        return;
    }
    for sub in command.get_subcommands() {
        let mut path = prefix.clone();
        path.push(sub.get_name().to_string());
        leaves(sub, path, found);
    }
}

#[test]
fn every_entry_is_complete_and_uniquely_identified() {
    let mut ids = BTreeSet::new();
    for command in registry::all() {
        assert_eq!(
            command.id,
            command.path.join("."),
            "{} is not its path joined with dots",
            command.id
        );
        assert!(ids.insert(command.id), "{} is registered twice", command.id);
        assert!(
            !command.summary.is_empty() && !command.description.is_empty(),
            "{} lacks prose",
            command.id
        );
        assert!(
            !command.examples.is_empty(),
            "{} has no example",
            command.id
        );
        let modifies = matches!(command.effect, registry::Effect::Modifies(_));
        assert_eq!(
            modifies, command.dry_run,
            "{}: every modifying command, and only those, supports --dry-run",
            command.id
        );
        if command.dry_run {
            assert!(
                command
                    .arguments
                    .iter()
                    .any(|argument| argument.name == "dry-run"),
                "{} supports --dry-run but does not declare it",
                command.id
            );
        }
        for argument in command.arguments {
            let kebab = argument
                .name
                .chars()
                .all(|character| character.is_ascii_lowercase() || character == '-');
            assert!(
                kebab,
                "{}: argument `{}` is not kebab-case",
                command.id, argument.name
            );
        }
    }
}

#[test]
fn every_parser_argument_is_in_the_registry_and_every_entry_parses() {
    let mut found = Vec::new();
    leaves(&parser::build(), Vec::new(), &mut found);
    let parsed: BTreeSet<String> = found.iter().map(|(path, _)| path.join(".")).collect();
    let declared: BTreeSet<String> = registry::all()
        .iter()
        .map(|command| command.id.to_string())
        .collect();
    assert_eq!(
        parsed, declared,
        "the parser's commands and the registry's differ"
    );

    for (path, command) in found {
        let words: Vec<&str> = path.iter().map(String::as_str).collect();
        let spec = registry::find(&words).expect("checked above");
        for argument in command.get_arguments() {
            let name = argument.get_id().as_str();
            if name == "help" || name == "version" {
                continue;
            }
            let declared = spec.all_arguments().any(|declared| declared.name == name);
            assert!(
                declared,
                "`{}` parses --{name}, which its registry entry does not declare",
                spec.command_line()
            );
        }
    }
}

#[test]
fn every_example_parses() {
    for command in registry::all() {
        for example in command.examples {
            let mut args = vec!["workbench"];
            args.extend_from_slice(example.args);
            let result = parser::build().try_get_matches_from(&args);
            assert!(
                result.is_ok(),
                "example `{}` does not parse: {}",
                args.join(" "),
                result.unwrap_err()
            );
            assert!(
                example.args.starts_with(command.path),
                "example `{}` belongs to another command than {}",
                args.join(" "),
                command.id
            );
        }
    }
}

/// The smallest command line that parses: the path plus a placeholder for each required argument.
fn minimal_args(command: &CommandSpec) -> Vec<String> {
    let mut args: Vec<String> = std::iter::once("workbench")
        .chain(command.path.iter().copied())
        .map(str::to_string)
        .collect();
    for argument in command
        .arguments
        .iter()
        .filter(|argument| argument.required)
    {
        let value = match argument.value {
            ValueKind::Choice(values) => values[0].to_string(),
            ValueKind::Integer => "1".to_string(),
            _ => "placeholder".to_string(),
        };
        match argument.kind {
            ArgumentKind::Positional => args.push(value),
            ArgumentKind::Option => args.extend([format!("--{}", argument.name), value]),
            ArgumentKind::Flag => args.push(format!("--{}", argument.name)),
        }
    }
    args
}

#[test]
fn every_declared_default_is_the_runtime_default() {
    for command in registry::all() {
        let args = minimal_args(command);
        let matches = parser::build()
            .try_get_matches_from(&args)
            .unwrap_or_else(|error| panic!("`{}` does not parse: {error}", args.join(" ")));
        let mut leaf = &matches;
        while let Some((_, sub)) = leaf.subcommand() {
            leaf = sub;
        }
        for argument in command.all_arguments() {
            let runtime: Option<String> = leaf
                .get_raw(argument.name)
                .and_then(|mut values| values.next())
                .map(|value| value.to_string_lossy().into_owned());
            let declared = argument.default.map(str::to_string);
            if argument.required {
                continue;
            }
            if argument.kind == ArgumentKind::Flag {
                assert_eq!(declared, None, "{}: a flag declares a default", command.id);
                continue;
            }
            assert_eq!(
                runtime, declared,
                "{}: --{} defaults differently at run time",
                command.id, argument.name
            );
        }
    }
}

#[test]
fn every_example_runs_and_its_output_matches_its_schema() {
    for command in registry::all() {
        for example in command.examples {
            let project = example_project();
            let outcome = project.run(example.args);
            let shown = registry::example_command_line(example);
            assert_eq!(
                outcome.status, 0,
                "`{shown}` failed:\nstdout: {}\nstderr: {}",
                outcome.stdout, outcome.stderr
            );
            support::assert_matches_output_schema(command.path, &outcome.json());
        }
    }
}
