//! References between records: `supersedes` and `established-by`.

use std::collections::HashMap;

use super::identity::Identities;
use super::kind::RecordKind;
use super::validate::{Record, Violation, shown};

/// Reports each of `record`'s references that does not resolve or names the wrong kind, and each
/// proposal it supersedes that is not marked `superseded` (reported on that proposal).
pub(super) fn check_references(record: &Record, identities: &Identities) -> Vec<Violation> {
    references(record)
        .into_iter()
        .filter_map(|(field, id)| {
            let Some(target) = identities.get(id) else {
                return Some(
                    record.violation(format!("reference \"{id}\" in {field} does not resolve")),
                );
            };
            match field {
                "established-by" if target.kind != RecordKind::Proposal => Some(record.violation(
                    format!("reference \"{id}\" in established-by must name a proposal"),
                )),
                "supersedes" if target.scalar("status") != Some("superseded") => {
                    Some(target.violation(format!(
                        "record superseded by {} must have status \"superseded\"",
                        superseding_name(record)
                    )))
                }
                _ => None,
            }
        })
        .collect()
}

/// Each reference `record` makes, as its field and the ID it names.
fn references<'a>(record: &Record<'a>) -> Vec<(&'static str, &'a str)> {
    let mut references = Vec::new();
    if record.kind == RecordKind::Proposal {
        for id in record.list("supersedes").unwrap_or_default() {
            references.push(("supersedes", id.as_str()));
        }
    }
    let carries_provenance = matches!(
        record.kind,
        RecordKind::Proposal | RecordKind::Policy | RecordKind::Decision
    );
    if carries_provenance && let Some(id) = record.scalar("established-by") {
        references.push(("established-by", id));
    }
    references
}

/// The superseding record's ID, or its path when it declares none.
fn superseding_name(record: &Record) -> String {
    record
        .get("id")
        .map(shown)
        .unwrap_or_else(|| record.artifact.path.clone())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visit {
    InProgress,
    Done,
}

/// Reports each cycle among proposals' `supersedes` references, on the proposal where the
/// depth-first search closes it.
pub(super) fn supersession_cycles(identities: &Identities) -> Vec<Violation> {
    let graph: Vec<(&str, Vec<&str>)> = identities
        .iter()
        .filter(|(_, record)| record.kind == RecordKind::Proposal)
        .map(|(id, record)| {
            let targets = record.list("supersedes").unwrap_or_default();
            let resolved = targets
                .iter()
                .map(String::as_str)
                .filter(|target| identities.get(target).is_some())
                .collect();
            (id, resolved)
        })
        .collect();
    let mut search = CycleSearch {
        edges: graph.iter().map(|(id, targets)| (*id, targets)).collect(),
        identities,
        visits: HashMap::new(),
        found: Vec::new(),
    };
    for (id, _) in &graph {
        if !search.visits.contains_key(id) {
            search.visit(id, &mut vec![id]);
        }
    }
    search.found
}

struct CycleSearch<'g, 'a> {
    edges: HashMap<&'a str, &'g Vec<&'a str>>,
    identities: &'g Identities<'a>,
    visits: HashMap<&'a str, Visit>,
    found: Vec<Violation>,
}

impl<'g, 'a> CycleSearch<'g, 'a> {
    /// `trail` is the path from the search's root to `id`, both included.
    fn visit(&mut self, id: &'a str, trail: &mut Vec<&'a str>) {
        let targets: &'g [&'a str] = match self.edges.get(id).copied() {
            Some(targets) => targets,
            None => &[],
        };
        self.visits.insert(id, Visit::InProgress);
        for &target in targets {
            match self.visits.get(target).copied() {
                Some(Visit::InProgress) => self.report(target, trail),
                Some(Visit::Done) => {}
                None => {
                    trail.push(target);
                    self.visit(target, trail);
                    trail.pop();
                }
            }
        }
        self.visits.insert(id, Visit::Done);
    }

    fn report(&mut self, target: &'a str, trail: &[&'a str]) {
        let start = trail
            .iter()
            .position(|id| *id == target)
            .expect("a record still being visited is on the trail");
        let mut cycle = trail[start..].to_vec();
        cycle.push(target);
        let owner = self
            .identities
            .get(target)
            .expect("the graph holds only declared IDs");
        self.found
            .push(owner.violation(format!("supersession cycle: {}", cycle.join(" -> "))));
    }
}
