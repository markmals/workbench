//! `workbench commits scopes`: the scopes a commit subject may use, before one is written.

use schemars::{JsonSchema, schema_for};
use serde::Serialize;

use super::scopes::{self, SCOPES_FILE, Scope, Source};
use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::registry::{CommandSpec, Effect, Example, Requirement};
use crate::response::{RenderText, Response};

pub const COMMAND: CommandSpec = CommandSpec {
    id: "commits.scopes",
    path: &["commits", "scopes"],
    summary: "List the scopes a commit subject may use, and where each comes from",
    description: "Lists every scope `workbench commits check` accepts, in the order it reads them: \
        the built-in process scopes (`source: built-in`); the scopes listed one per line in the \
        optional `.agents/commit-scopes` (`source: scopes-file`); and `<NNNN>-<slug>` for every \
        `proposals/<NNNN>-<slug>.md` (`source: proposal`, with the proposal's `path`). A scope the \
        scopes file repeats is listed once, with its first source. Run it before writing a commit \
        subject; a scope a project needs that no list has goes into `.agents/commit-scopes` in \
        the same commit. Reads only. An unreadable `.agents/commit-scopes` or `proposals/` is \
        PRECONDITION_FAILED.",
    arguments: &[],
    effect: Effect::Reads,
    requires: &[Requirement::Git],
    dry_run: false,
    idempotent: true,
    errors: &[ErrorCode::PreconditionFailed, ErrorCode::DependencyMissing],
    examples: &[Example {
        summary: "List the scopes this project's commit subjects may use",
        args: &["commits", "scopes"],
    }],
    synonyms: &[
        "list",
        "allowed",
        "accepted",
        "valid",
        "available",
        "prefix",
        "subject",
        "commit",
        "which",
    ],
    output_schema: || schema_for!(CommitScopes),
    run: list_scopes,
};

#[derive(Serialize, JsonSchema)]
pub struct CommitScopes {
    /// Every accepted scope, built-in first, then the scopes file's, then the proposals'.
    pub items: Vec<Scope>,
}

impl RenderText for CommitScopes {
    fn render_text(&self) -> String {
        let groups = [
            (Source::BuiltIn, "Built in"),
            (Source::ScopesFile, SCOPES_FILE),
            (Source::Proposal, "Proposals"),
        ];
        groups
            .iter()
            .filter_map(|(source, label)| {
                let names: Vec<&str> = self
                    .items
                    .iter()
                    .filter(|item| item.source == *source)
                    .map(|item| item.scope.as_str())
                    .collect();
                (!names.is_empty()).then(|| format!("{label}: {}\n", names.join(", ")))
            })
            .collect()
    }
}

fn list_scopes(invocation: &Invocation) -> Result<Response, Failure> {
    let project = invocation.project()?;
    let items = scopes::accepted_with_sources(&project.root)?;
    Ok(Response::new(&CommitScopes { items }))
}
