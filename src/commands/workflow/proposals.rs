//! Reading the project's proposals and choosing the one the work is about.

use std::cmp::Reverse;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use super::facts::{Proposal, ProposalSelection, ProposalSummary, PullRequest};
use crate::frontmatter::{self, Frontmatter};

const CLARIFICATION_MARKER: &str = "[NEEDS CLARIFICATION:";
const UNFINISHED_STATUSES: [&str; 5] = [
    "draft",
    "awaiting-implementation",
    "active-review",
    "returned-for-revisions",
    "accepted",
];

static TRAILING_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d+)$").expect("valid regex"));

/// Every proposal in `proposals/` whose frontmatter parses, in file-name order. The index
/// README is not a proposal.
pub fn read_proposals(root: &Path) -> Vec<Proposal> {
    let Ok(entries) = fs::read_dir(root.join("proposals")) else {
        return Vec::new();
    };
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|extension| extension == "md")
                && path.file_name().is_some_and(|name| name != "README.md")
        })
        .collect();
    files.sort();
    files
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .filter_map(|source| parse_proposal(&source))
        .collect()
}

fn parse_proposal(source: &str) -> Option<Proposal> {
    let frontmatter = frontmatter::parse(source).ok()?;
    let scalar = |key: &str| scalar(&frontmatter, key);
    Some(Proposal {
        id: scalar("id"),
        title: scalar("title"),
        status: scalar("status"),
        pull_request: scalar("pull-request"),
        supersedes: frontmatter
            .get("supersedes")
            .and_then(|value| value.as_list())
            .map(<[String]>::to_vec),
        clarification_count: frontmatter::without_code(source)
            .matches(CLARIFICATION_MARKER)
            .count(),
    })
}

fn scalar(frontmatter: &Frontmatter, key: &str) -> Option<String> {
    frontmatter
        .get(key)
        .and_then(|value| value.as_scalar())
        .map(str::to_string)
}

/// The proposal the work is about, and how certain that choice is: the one naming the branch's
/// pull request, else the highest-numbered unfinished one.
pub fn select_proposal(
    proposals: &[Proposal],
    pull_request: Option<&PullRequest>,
) -> (Option<Proposal>, ProposalSelection) {
    let ordered = newest_first(proposals);
    let matched = ordered
        .iter()
        .find(|proposal| names_pull_request(proposal, pull_request));
    let unfinished: Vec<&Proposal> = ordered
        .iter()
        .copied()
        .filter(|proposal| {
            proposal
                .status
                .as_deref()
                .is_some_and(|status| UNFINISHED_STATUSES.contains(&status))
        })
        .collect();
    let chosen = matched.copied().or(unfinished.first().copied());
    let other_candidates = match matched {
        Some(_) => Vec::new(),
        None => unfinished
            .iter()
            .skip(1)
            .map(|proposal| proposal.summary())
            .collect(),
    };
    let pull_request_mismatch = match (chosen, pull_request) {
        (Some(proposal), Some(_)) => {
            proposal.pull_request.is_some() && !names_pull_request(proposal, pull_request)
        }
        _ => false,
    };
    let selection = ProposalSelection {
        other_candidates,
        pull_request_mismatch,
    };
    (chosen.cloned(), selection)
}

/// The newest proposal that names `proposal` in its `supersedes`.
pub fn successor(proposals: &[Proposal], proposal: &Proposal) -> Option<ProposalSummary> {
    let id = proposal.id.as_deref()?;
    newest_first(proposals)
        .into_iter()
        .find(|candidate| {
            candidate
                .supersedes
                .as_ref()
                .is_some_and(|supersedes| supersedes.iter().any(|entry| entry == id))
        })
        .map(Proposal::summary)
}

fn newest_first(proposals: &[Proposal]) -> Vec<&Proposal> {
    let mut ordered: Vec<&Proposal> = proposals.iter().collect();
    ordered.sort_by_key(|proposal| Reverse(number(proposal)));
    ordered
}

fn number(proposal: &Proposal) -> i64 {
    proposal
        .id
        .as_deref()
        .and_then(|id| TRAILING_NUMBER.captures(id))
        .and_then(|captures| captures[1].parse().ok())
        .unwrap_or(-1)
}

fn names_pull_request(proposal: &Proposal, pull_request: Option<&PullRequest>) -> bool {
    let (Some(named), Some(pull_request)) = (proposal.pull_request.as_deref(), pull_request) else {
        return false;
    };
    let number = pull_request.number.to_string();
    named == number || named.ends_with(&format!("/pull/{number}"))
}
