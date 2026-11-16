//! One sync with the project's Beads remote: the sync every sync point runs. Its outcomes are
//! a contract: guides/record-validation.md documents the ones `workflow status` reports, and
//! the tracking-work skill tells agents what each one means.

use std::path::Path;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use regex::Regex;
use schemars::JsonSchema;
use serde::Serialize;

use super::bd::{bd, bd_error, interruption, parse_json, sentence};
use super::items::{DifferingItem, differing_items};
use super::remote::{Refusal, read_remotes, remote_lacks_beads_history};
use crate::failure::ErrorCode;
use crate::process::{Run, RunError};

static REJECTED_CREDENTIALS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)authenticat|unauthori[sz]ed|permission denied|access denied|forbidden|credential|could not read (username|password)",
    )
    .expect("valid regex")
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SyncOutcome {
    /// This replica pulled every change on the remote and published its own.
    Synced,
    /// `--no-sync` was passed; nothing was pulled or pushed.
    Skipped,
    /// The sync did not finish, or was refused before it started.
    Failed,
    /// Beads halted on a conflict it cannot settle; nothing was pushed.
    Conflict,
    /// A dirty working set stops Beads from publishing; nothing was pushed.
    Blocked,
}

impl SyncOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            SyncOutcome::Synced => "synced",
            SyncOutcome::Skipped => "skipped",
            SyncOutcome::Failed => "failed",
            SyncOutcome::Conflict => "conflict",
            SyncOutcome::Blocked => "blocked",
        }
    }
}

/// How one sync ended.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SyncResult {
    pub outcome: SyncOutcome,
    /// What happened, in words; for anything but `synced`, why.
    pub detail: String,
    /// For a sync that did not end `synced` or `skipped`: the error code `workbench beads sync`
    /// fails with for this cause, such as `UNAVAILABLE` for an unreachable remote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<ErrorCode>,
    /// Whether running the sync again can succeed without anyone changing anything.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    /// Why a sync that did not complete did not, for describing it as an error.
    #[serde(skip)]
    pub problem: Option<SyncProblem>,
}

#[derive(Debug)]
pub enum SyncProblem {
    Refused(Refusal),
    TimedOut,
    CredentialsRejected,
    /// The remote or the network was unreachable, push retries ran out, or Beads failed for
    /// another reason it did not make recognizable.
    Unreachable,
    BdMissing,
    Conflict(ConflictReport),
    Blocked,
}

impl SyncProblem {
    /// The error code this cause is reported with, wherever the sync ran.
    pub fn code(&self) -> ErrorCode {
        match self {
            SyncProblem::Refused(_) | SyncProblem::Blocked => ErrorCode::PreconditionFailed,
            SyncProblem::TimedOut => ErrorCode::Timeout,
            SyncProblem::CredentialsRejected => ErrorCode::Unauthenticated,
            SyncProblem::Unreachable => ErrorCode::Unavailable,
            SyncProblem::BdMissing => ErrorCode::DependencyMissing,
            SyncProblem::Conflict(_) => ErrorCode::Conflict,
        }
    }
}

/// What Beads says about a conflict it halted on.
#[derive(Debug)]
pub struct ConflictReport {
    pub tables: Vec<String>,
    /// The items that differ from the remote, or `None` when they could not be listed in time.
    pub differing: Option<Vec<DifferingItem>>,
}

impl SyncResult {
    pub fn skipped() -> SyncResult {
        let detail = "--no-sync was passed; nothing was pulled or pushed.".to_string();
        SyncResult::ended(SyncOutcome::Skipped, detail, None)
    }

    fn synced(detail: &str) -> SyncResult {
        SyncResult::ended(SyncOutcome::Synced, detail.into(), None)
    }

    fn failed(problem: SyncProblem, detail: String) -> SyncResult {
        SyncResult::ended(SyncOutcome::Failed, detail, Some(problem))
    }

    /// The outcome, with the cause's code and retryability when it did not sync: `failed
    /// (UNAVAILABLE, retryable)`.
    pub fn label(&self) -> String {
        match (self.code, self.retryable) {
            (Some(code), Some(true)) => {
                format!("{} ({}, retryable)", self.outcome.as_str(), code.as_str())
            }
            (Some(code), _) => format!("{} ({})", self.outcome.as_str(), code.as_str()),
            (None, _) => self.outcome.as_str().to_string(),
        }
    }

    /// The one place a result is made, so its `code` and `retryable` always match its cause.
    fn ended(outcome: SyncOutcome, detail: String, problem: Option<SyncProblem>) -> SyncResult {
        let code = problem.as_ref().map(SyncProblem::code);
        SyncResult {
            outcome,
            detail,
            code,
            retryable: code.map(ErrorCode::retryable),
            problem,
        }
    }

