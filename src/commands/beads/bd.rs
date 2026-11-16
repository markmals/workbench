//! Running `bd` and reading what it reports.

use std::path::Path;

use serde_json::Value;

use crate::process::{Program, Run, RunError};

/// A `bd` invocation in `root`. Every `bd` the CLI starts runs with `DOLT_REMOTE_INFO_BRANCH`
/// set and empty, so no sync publishes Dolt's `__dolt_remote_info__` branch to the remote,
/// whatever the caller's environment. Git, which Beads uses to reach the remote, never prompts
/// for credentials: rejected ones fail the sync rather than waiting out its time limit.
pub fn bd<'a>(root: &'a Path, args: &[&str]) -> Program<'a> {
    Program::new("bd", root)
        .args(args.iter().copied())
        .env("DOLT_REMOTE_INFO_BRANCH", "")
        .env("GIT_TERMINAL_PROMPT", "0")
}

/// `bd` prints machine-readable output with `--json`, but a failing or older `bd` may not.
pub fn parse_json(text: &str) -> Option<Value> {
    serde_json::from_str(text.trim()).ok()
}

/// The most specific reason `bd` gave for a failed command.
pub fn bd_error(run: &Run, command: &str) -> String {
    if let Some(output) = parse_json(&run.stdout) {
        let reason = [
            output.pointer("/failed/0/error"),
            output.get("message"),
            output.get("error"),
        ]
        .into_iter()
        .flatten()
        .find_map(Value::as_str)
        .filter(|reason| !reason.is_empty());
        if let Some(reason) = reason {
            return reason.to_string();
        }
    }
    let error_lines: Vec<&str> = run
        .stderr
        .lines()
        .filter(|line| line.starts_with("Error"))
        .collect();
    if !error_lines.is_empty() {
        return error_lines.join(" ");
    }
    let stderr = run.stderr.trim();
    if !stderr.is_empty() {
        return stderr.to_string();
    }
    match run.status {
        Some(status) => format!("bd {command} exited with status {status}"),
        None => format!("bd {command} was stopped by a signal"),
    }
}

/// Why `bd` did not run to completion, in words for a result.
pub fn interruption(error: &RunError, command: &str) -> String {
    match error {
        RunError::Missing => "bd is not installed".to_string(),
        RunError::TimedOut => format!("bd {command} did not finish in time"),
        RunError::Spawn(reason) => format!("bd could not be run: {reason}"),
    }
}

/// `text` as a sentence: with closing punctuation.
pub fn sentence(text: &str) -> String {
    let text = text.trim();
    if text.ends_with(['.', '!', '?']) {
        text.to_string()
    } else {
        format!("{text}.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(stdout: &str, stderr: &str) -> Run {
        Run {
            status: Some(1),
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }

    #[test]
    fn the_reason_is_the_most_specific_one_bd_gave() {
        let failed =
            r#"{"error":"1 of 1 failed","failed":[{"id":"wb-1","error":"already claimed"}]}"#;
        assert_eq!(bd_error(&run(failed, ""), "update"), "already claimed");
        let workspace =
            r#"{"error":"no_beads_directory","message":"No active beads workspace found."}"#;
        assert_eq!(
            bd_error(&run(workspace, ""), "where"),
            "No active beads workspace found."
        );
        assert_eq!(
            bd_error(&run("", "noise\nError updating wb-1: held\n"), "update"),
            "Error updating wb-1: held"
        );
        assert_eq!(
            bd_error(&run("", "cannot close wb-1\n"), "close"),
            "cannot close wb-1"
        );
        assert_eq!(
            bd_error(&run("", ""), "sync"),
            "bd sync exited with status 1"
        );
    }
}
