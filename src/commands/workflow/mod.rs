//! `workbench workflow status`: where the work stands, who acts next, and this machine's Beads
//! work. Its contract is "Find the next actor" in guides/record-validation.md.

mod derive;
mod facts;
mod gather;
mod proposals;
mod pull_request;
mod steps;
mod text;

use schemars::{JsonSchema, schema_for};
use serde::Serialize;

pub use derive::{DefaultBranch, DerivedStatus, Notice, default_branch_facts, derive_status};
pub use facts::{
    Facts, Proposal, ProposalSelection, ProposalSummary, PullRequest, PullRequestLookup,
    PullRequestState,
};
pub use steps::Actor;
pub use text::status_text;

use crate::commands::beads::report::{BeadsReport, collect_report};
use crate::commands::beads::time_limit;
use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::registry::{
    ArgumentKind, ArgumentSpec, CommandSpec, Effect, Example, Requirement, ValueKind,
};
use crate::response::{RenderText, Response};

pub static COMMANDS: &[CommandSpec] = &[CommandSpec {
    id: "workflow.status",
    path: &["workflow", "status"],
    summary: "Report where the work stands, what happens next and who does it, and this machine's Beads work",
    description: "Reads the branch, its pull request (through `gh`), and the active proposal, and \
        reports the phase, the situation, the next action, whether the agent or the human (`you`) \
        takes it, the skill that covers it, notices, and the `facts` they were derived from. A pull \
        request `gh` cannot look up — `gh` missing, not authenticated, or not answering — is \
        `unknown` with the reason, never `none`. Before reporting it runs one Beads sync with the \
        project's remote, the same one `workbench beads sync` runs; `--no-sync` skips it. `beads` \
        then holds the sync's `outcome` (`synced`, `skipped`, `failed`, `conflict`, or `blocked`) \
        and `detail`, the ready work, the items in progress and who holds them, and the project's \
        memories; or `unavailable` with the reason. `--timeout` limits the sync and each `gh` query \
        (default 20 seconds). Beads outcomes never change the exit status: the command exits 0 \
        whenever it can read the checkout.",
    arguments: &[
        ArgumentSpec {
            name: "no-sync",
            kind: ArgumentKind::Flag,
            value: ValueKind::Boolean,
            required: false,
            default: None,
            multiple: false,
            summary: "Report this machine's Beads replica without syncing it first",
        },
        ArgumentSpec {
            name: "timeout",
            kind: ArgumentKind::Option,
            value: ValueKind::Integer,
            required: false,
            default: Some("20"),
            multiple: false,
            summary: "Seconds the Beads sync, and each GitHub query, may take",
        },
    ],
    effect: Effect::Reads,
    requires: &[
        Requirement::Git,
        Requirement::Gh,
        Requirement::GhAuthentication,
        Requirement::Bd,
        Requirement::Network,
    ],
    dry_run: false,
    idempotent: true,
    errors: &[
        ErrorCode::InvalidArgument,
        ErrorCode::PreconditionFailed,
        ErrorCode::DependencyMissing,
    ],
    examples: &[
        Example {
            summary: "Orient at the start of a session",
            args: &["workflow", "status"],
        },
        Example {
            summary: "Report without syncing Beads, as when offline",
            args: &["workflow", "status", "--no-sync"],
        },
    ],
    synonyms: &[
        "where",
        "next",
        "phase",
        "orient",
        "start",
        "session",
        "progress",
        "ready",
        "memories",
        "overview",
        "situation",
        "todo",
    ],
    output_schema: || schema_for!(WorkflowStatus),
    run: report_status,
}];

/// The status report.
#[derive(Serialize, JsonSchema)]
pub struct WorkflowStatus {
    #[serde(flatten)]
    pub status: DerivedStatus,
    pub facts: Facts,
    pub beads: BeadsReport,
}

impl WorkflowStatus {
    pub fn new(facts: Facts, beads: BeadsReport) -> WorkflowStatus {
        WorkflowStatus {
            status: derive_status(&facts),
            facts,
            beads,
        }
    }
}

impl RenderText for WorkflowStatus {
    fn render_text(&self) -> String {
        format!(
            "{}\n\n{}",
            status_text(&self.facts, &self.status),
            self.beads.render_text()
        )
    }
}

fn report_status(invocation: &Invocation) -> Result<Response, Failure> {
    let limit = time_limit(invocation)?;
    let project = invocation.project()?;
    let facts = gather::gather_facts(&project.root, limit);
    let sync_limit = (!invocation.flag("no-sync")).then_some(limit);
    let beads = collect_report(&project.root, sync_limit);
    Ok(Response::new(&WorkflowStatus::new(facts, beads)))
}
