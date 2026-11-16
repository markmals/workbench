//! `workbench beads close`: closing an item as this machine's claimant, visibly to the others.

use schemars::schema_for;
use serde_json::json;

use super::claim_command::ITEM_ID;
use super::claims::close_item;
use super::receipts::{ItemChange, ItemReceipt, preview};
use super::workspace::claimant;
use super::{TIMEOUT, open_own_replica, time_limit};
use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::registry::{
    ArgumentKind, ArgumentSpec, CommandSpec, DRY_RUN, Effect, Example, Requirement, ValueKind,
};
use crate::response::Response;

pub const COMMAND: CommandSpec = CommandSpec {
    id: "beads.close",
    path: &["beads", "close"],
    summary: "Close a Beads item as this machine's claimant, then sync so other machines see it",
    description: "Closes the item with the reason as this machine's claimant (`BEADS_ACTOR`, or \
        `<login user>@<short host name>` when it is unset) — a plain `bd close` runs as the Git \
        user name and is refused for an item the claimant holds — then syncs. The receipt's \
        `status` is `closed` (`published: true`) or, when the sync failed, `closed-unpublished` \
        (`published: false`, with the reason on stderr); both exit 0. When Beads refuses because \
        another claimant holds the item the error is CONFLICT, with `details.holder`; an unknown \
        id is NOT_FOUND; another refusal, or a replica whose Beads remote is not the checkout's \
        `origin`, is PRECONDITION_FAILED. A refused close syncs nothing. The sync is limited to \
        `--timeout` seconds (default 20). `--dry-run` reports the item, its current claimant, and \
        the claimant it would close as.",
    arguments: &[
        ITEM_ID,
        ArgumentSpec {
            name: "reason",
            kind: ArgumentKind::Option,
            value: ValueKind::String,
            required: true,
            default: None,
            multiple: false,
            summary: "Why the item is closed, such as the pull request that resolved it",
        },
        DRY_RUN,
        TIMEOUT,
    ],
    effect: Effect::Modifies("the item's status, in this replica and on the Beads remote"),
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
            summary: "Close wb-1 once its fix is in a pull request",
            args: &["beads", "close", "wb-1", "--reason", "Fixed in PR #42"],
        },
        Example {
            summary: "See who holds wb-1 and who the close would be made as",
            args: &["beads", "close", "wb-1", "--reason", "Done", "--dry-run"],
        },
    ],
    synonyms: &[
        "finish", "done", "complete", "resolve", "item", "issue", "task", "end",
    ],
    output_schema: || schema_for!(ItemReceipt),
    run: close,
};

const CANNOT_PREDICT: &[&str] = &[
    "Whether Beads accepts the close: it refuses an item another claimant holds, and the holder \
     can change before the close runs.",
    "Whether the sync after the close succeeds; a close whose sync fails is this machine's only \
     (`closed-unpublished`).",
];

fn close(invocation: &Invocation) -> Result<Response, Failure> {
    let id = invocation.string("id").expect("id is required");
    let reason = invocation.string("reason").expect("reason is required");
    if reason.trim().is_empty() {
        return Err(
            Failure::invalid_argument("--reason must say why the item is closed")
                .with_details(json!({ "argument": "reason" })),
        );
    }
    let limit = time_limit(invocation)?;
    let replica = open_own_replica(invocation, limit)?;
    let actor = claimant(&replica.root)?;
    if invocation.dry_run() {
        let preview = preview(replica, &id, actor, CANNOT_PREDICT)?;
        return Ok(Response::new(&ItemReceipt::DryRun(preview)));
    }
    let closed = close_item(&replica.root, &id, &reason, &actor, limit)?;
    Ok(ItemChange::new(replica, &id, &actor, closed.sync, "closed").into_response())
}
