//! The receipt `project init` returns, and its text rendering.

use schemars::JsonSchema;
use serde::Serialize;

use crate::commands::project::vendored::{TemplateMatch, VendoredFile};
use crate::response::RenderText;

/// What `project init` did, or for a dry run, what it would do.
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub status: Status,
    /// The project root the paths are relative to.
    pub project: String,
    /// The documentation site host requested with `--site`, or null.
    pub site: Option<String>,
    /// Files and links created, relative to the project root.
    pub created: Vec<String>,
    /// Existing files given missing entries.
    pub added_to: Vec<Addition>,
    /// Items already in place, left as they were.
    pub unchanged: Vec<String>,
    /// Existing content left as it was that a person needs to complete by hand.
    pub attention: Vec<Notice>,
    /// Files that could not be written, with the reason. Empty unless the command failed.
    pub failed: Vec<Notice>,
    /// Workbench files the project still vendors.
    pub vendored: Vec<VendoredFile>,
    /// Project-owned pieces Workbench relies on but does not create.
    pub missing: Vec<MissingPiece>,
    /// What a dry run cannot predict; empty for a real run.
    pub cannot_predict: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    /// It created or added to files.
    Initialized,
    /// Everything was already in place; nothing was written.
    Unchanged,
    /// `--dry-run`: nothing was written.
    DryRun,
}

/// An existing file and the entries added to it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Addition {
    pub path: String,
    /// The keys or lines added, such as `tools.hk`, `tasks."beads:sync"`, or `.workbench`.
    pub entries: Vec<String>,
}

/// Something about one file a person needs to know or do.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Notice {
    pub path: String,
    pub message: String,
}

/// A project-owned piece Workbench relies on but does not create.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct MissingPiece {
    /// `setup`, `check`, or `fmt` for a mise task; `release` for the Knope release flow.
    pub piece: String,
    pub message: String,
}

impl RenderText for Receipt {
    fn render_text(&self) -> String {
        let mut text = match self.status {
            Status::Initialized => format!("Initialized Workbench in {}\n", self.project),
            Status::Unchanged => format!("Workbench is already set up in {}\n", self.project),
            Status::DryRun => format!("Dry run in {}: nothing was written\n", self.project),
        };
        section(&mut text, "Created", self.created.iter().cloned());
        section(
            &mut text,
            "Added to",
            self.added_to
                .iter()
                .map(|addition| format!("{}: {}", addition.path, addition.entries.join(", "))),
        );
        section(&mut text, "Unchanged", self.unchanged.iter().cloned());
        section(&mut text, "Needs attention", notices(&self.attention));
        section(&mut text, "Could not write", notices(&self.failed));
        section(
            &mut text,
            "Vendored Workbench files",
            self.vendored.iter().map(vendored_line),
        );
        section(
            &mut text,
            "Project-owned pieces to add",
            self.missing
                .iter()
                .map(|piece| format!("{}: {}", piece.piece, piece.message)),
        );
        section(
            &mut text,
            "Cannot predict",
            self.cannot_predict.iter().cloned(),
        );
        text
    }
}

fn notices(notices: &[Notice]) -> impl Iterator<Item = String> + '_ {
    notices
        .iter()
        .map(|notice| format!("{}: {}", notice.path, notice.message))
}

fn vendored_line(file: &VendoredFile) -> String {
    let template = match file.template {
        Some(TemplateMatch::Identical) => " (identical)",
        Some(TemplateMatch::Different) => " (different)",
        None => "",
    };
    format!("{}{template}: {}", file.path, file.message)
}

fn section(text: &mut String, heading: &str, lines: impl Iterator<Item = String>) {
    let lines: Vec<String> = lines.collect();
    if lines.is_empty() {
        return;
    }
    text.push_str(&format!("\n{heading}:\n"));
    for line in lines {
        text.push_str(&format!("  {line}\n"));
    }
}
