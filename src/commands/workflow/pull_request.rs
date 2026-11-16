//! The branch's pull request, through `gh`. When `gh` cannot tell, the pull request is unknown
//! and the reason says why; that is never reported as "no pull request".

use std::path::Path;
use std::time::Duration;

use serde_json::Value;

use super::facts::{PullRequest, PullRequestLookup};
use crate::process::{Program, Run, RunError};

/// A `gh` query, limited to `limit`, or why it gave no answer.
pub fn gh(root: &Path, args: &[&str], limit: Duration) -> Result<Run, String> {
    Program::new("gh", root)
        .args(args.iter().copied())
        .timeout(limit)
        .run()
        .map_err(|error| match error {
            RunError::TimedOut => format!("gh did not answer within {} seconds", limit.as_secs()),
            RunError::Missing | RunError::Spawn(_) => "gh is unavailable".to_string(),
        })
}

pub fn look_up(root: &Path, branch: Option<&str>, limit: Duration) -> PullRequestLookup {
    let unknown = |reason: &str| PullRequestLookup::Unknown {
        reason: reason.to_string(),
    };
    match gh(root, &["--version"], limit) {
        Ok(run) if run.success() => {}
        Ok(_) => return unknown("gh is unavailable"),
        Err(reason) => return unknown(&reason),
    }
    match gh(root, &["auth", "status"], limit) {
        Ok(run) if run.success() => {}
        Ok(_) => return unknown("gh is not authenticated"),
        Err(reason) => return unknown(&reason),
    }
    let Some(branch) = branch else {
        return unknown("branch is unknown");
    };
    let fields = "number,isDraft,url,title,state,baseRefName";
    let args = [
        "pr", "list", "--state", "all", "--head", branch, "--limit", "2", "--json", fields,
    ];
    match gh(root, &args, limit) {
        Ok(run) if run.success() => parse_pull_requests(&run.stdout, branch),
        Ok(_) => unknown("gh could not list pull requests"),
        Err(reason) => unknown(&reason),
    }
}

/// The one pull request `gh pr list` found for `branch`, none, or unknown when the branch name
/// was reused or `gh` printed something else.
fn parse_pull_requests(source: &str, branch: &str) -> PullRequestLookup {
    let invalid = || PullRequestLookup::Unknown {
        reason: "gh returned invalid pull request data".to_string(),
    };
    let Ok(Value::Array(pull_requests)) = serde_json::from_str::<Value>(source.trim()) else {
        return invalid();
    };
    match pull_requests.as_slice() {
        [] => PullRequestLookup::None,
        [single] => serde_json::from_value::<PullRequest>(single.clone())
            .map(PullRequestLookup::Found)
            .unwrap_or_else(|_| invalid()),
        _ => PullRequestLookup::Unknown {
            reason: format!(
                "Multiple pull requests found for branch {branch}; inspect their proposal links"
            ),
        },
    }
}
