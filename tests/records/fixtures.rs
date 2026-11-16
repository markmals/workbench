//! Records to validate without a project on disk.

use workbench::commands::records::validate::{Artifact, Violation, validate};
use workbench::frontmatter::{Frontmatter, FrontmatterValue};

pub fn scalar(value: &str) -> FrontmatterValue {
    FrontmatterValue::Scalar(value.to_string())
}

pub fn list(values: &[&str]) -> FrontmatterValue {
    FrontmatterValue::List(values.iter().map(|value| value.to_string()).collect())
}

pub fn artifact(path: &str, fields: &[(&str, FrontmatterValue)], content: &str) -> Artifact {
    let frontmatter: Frontmatter = fields
        .iter()
        .map(|(key, value)| (key.to_string(), value.clone()))
        .collect();
    Artifact {
        path: path.to_string(),
        frontmatter,
        content: content.to_string(),
    }
}

pub fn set(artifact: &mut Artifact, key: &str, value: FrontmatterValue) {
    artifact.frontmatter.insert(key.to_string(), value);
}

pub fn violation(path: &str, message: &str) -> Violation {
    Violation {
        path: path.to_string(),
        message: message.to_string(),
    }
}

/// The violations among `artifacts` whose message contains `needle`.
pub fn violations_mentioning(artifacts: &[Artifact], needle: &str) -> Vec<Violation> {
    validate(artifacts)
        .into_iter()
        .filter(|violation| violation.message.contains(needle))
        .collect()
}

/// One valid record of every kind; the second proposal supersedes the first.
pub fn valid_artifacts() -> Vec<Artifact> {
    vec![
        artifact(
            "proposals/0001-records.md",
            &[
                ("id", scalar("proposal.0001")),
                ("title", scalar("Record Validation")),
                ("authors", list(&["Ada"])),
                ("status", scalar("superseded")),
                ("pull-request", scalar("42")),
                ("supersedes", list(&[])),
            ],
            "",
        ),
        artifact(
            "proposals/0002-replaces.md",
            &[
                ("id", scalar("proposal.0002")),
                ("title", scalar("Replacement Record")),
                ("authors", list(&["Ada"])),
                ("status", scalar("accepted")),
                ("pull-request", scalar("43")),
                ("supersedes", list(&["proposal.0001"])),
            ],
            "",
        ),
        artifact(
            "policies/0001-records.md",
            &[
                ("id", scalar("policy.0001")),
                ("title", scalar("Record Validation Policy")),
                ("status", scalar("active")),
                ("established-by", scalar("proposal.0002")),
            ],
            "",
        ),
        artifact(
            "decisions/0001-records.md",
            &[
                ("id", scalar("decision.0001")),
                ("title", scalar("Record Validation Decision")),
                ("status", scalar("accepted")),
                ("established-by", scalar("proposal.0002")),
            ],
            "",
        ),
        artifact(
            "guides/records.md",
            &[
                ("id", scalar("guide.records")),
                ("title", scalar("Record Validation Guide")),
            ],
            "",
        ),
        artifact(
            "VISION.md",
            &[
                ("title", scalar("Workbench Vision")),
                ("updated", scalar("2026-08-25")),
            ],
            "",
        ),
    ]
}

/// A lone proposal with `status` and `content`.
pub fn proposal_artifact(status: &str, content: &str) -> Artifact {
    artifact(
        "proposals/0001-records.md",
        &[
            ("id", scalar("proposal.0001")),
            ("title", scalar("Record Validation")),
            ("authors", list(&["Ada"])),
            ("status", scalar(status)),
            ("pull-request", scalar("42")),
            ("supersedes", list(&[])),
        ],
        content,
    )
}
