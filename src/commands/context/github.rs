//! The branch and its pull request.

use std::path::Path;
use std::time::Instant;

use serde::Deserialize;

use super::report::PullRequest;
use crate::process::{Program, Run, RunError};

/// The checked-out branch, or `None` when HEAD is detached.
pub fn branch(root: &Path) -> Option<String> {
    let run = Program::new("git", root)
        .args(["symbolic-ref", "--quiet", "--short", "HEAD"])
        .run()
        .ok()?;
    run.success().then(|| run.stdout.trim().to_string())
}

/// The branch's pull request as `gh` lists it. Never fails: what stops the lookup becomes the
/// reason the pull request is unknown.
pub fn pull_request(
    root: &Path,
    branch: Option<&str>,
    has_origin: bool,
    deadline: Instant,
    timeout: u64,
) -> PullRequest {
    let Some(branch) = branch else {
        return unknown("HEAD is detached, so there is no branch to look up");
    };
    if !has_origin {
        return unknown("the checkout has no `origin` remote to look the pull request up on");
    }
    let listing = Program::new("gh", root)
        .args([
            "pr", "list", "--state", "all", "--head", branch, "--limit", "2",
        ])
        .args(["--json", "number,url,title,state,isDraft"])
        .deadline(deadline)
        .run();
    match listing {
        Ok(run) if run.success() => parse(&run.stdout, branch),
        Ok(run) => unknown(&failure_reason(&run)),
        Err(RunError::Missing) => unknown("`gh` is not installed"),
        Err(RunError::TimedOut) => unknown(&format!(
            "`gh` did not answer within {timeout} second{}",
            if timeout == 1 { "" } else { "s" }
        )),
        Err(RunError::Spawn(reason)) => unknown(&format!("`gh` could not start: {reason}")),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Listed {
    number: u64,
    url: String,
    title: String,
    state: String,
    is_draft: bool,
}

fn parse(listing: &str, branch: &str) -> PullRequest {
    let Ok(listed) = serde_json::from_str::<Vec<Listed>>(listing) else {
        return unknown("`gh` returned pull request data workbench cannot read");
    };
    match listed.as_slice() {
        [] => PullRequest::None,
        [only] => PullRequest::Found {
            number: only.number,
            url: only.url.clone(),
            title: only.title.clone(),
            state: only.state.clone(),
            draft: only.is_draft,
        },
        several => unknown(&format!(
            "at least {} pull requests have the head branch `{branch}`; inspect them with `gh pr list --state all --head {branch}`",
            several.len()
        )),
    }
}

/// `gh` exits 4 when it needs authentication.
fn failure_reason(run: &Run) -> String {
    if run.status == Some(4) || run.stderr.contains("gh auth login") {
        return "`gh` is not authenticated; run `gh auth login`".into();
    }
    let detail = run.stderr.lines().next().unwrap_or("no output").trim();
    format!("`gh pr list` failed: {detail}")
}

fn unknown(reason: &str) -> PullRequest {
    PullRequest::Unknown {
        reason: reason.to_string(),
    }
}
