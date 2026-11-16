//! The scopes a project's commit subjects may use.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde_json::json;

use crate::failure::Failure;

/// The process scopes every Workbench project accepts without listing them.
pub const BUILT_IN: [&str; 14] = [
    "agents",
    "ci",
    "decision",
    "deps",
    "docs",
    "guides",
    "policy",
    "proposal",
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

/// Every accepted scope: the built-in ones, then the scopes file's, then one per proposal.
pub fn accepted(root: &Path) -> Result<Vec<String>, Failure> {
    let mut scopes: Vec<String> = BUILT_IN.iter().map(ToString::to_string).collect();
    let listed = match read_optional(&root.join(SCOPES_FILE))? {
        Some(source) => parse(&source),
        None => Vec::new(),
    };
    for scope in listed.into_iter().chain(proposal_scopes(root)?) {
        if !scopes.contains(&scope) {
            scopes.push(scope);
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
