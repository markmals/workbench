//! Record IDs: each matches its filename, and the first record to declare one owns it.

use std::collections::HashMap;

use super::kind::RecordKind;
use super::validate::{Record, Violation};

/// The records that declared an ID, in declaration order.
#[derive(Default)]
pub(super) struct Identities<'a> {
    declared: Vec<(&'a str, Record<'a>)>,
    by_id: HashMap<&'a str, usize>,
}

impl<'a> Identities<'a> {
    /// Takes `record`'s ID unless an earlier record declared it, and reports a filename that
    /// breaks its kind's naming rule, an ID that does not match the filename, or a duplicate.
    /// The vision, and a record whose ID is missing or not a scalar, declare nothing.
    pub fn declare(&mut self, record: Record<'a>) -> Vec<Violation> {
        if record.kind == RecordKind::Vision {
            return Vec::new();
        }
        let Some(id) = record.scalar("id") else {
            return Vec::new();
        };

        let mut violations = Vec::new();
        match record.kind.expected_id(&record.artifact.path) {
            None => violations.push(record.violation(record.kind.filename_rule())),
            Some(expected) if expected != id => violations.push(record.violation(format!(
                "ID \"{id}\" must be \"{expected}\" for this filename"
            ))),
            Some(_) => {}
        }
        match self.get(id) {
            Some(first) => violations.push(record.violation(format!(
                "duplicate ID \"{id}\"; first declared in {}",
                first.artifact.path
            ))),
            None => {
                self.by_id.insert(id, self.declared.len());
                self.declared.push((id, record));
            }
        }
        violations
    }

    pub fn get(&self, id: &str) -> Option<&Record<'a>> {
        self.by_id.get(id).map(|&index| &self.declared[index].1)
    }

    /// Each declared ID with the record that owns it, in declaration order.
    pub fn iter(&self) -> impl Iterator<Item = (&'a str, &Record<'a>)> {
        self.declared.iter().map(|(id, record)| (*id, record))
    }
}
