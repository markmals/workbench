//! `workbench commits`: commit subjects and their Scoped Commits scopes.

pub mod range;
mod report;
mod scopes;
mod scopes_command;
mod subject;

use std::io::ErrorKind;

use schemars::schema_for;
use serde_json::json;

use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::registry::{
    ArgumentKind, ArgumentSpec, CommandSpec, Effect, Example, Requirement, ValueKind,
};
use crate::response::Response;
use range::Range;
use report::{CommitsCheck, RangeArguments};

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        id: "commits.check",
        path: &["commits", "check"],
        summary: "Check commit subjects against Scoped Commits: one message file, or every commit in a range",
        description: "Checks that each commit subject is a Scoped Commit, `<scope>: <imperative \
        description>`, naming an accepted scope. Give exactly one form. `--message-file <path>` \
        checks the message a commit-msg hook is about to commit: its subject is the first line \
        that is neither blank nor a `#` comment, and a message with no such line passes. \
        `<base> <head>` checks the subject of every commit reachable from head and not from base, \
        and reports every bad subject, not only the first. Accepted scopes are the built-in \
        process scopes (agents, ci, decision, deps, docs, guides, policy, proposal, release, \
        rules, skills, templates, treewide, vision); the scopes listed one per line in the \
        optional `.agents/commit-scopes` (blank lines and `#` comments ignored); and \
        `<NNNN>-<slug>` for every `proposals/<NNNN>-<slug>.md`. Merge, revert, `fixup!`, and \
        `squash!` subjects are exempt; Conventional Commits `type(scope):` subjects are \
        rejected by name. The result lists each subject with its verdict, the violations, and \
        every accepted scope. Any violation fails with CHECK_FAILED after the result is \
        written. A missing message file or an unknown revision is NOT_FOUND; both forms, \
        neither, or a base without a head is INVALID_ARGUMENT.",
        arguments: &[
            ArgumentSpec {
                name: "base",
                kind: ArgumentKind::Positional,
                value: ValueKind::String,
                required: false,
                default: None,
                multiple: false,
                summary: "The range's base: a commit, branch, or tag the checked commits are not reachable from",
            },
            ArgumentSpec {
                name: "head",
                kind: ArgumentKind::Positional,
                value: ValueKind::String,
                required: false,
                default: None,
                multiple: false,
                summary: "The range's head: the last commit to check",
            },
            ArgumentSpec {
                name: "message-file",
                kind: ArgumentKind::Option,
                value: ValueKind::Path,
                required: false,
                default: None,
                multiple: false,
                summary: "A commit message to check instead of a range, such as the commit-msg hook's file",
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
        examples: &[
            Example {
                summary: "Check the message a commit-msg hook is about to commit",
                args: &["commits", "check", "--message-file", ".git/COMMIT_EDITMSG"],
            },
            Example {
                summary: "Check every commit a branch adds to main",
                args: &["commits", "check", "main", "feature"],
            },
        ],
        synonyms: &[
            "commit",
            "message",
            "subject",
            "scope",
            "lint",
            "validate",
            "verify",
            "hook",
            "conventional",
            "pull request",
            "branch",
        ],
        output_schema: || schema_for!(CommitsCheck),
        run: check,
    },
    scopes_command::COMMAND,
];

/// What `commits check` was asked to check.
enum Form {
    MessageFile(String),
    Range { base: String, head: String },
}

fn check(invocation: &Invocation) -> Result<Response, Failure> {
    let form = form(invocation)?;
    let project = invocation.project()?;
    let accepted_scopes = scopes::accepted(&project.root)?;
    let project_root = project.root.display().to_string();
    let check = match form {
        Form::MessageFile(path) => {
            let message = read_message(invocation, &path)?;
            let subject = subject::subject_of(&message).map(str::to_string);
            let mut check =
                CommitsCheck::judge(project_root, vec![(None, subject)], accepted_scopes);
            check.message_file = Some(path);
            check
        }
        Form::Range { base, head } => {
            // A commit made with an empty message has no subject to check.
            let subjects = Range::resolve(&project.root, &base, &head)?
                .subjects()?
                .into_iter()
                .map(|(commit, subject)| (Some(commit), Some(subject).filter(|s| !s.is_empty())))
                .collect();
            let mut check = CommitsCheck::judge(project_root, subjects, accepted_scopes);
            check.range = Some(RangeArguments { base, head });
            check
        }
    };
    let failure = check.failure();
    let response = Response::new(&check);
    Ok(match failure {
        Some(failure) => response.failing(failure),
        None => response,
    })
}

fn form(invocation: &Invocation) -> Result<Form, Failure> {
    let message_file = invocation.string("message-file");
    let base = invocation.string("base");
    let head = invocation.string("head");
    match (message_file, base, head) {
        (Some(path), None, None) => Ok(Form::MessageFile(path)),
        (None, Some(base), Some(head)) => Ok(Form::Range { base, head }),
        (None, Some(_), None) => Err(wrong_form(
            "a range needs both <base> and <head>",
            &["head"],
        )),
        (None, None, _) => Err(wrong_form(
            "nothing to check: pass --message-file <path>, or <base> <head>",
            &["message-file", "base", "head"],
        )),
        (Some(_), _, _) => Err(wrong_form(
            "pass either --message-file <path> or <base> <head>, not both",
            &[],
        )),
    }
}

fn wrong_form(message: &str, missing: &[&str]) -> Failure {
    Failure::invalid_argument(message)
        .with_details(json!({
            "missing": missing,
            "forms": ["--message-file <path>", "<base> <head>"],
        }))
        .suggest("workbench commits check --message-file .git/COMMIT_EDITMSG")
        .suggest("workbench commits check origin/main HEAD")
}

fn read_message(invocation: &Invocation, path: &str) -> Result<String, Failure> {
    let full = invocation.cwd.join(path);
    let bytes = std::fs::read(&full).map_err(|error| {
        let details = json!({ "path": full.display().to_string() });
        if error.kind() == ErrorKind::NotFound {
            Failure::not_found(format!("no commit-message file at {}", full.display()))
                .with_details(details)
        } else {
            Failure::invalid_argument(format!(
                "cannot read the commit-message file {}: {error}",
                full.display()
            ))
            .with_details(details)
        }
    })?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
