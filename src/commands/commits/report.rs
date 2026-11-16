//! The result of `commits check`: each subject's verdict and the violations among them.

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::json;

use super::subject::{self, Judgement, Rule};
use crate::failure::{ErrorCode, Failure};
use crate::response::RenderText;

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommitsCheck {
    /// The project whose scopes applied: the Git top level of the current directory.
    pub project: String,
    /// The commit-message file checked, as given; absent when a range was checked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_file: Option<String>,
    /// The range checked, as given; absent when a message file was checked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<RangeArguments>,
    /// Every subject checked, oldest commit first.
    pub commits: Vec<CheckedSubject>,
    /// The subjects that break Scoped Commits, in the same order.
    pub violations: Vec<Violation>,
    /// Every scope a subject may use: the built-in process scopes, then those in
    /// `.agents/commit-scopes`, then `<NNNN>-<slug>` for each `proposals/<NNNN>-<slug>.md`.
    pub accepted_scopes: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct RangeArguments {
    pub base: String,
    pub head: String,
}

#[derive(Serialize, JsonSchema)]
pub struct CheckedSubject {
    /// The full commit hash; absent for a message file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Absent when the message holds only blank lines and `#` comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The scope a well-formed subject names; absent otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub verdict: Verdict,
}

/// `accepted`; `exempt` for merge, revert, `fixup!`, and `squash!` subjects; `rejected`; or
/// `noSubject` for a message with nothing to check.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Verdict {
    Accepted,
    Exempt,
    Rejected,
    NoSubject,
}

#[derive(Serialize, JsonSchema)]
pub struct Violation {
    /// The full commit hash; absent for a message file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    pub subject: String,
    /// `conventionalCommit`, `unscoped`, or `unknownScope`.
    pub rule: Rule,
    /// What is wrong; an unknown scope's message lists every accepted scope.
    pub message: String,
}

impl CommitsCheck {
    /// Judges each `(commit, subject)` pair against `accepted_scopes`.
    pub fn judge(
        project: String,
        subjects: Vec<(Option<String>, Option<String>)>,
        accepted_scopes: Vec<String>,
    ) -> CommitsCheck {
        let mut commits = Vec::new();
        let mut violations = Vec::new();
        for (commit, subject) in subjects {
            let (checked, violation) = judge_one(commit, subject, &accepted_scopes);
            commits.push(checked);
            violations.extend(violation);
        }
        CommitsCheck {
            project,
            message_file: None,
            range: None,
            commits,
            violations,
            accepted_scopes,
        }
    }

    /// CHECK_FAILED when any subject breaks Scoped Commits.
    pub fn failure(&self) -> Option<Failure> {
        let message = match self.violations.as_slice() {
            [] => return None,
            [only] => format!("'{}': {}", only.subject, only.message),
            many => format!(
                "{} of {} commit subjects break Scoped Commits; `violations` lists each. \
                 Accepted scopes: {}.",
                many.len(),
                self.commits.len(),
                self.accepted_scopes.join(", ")
            ),
        };
        Some(
            Failure::new(ErrorCode::CheckFailed, message)
                .with_details(json!({
                    "violations": self.violations.len(),
                    "acceptedScopes": self.accepted_scopes,
                }))
                .suggest("Write each subject as '<scope>: <imperative description>' with an accepted scope")
                .suggest("A part of this repository the process scopes do not name gets a line in .agents/commit-scopes"),
        )
    }
}

fn judge_one(
    commit: Option<String>,
    subject: Option<String>,
    accepted_scopes: &[String],
) -> (CheckedSubject, Option<Violation>) {
    let Some(subject) = subject else {
        let checked = CheckedSubject {
            commit,
            subject: None,
            scope: None,
            verdict: Verdict::NoSubject,
        };
        return (checked, None);
    };
    let (verdict, scope, violation) = match subject::judge(&subject, accepted_scopes) {
        Judgement::Accepted { scope } => (Verdict::Accepted, Some(scope.to_string()), None),
        Judgement::Exempt => (Verdict::Exempt, None, None),
        Judgement::Rejected(rejection) => {
            let violation = Violation {
                commit: commit.clone(),
                subject: subject.clone(),
                rule: rejection.rule(),
                message: rejection.explain(accepted_scopes),
            };
            let scope = rejection.scope().map(str::to_string);
            (Verdict::Rejected, scope, Some(violation))
        }
    };
    let checked = CheckedSubject {
        commit,
        subject: Some(subject),
        scope,
        verdict,
    };
    (checked, violation)
}

impl RenderText for CommitsCheck {
    fn render_text(&self) -> String {
        let mut text = String::new();
        // Violations follow the rejected subjects' order, so each pairs with the next one.
        let mut violations = self.violations.iter();
        for checked in &self.commits {
            let verdict = match checked.verdict {
                Verdict::Accepted => "accepted",
                Verdict::Exempt => "exempt",
                Verdict::Rejected => "rejected",
                Verdict::NoSubject => "empty",
            };
            let subject = checked
                .subject
                .as_deref()
                .unwrap_or("(the message has no subject to check)");
            let line = match checked.commit.as_deref() {
                Some(commit) => format!("{verdict:<9} {} {subject}\n", short(commit)),
                None => format!("{verdict:<9} {subject}\n"),
            };
            text.push_str(&line);
            if checked.verdict == Verdict::Rejected
                && let Some(violation) = violations.next()
            {
                text.push_str(&format!("          {}\n", violation.message));
            }
        }
        text.push_str(&match self.violations.len() {
            0 => format!(
                "\nAll {} subjects follow Scoped Commits.",
                self.commits.len()
            ),
            count => format!(
                "\n{count} of {} subjects break Scoped Commits.",
                self.commits.len()
            ),
        });
        text
    }
}

fn short(commit: &str) -> &str {
    &commit[..commit.len().min(7)]
}
