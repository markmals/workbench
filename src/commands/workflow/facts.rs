//! The facts `workflow status` derives the phase from: the branch, its pull request, and the
//! active proposal.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Facts {
    /// The checked-out branch, or `null` when Git cannot name it.
    pub branch: Option<String>,
    /// The repository's default branch, or `null` when it cannot be determined.
    pub default_branch: Option<String>,
    /// Whether the branch is the default branch; `null` when that is unknown.
    pub is_default_branch: Option<bool>,
    pub has_upstream: bool,
    /// Commits on the branch that its upstream does not have.
    pub unpushed_commits: u64,
    pub pull_request: PullRequestLookup,
    /// The proposal the work is about, if any.
    pub proposal: Option<Proposal>,
    /// The proposal that supersedes a superseded one.
    pub proposal_successor: Option<ProposalSummary>,
    pub proposal_selection: ProposalSelection,
}

/// The branch's pull request, as far as `gh` could tell.
#[derive(Clone, Debug, PartialEq, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum PullRequestLookup {
    /// The branch has exactly this pull request, open or not.
    Found(PullRequest),
    /// `gh` positively found no pull request for the branch.
    None,
    /// `gh` could not tell, so whether a pull request exists is unknown.
    Unknown { reason: String },
}

impl PullRequestLookup {
    pub fn found(&self) -> Option<&PullRequest> {
        match self {
            PullRequestLookup::Found(pull_request) => Some(pull_request),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    pub number: u64,
    pub state: PullRequestState,
    pub base_ref_name: String,
    pub is_draft: bool,
    pub url: String,
    pub title: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

/// A proposal as its frontmatter states it. A field with the wrong shape reads as absent:
/// reporting status must not fail on a record `records validate` would reject.
#[derive(Clone, Debug, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    /// The `pull-request` the proposal names: a number or a URL.
    pub pull_request: Option<String>,
    pub supersedes: Option<Vec<String>>,
    /// Unresolved `[NEEDS CLARIFICATION: …]` markers in its prose.
    pub clarification_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, JsonSchema)]
pub struct ProposalSummary {
    pub id: Option<String>,
    pub title: Option<String>,
}

/// How the proposal was chosen, when the choice was not certain.
#[derive(Clone, Debug, Default, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProposalSelection {
    /// Other unfinished proposals the heuristic passed over.
    pub other_candidates: Vec<ProposalSummary>,
    /// Whether the chosen proposal names a different pull request than the branch's.
    pub pull_request_mismatch: bool,
}

impl Proposal {
    pub fn summary(&self) -> ProposalSummary {
        ProposalSummary {
            id: self.id.clone(),
            title: self.title.clone(),
        }
    }
}
