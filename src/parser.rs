//! The Clap parser, generated from the registry.

use clap::builder::PossibleValuesParser;
use clap::{Arg, ArgAction, Command, value_parser};

use crate::registry::{
    self, ArgumentKind, ArgumentSpec, CommandSpec, GLOBAL_ARGUMENTS, RESOURCES, ValueKind,
};

const ORIENTATION: &str = "\
Workbench is a development process for working with coding agents. This CLI is how agents and \
hooks run its operations in a project: workflow status, record validation, commit and change-file \
checks, project setup, and Beads sync and claims.

Results are JSON on stdout; diagnostics and structured errors go to stderr. Pass --output text \
for a human rendering.";

const DISCOVERY: &str = "\
Discover before you run:
  workbench cli search \"<task>\"            find the command for a task
  workbench schema <resource> <operation>  arguments, defaults, effects, errors, examples
  workbench docs search \"<topic>\"          search the process contract, rules, skills, templates
  workbench context show                   the project, pin, link, harnesses, and Beads setup

Modifying commands accept --dry-run. No command prompts.";

pub fn build() -> Command {
    let mut root = Command::new("workbench")
        .version(env!("CARGO_PKG_VERSION"))
        .about(ORIENTATION)
        .after_help(DISCOVERY)
        .subcommand_required(true)
        .disable_help_subcommand(true);
    for argument in GLOBAL_ARGUMENTS {
        root = root.arg(clap_arg(argument).global(true));
    }

    let commands = registry::all();
    for (resource, summary) in RESOURCES {
        let members: Vec<&CommandSpec> = commands
            .iter()
            .copied()
            .filter(|command| command.path[0] == *resource)
            .collect();
        root = match members.as_slice() {
            [] => root,
            [single] if single.path.len() == 1 => root.subcommand(leaf(single)),
            _ => {
                let mut group = Command::new(*resource)
                    .about(*summary)
                    .subcommand_required(true)
                    .disable_help_subcommand(true);
                for member in members {
                    group = group.subcommand(leaf(member));
                }
                root.subcommand(group)
            }
        };
    }
    root
}

fn leaf(spec: &CommandSpec) -> Command {
    let name = *spec
        .path
        .last()
        .expect("a command path has at least one word");
    let mut command = Command::new(name)
        .about(spec.summary)
        .long_about(format!("{}\n\n{}", spec.summary, spec.description))
        .after_long_help(examples_help(spec))
        .after_help(examples_help(spec));
    for argument in spec.arguments {
        command = command.arg(clap_arg(argument));
    }
    command
}

fn examples_help(spec: &CommandSpec) -> String {
    let mut help = format!("Effect: {}\n\nExamples:", spec.effect_label());
    for example in spec.examples {
        help.push_str(&format!(
            "\n  {}\n      {}",
            registry::example_command_line(example),
            example.summary
        ));
    }
    help
}

fn clap_arg(spec: &ArgumentSpec) -> Arg {
    let mut arg = Arg::new(spec.name).help(spec.summary);
    arg = match spec.kind {
        ArgumentKind::Positional => arg.value_name(spec.name.to_uppercase()),
        ArgumentKind::Option => arg
            .long(spec.name)
            .value_name(spec.value.type_name().to_uppercase()),
        ArgumentKind::Flag => arg.long(spec.name),
    };
    arg = match spec.value {
        ValueKind::Boolean => arg.action(ArgAction::SetTrue),
        ValueKind::Integer => arg.value_parser(value_parser!(u64)),
        ValueKind::Choice(values) => {
            arg.value_parser(PossibleValuesParser::new(values.iter().copied()))
        }
        ValueKind::String | ValueKind::Path | ValueKind::Text => arg,
    };
    if spec.kind != ArgumentKind::Flag {
        arg = arg.required(spec.required);
        if spec.multiple {
            arg = match spec.kind {
                ArgumentKind::Positional => arg.num_args(if spec.required { 1.. } else { 0.. }),
                _ => arg.action(ArgAction::Append),
            };
        }
        if let Some(default) = spec.default {
            arg = arg.default_value(default);
        }
    }
    arg
}
