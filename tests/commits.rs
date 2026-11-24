//! `workbench commits check`: Scoped Commits subjects, for one message file or a commit range.

mod support;

use serde_json::Value;
use support::{Fixture, Outcome, assert_matches_output_schema};

const BUILT_IN_SCOPES: [&str; 13] = [
    "agents",
    "ci",
    "decision",
    "deps",
    "docs",
    "guides",
    "policy",
    "release",
    "rules",
    "skills",
    "templates",
    "treewide",
    "vision",
];

fn check_message(fixture: &Fixture, message: &str) -> Outcome {
    fixture.write("COMMIT_EDITMSG", message);
    fixture.run(&["commits", "check", "--message-file", "COMMIT_EDITMSG"])
}

/// The one subject a message-file check reports.
fn checked(outcome: &Outcome) -> Value {
    let result = outcome.json();
    assert_matches_output_schema(&["commits", "check"], &result);
    let commits = result["commits"].as_array().expect("a commits list");
    assert_eq!(commits.len(), 1, "{result}");
    commits[0].clone()
}

fn accepted_scopes(outcome: &Outcome) -> Vec<String> {
    outcome.json()["acceptedScopes"]
        .as_array()
        .expect("acceptedScopes")
        .iter()
        .map(|scope| scope.as_str().unwrap().to_string())
        .collect()
}

fn assert_passes(fixture: &Fixture, message: &str) -> Value {
    let outcome = check_message(fixture, message);
    assert_eq!(
        outcome.status, 0,
        "{message:?} was rejected:\n{}\n{}",
        outcome.stdout, outcome.stderr
    );
    checked(&outcome)
}

/// Checks that `message` fails with one violation of `rule` and returns that violation.
fn assert_rejected(fixture: &Fixture, message: &str, rule: &str) -> Value {
    let outcome = check_message(fixture, message);
    assert_eq!(outcome.status, 1, "{message:?} was not rejected");
    assert_eq!(outcome.error_code(), "CHECK_FAILED");
    assert_eq!(checked(&outcome)["verdict"], "rejected");
    let violations = outcome.json()["violations"].clone();
    assert_eq!(violations.as_array().unwrap().len(), 1, "{violations}");
    assert_eq!(violations[0]["rule"], rule, "{violations}");
    violations[0].clone()
}

#[path = "commits/message_file.rs"]
mod message_file;
#[path = "commits/range.rs"]
mod range;
#[path = "commits/scopes.rs"]
mod scopes;
