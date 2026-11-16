//! Reading Beads items: one item with `bd show`, and differences between two states with
//! `bd diff`.

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::bd::{bd, bd_error, interruption, parse_json};
use crate::process::Run;

static NOT_FOUND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)not found|no issues? found|no issue matching").expect("valid regex")
});

/// The fields of an item the claim and close commands act on.
#[derive(Clone, Debug, Deserialize)]
pub struct Item {
    pub title: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
}

impl Item {
    /// Who holds the item: its assignee, when it has one.
    pub fn holder(&self) -> Option<&str> {
        self.assignee
            .as_deref()
            .filter(|assignee| !assignee.is_empty())
    }

    pub fn is_in_progress(&self) -> bool {
        self.status.as_deref() == Some("in_progress")
    }
}

/// What reading an item found.
pub enum ItemRead {
    Found(Item),
    /// Beads has no item with the id; the reason is Beads' own.
    NotFound(String),
    /// The item could not be read, for a reason other than its absence.
    Unreadable(String),
}

pub fn read_item(root: &Path, id: &str) -> ItemRead {
    let show = match bd(root, &["show", id, "--json"]).run() {
        Ok(run) => run,
        Err(error) => return ItemRead::Unreadable(interruption(&error, "show")),
    };
    if !show.success() {
        let reason = bd_error(&show, "show");
        return if NOT_FOUND.is_match(&reason) {
            ItemRead::NotFound(reason)
        } else {
            ItemRead::Unreadable(reason)
        };
    }
    let items: Option<Vec<Item>> =
        parse_json(&show.stdout).and_then(|value| serde_json::from_value(value).ok());
    match items {
        Some(items) => items
            .into_iter()
            .next()
            .map(ItemRead::Found)
            .unwrap_or_else(|| ItemRead::NotFound(format!("no item {id}"))),
        None => ItemRead::Unreadable(format!(
            "bd show {id} printed output that is not the JSON list it prints for an item"
        )),
    }
}

/// An item that differs between two states of the database.
#[derive(Clone, Debug, PartialEq, Serialize, JsonSchema)]
pub struct DifferingItem {
    pub id: String,
    /// Its title on whichever side has it.
    pub title: Option<String>,
    /// How it differs, as Beads names it: `added`, `removed`, or `modified`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<String>,
}

impl DifferingItem {
    pub fn describe(&self) -> String {
        match &self.title {
            Some(title) => format!("{} ({title})", self.id),
            None => self.id.clone(),
        }
    }
}

/// The items a finished `bd diff --json` lists. Beads prints a sentence instead of JSON when
/// nothing differs, so output that is not a JSON list means no differences.
pub fn differing_items(diff: &Run) -> Vec<DifferingItem> {
    let Some(Value::Array(entries)) = parse_json(&diff.stdout) else {
        return Vec::new();
    };
    entries.iter().filter_map(differing_item).collect()
}

fn differing_item(entry: &Value) -> Option<DifferingItem> {
    let title = ["/NewValue/title", "/OldValue/title"]
        .into_iter()
        .find_map(|pointer| entry.pointer(pointer).and_then(Value::as_str));
    Some(DifferingItem {
        id: entry["IssueID"].as_str()?.to_string(),
        title: title.map(str::to_string),
        change: entry["DiffType"].as_str().map(str::to_string),
    })
}
