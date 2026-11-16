//! `workbench beads`: this machine's Beads replica. It syncs with the project's own remote,
//! and claims and closes items as this machine's claimant so other machines see them.

mod bd;
mod claim_command;
mod claims;
mod close_command;
mod items;
mod receipts;
mod remote;
pub mod report;
mod repository;
pub mod sync;
mod sync_command;
mod workspace;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_json::json;

use crate::failure::Failure;
use crate::invocation::Invocation;
use crate::registry::{ArgumentKind, ArgumentSpec, CommandSpec, ValueKind};

pub static COMMANDS: &[CommandSpec] = &[
    sync_command::COMMAND,
    claim_command::COMMAND,
    close_command::COMMAND,
];

/// The time limit for each sync with the Beads remote.
pub const TIMEOUT: ArgumentSpec = ArgumentSpec {
    name: "timeout",
    kind: ArgumentKind::Option,
    value: ValueKind::Integer,
    required: false,
    default: Some("20"),
    multiple: false,
    summary: "Seconds each sync with the Beads remote may take before it fails",
};

/// `--timeout`, which must allow at least a second.
pub fn time_limit(invocation: &Invocation) -> Result<Duration, Failure> {
    let seconds = invocation
        .integer("timeout")
        .expect("timeout has a default");
    if seconds == 0 {
        return Err(
            Failure::invalid_argument("--timeout must be at least 1 second")
                .with_details(json!({ "argument": "timeout", "minimum": 1, "value": seconds })),
        );
    }
    Ok(Duration::from_secs(seconds))
}

/// The replica a Beads command acts on, once it is known to be usable.
struct Replica {
    root: PathBuf,
    /// The project root, as receipts report it.
    project: String,
    /// The Beads remote, without credentials, or `None` when neither the replica nor the
    /// checkout has one.
    remote: Option<String>,
    /// Why no sync, claim, or close may run here, if one may not.
    refusal: Option<remote::Refusal>,
}

/// Resolves the project and its replica, and whether its Beads remote is the project's own.
fn open_replica(invocation: &Invocation, limit: Duration) -> Result<Replica, Failure> {
    let project = invocation.project()?;
    workspace::check_workspace(&project.root).map_err(|unavailable| unavailable.to_failure())?;
    let remotes = remote::read_remotes(&project.root, Instant::now() + limit);
    Ok(Replica {
        project: project.root.display().to_string(),
        remote: remotes.target(),
        refusal: remotes.refusal(&project.root),
        root: project.root,
    })
}

/// [`open_replica`], refusing a replica whose Beads remote is not the project's own before
/// anything changes.
fn open_own_replica(invocation: &Invocation, limit: Duration) -> Result<Replica, Failure> {
    let replica = open_replica(invocation, limit)?;
    match &replica.refusal {
        Some(refusal) => Err(refusal.to_failure()),
        None => Ok(replica),
    }
}
