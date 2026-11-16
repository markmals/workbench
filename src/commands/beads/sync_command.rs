//! `workbench beads sync`: one sync with the project's Beads remote.

use schemars::{JsonSchema, schema_for};
use serde::Serialize;
use serde_json::{Value, json};

use super::bd::{bd, bd_error, interruption};
use super::items::{DifferingItem, differing_items};
use super::report::outcome_text;
use super::sync::{SyncOutcome, SyncProblem, SyncResult, sync_replica};
use super::{Replica, TIMEOUT, open_replica, time_limit};
use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::registry::{CommandSpec, DRY_RUN, Effect, Example, Requirement};
use crate::response::{RenderText, Response};

pub const COMMAND: CommandSpec = CommandSpec {
    id: "beads.sync",
    path: &["beads", "sync"],
    summary: "Sync this machine's Beads replica with the project's remote",
    description: "Pulls every change other machines published to the project's Beads remote \
        (`refs/dolt/data` on the checkout's `origin`) and publishes this replica's, with `bd sync`; \
        when the remote has no Beads history yet, the first sync publishes it with `bd dolt push`. \
        Refuses, changing nothing, when the replica's Beads remote is not the checkout's `origin` or \
        there is no `origin`. Limited to `--timeout` seconds (default 20). Exits 0 only when the \
        receipt's `status` is `synced`. Otherwise the error's `details.outcome` is `failed`, \
        `conflict`, or `blocked`, and its code says why: UNAVAILABLE (remote or network \
        unreachable, push retries exhausted), TIMEOUT, UNAUTHENTICATED (credentials rejected), \
        PRECONDITION_FAILED (a foreign remote or no `origin`, no Beads workspace, or a dirty \
        working set that blocks publishing), DEPENDENCY_MISSING (no `bd`), or CONFLICT (a halted \
        merge; `details.tables` and `details.differing` name what conflicts). `--dry-run` reports \
        the remote and the changes this replica has not published, without syncing.",
    arguments: &[DRY_RUN, TIMEOUT],
    effect: Effect::Modifies("this machine's Beads replica and the project's Beads remote"),
    requires: &[Requirement::Git, Requirement::Bd, Requirement::Network],
    dry_run: true,
    idempotent: true,
    errors: &[
        ErrorCode::InvalidArgument,
        ErrorCode::PreconditionFailed,
        ErrorCode::DependencyMissing,
        ErrorCode::Unauthenticated,
        ErrorCode::Conflict,
        ErrorCode::Timeout,
        ErrorCode::Unavailable,
    ],
    examples: &[
        Example {
            summary: "Publish what this machine changed and pull what the others did",
            args: &["beads", "sync"],
        },
        Example {
            summary: "See the remote and the unpublished changes without syncing",
            args: &["beads", "sync", "--dry-run"],
        },
    ],
    synonyms: &[
        "publish", "push", "pull", "replica", "remote", "dolt", "share", "upload", "fetch",
    ],
    output_schema: || schema_for!(SyncReceipt),
    run: sync,
};

/// A completed sync, or a dry run's description of one.
#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum SyncReceipt {
    Synced(Synced),
    DryRun(SyncPreview),
}

