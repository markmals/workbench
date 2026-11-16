//! The receipts `beads claim` and `beads close` return, and their dry runs.

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::json;

use super::Replica;
use super::claims::unknown_item;
use super::items::{ItemRead, read_item};
use super::report::outcome_text;
use super::sync::SyncResult;
use crate::failure::Failure;
use crate::response::{RenderText, Response};

/// A claim or close, made or previewed.
#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum ItemReceipt {
    Changed(ItemChange),
    DryRun(ItemPreview),
}

/// A change Beads accepted.
#[derive(Serialize, JsonSchema)]
pub struct ItemChange {
    /// `claimed` or `closed` when other machines can see the change; `claimed-unpublished` or
    /// `closed-unpublished` when only this machine has it because the sync failed.
    pub status: String,
    /// The project root.
    pub project: String,
    /// The Beads remote, without credentials; `null` when there is none.
    pub remote: Option<String>,
    /// The item.
    pub id: String,
    /// Who the change was made as.
    pub claimant: String,
    /// Whether other machines can see the change.
    pub published: bool,
    /// The sync that published the change, or failed to.
    pub sync: SyncResult,
    pub detail: String,
}

/// What a claim or close would act on.
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemPreview {
    /// `dry-run`.
    pub status: String,
    pub project: String,
    pub remote: Option<String>,
    pub id: String,
    /// The claimant the change would be made as.
    pub claimant: String,
    pub title: Option<String>,
    /// The item's status in this replica: `open`, `in_progress`, `closed`, and so on.
    pub item_status: Option<String>,
    /// Who holds the item in this replica now, if anyone.
    pub current_claimant: Option<String>,
    /// What the dry run cannot know until the change runs.
    pub cannot_predict: Vec<String>,
}

impl ItemChange {
    /// The receipt, with the reason on stderr when only this machine has the change.
    pub fn into_response(self) -> Response {
        let warning = (!self.published).then(|| self.detail.clone());
        let response = Response::new(&ItemReceipt::Changed(self));
        match warning {
            Some(warning) => response.warn(warning),
            None => response,
        }
    }

    pub fn new(
        replica: Replica,
        id: &str,
        claimant: &str,
        sync: SyncResult,
        verb: &str,
    ) -> ItemChange {
        let published = sync.outcome == super::sync::SyncOutcome::Synced;
        let (status, detail) = if published {
            (
                verb.to_string(),
                format!("{id} is {verb} as {claimant}, and other machines can see it."),
            )
        } else {
            (
                format!("{verb}-unpublished"),
                format!(
                    "{id} is {verb} as {claimant} on this machine only; the sync ended {}: {} \
                     Run `workbench beads sync` until it succeeds.",
                    sync.label(),
                    sync.detail
                ),
            )
        };
        ItemChange {
            status,
            project: replica.project,
            remote: replica.remote,
            id: id.to_string(),
            claimant: claimant.to_string(),
            published,
            sync,
            detail,
        }
    }
}

/// Describes the item a claim or close would act on, without changing anything.
pub fn preview(
    replica: Replica,
    id: &str,
    claimant: String,
    cannot_predict: &[&str],
) -> Result<ItemPreview, Failure> {
    let item = match read_item(&replica.root, id) {
        ItemRead::Found(item) => item,
        ItemRead::NotFound(reason) => return Err(unknown_item(id, &reason)),
        ItemRead::Unreadable(reason) => {
            return Err(Failure::precondition(format!("cannot read {id}: {reason}"))
                .with_details(json!({ "id": id })));
        }
    };
    Ok(ItemPreview {
        status: "dry-run".into(),
        project: replica.project,
        remote: replica.remote,
        id: id.to_string(),
        current_claimant: item.holder().map(str::to_string),
        title: item.title,
        item_status: item.status,
        claimant,
        cannot_predict: cannot_predict.iter().map(|line| line.to_string()).collect(),
    })
}

impl RenderText for ItemReceipt {
    fn render_text(&self) -> String {
        match self {
            ItemReceipt::Changed(change) => outcome_text(&change.status, &change.detail),
            ItemReceipt::DryRun(preview) => {
                let holder = preview.current_claimant.as_deref().unwrap_or("nobody");
                let mut lines = vec![
                    format!(
                        "Dry run: {}  {}  ·  {}",
                        preview.id,
                        preview.title.as_deref().unwrap_or(""),
                        preview.item_status.as_deref().unwrap_or("unknown status")
                    ),
                    format!("Held by {holder}; would act as {}.", preview.claimant),
                    "Cannot predict:".to_string(),
                ];
                lines.extend(
                    preview
                        .cannot_predict
                        .iter()
                        .map(|line| format!("  {line}")),
                );
                lines.join("\n")
            }
        }
    }
}
