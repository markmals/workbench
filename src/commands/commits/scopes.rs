//! The scopes a project's commit subjects may use.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::json;

use crate::failure::Failure;

/// The process scopes every Workbench project accepts without listing them.
pub const BUILT_IN: [&str; 13] = [
    "agents",
    "ci",
    "decision",
    "deps",
    "docs",
    "guides",
    "policy",
    "release",
    "rules",
    "skills",
    "templates",
    "treewide",
    "vision",
];

/// The project's own scopes: one per line, blank lines and `#` comments ignored.
pub const SCOPES_FILE: &str = ".agents/commit-scopes";

/// `proposals/<NNNN>-<slug>.md` is the scope `<NNNN>-<slug>` for the work implementing it.
static PROPOSAL_FILE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([0-9]{4}-[a-z0-9-]+)\.md$").expect("a valid pattern"));

/// Where an accepted scope comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    /// One of Workbench's process scopes, accepted in every project.
    BuiltIn,
    /// Listed in the project's `.agents/commit-scopes`.
    ScopesFile,
    /// The `<NNNN>-<slug>` of a proposal in `proposals/`.
    Proposal,
}

/// One accepted scope.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Scope {
    /// The scope, as written before the colon in a subject.
    pub scope: String,
    pub source: Source,
    /// For a proposal scope, the proposal file it names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Every accepted scope: the built-in ones, then the scopes file's, then one per proposal.
pub fn accepted(root: &Path) -> Result<Vec<String>, Failure> {
    Ok(accepted_with_sources(root)?
        .into_iter()
        .map(|scope| scope.scope)
        .collect())
}

/// Every accepted scope with where it comes from, in the order `accepted` gives. A scope the
/// scopes file repeats keeps its first source.
pub fn accepted_with_sources(root: &Path) -> Result<Vec<Scope>, Failure> {
    let built_in = BUILT_IN
        .iter()
        .map(|scope| (scope.to_string(), Source::BuiltIn, None));
    let listed = match read_optional(&root.join(SCOPES_FILE))? {
        Some(source) => parse(&source),
        None => Vec::new(),
    };
    let listed = listed
        .into_iter()
        .map(|scope| (scope, Source::ScopesFile, None));
    let proposals = proposal_scopes(root)?.into_iter().map(|scope| {
        (
            scope.clone(),
            Source::Proposal,
            Some(format!("proposals/{scope}.md")),
        )
    });

    let mut scopes: Vec<Scope> = Vec::new();
    for (scope, source, path) in built_in.chain(listed).chain(proposals) {
        if !scopes.iter().any(|known| known.scope == scope) {
            scopes.push(Scope {
                scope,
                source,
                path,
            });
        }
    }
    Ok(scopes)
}

pub fn parse(source: &str) -> Vec<String> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn read_optional(path: &Path) -> Result<Option<String>, Failure> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(unreadable(path, &error)),
    }
}

fn proposal_scopes(root: &Path) -> Result<Vec<String>, Failure> {
    let directory = root.join("proposals");
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(unreadable(&directory, &error)),
    };
    let mut scopes: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            PROPOSAL_FILE
                .captures(&name)
                .map(|captures| captures[1].to_string())
        })
        .collect();
    scopes.sort();
    Ok(scopes)
}

fn unreadable(path: &Path, error: &std::io::Error) -> Failure {
    Failure::precondition(format!(
        "cannot read {} for the accepted commit scopes: {error}",
        path.display()
    ))
    .with_details(json!({ "path": path.display().to_string() }))
    .suggest(format!("Make {} readable", path.display()))
}
