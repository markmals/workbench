//! `workbench docs search`: finding the package's contract, rules, skills, templates, and reference.

use std::fs;
use std::path::{Path, PathBuf};

use schemars::{JsonSchema, schema_for};
use serde::Serialize;

use crate::commands::cli::{CURSOR, LIMIT};
use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::registry::{
    ArgumentKind, ArgumentSpec, CommandSpec, Effect, Example, Requirement, ValueKind,
};
use crate::response::{RenderText, Response};
use crate::search::{self, Page};

pub static COMMANDS: &[CommandSpec] = &[CommandSpec {
    id: "docs.search",
    path: &["docs", "search"],
    summary: "Search the Workbench package's contract, rules, skills, templates, and CLI reference",
    description: "Searches the Markdown in the project's Workbench package (the `.workbench` link) and \
        returns the best-matching files with a `path` under `.workbench/`, a `title`, and a `snippet` \
        around the best match. Read the file at `path` for detail. Runs offline and never changes \
        anything. Fails with PRECONDITION_FAILED when `.workbench` is missing; `mise install` restores it.",
    arguments: &[
        ArgumentSpec {
            name: "query",
            kind: ArgumentKind::Positional,
            value: ValueKind::String,
            required: true,
            default: None,
            multiple: true,
            summary: "The topic, in plain words; several words need no quotes",
        },
        LIMIT,
        CURSOR,
    ],
    effect: Effect::Reads,
    requires: &[Requirement::Git, Requirement::WorkbenchLink],
    dry_run: false,
    idempotent: true,
    errors: &[
        ErrorCode::InvalidArgument,
        ErrorCode::PreconditionFailed,
        ErrorCode::DependencyMissing,
    ],
    examples: &[Example {
        summary: "Find guidance on writing a proposal",
        args: &["docs", "search", "write a proposal"],
    }],
    synonyms: &[
        "documentation",
        "guide",
        "skill",
        "rule",
        "template",
        "manual",
        "read",
        "learn",
        "contract",
    ],
    output_schema: || schema_for!(Page<DocumentMatch>),
    run: search_documents,
}];

/// One package file that matches the query.
#[derive(Serialize, JsonSchema)]
pub struct DocumentMatch {
    /// The file to read, relative to the project root: `.workbench/skills/<name>/SKILL.md`.
    pub path: String,
    pub title: String,
    /// The line that best matches the query.
    pub snippet: String,
    /// Relevance from 0 to 1.
    pub score: f64,
}

impl RenderText for Page<DocumentMatch> {
    fn render_text(&self) -> String {
        if self.items.is_empty() {
            return "No package document matches.".into();
        }
        let mut text = String::new();
        for item in &self.items {
            text.push_str(&format!(
                "{}  {}\n    {}\n",
                item.path, item.title, item.snippet
            ));
        }
        if self.has_more {
            text.push_str("More matches follow; pass --cursor with nextCursor or raise --limit.\n");
        }
        text
    }
}

fn search_documents(invocation: &Invocation) -> Result<Response, Failure> {
    let project = invocation.project()?;
    let package = project.package_root()?;
    let query = invocation.strings("query").join(" ");
    let terms = search::query_words(&query);

    let mut documents = Vec::new();
    collect_markdown(&package, &mut documents);
    let mut ranked: Vec<DocumentMatch> = documents
        .iter()
        .filter_map(|file| score_document(&package, file, &terms))
        .collect();
    ranked.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
    });

    let limit = invocation.integer("limit").expect("limit has a default");
    let page = search::page(ranked, invocation.string("cursor").as_deref(), limit)?;
    Ok(Response::new(&page))
}

fn collect_markdown(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_markdown(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "md") {
            files.push(path);
        }
    }
}

fn score_document(package: &Path, file: &Path, terms: &[String]) -> Option<DocumentMatch> {
    if terms.is_empty() {
        return None;
    }
    let source = fs::read_to_string(file).ok()?;
    let relative = file
        .strip_prefix(package)
        .ok()?
        .to_string_lossy()
        .replace('\\', "/");
    let (title, description, body) = split_document(&source, &relative);

    let title_words = search::words(&title);
    let path_words = search::words(&relative);
    let description_words = search::words(&description);
    let body_words = search::words(body);

    let total: f64 = terms
        .iter()
        .map(|term| {
            let occurrences = body_words
                .iter()
                .filter(|word| search::word_match(term, word) > 0.0)
                .count();
            let body = if occurrences == 0 {
                0.0
            } else {
                0.3 + 0.03 * occurrences.min(5) as f64
            };
            [
                search::best_match(term, &title_words),
                search::best_match(term, &path_words) * 0.8,
                search::best_match(term, &description_words) * 0.7,
                body,
            ]
            .into_iter()
            .fold(0.0, f64::max)
        })
        .sum();
    let score = total / terms.len() as f64;
    if score <= 0.0 {
        return None;
    }
    Some(DocumentMatch {
        path: format!(".workbench/{relative}"),
        title,
        snippet: snippet(body, terms),
        score: (score * 1000.0).round() / 1000.0,
    })
}

/// The document's title, its frontmatter description, and its body after the frontmatter.
fn split_document<'a>(source: &'a str, relative: &str) -> (String, String, &'a str) {
    let mut title = None;
    let mut description = String::new();
    let mut body = source;
    if let Some(rest) = source.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---")
    {
        for line in rest[..end].lines() {
            if let Some((key, value)) = line.split_once(':') {
                let value = value.trim().trim_matches('"').to_string();
                match key.trim() {
                    "title" | "name" if title.is_none() => title = Some(value),
                    "description" => description = value,
                    _ => {}
                }
            }
        }
        body = rest[end + 4..].trim_start_matches(['-', '\n']);
    }
    let heading = body
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim).map(str::to_string));
    let title = heading.or(title).unwrap_or_else(|| relative.to_string());
    (title, description, body)
}

fn snippet(body: &str, terms: &[String]) -> String {
    let best = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("```"))
        .map(|line| {
            let words = search::words(line);
            let hits = terms
                .iter()
                .filter(|term| search::best_match(term, &words) > 0.0)
                .count();
            (hits, line)
        })
        .filter(|(hits, _)| *hits > 0)
        .max_by_key(|(hits, _)| *hits)
        .map(|(_, line)| line)
        .unwrap_or_default();
    truncate(best, 240)
}

fn truncate(line: &str, limit: usize) -> String {
    if line.chars().count() <= limit {
        return line.to_string();
    }
    let cut: String = line.chars().take(limit).collect();
    format!("{}…", cut.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_come_from_the_first_heading_then_frontmatter() {
        let (title, description, _) = split_document(
            "---\nname: writing-a-proposal\ndescription: Use when\n---\n\n# Writing a Proposal\n",
            "x.md",
        );
        assert_eq!(title, "Writing a Proposal");
        assert_eq!(description, "Use when");
        let (title, _, _) = split_document("---\nname: a-skill\n---\nNo heading\n", "x.md");
        assert_eq!(title, "a-skill");
        let (title, _, _) = split_document("Plain\n", "rules/x.md");
        assert_eq!(title, "rules/x.md");
    }

    #[test]
    fn the_snippet_is_the_line_with_the_most_query_words() {
        let body = "# Heading about claims\nNothing here.\nClaim an item before you start the work.\nClaim it.\n";
        let terms = search::query_words("claim item work");
        assert_eq!(
            snippet(body, &terms),
            "Claim an item before you start the work."
        );
    }
}
