//! Which phase the facts put the work in, what happens next, and what deserves a notice.

use schemars::JsonSchema;
use serde::Serialize;

use super::facts::{Facts, ProposalSummary, PullRequest, PullRequestLookup, PullRequestState};
use super::steps::{self, Actor, Step};

/// The phase, the situation, the next action, and who takes it.
#[derive(Clone, Debug, PartialEq, Serialize, JsonSchema)]
pub struct DerivedStatus {
    pub phase: String,
    pub title: String,
    pub situation: String,
    pub next: String,
    pub actor: Actor,
    /// The skill that covers the next action, by name.
    pub skill: Option<String>,
    /// Why the pull request is unknown, when it is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub notices: Vec<Notice>,
}

/// Something the next actor should know besides the phase.
#[derive(Clone, Debug, PartialEq, Serialize, JsonSchema)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum Notice {
    /// The proposal has unresolved clarification markers.
    Clarifications { count: usize },
    /// The branch has commits its upstream does not.
    UnpushedCommits { count: u64 },
    /// The proposal is in active review, but its pull request is still a draft.
    DraftActiveReview,
    /// The proposal was chosen by a heuristic, or names another pull request.
    AmbiguousProposalSelection {
        selected: Option<String>,
        other_candidates: Vec<ProposalSummary>,
        pull_request_mismatch: bool,
        proposal_pull_request: Option<String>,
        pull_request_number: Option<u64>,
    },
}

/// Which branch this repository treats as its default, and whether the branch is it.
#[derive(Clone, Debug, PartialEq)]
pub struct DefaultBranch {
    pub default_branch: Option<String>,
    pub is_default_branch: Option<bool>,
}

// Ordered: `main` wins when a repository somehow carries both.
pub const CONVENTIONAL_DEFAULT_BRANCHES: [&str; 2] = ["main", "master"];

/// The default branch from the most authoritative source available: `origin/HEAD`, then
/// `gh repo view`, then a conventional branch that exists locally, then the current branch's
/// own name. The last two keep a repository with no remote yet — a fresh `git init -b main` —
/// from reading as "role unknown": the local probe covers a feature branch cut from `main`,
/// and the current-branch case covers `main` itself before its first commit creates it.
pub fn default_branch_facts(
    branch: Option<&str>,
    origin_head: Option<&str>,
    gh_default: Option<&str>,
    local_default: Option<&str>,
) -> DefaultBranch {
    let conventional = branch.filter(|name| CONVENTIONAL_DEFAULT_BRANCHES.contains(name));
    let default_branch = origin_head
        .and_then(|reference| reference.rsplit('/').next())
        .or(gh_default)
        .or(local_default)
        .or(conventional)
        .map(str::to_string);
    DefaultBranch {
        is_default_branch: default_branch
            .as_deref()
            .map(|default| branch == Some(default)),
        default_branch,
    }
}

pub fn derive_status(facts: &Facts) -> DerivedStatus {
    let notices = notices(facts);
    let derived = |step: &Step| from_step(step, notices.clone());
    match facts.is_default_branch {
        Some(true) => return derived(&steps::DEFAULT_BRANCH),
        None => return derived(&steps::UNKNOWN_BRANCH_ROLE),
        Some(false) => {}
    }
    let pull_request = match &facts.pull_request {
        PullRequestLookup::Unknown { reason } => return unknown_pull_request(reason, notices),
        PullRequestLookup::None => return derived(&steps::NO_PULL_REQUEST),
        PullRequestLookup::Found(pull_request) => pull_request,
    };
    let Some(proposal) = &facts.proposal else {
        return derived(&steps::NO_PROPOSAL);
    };
    let status = proposal.status.as_deref().unwrap_or_default();
    if let Some(incomplete) = integration_incomplete(status, pull_request, facts) {
        return DerivedStatus {
            notices,
            ..incomplete
        };
    }
    if status == "superseded" {
        return superseded(facts.proposal_successor.as_ref(), notices);
    }
    derived(step_for(status, pull_request))
}

