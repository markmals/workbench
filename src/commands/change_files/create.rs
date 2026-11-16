//! `workbench change-files create`: writing one change file without Knope's prompts.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::json;

use super::change_file;
use super::knope::{KNOPE_TOML, KnopeConfig, KnopePackage};
use crate::failure::Failure;
use crate::invocation::Invocation;
use crate::project::Project;
use crate::response::{RenderText, Response};

/// What `change-files create` wrote, or would write.
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangeFileReceipt {
    /// `created`, or `dry-run` when nothing was written.
    pub status: Status,
    /// The project root.
    pub project: String,
    /// The change file, relative to the project root: `.changeset/<name>.md`.
    pub path: String,
    /// The change type: `major`, `minor`, `patch`, or one the package's knope.toml sections add.
    #[serde(rename = "type")]
    pub change_type: String,
    /// The package the change affects: `default` in a single-package project.
    pub package: String,
    /// The file a dry run would write.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// What a dry run cannot predict; empty when it predicts everything.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cannot_predict: Option<Vec<String>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
pub enum Status {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "dry-run")]
    DryRun,
}

impl RenderText for ChangeFileReceipt {
    fn render_text(&self) -> String {
        match (&self.status, &self.content) {
            (Status::DryRun, Some(content)) => {
                format!("Would create {}:\n\n{content}", self.path)
            }
            _ => format!(
                "Created {} ({}: {})",
                self.path, self.package, self.change_type
            ),
        }
    }
}

pub fn create(invocation: &Invocation) -> Result<Response, Failure> {
    let project = invocation.project()?;
    let config = KnopeConfig::load(&project.root)?.ok_or_else(missing_knope_toml)?;
    let package = choose_package(&config, invocation.string("package"))?;
    let change_type = invocation.string("type").expect("type is required");
    check_type(package, &change_type)?;
    let summary = summary(invocation.string("summary").expect("summary is required"))?;
    let path = format!(".changeset/{}.md", change_file::file_stem(&summary));
    if project.path(&path).symlink_metadata().is_ok() {
        return Err(already_exists(&path));
    }
    let body = invocation.text("body")?;
    let content = change_file::render(&package.name, &change_type, &summary, body.as_deref());

    let mut receipt = ChangeFileReceipt {
        status: Status::Created,
        project: project.root.display().to_string(),
        path,
        change_type,
        package: package.name.clone(),
        content: None,
        cannot_predict: None,
    };
    if invocation.dry_run() {
        receipt.status = Status::DryRun;
        receipt.content = Some(content);
        receipt.cannot_predict = Some(Vec::new());
    } else {
        write_new(&project, &receipt.path, &content)?;
    }
    Ok(Response::new(&receipt))
}

fn missing_knope_toml() -> Failure {
    Failure::precondition("the project has no knope.toml, so its packages and change types are unknown")
        .with_details(json!({ "path": KNOPE_TOML }))
        .suggest("Add knope.toml declaring the project's package: `[package]` with its `versioned_files` and `changelog`; see https://knope.tech/reference/config-file/packages/")
}

fn choose_package(
    config: &KnopeConfig,
    requested: Option<String>,
) -> Result<&KnopePackage, Failure> {
    let names = config.package_names();
    match (requested, config.packages.as_slice()) {
        (None, [only]) => Ok(only),
        (None, _) => Err(Failure::invalid_argument(
            "knope.toml declares several packages; name the one this change affects with --package",
        )
        .with_details(json!({ "missing": ["package"], "acceptedValues": names }))),
        (Some(name), _) => config.package(&name).ok_or_else(|| {
            Failure::invalid_argument(format!("`{name}` is not a package in knope.toml"))
                .with_details(
                    json!({ "argument": "package", "value": name, "acceptedValues": names }),
                )
        }),
    }
}

fn check_type(package: &KnopePackage, change_type: &str) -> Result<(), Failure> {
    if package.accepts(change_type) {
        return Ok(());
    }
    Err(Failure::invalid_argument(format!(
        "`{change_type}` is not a change type of the `{}` package",
        package.name
    ))
    .with_details(json!({
        "argument": "type",
        "value": change_type,
        "acceptedValues": package.change_types,
    })))
}

/// The summary, trimmed: one line that names a file.
fn summary(summary: String) -> Result<String, Failure> {
    let summary = summary.trim();
    let problem = if summary.contains(['\n', '\r']) {
        "--summary is the change file's heading, so it must be one line"
    } else if change_file::file_stem(summary).is_empty() {
        "--summary needs at least one ASCII letter or digit to name the change file"
    } else {
        return Ok(summary.to_string());
    };
    Err(Failure::invalid_argument(problem)
        .with_details(json!({ "argument": "summary", "value": summary })))
}

fn already_exists(path: &str) -> Failure {
    Failure::invalid_argument(format!(
        "{path} already exists, and change-files create never overwrites a file"
    ))
    .with_details(json!({ "argument": "summary", "path": path }))
    .suggest(format!("Edit {path} if it describes this change"))
    .suggest("Otherwise pass a --summary that names a different file")
}

fn write_new(project: &Project, path: &str, content: &str) -> Result<(), Failure> {
    let full = project.path(path);
    let unwritable = |error: std::io::Error| {
        Failure::precondition(format!("cannot write {path}: {error}"))
            .with_details(json!({ "path": path }))
    };
    if let Some(directory) = full.parent() {
        fs::create_dir_all(directory).map_err(unwritable)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&full)
        .map_err(|error| match error.kind() {
            ErrorKind::AlreadyExists => already_exists(path),
            _ => unwritable(error),
        })?;
    file.write_all(content.as_bytes()).map_err(unwritable)
}