#[derive(Serialize, JsonSchema)]
pub struct Synced {
    /// `synced`.
    pub status: SyncOutcome,
    /// The project root.
    pub project: String,
    /// The Beads remote, without credentials; `null` when there is none.
    pub remote: Option<String>,
    pub detail: String,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SyncPreview {
    /// `dry-run`.
    pub status: String,
    pub project: String,
    pub remote: Option<String>,
    pub unpublished: UnpublishedChanges,
    /// What the dry run cannot know until the sync runs.
    pub cannot_predict: Vec<String>,
}

/// The changes this replica has not published, as far as it knows.
#[derive(Serialize, JsonSchema)]
pub struct UnpublishedChanges {
    /// Whether they could be determined. Beads compares this replica with the remote as last
    /// fetched (`bd diff origin/main main`), by item rather than by commit.
    pub determined: bool,
    /// The items that differ from the remote as last fetched.
    pub items: Vec<DifferingItem>,
    /// Why they could not be determined.
    pub reason: Option<String>,
}

fn sync(invocation: &Invocation) -> Result<Response, Failure> {
    let limit = time_limit(invocation)?;
    let replica = open_replica(invocation, limit)?;
    if let Some(refusal) = &replica.refusal {
        return Err(as_failed_sync(refusal.to_failure()));
    }
    if invocation.dry_run() {
        return Ok(Response::new(&SyncReceipt::DryRun(preview(&replica))));
    }
    let result = sync_replica(&replica.root, limit);
    if result.outcome != SyncOutcome::Synced {
        return Err(sync_failure(result, replica.remote));
    }
    Ok(Response::new(&SyncReceipt::Synced(Synced {
        status: result.outcome,
        project: replica.project,
        remote: replica.remote,
        detail: result.detail,
    })))
}

fn preview(replica: &Replica) -> SyncPreview {
    SyncPreview {
        status: "dry-run".into(),
        project: replica.project.clone(),
        remote: replica.remote.clone(),
        unpublished: unpublished_changes(replica),
        cannot_predict: vec![
            "Merge conflicts: what the remote holds is known only once the sync pulls it.".into(),
            "Whether the remote is reachable and accepts the push within the time limit.".into(),
            "Whether the remote has Beads history yet; a sync to one without it publishes this \
             replica with `bd dolt push`."
                .into(),
        ],
    }
}

fn unpublished_changes(replica: &Replica) -> UnpublishedChanges {
    let failure = match bd(&replica.root, &["diff", "origin/main", "main", "--json"]).run() {
        Ok(diff) if diff.success() => {
            return UnpublishedChanges {
                determined: true,
                items: differing_items(&diff),
                reason: None,
            };
        }
        Ok(diff) => bd_error(&diff, "diff"),
        Err(error) => interruption(&error, "diff"),
    };
    UnpublishedChanges {
        determined: false,
        items: Vec::new(),
        reason: Some(format!(
            "this replica cannot be compared with the remote as last fetched: {failure}"
        )),
    }
}

/// A sync refused before it ran is a `failed` sync too.
fn as_failed_sync(mut failure: Failure) -> Failure {
    if let Value::Object(details) = &mut failure.details {
        details.entry("outcome").or_insert(json!("failed"));
    }
    failure
}

/// The error for a sync that did not end `synced`, with the code its cause calls for: the same
/// code its `sync` object carries in status reports and receipts.
fn sync_failure(result: SyncResult, remote: Option<String>) -> Failure {
    let message = outcome_text(result.outcome.as_str(), &result.detail).replace("\n  ", "\n");
    let mut details = json!({ "outcome": result.outcome, "remote": remote });
    let code = result.code.unwrap_or(ErrorCode::Unavailable);
    let suggestion = match result.problem {
        Some(SyncProblem::Refused(refusal)) => return as_failed_sync(refusal.to_failure()),
        Some(SyncProblem::BdMissing) => return as_failed_sync(Failure::dependency_missing("bd")),
        Some(SyncProblem::TimedOut) => "Retry, or allow more time with --timeout",
        Some(SyncProblem::CredentialsRejected) => {
            "Beads uses Git's credentials for origin; check that `git fetch origin` works"
        }
        Some(SyncProblem::Conflict(report)) => {
            details["tables"] = json!(report.tables);
            details["differing"] = json!(report.differing);
            "Back up with `bd export --all -o <path>`, settle the conflict as the tracking-work \
             skill describes, then run `workbench beads sync`"
        }
        Some(SyncProblem::Blocked) => {
            "Back up, clear the dirty working set as the tracking-work skill describes, then run \
             `workbench beads sync`"
        }
        Some(SyncProblem::Unreachable) | None => {
            "Continue, and say that Beads may be stale; run `workbench beads sync` again later"
        }
    };
    Failure::new(code, message)
        .with_details(details)
        .suggest(suggestion)
}

impl RenderText for SyncReceipt {
    fn render_text(&self) -> String {
        match self {
            SyncReceipt::Synced(synced) => outcome_text("synced", &synced.detail),
            SyncReceipt::DryRun(preview) => preview.render_text(),
        }
    }
}

impl RenderText for SyncPreview {
    fn render_text(&self) -> String {
        let remote = self.remote.as_deref().unwrap_or("no remote");
        let mut lines = vec![format!("Dry run: would sync with {remote}.")];
        let unpublished = &self.unpublished;
        match (&unpublished.reason, unpublished.items.is_empty()) {
            (Some(reason), _) => lines.push(format!("Unpublished changes: unknown; {reason}.")),
            (None, true) => lines.push("Unpublished changes: none.".into()),
            (None, false) => {
                lines.push("Unpublished changes:".into());
                lines.extend(unpublished.items.iter().map(|item| {
                    let change = item.change.as_deref().unwrap_or("changed");
                    format!("  {change}  {}", item.describe())
                }));
            }
        }
        lines.push("Cannot predict:".into());
        lines.extend(self.cannot_predict.iter().map(|item| format!("  {item}")));
        lines.join("\n")
    }
}
