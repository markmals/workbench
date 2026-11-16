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
    /// Why a sync that did not complete did not, for choosing an error code.
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

/// What Beads says about a conflict it halted on.
#[derive(Debug)]
pub struct ConflictReport {
    pub tables: Vec<String>,
    /// The items that differ from the remote, or `None` when they could not be listed in time.
    pub differing: Option<Vec<DifferingItem>>,
}

impl SyncResult {
    pub fn skipped() -> SyncResult {
        SyncResult {
            outcome: SyncOutcome::Skipped,
            detail: "--no-sync was passed; nothing was pulled or pushed.".into(),
            problem: None,
        }
    }

    fn synced(detail: &str) -> SyncResult {
        SyncResult {
            outcome: SyncOutcome::Synced,
            detail: detail.into(),
            problem: None,
        }
    }

    fn failed(problem: SyncProblem, detail: String) -> SyncResult {
        SyncResult {
            outcome: SyncOutcome::Failed,
            detail,
            problem: Some(problem),
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
        Some(4) => SyncResult {
            outcome: SyncOutcome::Blocked,
            detail: format!("{} Nothing was pushed.", sentence(&bd_error(&sync, "sync"))),
            problem: Some(SyncProblem::Blocked),
        },
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
    SyncResult {
        outcome: SyncOutcome::Conflict,
        detail: lines.join("\n"),
        problem: Some(SyncProblem::Conflict(ConflictReport { tables, differing })),
    }
}
