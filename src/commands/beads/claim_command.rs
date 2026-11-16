//! `workbench beads claim`: claiming an item as this machine's claimant, visibly to the others.

use schemars::schema_for;
use serde_json::json;

use super::claims::{Claim, claim_item, held};
use super::receipts::{ItemChange, ItemReceipt, preview};
use super::workspace::claimant;
use super::{TIMEOUT, open_own_replica, time_limit};
use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::registry::{
    ArgumentKind, ArgumentSpec, CommandSpec, DRY_RUN, Effect, Example, Requirement, ValueKind,
};
use crate::response::Response;

/// The item a claim or close acts on.
pub const ITEM_ID: ArgumentSpec = ArgumentSpec {
    name: "id",
    kind: ArgumentKind::Positional,
    value: ValueKind::String,
    required: true,
    default: None,
    multiple: false,
    summary: "The Beads item's id, such as wb-1",
};

pub const COMMAND: CommandSpec = CommandSpec {
    id: "beads.claim",
    path: &["beads", "claim"],
    summary: "Claim a Beads item as this machine's claimant, so other machines see the claim",
    description: "Syncs, claims the item as this machine's claimant (`BEADS_ACTOR`, or \
        `<login user>@<short host name>` when it is unset), syncs again, and reads the item back to \
        confirm who holds it. The receipt's `status` is `claimed` (`published: true`) or, when the \
        second sync failed, `claimed-unpublished` (`published: false`, with the reason on stderr): \
        both exit 0 and the item is yours. CONFLICT (exit 4) means another claimant already held \
        the item (`details.outcome: held`) or a concurrent claim from another machine won \
        (`lost`); `details.holder` names the holder. Do not start the work then. An unknown id is \
        NOT_FOUND; another refusal from Beads, or a replica whose Beads remote is not the \
        checkout's `origin`, is PRECONDITION_FAILED and changes nothing. Each sync is limited to \
        `--timeout` seconds (default 20). `--dry-run` reports the item, its current claimant, and \
        the claimant it would claim as.",
    arguments: &[ITEM_ID, DRY_RUN, TIMEOUT],
    effect: Effect::Modifies(
        "the item's assignee and status, in this replica and on the Beads remote",
    ),
    requires: &[Requirement::Git, Requirement::Bd, Requirement::Network],
    dry_run: true,
    idempotent: true,
    errors: &[
        ErrorCode::InvalidArgument,
        ErrorCode::NotFound,
        ErrorCode::PreconditionFailed,
        ErrorCode::DependencyMissing,
        ErrorCode::Conflict,
    ],
    examples: &[
        Example {
            summary: "Claim wb-1 before starting it",
            args: &["beads", "claim", "wb-1"],
        },
        Example {
            summary: "See who holds wb-1 and who the claim would be made as",
            args: &["beads", "claim", "wb-1", "--dry-run"],
        },
    ],
    synonyms: &[
        "take", "start", "assign", "own", "pick", "begin", "work", "item", "issue", "task",
    ],
    output_schema: || schema_for!(ItemReceipt),
    run: claim,
};

const CANNOT_PREDICT: &[&str] = &[
    "Whether another machine claims the item first: the sync before the claim may bring a claim \
     made elsewhere, and a concurrent claim can still win before the second sync.",
    "Whether the syncs succeed; a claim whose sync fails is this machine's only \
     (`claimed-unpublished`).",
];

fn claim(invocation: &Invocation) -> Result<Response, Failure> {
    let id = invocation.string("id").expect("id is required");
    let limit = time_limit(invocation)?;
    let replica = open_own_replica(invocation, limit)?;
    let actor = claimant(&replica.root)?;
    if invocation.dry_run() {
        let preview = preview(replica, &id, actor, CANNOT_PREDICT)?;
        return Ok(Response::new(&ItemReceipt::DryRun(preview)));
    }
    match claim_item(&replica.root, &id, &actor, limit)? {
        Claim::Made { sync } => {
            Ok(ItemChange::new(replica, &id, &actor, sync, "claimed").into_response())
        }
        Claim::Held { holder } => Err(held(&id, &holder, &actor, "held", "it is not available")),
        Claim::Lost { holder, status } => Err(lost(&id, holder, status, &actor)),
    }
}

fn lost(id: &str, holder: Option<String>, status: Option<String>, actor: &str) -> Failure {
    let now = match (&holder, &status) {
        (Some(holder), _) => format!("is now held by {holder}"),
        (None, Some(status)) => format!("is no longer claimed (status {status})"),
        (None, None) => "no longer exists".to_string(),
    };
    Failure::new(
        ErrorCode::Conflict,
        format!("{id} {now}: a change from another machine won the claim (lost)."),
    )
    .with_details(json!({
        "outcome": "lost",
        "id": id,
        "holder": holder,
        "itemStatus": status,
        "claimant": actor,
    }))
    .suggest("Do not start the work; pick other work, or ask the human")
}
