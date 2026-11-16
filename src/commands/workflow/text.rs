//! The status report as text, for `mise run status` and `--output text`.

use super::derive::{DerivedStatus, Notice};
use super::facts::{Facts, PullRequestLookup, PullRequestState};

/// The phase report: the proposal, pull request, and branch, then the phase, next action, and
/// notices.
pub fn status_text(facts: &Facts, derived: &DerivedStatus) -> String {
    let mut lines = vec![
        proposal_line(facts),
        pull_request_line(&facts.pull_request),
        format!(
            "branch {}",
            facts.branch.as_deref().unwrap_or("unavailable")
        ),
        String::new(),
        format!("Phase {} — {}", derived.phase, derived.title),
        format!("  {}", derived.situation),
        String::new(),
        format!("Next ({}): {}", derived.actor.as_str(), derived.next),
    ];
    if let Some(skill) = &derived.skill {
        lines.push(format!("              Skill: {skill}"));
    }
    lines.extend(derived.notices.iter().map(notice_line));
    lines.join("\n")
}

fn proposal_line(facts: &Facts) -> String {
    let Some(proposal) = &facts.proposal else {
        return "proposal — none".to_string();
    };
    let field = |value: &Option<String>| value.as_deref().unwrap_or("?").to_string();
    format!(
        "{}  {}  ·  {}",
        field(&proposal.id),
        field(&proposal.title),
        field(&proposal.status)
    )
}

fn pull_request_line(lookup: &PullRequestLookup) -> String {
    match lookup {
        PullRequestLookup::Found(pull_request) => {
            let state = match pull_request.state {
                PullRequestState::Open if pull_request.is_draft => "draft",
                PullRequestState::Open => "open",
                PullRequestState::Closed => "closed",
                PullRequestState::Merged => "merged",
            };
            format!(
                "PR #{} ({state})  {}",
                pull_request.number, pull_request.url
            )
        }
        PullRequestLookup::None => "PR — none".to_string(),
        PullRequestLookup::Unknown { reason } => format!("PR — unknown ({reason})"),
    }
}

fn notice_line(notice: &Notice) -> String {
    match notice {
        Notice::Clarifications { count } => format!("Clarifications: {count} unresolved"),
        Notice::UnpushedCommits { count } => format!("Unpushed commits: {count}"),
        Notice::DraftActiveReview => {
            "Warning: proposal is active-review, but the pull request remains draft.".to_string()
        }
        Notice::AmbiguousProposalSelection {
            selected,
            other_candidates,
            pull_request_mismatch,
            proposal_pull_request,
            pull_request_number,
        } => {
            let selected = selected.as_deref().unwrap_or("?");
            let candidates: Vec<String> = other_candidates
                .iter()
                .map(|candidate| {
                    format!(
                        "{} ({})",
                        candidate.id.as_deref().unwrap_or("?"),
                        candidate.title.as_deref().unwrap_or("?")
                    )
                })
                .collect();
            let candidates = candidates.join(", ");
            if !pull_request_mismatch {
                return format!(
                    "Warning: heuristic selected {selected}. Other candidates: {candidates}."
                );
            }
            let mismatch = format!(
                "selected {selected} references {}, not current PR #{}",
                proposal_pull_request.as_deref().unwrap_or("?"),
                pull_request_number
                    .map(|number| number.to_string())
                    .unwrap_or_else(|| "?".into())
            );
            if candidates.is_empty() {
                format!("Warning: {mismatch}.")
            } else {
                format!("Warning: {mismatch}. Other candidates: {candidates}.")
            }
        }
    }
}
