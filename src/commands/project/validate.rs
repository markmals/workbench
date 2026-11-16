//! `workbench project validate`: problems in how the project uses Workbench.

use std::path::Path;

use schemars::JsonSchema;
use serde::Serialize;

use super::setup::{self, Declaration, DeclarationState, Link, LinkState, PIN_TOOL, Pin};
use super::vendored::{self, TemplateMatch, VendoredFile};
use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::response::{RenderText, Response};

/// The project's Workbench setup problems.
#[derive(Serialize, JsonSchema)]
pub struct Validation {
    /// Every violation, in the order the checks found them.
    pub violations: Vec<Violation>,
    /// How many violations there are; zero means the setup is valid.
    pub count: usize,
}

/// One problem in one file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Violation {
    /// The file or directory, relative to the project root.
    pub path: String,
    pub message: String,
}

impl RenderText for Validation {
    fn render_text(&self) -> String {
        let mut text = String::new();
        for violation in &self.violations {
            text.push_str(&format!("{}: {}\n", violation.path, violation.message));
        }
        let plural = if self.count == 1 { "" } else { "s" };
        text.push_str(&format!("{} violation{plural}\n", self.count));
        text
    }
}

pub fn run(invocation: &Invocation) -> Result<Response, Failure> {
    let project = invocation.project()?;
    let root = project.root.as_path();
    let link = setup::read_link(root);

    let mut violations = pin_and_link(root, &link);
    violations.extend(harness_declarations(root));
    violations.extend(
        vendored::find(root, &link)
            .into_iter()
            .filter_map(vendored_violation),
    );

    let validation = Validation {
        count: violations.len(),
        violations,
    };
    let response = Response::new(&validation);
    if validation.count == 0 {
        return Ok(response);
    }
    let plural = if validation.count == 1 { "" } else { "s" };
    Ok(response.failing(
        Failure::new(
            ErrorCode::CheckFailed,
            format!(
                "the project's Workbench setup has {} violation{plural}",
                validation.count
            ),
        )
        .suggest("Fix each violation on stdout, then rerun `workbench project validate`"),
    ))
}

/// The pin and the link, except in Workbench's own repository, which links its working tree.
fn pin_and_link(root: &Path, link: &Link) -> Vec<Violation> {
    if link.working_tree {
        return Vec::new();
    }
    let mut violations = Vec::new();
    let pin = setup::read_pin(root);
    match &pin {
        Pin::Pinned { .. } => {}
        Pin::Unpinned => violations.push(violation(
            "mise.toml",
            format!(
                "pins no `{PIN_TOOL}` release, so this project has no Workbench to load; run \
                 `mise use {PIN_TOOL}@{}`",
                env!("CARGO_PKG_VERSION")
            ),
        )),
        Pin::Unreadable { reason } => violations.push(violation("mise.toml", reason.clone())),
    }
    let restore = "harnesses load no Workbench content; run `mise install`, which links it to \
                   the pinned release";
    match (link.state, &pin) {
        (LinkState::Missing, _) => {
            violations.push(violation(".workbench", format!("is missing, so {restore}")));
        }
        (LinkState::Dangling, _) => violations.push(violation(
            ".workbench",
            format!(
                "is dangling: it points at {}, which does not exist, so {restore}",
                link.target.as_deref().unwrap_or("nothing")
            ),
        )),
        (LinkState::Linked, Pin::Pinned { version }) if link.version.as_ref() != Some(version) => {
            violations.push(violation(
                ".workbench",
                format!(
                    "links Workbench {} but mise.toml pins {version}; run `mise install`, which \
                     re-links it to the pinned release",
                    link.version.as_deref().unwrap_or("of an unknown version"),
                ),
            ))
        }
        (LinkState::Linked, _) => {}
    }
    violations
}

fn harness_declarations(root: &Path) -> Vec<Violation> {
    [
        ("Claude Code", setup::claude_declaration(root)),
        ("Oh My Pi", setup::omp_declaration(root)),
    ]
    .into_iter()
    .filter_map(|(harness, declaration)| declaration_violation(harness, declaration))
    .collect()
}

fn declaration_violation(harness: &str, declaration: Declaration) -> Option<Violation> {
    let detail = declaration.detail.unwrap_or_default();
    let (problem, fix) = match declaration.state {
        DeclarationState::Declared => return None,
        DeclarationState::Missing => (
            "is missing".to_string(),
            "`workbench project init` creates it",
        ),
        DeclarationState::Incomplete => (detail, "`workbench project init` adds the declaration"),
        DeclarationState::Unreadable => (detail, "fix it, then run `workbench project init`"),
    };
    Some(violation(
        &declaration.path,
        format!("{problem}, so {harness} does not load Workbench; {fix}"),
    ))
}

/// Every vendored file but a template override. Skills and rules carry the collision message.
fn vendored_violation(file: VendoredFile) -> Option<Violation> {
    if file.template == Some(TemplateMatch::Different) {
        return None;
    }
    Some(Violation {
        path: file.path,
        message: file.message,
    })
}

fn violation(path: &str, message: impl Into<String>) -> Violation {
    Violation {
        path: path.to_string(),
        message: message.into(),
    }
}
