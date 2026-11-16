//! `workbench project init`: adding what Workbench owns to a project, never overwriting.

mod apply;
mod claude_settings;
mod json_members;
mod mise;
mod plan;
mod receipt;
mod scaffold;

use std::fs;
use std::path::Path;

use serde_json::json;
use toml_edit::DocumentMut;

use super::package_templates::package_template;
use super::setup::{self, Link, PIN_TOOL, Pin};
use super::vendored;
use crate::failure::Failure;
use crate::invocation::Invocation;
use crate::response::Response;
use plan::{Operation, Plan};
pub use receipt::Receipt;
use receipt::{Addition, MissingPiece, Notice, Status};
use scaffold::Site;

const CANNOT_PREDICT: &[&str] = &[
    "Whether each file can be written: permissions and free space show only when writing.",
    "Changes another process makes to these files before the real run.",
];

pub fn run(invocation: &Invocation) -> Result<Response, Failure> {
    let project = invocation.project()?;
    let root = project.root.as_path();
    let mise = pinned_mise(root)?;
    let site = match invocation.string("site") {
        None => None,
        Some(name) => Some(Site::parse(&name).ok_or_else(|| {
            Failure::internal(format!("the parser accepted an unknown site host `{name}`"))
        })?),
    };
    let link = setup::read_link(root);
    let missing = missing_pieces(root, &mise);
    let files = scaffold::files(&scaffold_project(root, &link), site);
    let plan = plan::plan(plan::Project {
        root,
        mise,
        site,
        files,
    });
    let dry_run = invocation.dry_run();
    let failed = if dry_run {
        Vec::new()
    } else {
        apply::apply(root, &plan.operations)
    };

    let receipt = receipt(root, site, plan, failed, dry_run, &link, missing);
    let response = Response::new(&receipt);
    if receipt.failed.is_empty() {
        return Ok(response);
    }
    Ok(response.failing(write_failure(&receipt.failed)))
}

/// `mise.toml`, when it pins Workbench; otherwise the failure that says how to pin it.
fn pinned_mise(root: &Path) -> Result<DocumentMut, Failure> {
    match setup::read_pin(root) {
        Pin::Pinned { .. } => {}
        Pin::Unpinned => {
            return Err(Failure::precondition(format!(
                "mise.toml does not pin `{PIN_TOOL}`, so this is not yet a Workbench project"
            ))
            .with_details(json!({ "file": "mise.toml", "tool": PIN_TOOL }))
            .suggest(format!(
                "Run `mise use {PIN_TOOL}@{}`, then rerun `workbench project init`",
                env!("CARGO_PKG_VERSION")
            )));
        }
        Pin::Unreadable { reason } => {
            return Err(Failure::precondition(reason)
                .with_details(json!({ "file": "mise.toml" }))
                .suggest("Fix mise.toml, then rerun `workbench project init`"));
        }
    }
    let source = fs::read_to_string(root.join("mise.toml"))
        .map_err(|error| Failure::precondition(format!("cannot read mise.toml: {error}")))?;
    source
        .parse()
        .map_err(|error| Failure::precondition(format!("mise.toml is not valid TOML: {error}")))
}

fn scaffold_project(root: &Path, link: &Link) -> scaffold::Project {
    let vision = fs::read_to_string(root.join(".agents/templates/VISION.md"))
        .ok()
        .or_else(|| package_template(link, "VISION.md"));
    let name = root
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    scaffold::Project {
        name,
        repository: setup::origin(root).and_then(|origin| scaffold::repository_url(&origin)),
        vision,
        today: scaffold::today(),
    }
}

fn missing_pieces(root: &Path, mise: &DocumentMut) -> Vec<MissingPiece> {
    let mut missing: Vec<MissingPiece> = mise::missing_project_tasks(mise)
        .into_iter()
        .map(|task| MissingPiece {
            piece: task.to_string(),
            message: format!("mise.toml has no `{task}` task: {}", task_purpose(task)),
        })
        .collect();
    if !root.join("knope.toml").exists() {
        missing.push(MissingPiece {
            piece: "release".into(),
            message: "there is no `knope.toml`: add a release flow that consumes change files \
                      with Knope"
                .into(),
        });
    }
    missing
}

fn task_purpose(task: &str) -> &'static str {
    match task {
        "setup" => {
            "the idempotent bootstrap of a machine, which restores the Beads replica and installs \
             the Git hooks with `hk install --mise`"
        }
        "check" => "the project's whole blocking gate, which CI and the agent run",
        _ => "the project's formatter, which accepts file paths",
    }
}

fn receipt(
    root: &Path,
    site: Option<Site>,
    plan: Plan,
    failed: Vec<Notice>,
    dry_run: bool,
    link: &Link,
    missing: Vec<MissingPiece>,
) -> Receipt {
    let status = match (dry_run, plan.operations.is_empty()) {
        (true, _) => Status::DryRun,
        (false, true) => Status::Unchanged,
        (false, false) => Status::Initialized,
    };
    let written =
        |operation: &&Operation| !failed.iter().any(|notice| notice.path == operation.path());
    let mut created = Vec::new();
    let mut added_to = Vec::new();
    for operation in plan.operations.iter().filter(written) {
        match operation {
            Operation::AddTo { path, entries, .. } => added_to.push(Addition {
                path: path.clone(),
                entries: entries.clone(),
            }),
            other => created.push(other.path().to_string()),
        }
    }
    Receipt {
        status,
        project: root.display().to_string(),
        site: site.map(|site| site.name().to_string()),
        created,
        added_to,
        unchanged: plan.unchanged,
        attention: plan.attention,
        failed,
        vendored: vendored::find(root, link),
        missing,
        cannot_predict: if dry_run {
            CANNOT_PREDICT.iter().map(|line| line.to_string()).collect()
        } else {
            Vec::new()
        },
    }
}

fn write_failure(failed: &[Notice]) -> Failure {
    let paths: Vec<&str> = failed.iter().map(|notice| notice.path.as_str()).collect();
    Failure::precondition(format!("could not write {}", paths.join(", ")))
        .with_details(json!({ "failed": failed.iter().map(|notice| json!({ "path": notice.path, "message": notice.message })).collect::<Vec<_>>() }))
        .suggest(
            "Make those paths writable, then rerun `workbench project init`; it adds only what is \
             still missing",
        )
}
