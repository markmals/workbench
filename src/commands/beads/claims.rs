//! Claiming and closing items as this machine's claimant. `bd update --claim` and `bd close`
//! both act as `BEADS_ACTOR`, and `bd close` refuses an item another actor holds, so both run
//! with the claimant set rather than as the Git user name a plain `bd` uses.

use std::path::Path;
use std::time::Duration;

use serde_json::json;

use super::bd::{bd, bd_error, interruption};
use super::items::{ItemRead, read_item};
use super::sync::{SyncResult, sync_replica};
use crate::failure::{ErrorCode, Failure};
use crate::process::Program;

/// How a claim that Beads accepted, or refused because of a holder, ended.
pub enum Claim {
    /// The claim is this machine's; `sync` is the sync that published it, or failed to.
    Made { sync: SyncResult },
    /// Another claimant already held the item.
    Held { holder: String },
    /// After syncing, the item is no longer this claimant's: a change from another machine won.
    Lost {
        holder: Option<String>,
        status: Option<String>,
    },
}

/// A close that Beads accepted, and the sync that published it, or failed to.
pub struct Close {
    pub sync: SyncResult,
}

/// Claims `id` so other machines see it: sync, claim as `actor`, sync again, then re-read who
/// holds it. The caller has checked that the replica's Beads remote is the project's own. Fails
/// when Beads rejects the claim for any reason other than another claimant holding the item.
pub fn claim_item(root: &Path, id: &str, actor: &str, limit: Duration) -> Result<Claim, Failure> {
    sync_replica(root, limit);
    let claim = bd(root, &["update", id, "--claim", "--json"]);
    if let Some(reason) = refusal_as_claimant(claim, actor, "update --claim") {
        return match read_item(root, id) {
            ItemRead::Found(item) if item.is_in_progress() => match item.holder() {
                Some(holder) if holder != actor => Ok(Claim::Held {
                    holder: holder.to_string(),
                }),
                _ => Err(refused(id, &reason)),
            },
            read => Err(refusal_failure(id, &reason, read)),
        };
    }
    let sync = sync_replica(root, limit);
    match read_item(root, id) {
        ItemRead::Found(item) if item.is_in_progress() && item.holder() == Some(actor) => {
            Ok(Claim::Made { sync })
        }
        ItemRead::Found(item) => Ok(Claim::Lost {
            holder: item
                .is_in_progress()
                .then(|| item.holder().map(str::to_string))
                .flatten(),
            status: item.status,
        }),
        ItemRead::NotFound(_) => Ok(Claim::Lost {
            holder: None,
            status: None,
        }),
        ItemRead::Unreadable(reason) => Err(Failure::precondition(format!(
            "{id} was claimed, but could not be read back to confirm who holds it: {reason}"
        ))
        .with_details(json!({ "id": id, "claimant": actor }))
        .suggest(format!(
            "Read the item with `bd show {id}` before starting the work"
        ))),
    }
}

/// Closes `id` as `actor`, then syncs so other machines see it. The caller has checked that the
/// replica's Beads remote is the project's own. When Beads refuses, nothing is synced.
pub fn close_item(
    root: &Path,
    id: &str,
    reason: &str,
    actor: &str,
    limit: Duration,
) -> Result<Close, Failure> {
    let reason_argument = format!("--reason={reason}");
    let close = bd(root, &["close", id, &reason_argument, "--json"]);
    if let Some(refusal) = refusal_as_claimant(close, actor, "close") {
        return Err(match read_item(root, id) {
            ItemRead::Found(item) => match item.holder() {
                Some(holder) if holder != actor => held(id, holder, actor, "held", &refusal),
                _ => refused(id, &refusal),
            },
            read => refusal_failure(id, &refusal, read),
        });
    }
    Ok(Close {
        sync: sync_replica(root, limit),
    })
}

/// Runs a change as `actor`; what Beads said when it refused the change, if it did.
fn refusal_as_claimant(program: Program, actor: &str, command: &str) -> Option<String> {
    match program.env("BEADS_ACTOR", actor).run() {
        Ok(run) if run.success() => None,
        Ok(run) => Some(bd_error(&run, command)),
        Err(error) => Some(interruption(&error, command)),
    }
}

/// The failure for an item another claimant holds.
pub fn held(id: &str, holder: &str, claimant: &str, outcome: &str, reason: &str) -> Failure {
    Failure::new(
        ErrorCode::Conflict,
        format!("{id} is held by {holder}: {reason}"),
    )
    .with_details(json!({
        "outcome": outcome,
        "id": id,
        "holder": holder,
        "claimant": claimant,
    }))
    .suggest("The item is not available; pick other work, or ask the human")
}

fn refused(id: &str, reason: &str) -> Failure {
    Failure::precondition(format!("Beads refused the change to {id}: {reason}"))
        .with_details(json!({ "id": id, "reason": reason }))
}

fn refusal_failure(id: &str, reason: &str, read: ItemRead) -> Failure {
    match read {
        ItemRead::NotFound(_) => unknown_item(id, reason),
        ItemRead::Unreadable(unreadable) => refused(id, &format!("{reason} ({unreadable})")),
        ItemRead::Found(_) => refused(id, reason),
    }
}

/// The failure for an id Beads has no item for.
pub fn unknown_item(id: &str, reason: &str) -> Failure {
    Failure::not_found(format!("no Beads item {id}: {reason}"))
        .with_details(json!({ "id": id }))
        .suggest("Find the item's id with `bd search` or `workbench workflow status`")
}
