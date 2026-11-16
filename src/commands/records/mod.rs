//! `workbench records`: the project's records.

mod collect;
mod fields;
mod identity;
mod kind;
mod links;
pub mod validate;

use schemars::{JsonSchema, schema_for};
use serde::Serialize;
use serde_json::json;

use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::registry::{CommandSpec, Effect, Example, Requirement};
use crate::response::{RenderText, Response};
use validate::Violation;

pub static COMMANDS: &[CommandSpec] = &[CommandSpec {
    id: "records.validate",
    path: &["records", "validate"],
    summary: "Check the frontmatter of the project's proposals, policies, decisions, guides, and vision",
    description: "Checks the records at the project root, the Git top level of the current directory: \
        `VISION.md` and the Markdown files directly inside `proposals/`, `policies/`, `decisions/`, \
        and `guides/`. Directory READMEs are indexes and are skipped; absent or empty record \
        directories are allowed; a Markdown file nested deeper is a violation; symlinked files are \
        checked and symlinked directories are not traversed. Each record must have parsable \
        frontmatter with its kind's required keys and an allowed status, an ID that matches its \
        filename and is unique, and `supersedes` and `established-by` references that resolve. A \
        proposal another supersedes must have status `superseded`, supersession must not form a \
        cycle, and a proposal past drafting must not keep an unresolved clarification marker in its \
        prose. The result lists every violation with its `path` relative to the project root and a \
        `message`, and their `count`. With any violation the result still goes to stdout and the \
        command fails with CHECK_FAILED (exit 1). Never changes anything. Fails with \
        PRECONDITION_FAILED outside a Git checkout or when a record cannot be read, and with \
        DEPENDENCY_MISSING when git is not installed.",
    arguments: &[],
    effect: Effect::Reads,
    requires: &[Requirement::Git],
    dry_run: false,
    idempotent: true,
    errors: &[
        ErrorCode::CheckFailed,
        ErrorCode::PreconditionFailed,
        ErrorCode::DependencyMissing,
    ],
    examples: &[Example {
        summary: "Check every record in the project",
        args: &["records", "validate"],
    }],
    synonyms: &[
        "lint",
        "check",
        "verify",
        "records",
        "frontmatter",
        "metadata",
        "proposals",
        "policies",
        "decisions",
        "guides",
        "vision",
        "violations",
    ],
    output_schema: || schema_for!(ValidationReport),
    run: validate_records,
}];

/// Every problem the validator found in the project's records.
#[derive(Serialize, JsonSchema)]
pub struct ValidationReport {
    /// Every violation, in the order the validator found them.
    pub violations: Vec<Violation>,
    /// How many violations there are; zero means the records are valid.
    pub count: usize,
}

impl RenderText for ValidationReport {
    fn render_text(&self) -> String {
        let mut text = String::new();
        for violation in &self.violations {
            text.push_str(&format!("{}: {}\n", violation.path, violation.message));
        }
        text.push_str(&count_line(self.count));
        text
    }
}

fn count_line(count: usize) -> String {
    let plural = if count == 1 { "" } else { "s" };
    format!("{count} violation{plural}\n")
}

fn validate_records(invocation: &Invocation) -> Result<Response, Failure> {
    let project = invocation.project()?;
    let collected = collect::collect(&project.root)?;
    let mut violations = collected.violations;
    violations.extend(validate::validate(&collected.artifacts));
    let report = ValidationReport {
        count: violations.len(),
        violations,
    };

    let response = Response::new(&report);
    if report.count == 0 {
        return Ok(response);
    }
    let failure = Failure::new(
        ErrorCode::CheckFailed,
        format!("the records have {}", count_line(report.count).trim_end()),
    )
    .with_details(json!({ "count": report.count }))
    .suggest(
        "Fix the record at each violation's path, then run `workbench records validate` again",
    );
    Ok(response.failing(failure))
}
