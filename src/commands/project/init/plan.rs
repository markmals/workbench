//! What `project init` would change in a project: every write it would make, what it would leave
//! alone, and what it cannot add without overwriting. Reads the project; writes nothing.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use toml_edit::DocumentMut;

use super::Notice;
use super::claude_settings;
use super::mise;
use super::scaffold::Site;
use crate::commands::project::omp_config;
use crate::commands::project::setup::{
    self, CLAUDE_SETTINGS, MARKETPLACE, OMP_CONFIG, PACKAGE_PATH, PLUGIN,
};

/// One write `project init` makes.
pub enum Operation {
    /// Create a file that does not exist.
    Create { path: String, contents: String },
    /// Replace an existing file's contents with the same contents plus `entries`.
    AddTo {
        path: String,
        contents: String,
        entries: Vec<String>,
    },
    /// Create a symbolic link to `target`.
    Link { path: String, target: String },
}

impl Operation {
    pub fn path(&self) -> &str {
        match self {
            Operation::Create { path, .. }
            | Operation::AddTo { path, .. }
            | Operation::Link { path, .. } => path,
        }
    }
}

#[derive(Default)]
pub struct Plan {
    pub operations: Vec<Operation>,
    pub unchanged: Vec<String>,
    pub attention: Vec<Notice>,
}

impl Plan {
    fn create(&mut self, path: &str, contents: String) {
        let path = path.to_string();
        self.operations.push(Operation::Create { path, contents });
    }

    fn add_to(&mut self, path: &str, contents: String, entries: Vec<String>) {
        if entries.is_empty() {
            self.unchanged.push(path.to_string());
        } else {
            let path = path.to_string();
            self.operations.push(Operation::AddTo {
                path,
                contents,
                entries,
            });
        }
    }

    fn attention(&mut self, path: &str, message: impl Into<String>) {
        self.attention.push(Notice {
            path: path.to_string(),
            message: message.into(),
        });
    }
}

/// The project as `project init` finds it.
pub struct Project<'a> {
    pub root: &'a Path,
    /// `mise.toml`, already known to pin Workbench.
    pub mise: DocumentMut,
    pub site: Option<Site>,
    /// The scaffold files, from `scaffold::files`.
    pub files: Vec<(&'static str, Option<String>)>,
}

pub fn plan(project: Project) -> Plan {
    let mut plan = Plan::default();
    plan_mise(&mut plan, project.mise, project.site);
    plan_gitignore(&mut plan, project.root);
    plan_claude_settings(&mut plan, project.root);
    plan_omp_config(&mut plan, project.root);
    plan_claude_links(&mut plan, project.root);
    plan_files(&mut plan, project.root, project.files);
    plan
}

fn plan_mise(plan: &mut Plan, mut document: DocumentMut, site: Option<Site>) {
    let additions = mise::add_missing(&mut document, site.is_some());
    for conflict in additions.conflicts {
        plan.attention("mise.toml", conflict);
    }
    plan.add_to("mise.toml", document.to_string(), additions.added);
}

/// `.gitignore` with the package link, which is machine-local.
fn plan_gitignore(plan: &mut Plan, root: &Path) {
    let (path, entry) = (".gitignore", ".workbench");
    let existing = match read_existing(root, path) {
        Ok(existing) => existing,
        Err(reason) => return plan.attention(path, reason),
    };
    let Some(mut contents) = existing else {
        return plan.create(path, format!("{entry}\n"));
    };
    if contents.lines().any(ignores_link) {
        return plan.unchanged.push(path.to_string());
    }
    if !contents.is_empty() && !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str(entry);
    contents.push('\n');
    plan.add_to(path, contents, vec![entry.to_string()]);
}

/// Whether a `.gitignore` line ignores the link. `.workbench/` would not: it matches only a
/// directory, and Git sees a symbolic link.
fn ignores_link(line: &str) -> bool {
    matches!(line.trim(), ".workbench" | "/.workbench")
}

fn plan_claude_settings(plan: &mut Plan, root: &Path) {
    let path = CLAUDE_SETTINGS;
    match read_existing(root, path) {
        Err(reason) => plan.attention(path, reason),
        Ok(None) => plan.create(path, claude_settings::new_settings()),
        Ok(Some(source)) => match claude_settings::add_declaration(&source) {
            Ok(edit) => {
                for conflict in edit.conflicts {
                    plan.attention(path, conflict);
                }
                plan.add_to(path, edit.contents, edit.entries);
            }
            Err(reason) => plan.attention(
                path,
                format!(
                    "{reason}, so nothing was added; declare the `{MARKETPLACE}` marketplace and \
                     enable `{PLUGIN}` by hand"
                ),
            ),
        },
    }
}

fn plan_omp_config(plan: &mut Plan, root: &Path) {
    let path = OMP_CONFIG;
    let source = match read_existing(root, path) {
        Err(reason) => return plan.attention(path, reason),
        Ok(None) => return plan.create(path, omp_config::with_extension("", PACKAGE_PATH)),
        Ok(Some(source)) => source,
    };
    let declared = omp_config::extensions(&source)
        .unwrap_or_default()
        .iter()
        .any(|entry| setup::is_package_link(entry));
    if declared {
        plan.unchanged.push(path.to_string());
    } else {
        let contents = omp_config::with_extension(&source, PACKAGE_PATH);
        plan.add_to(path, contents, vec![PACKAGE_PATH.to_string()]);
    }
}

/// `.claude/skills` and `.claude/rules`, for the `.agents/` directories that exist.
fn plan_claude_links(plan: &mut Plan, root: &Path) {
    for kind in ["skills", "rules"] {
        if !root.join(".agents").join(kind).is_dir() {
            continue;
        }
        let path = format!(".claude/{kind}");
        if fs::symlink_metadata(root.join(&path)).is_ok() {
            plan.unchanged.push(path);
        } else {
            let target = format!("../.agents/{kind}");
            plan.operations.push(Operation::Link { path, target });
        }
    }
}

fn plan_files(plan: &mut Plan, root: &Path, files: Vec<(&'static str, Option<String>)>) {
    for (path, contents) in files {
        if fs::symlink_metadata(root.join(path)).is_ok() {
            plan.unchanged.push(path.to_string());
            if path == "AGENTS.md" {
                check_agents_line(plan, root);
            }
            continue;
        }
        match contents {
            Some(contents) => plan.create(path, contents),
            None => plan.attention(
                path,
                "the package at `.workbench` has no template for it, so it was not created",
            ),
        }
    }
}

fn check_agents_line(plan: &mut Plan, root: &Path) {
    let says_where = fs::read_to_string(root.join("AGENTS.md"))
        .is_ok_and(|agents| agents.contains(".workbench"));
    if !says_where {
        plan.attention(
            "AGENTS.md",
            "it does not say where Workbench loads from; add that Workbench loads from \
             `.workbench`, a gitignored link `mise install` restores",
        );
    }
}

/// The file's text, `None` when it does not exist, or why it cannot be read.
fn read_existing(root: &Path, relative: &str) -> Result<Option<String>, String> {
    match fs::read_to_string(root.join(relative)) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("it cannot be read ({error}), so nothing was added")),
    }
}
