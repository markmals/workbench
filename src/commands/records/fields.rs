//! One record's frontmatter against its kind's schema.

use std::sync::LazyLock;

use regex::Regex;

use super::kind::RecordKind;
use super::validate::{Record, Violation, shown};

static DATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$").expect("valid regex"));

/// The record's frontmatter problems: retired fields, missing or mistyped keys, its status, and
/// the vision's date.
pub(super) fn check(record: &Record) -> Vec<Violation> {
    let mut messages = retired_fields(record);
    messages.extend(key_problems(record));
    messages.extend(status_problem(record));
    messages.extend(date_problem(record));
    messages
        .into_iter()
        .map(|message| record.violation(message))
        .collect()
}

/// Fields earlier versions of the process used, and what replaces them.
fn retired_fields(record: &Record) -> Vec<String> {
    let mut messages = Vec::new();
    if record.kind.is_current_record() {
        if record.has("supersedes") {
            messages.push(
                "remove \"supersedes\"; revise the current record in place or retire it and \
                 repair its links"
                    .to_string(),
            );
        }
        if record.has("established-by") && record.scalar("established-by").is_none() {
            messages.push("key \"established-by\" must be a scalar".to_string());
        }
    }
    if record.kind == RecordKind::Guide && record.has("describes") {
        messages.push(
            "remove \"describes\"; use ordinary links to current requirements instead".to_string(),
        );
    }
    messages
}

fn key_problems(record: &Record) -> Vec<String> {
    let schema = record.kind.schema();
    let missing = schema
        .required
        .iter()
        .filter(|key| !record.has(key))
        .map(|key| format!("missing required key \"{key}\""));
    let not_scalar = schema
        .required
        .iter()
        .filter(|key| !schema.lists.contains(key))
        .filter(|key| record.has(key) && record.scalar(key).is_none())
        .map(|key| format!("key \"{key}\" must be a scalar"));
    let not_list = schema
        .lists
        .iter()
        .chain(schema.optional_lists)
        .filter(|key| record.has(key) && record.list(key).is_none())
        .map(|key| format!("key \"{key}\" must be an inline list"));
    let empty_list = schema
        .non_empty_lists
        .iter()
        .filter(|key| record.list(key).is_some_and(<[String]>::is_empty))
        .map(|key| format!("key \"{key}\" must be a non-empty inline list"));
    missing
        .chain(not_scalar)
        .chain(not_list)
        .chain(empty_list)
        .collect()
}

fn status_problem(record: &Record) -> Option<String> {
    let allowed = record.kind.schema().statuses?;
    let status = record.get("status")?;
    let scalar = status.as_scalar();
    if scalar.is_some_and(|status| allowed.contains(&status)) {
        return None;
    }
    let kind = record.kind.name();
    if record.kind.is_current_record() && scalar == Some("superseded") {
        return Some(format!(
            "status \"superseded\" is obsolete for {kind}; consolidate still-applicable \
             requirements into the current record, then delete the retired record and repair its \
             links"
        ));
    }
    Some(format!("invalid status \"{}\" for {kind}", shown(status)))
}

fn date_problem(record: &Record) -> Option<String> {
    if record.kind != RecordKind::Vision {
        return None;
    }
    let updated = record.get("updated")?;
    (!DATE.is_match(&shown(updated))).then(|| "key \"updated\" must use YYYY-MM-DD".to_string())
}