    /// `bd` reported a failure: rejected credentials, or a remote it could not reach. Dolt
    /// appends `hint:` lines about credentials to every failed fetch, so only the other lines
    /// can say the credentials were rejected.
    fn failed_with(reason: &str) -> SyncResult {
        let rejected = reason
            .lines()
            .filter(|line| !line.trim_start().starts_with("hint:"))
            .any(|line| REJECTED_CREDENTIALS.is_match(line));
        let problem = if rejected {
            SyncProblem::CredentialsRejected
        } else {
            SyncProblem::Unreachable
        };
        SyncResult::failed(problem, reason.to_string())
    }
}

/// Runs one sync with the project's Beads remote: `bd sync`, or `bd dolt push` when the remote
/// has no Beads history yet, so the project's first sync publishes it. A replica whose remote
/// is not the checkout's own `origin` is never synced. The whole sync, including describing a
/// conflict, stays within `limit`. Every way the sync can end is one of the outcomes.
pub fn sync_replica(root: &Path, limit: Duration) -> SyncResult {
    let deadline = Instant::now() + limit;
    if let Some(refusal) = read_remotes(root, deadline).refusal(root) {
        let detail = refusal.message();
        return SyncResult::failed(SyncProblem::Refused(refusal), detail);
    }
    let sync = match bd(root, &["sync", "--json"]).deadline(deadline).run() {
        Ok(run) => run,
        Err(error) => return interrupted(&error, "sync", limit),
    };
    match sync.status {
        Some(0) => SyncResult::synced(synced_detail(&sync)),
        Some(2) => halted(root, &sync, deadline),
        Some(4) => SyncResult::ended(
            SyncOutcome::Blocked,
            format!("{} Nothing was pushed.", sentence(&bd_error(&sync, "sync"))),
            Some(SyncProblem::Blocked),
        ),
        Some(1) if remote_lacks_beads_history(root, deadline) => {
            first_publish(root, deadline, limit)
        }
        _ => SyncResult::failed_with(&bd_error(&sync, "sync")),
    }
}

fn synced_detail(sync: &Run) -> &'static str {
    let status =
        parse_json(&sync.stdout).and_then(|output| output["status"].as_str().map(str::to_string));
    match status.as_deref() {
        Some("no-remote") => {
            "There is no remote to sync with: neither this replica nor the checkout has one."
        }
        _ => "Pulled from and pushed to the remote.",
    }
}

/// Publishes this replica to a remote that has no Beads history yet.
fn first_publish(root: &Path, deadline: Instant, limit: Duration) -> SyncResult {
    match bd(root, &["dolt", "push"]).deadline(deadline).run() {
        Ok(push) if push.success() => {
            SyncResult::synced("Published this replica to a remote that had no Beads history.")
        }
        Ok(push) => SyncResult::failed_with(&bd_error(&push, "dolt push")),
        Err(error) => interrupted(&error, "dolt push", limit),
    }
}

fn interrupted(error: &RunError, command: &str, limit: Duration) -> SyncResult {
    match error {
        RunError::TimedOut => SyncResult::failed(
            SyncProblem::TimedOut,
            format!(
                "The sync did not finish within {}; nothing is known to have been pushed.",
                seconds(limit)
            ),
        ),
        RunError::Missing | RunError::Spawn(_) => {
            SyncResult::failed(SyncProblem::BdMissing, interruption(error, command))
        }
    }
}

fn seconds(limit: Duration) -> String {
    match limit.as_secs() {
        1 => "1 second".to_string(),
        seconds => format!("{seconds} seconds"),
    }
}

/// A sync Beads halted on a conflict: the conflicted tables, and what differs from the remote.
fn halted(root: &Path, sync: &Run, deadline: Instant) -> SyncResult {
    let tables: Vec<String> = parse_json(&sync.stdout)
        .and_then(|output| serde_json::from_value(output["conflicts"].clone()).ok())
        .unwrap_or_default();
    let place = if tables.is_empty() {
        "the database".to_string()
    } else {
        tables.join(", ")
    };
    let mut lines = vec![format!(
        "Beads halted on a conflict in {place}; nothing was pushed."
    )];
    // Embedded storage aborts the halted merge, so the conflicted rows are gone; what differs
    // from the remote's last fetched state is the closest Beads can show.
    let differing = bd(root, &["diff", "main", "origin/main", "--json"])
        .deadline(deadline)
        .run()
        .ok()
        .filter(Run::success)
        .map(|diff| differing_items(&diff));
    match &differing {
        Some(items) if !items.is_empty() => {
            let described: Vec<String> = items.iter().map(DifferingItem::describe).collect();
            lines.push(format!("Differs from the remote: {}", described.join(", ")));
        }
        Some(_) => {}
        None => {
            lines.push("Run `bd diff main origin/main` to see what differs from the remote.".into())
        }
    }
    SyncResult::ended(
        SyncOutcome::Conflict,
        lines.join("\n"),
        Some(SyncProblem::Conflict(ConflictReport { tables, differing })),
    )
}
