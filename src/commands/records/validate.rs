//! Validating records against each other and the rules for their kind.

use schemars::JsonSchema;
use serde::Serialize;

use super::kind::RecordKind;
use super::{fields, identity::Identities, links};
use crate::frontmatter::{Frontmatter, FrontmatterValue, without_code};

/// One record file as the validator reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct Artifact {
    /// Relative to the project root, with `/` separators: `proposals/0001-records.md`.
    pub path: String,
    pub frontmatter: Frontmatter,
    /// The whole file, frontmatter included.
    pub content: String,
}

/// One problem in one record file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Violation {
    /// The file, relative to the project root.
    pub path: String,
    pub message: String,
}

/// Every violation among `artifacts`: each record's frontmatter against its kind, IDs against
/// filenames and each other, references, supersession, and clarification markers. Artifacts at
/// paths where no kind of record lives are ignored. Violations come record by record, in path
/// order: shape and identity first, then references and markers, then supersession cycles.
pub fn validate(artifacts: &[Artifact]) -> Vec<Violation> {
    let mut records: Vec<Record> = artifacts.iter().filter_map(Record::classify).collect();
    records.sort_by(|left, right| left.artifact.path.cmp(&right.artifact.path));

    let mut violations = Vec::new();
    let mut identities = Identities::default();
    for record in &records {
        violations.extend(fields::check(record));
        violations.extend(identities.declare(*record));
    }
    for record in &records {
        violations.extend(links::check_references(record, &identities));
        violations.extend(unresolved_clarification(record));
    }
    violations.extend(links::supersession_cycles(&identities));
    violations
}

/// An artifact at a path where a kind of record lives.
#[derive(Clone, Copy)]
pub(super) struct Record<'a> {
    pub artifact: &'a Artifact,
    pub kind: RecordKind,
}

impl<'a> Record<'a> {
    fn classify(artifact: &'a Artifact) -> Option<Record<'a>> {
        RecordKind::of_path(&artifact.path).map(|kind| Record { artifact, kind })
    }

    pub fn get(&self, key: &str) -> Option<&'a FrontmatterValue> {
        self.artifact.frontmatter.get(key)
    }

    pub fn has(&self, key: &str) -> bool {
        self.artifact.frontmatter.contains_key(key)
    }

    pub fn scalar(&self, key: &str) -> Option<&'a str> {
        self.get(key).and_then(FrontmatterValue::as_scalar)
    }

    pub fn list(&self, key: &str) -> Option<&'a [String]> {
        self.get(key).and_then(FrontmatterValue::as_list)
    }

    pub fn violation(&self, message: impl Into<String>) -> Violation {
        Violation {
            path: self.artifact.path.clone(),
            message: message.into(),
        }
    }
}

/// A value as a message quotes it: a list's items joined by commas.
pub(super) fn shown(value: &FrontmatterValue) -> String {
    match value {
        FrontmatterValue::Scalar(value) => value.clone(),
        FrontmatterValue::List(values) => values.join(","),
    }
}

const CLARIFICATION_MARKER: &str = "[NEEDS CLARIFICATION:";

/// Proposals in these statuses have a finished design, so an open question is a defect.
const FINAL_PROPOSAL_STATUSES: [&str; 5] = [
    "awaiting-implementation",
    "active-review",
    "accepted",
    "implemented",
    "superseded",
];

/// Only prose counts: a proposal that documents the marker writes it in code.
fn unresolved_clarification(record: &Record) -> Option<Violation> {
    if record.kind != RecordKind::Proposal {
        return None;
    }
    let status = record.scalar("status")?;
    let unresolved = FINAL_PROPOSAL_STATUSES.contains(&status)
        && without_code(&record.artifact.content).contains(CLARIFICATION_MARKER);
    unresolved.then(|| {
        record.violation(format!(
            "proposal status \"{status}\" does not allow a [NEEDS CLARIFICATION:] marker"
        ))
    })
}
