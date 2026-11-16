//! The Beads part of `workbench workflow status`. Its contract is "Beads in the status report"
//! in guides/record-validation.md.

use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::bd::{bd, bd_error, interruption, parse_json};
use super::sync::{SyncOutcome, SyncResult, sync_replica};
use super::workspace::check_workspace;
use crate::response::RenderText;

// `bd prime` addresses session-start hook hosts in its first line; it is not a memory.
static PRIME_HOST_NOTICE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\[bd prime\][^\n]*\n+").expect("valid regex"));

/// This machine's Beads replica after the sync, or why it cannot be read.
#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum BeadsReport {
    Available(BeadsState),
    Unavailable(BeadsUnavailable),
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BeadsState {
    pub sync: SyncResult,
    /// Work no open item blocks, from `bd ready`.
    pub ready: Vec<ReadyItem>,
    /// Items in progress and who holds them, from `bd list --status in_progress`.
    pub in_progress: Vec<InProgressItem>,
    /// The project's memories, from `bd prime --memories-only`.
    pub memories: String,
}

#[derive(Serialize, JsonSchema)]
pub struct BeadsUnavailable {
    /// Why: `bd` is missing, there is no workspace or database, or a read after the sync failed.
    pub unavailable: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ReadyItem {
    pub id: String,
    pub title: String,
    pub priority: i64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InProgressItem {
    pub id: String,
    pub title: String,
    pub assignee: Option<String>,
}

/// Syncs, unless `sync_limit` is `None`, then reads this machine's replica: ready work, items
/// in progress, and memories. A failed read makes the whole report unavailable rather than
/// partial; its reason names the failed read and keeps the sync's outcome, which came first.
pub fn collect_report(root: &Path, sync_limit: Option<Duration>) -> BeadsReport {
    if let Err(unavailable) = check_workspace(root) {
        return unavailable_because(unavailable.reason);
    }
    let sync = match sync_limit {
        Some(limit) => sync_replica(root, limit),
        None => SyncResult::skipped(),
    };
    match read_replica(root) {
        Ok(replica) => BeadsReport::Available(BeadsState {
            sync,
            ready: replica.ready,
            in_progress: replica.in_progress,
            memories: replica.memories,
        }),
        Err(failure) => unavailable_because(format!(
            "reading this replica failed (bd {}: {}); {}",
            failure.command,
            failure.reason,
            sync_before(&sync)
        )),
    }
}

struct Replica {
    ready: Vec<ReadyItem>,
    in_progress: Vec<InProgressItem>,
    memories: String,
}

/// A read of the replica that failed: the `bd` command, and why.
struct ReadFailure {
    command: &'static str,
    reason: String,
}

fn read_replica(root: &Path) -> Result<Replica, ReadFailure> {
    let ready = read_list(root, "ready", &["ready", "--limit", "0", "--json"])?;
    let in_progress = read_list(
        root,
        "list",
        &["list", "--status", "in_progress", "--limit", "0", "--json"],
    )?;
    let prime = read(root, "prime", &["prime", "--memories-only"])?;
    let memories = PRIME_HOST_NOTICE.replace(&prime, "").trim().to_string();
    Ok(Replica {
        ready,
        in_progress,
        memories,
    })
}

fn read(root: &Path, command: &'static str, args: &[&str]) -> Result<String, ReadFailure> {
    let reason = match bd(root, args).run() {
        Ok(run) if run.success() => return Ok(run.stdout),
        Ok(run) => bd_error(&run, command),
        Err(error) => interruption(&error, command),
    };
    Err(ReadFailure { command, reason })
}

fn read_list<T: DeserializeOwned>(
    root: &Path,
    command: &'static str,
    args: &[&str],
) -> Result<Vec<T>, ReadFailure> {
    let output = read(root, command, args)?;
    parse_json(&output)
        .and_then(|value| serde_json::from_value(value).ok())
        .ok_or_else(|| ReadFailure {
            command,
            reason: "its output is not the JSON list bd prints".to_string(),
        })
}

fn sync_before(sync: &SyncResult) -> String {
    match sync.outcome {
        SyncOutcome::Skipped => "no sync ran before it (--no-sync)".to_string(),
        outcome => format!(
            "the sync before it ended {}: {}",
            outcome.as_str(),
            sync.detail
        ),
    }
}

fn unavailable_because(reason: String) -> BeadsReport {
    BeadsReport::Unavailable(BeadsUnavailable {
        unavailable: reason,
    })
}

/// One outcome line, then any further detail lines indented beneath it.
pub fn outcome_text(outcome: &str, detail: &str) -> String {
    let mut lines = detail.lines();
    let first = lines.next().unwrap_or_default();
    let mut text = format!("Beads: {outcome}. {first}");
    for line in lines {
        text.push_str("\n  ");
        text.push_str(line);
    }
    text
}

impl RenderText for BeadsReport {
    fn render_text(&self) -> String {
        match self {
            BeadsReport::Unavailable(BeadsUnavailable { unavailable }) => outcome_text(
                "unavailable",
                &format!(
                    "{}. Run `mise run setup`.",
                    unavailable.trim_end_matches('.')
                ),
            ),
            BeadsReport::Available(state) => state.render_text(),
        }
    }
}

impl RenderText for BeadsState {
    fn render_text(&self) -> String {
        let detail = match self.sync.outcome {
            SyncOutcome::Synced => self.sync.detail.clone(),
            _ => format!(
                "{}\nWhat follows is this machine's replica, which may be stale.",
                self.sync.detail
            ),
        };
        let mut lines = vec![outcome_text(&self.sync.label(), &detail), String::new()];
        lines.push(heading("Ready", self.ready.is_empty()));
        for item in &self.ready {
            lines.push(format!("  {}  P{}  {}", item.id, item.priority, item.title));
        }
        lines.push(heading("In progress", self.in_progress.is_empty()));
        for item in &self.in_progress {
            let holder = item.assignee.as_deref().unwrap_or("unassigned");
            lines.push(format!("  {}  {holder}  {}", item.id, item.title));
        }
        lines.push(String::new());
        lines.push(self.memories.clone());
        lines.join("\n")
    }
}

fn heading(name: &str, empty: bool) -> String {
    if empty {
        format!("{name}: none")
    } else {
        format!("{name}:")
    }
}
