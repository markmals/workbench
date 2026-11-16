//! One run of `workbench`: parse, dispatch, and report on the standard streams.

use std::ffi::OsString;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use clap::ArgMatches;
use clap::error::{ContextKind, ContextValue, ErrorKind};
use serde_json::{Value, json};

use crate::failure::{ErrorCode, Failure};
use crate::invocation::{Invocation, OutputFormat};
use crate::parser;
use crate::registry;
use crate::response::Response;

/// Runs `workbench` with `args` (including the program name) and returns the exit status.
pub fn run(args: Vec<OsString>, cwd: PathBuf) -> i32 {
    let output = requested_output(&args);
    let mut command = parser::build();
    if args.len() <= 1 {
        let _ = command.print_help();
        return 0;
    }

    let matches = match command.try_get_matches_from_mut(&args) {
        Ok(matches) => matches,
        Err(error) => match error.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                let _ = error.print();
                return 0;
            }
            _ => return report(output, None, Err(parse_failure(&error, &args))),
        },
    };

    let (path, leaf) = leaf_matches(&matches);
    let words: Vec<&str> = path.iter().map(String::as_str).collect();
    let Some(spec) = registry::find(&words) else {
        let failure = Failure::internal(format!("no registry entry for `{}`", words.join(" ")));
        return report(output, None, Err(failure));
    };
    let invocation = Invocation::new(spec, leaf, output, cwd);
    report(output, Some(spec.id), (spec.run)(&invocation))
}

fn leaf_matches(matches: &ArgMatches) -> (Vec<String>, &ArgMatches) {
    let mut path = Vec::new();
    let mut current = matches;
    while let Some((name, sub)) = current.subcommand() {
        path.push(name.to_string());
        current = sub;
    }
    (path, current)
}

/// `--output` as written, read before parsing so a parse error can honor it.
fn requested_output(args: &[OsString]) -> OutputFormat {
    let words: Vec<String> = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    for (index, word) in words.iter().enumerate() {
        let value = if word == "--output" {
            words.get(index + 1).map(String::as_str)
        } else {
            word.strip_prefix("--output=")
        };
        if value == Some("text") {
            return OutputFormat::Text;
        }
    }
    OutputFormat::Json
}

fn parse_failure(error: &clap::Error, args: &[OsString]) -> Failure {
    let message = error
        .to_string()
        .lines()
        .next()
        .unwrap_or("invalid arguments")
        .trim_start_matches("error: ")
        .to_string();
    let mut details = serde_json::Map::new();
    if let Some(ContextValue::String(argument)) = error.get(ContextKind::InvalidArg) {
        details.insert("argument".into(), json!(argument));
    }
    if let Some(ContextValue::Strings(arguments)) = error.get(ContextKind::InvalidArg) {
        details.insert("missing".into(), json!(arguments));
    }
    if let Some(ContextValue::String(value)) = error.get(ContextKind::InvalidValue) {
        details.insert("value".into(), json!(value));
    }
    if let Some(ContextValue::Strings(values)) = error.get(ContextKind::ValidValue) {
        details.insert("acceptedValues".into(), json!(values));
    }
    if let Some(ContextValue::Strings(values)) = error.get(ContextKind::ValidSubcommand) {
        details.insert("validSubcommands".into(), json!(values));
    }

    let words: Vec<String> = args
        .iter()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .take_while(|word| !word.starts_with('-'))
        .collect();
    let mut failure = Failure::invalid_argument(message);
    if let Some(spec) = longest_known_command(&words) {
        details.insert("command".into(), json!(spec.command_line()));
        details.insert(
            "arguments".into(),
            json!(
                spec.all_arguments()
                    .map(|argument| argument.describe())
                    .collect::<Vec<_>>()
            ),
        );
        failure = failure.suggest(format!("workbench schema {}", spec.path.join(" ")));
    } else {
        let prefix: Vec<&str> = words.iter().map(String::as_str).collect();
        let operations: Vec<String> = registry::all()
            .into_iter()
            .filter(|command| prefix.is_empty() || command.path.starts_with(&prefix[..1]))
            .map(|command| command.command_line())
            .collect();
        details.insert("commands".into(), json!(operations));
        failure = failure.suggest("workbench cli search \"<what you want to do>\"");
    }
    if error.kind() == ErrorKind::MissingRequiredArgument {
        failure = failure.suggest("Pass every required argument; `details.arguments` lists them");
    }
    failure.with_details(Value::Object(details))
}

fn longest_known_command(words: &[String]) -> Option<&'static registry::CommandSpec> {
    (1..=words.len()).rev().find_map(|length| {
        let path: Vec<&str> = words[..length].iter().map(String::as_str).collect();
        registry::find(&path)
    })
}

fn report(output: OutputFormat, command: Option<&str>, outcome: Result<Response, Failure>) -> i32 {
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    let (response, failure) = match outcome {
        Ok(mut response) => {
            let failure = response.failure.take();
            (Some(response), failure)
        }
        Err(failure) => (None, Some(failure)),
    };

    if let Some(response) = &response {
        let rendered = match output {
            OutputFormat::Json => json_text(&response.result, std::io::stdout().is_terminal()),
            OutputFormat::Text => response.text.clone(),
        };
        let _ = writeln!(stdout, "{}", rendered.trim_end_matches('\n'));
        for warning in &response.warnings {
            let _ = writeln!(stderr, "warning: {warning}");
        }
    }
    let _ = stdout.flush();

    let Some(mut failure) = failure else {
        return 0;
    };
    if failure.code == ErrorCode::Internal {
        failure = failure.suggest(format!(
            "Report this defect in workbench {}{} at https://github.com/markmals/workbench/issues",
            env!("CARGO_PKG_VERSION"),
            command.map(|id| format!(" ({id})")).unwrap_or_default()
        ));
    }
    // `--output text` renders the result for humans; the error stays the structured object, so
    // a caller can always parse stderr whatever format it asked the result in.
    let rendered = json_text(&failure.to_json(), std::io::stderr().is_terminal());
    let _ = writeln!(stderr, "{rendered}");
    failure.code.exit_status()
}

fn json_text(value: &Value, indented: bool) -> String {
    if indented {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
    .expect("JSON values serialize")
}
