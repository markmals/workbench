//! Gathering the facts from Git, `gh`, and the proposals.

use std::path::Path;
use std::time::Duration;

use super::derive::{CONVENTIONAL_DEFAULT_BRANCHES, default_branch_facts};
use super::facts::Facts;
use super::proposals::{read_proposals, select_proposal, successor};
use super::pull_request::{gh, look_up};
use crate::process::Program;

/// Reads the facts in `root`, giving each `gh` query up to `limit`.
pub fn gather_facts(root: &Path, limit: Duration) -> Facts {
    let branch = git(root, &["rev-parse", "--abbrev-ref", "HEAD"]);
    let default = default_branch(root, branch.as_deref(), limit);
    let upstream = git(
        root,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    );
    let unpushed_commits = match upstream {
        Some(_) => git(root, &["rev-list", "--count", "@{upstream}..HEAD"])
            .and_then(|count| count.parse().ok())
            .unwrap_or(0),
        None => 0,
    };
    let pull_request = look_up(root, branch.as_deref(), limit);
    let proposals = read_proposals(root);
    let (proposal, proposal_selection) = select_proposal(&proposals, pull_request.found());
    let proposal_successor = proposal
        .as_ref()
        .filter(|proposal| proposal.status.as_deref() == Some("superseded"))
        .and_then(|proposal| successor(&proposals, proposal));
    Facts {
        branch,
        default_branch: default.default_branch,
        is_default_branch: default.is_default_branch,
        has_upstream: upstream.is_some(),
        unpushed_commits,
        pull_request,
        proposal,
        proposal_successor,
        proposal_selection,
    }
}

fn default_branch(
    root: &Path,
    branch: Option<&str>,
    limit: Duration,
) -> super::derive::DefaultBranch {
    let origin_head = git(root, &["symbolic-ref", "refs/remotes/origin/HEAD"]);
    let gh_default = match origin_head {
        Some(_) => None,
        None => gh(
            root,
            &[
                "repo",
                "view",
                "--json",
                "defaultBranchRef",
                "--jq",
                ".defaultBranchRef.name",
            ],
            limit,
        )
        .ok()
        .filter(|run| run.success())
        .map(|run| run.stdout.trim().to_string())
        .filter(|name| !name.is_empty()),
    };
    let local_default = match (&origin_head, &gh_default) {
        (None, None) => CONVENTIONAL_DEFAULT_BRANCHES.into_iter().find(|candidate| {
            git(
                root,
                &[
                    "rev-parse",
                    "--verify",
                    "--quiet",
                    &format!("refs/heads/{candidate}"),
                ],
            )
            .is_some()
        }),
        _ => None,
    };
    default_branch_facts(
        branch,
        origin_head.as_deref(),
        gh_default.as_deref(),
        local_default,
    )
}

/// What a Git query printed, trimmed, or `None` when it failed or printed nothing.
fn git(root: &Path, args: &[&str]) -> Option<String> {
    Program::new("git", root)
        .args(args.iter().copied())
        .run()
        .ok()
        .filter(|run| run.success())
        .map(|run| run.stdout.trim().to_string())
        .filter(|output| !output.is_empty())
}
