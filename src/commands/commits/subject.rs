//! Scoped Commits rules for one commit subject.

use std::sync::LazyLock;

use regex::Regex;
use schemars::JsonSchema;
use serde::Serialize;

/// Git-generated mechanical commits are exempt from the subject format.
static EXEMPT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:merge[: ]|revert[: ]|fixup! |squash! )").expect("a valid pattern")
});

/// Conventional Commits `type(scope):` is a different convention, rejected by name because its
/// failure mode is a subject that looks correct to its author.
static CONVENTIONAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z][A-Za-z0-9-]*\([A-Za-z0-9._/-]+\)!?:\s").expect("a valid pattern")
});

static SCOPED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([^\s:]+):[ \t]+\S").expect("a valid pattern"));

/// The rule a rejected subject breaks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Rule {
    /// A Conventional Commits `type(scope):` subject.
    ConventionalCommit,
    /// No `<scope>: ` prefix.
    Unscoped,
    /// A scope that is not accepted.
    UnknownScope,
}

/// What the rules make of one subject.
#[derive(Debug, PartialEq, Eq)]
pub enum Judgement<'a> {
    Accepted { scope: &'a str },
    Exempt,
    Rejected(Rejection<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Rejection<'a> {
    ConventionalCommit,
    Unscoped,
    UnknownScope(&'a str),
}

/// The first line that is neither blank nor a Git comment. A message template holding nothing
/// else has no subject to check.
pub fn subject_of(message: &str) -> Option<&str> {
    message
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
}

fn scope_of(subject: &str) -> Option<&str> {
    SCOPED
        .captures(subject)
        .and_then(|captures| captures.get(1))
        .map(|scope| scope.as_str())
}

pub fn judge<'a>(subject: &'a str, accepted_scopes: &[String]) -> Judgement<'a> {
    if EXEMPT.is_match(subject) {
        return Judgement::Exempt;
    }
    if CONVENTIONAL.is_match(subject) {
        return Judgement::Rejected(Rejection::ConventionalCommit);
    }
    match scope_of(subject) {
        None => Judgement::Rejected(Rejection::Unscoped),
        Some(scope) if accepted_scopes.iter().any(|accepted| accepted == scope) => {
            Judgement::Accepted { scope }
        }
        Some(scope) => Judgement::Rejected(Rejection::UnknownScope(scope)),
    }
}

impl Rejection<'_> {
    pub fn rule(&self) -> Rule {
        match self {
            Rejection::ConventionalCommit => Rule::ConventionalCommit,
            Rejection::Unscoped => Rule::Unscoped,
            Rejection::UnknownScope(_) => Rule::UnknownScope,
        }
    }

    /// The scope the subject names, when it is well formed.
    pub fn scope(&self) -> Option<&str> {
        match self {
            Rejection::UnknownScope(scope) => Some(scope),
            _ => None,
        }
    }

    /// Why the subject was rejected; an unknown scope lists every accepted one.
    pub fn explain(&self, accepted_scopes: &[String]) -> String {
        match self {
            Rejection::ConventionalCommit => "Conventional Commits 'type(scope):' subjects are \
                not allowed; use Scoped Commits '<scope>: <imperative description>'."
                .to_string(),
            Rejection::Unscoped => {
                "subject must be a Scoped Commit: '<scope>: <imperative description>'.".to_string()
            }
            Rejection::UnknownScope(scope) => format!(
                "scope '{scope}' is not allowed. Accepted scopes: {}.",
                accepted_scopes.join(", ")
            ),
        }
    }
}