fn step_for(status: &str, pull_request: &PullRequest) -> &'static Step {
    let state = pull_request.state;
    match status {
        "awaiting-implementation" if !pull_request.is_draft => {
            &steps::AWAITING_IMPLEMENTATION_READY
        }
        "implemented" => match state {
            PullRequestState::Merged => &steps::MERGED,
            PullRequestState::Closed => &steps::CLOSED_WITHOUT_MERGE,
            PullRequestState::Open if pull_request.is_draft => &steps::DRAFT_IMPLEMENTATION,
            PullRequestState::Open => &steps::READY_TO_MERGE,
        },
        "accepted" if state == PullRequestState::Merged => &steps::ACCEPTED_AFTER_MERGE,
        "accepted" if state == PullRequestState::Closed => &steps::CLOSED_WITHOUT_MERGE,
        "accepted" if pull_request.is_draft => &steps::ACCEPTED_DRAFT,
        status => steps::for_status(status).unwrap_or(&steps::UNKNOWN_STATUS),
    }
}

/// A merged pull request for accepted or implemented work that did not merge into the default
/// branch, or whose default branch is unknown: delivery is not established.
fn integration_incomplete(
    status: &str,
    pull_request: &PullRequest,
    facts: &Facts,
) -> Option<DerivedStatus> {
    let merged_work = matches!(status, "accepted" | "implemented")
        && pull_request.state == PullRequestState::Merged;
    let into_default = facts.default_branch.as_deref() == Some(pull_request.base_ref_name.as_str());
    if !merged_work || into_default {
        return None;
    }
    let next = match facts.default_branch.as_deref() {
        Some(destination) => format!(
            "The pull request merged into {}, not {destination}. Ask the human how to integrate the accepted change into {destination}; do not claim delivery.",
            pull_request.base_ref_name
        ),
        None => "Confirm the default branch before deciding whether the pull request merged into it; do not claim delivery.".to_string(),
    };
    Some(DerivedStatus {
        phase: "5".into(),
        title: "Integration incomplete".into(),
        situation: "The feature pull request is merged, but integration into the default branch is not established.".into(),
        next,
        actor: Actor::You,
        skill: None,
        reason: None,
        notices: Vec::new(),
    })
}

fn unknown_pull_request(reason: &str, notices: Vec<Notice>) -> DerivedStatus {
    DerivedStatus {
        phase: "1".into(),
        title: "Preparation".into(),
        situation: format!("The pull request state is unknown because {reason}."),
        next: "Check gh access and the branch's pull request association before continuing.".into(),
        actor: Actor::Agent,
        skill: None,
        reason: Some(reason.to_string()),
        notices,
    }
}

fn superseded(successor: Option<&ProposalSummary>, notices: Vec<Notice>) -> DerivedStatus {
    let Some(successor) = successor else {
        return from_step(&steps::SUPERSEDED, notices);
    };
    DerivedStatus {
        situation: format!(
            "The work is closed. Its successor is {} — {}.",
            successor.id.as_deref().unwrap_or("an unnamed proposal"),
            successor.title.as_deref().unwrap_or("untitled")
        ),
        next: "Nothing outstanding. Continue with the successor if further work is needed.".into(),
        ..from_step(&steps::SUPERSEDED, notices)
    }
}

fn from_step(step: &Step, notices: Vec<Notice>) -> DerivedStatus {
    DerivedStatus {
        phase: step.phase.into(),
        title: step.title.into(),
        situation: step.situation.into(),
        next: step.next.into(),
        actor: step.actor,
        skill: step.skill.map(str::to_string),
        reason: None,
        notices,
    }
}

fn notices(facts: &Facts) -> Vec<Notice> {
    let proposal = facts.proposal.as_ref();
    let pull_request = facts.pull_request.found();
    let mut notices = Vec::new();
    if let Some(count) = proposal
        .map(|proposal| proposal.clarification_count)
        .filter(|count| *count > 0)
    {
        notices.push(Notice::Clarifications { count });
    }
    if facts.unpushed_commits > 0 {
        notices.push(Notice::UnpushedCommits {
            count: facts.unpushed_commits,
        });
    }
    let active_review =
        proposal.and_then(|proposal| proposal.status.as_deref()) == Some("active-review");
    if active_review && pull_request.is_some_and(|pull_request| pull_request.is_draft) {
        notices.push(Notice::DraftActiveReview);
    }
    let selection = &facts.proposal_selection;
    if let Some(proposal) = proposal
        && (!selection.other_candidates.is_empty() || selection.pull_request_mismatch)
    {
        notices.push(Notice::AmbiguousProposalSelection {
            selected: proposal.id.clone(),
            other_candidates: selection.other_candidates.clone(),
            pull_request_mismatch: selection.pull_request_mismatch,
            proposal_pull_request: proposal.pull_request.clone(),
            pull_request_number: pull_request.map(|pull_request| pull_request.number),
        });
    }
    notices
}
