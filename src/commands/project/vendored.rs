//! Workbench files a project still carries that the package now supplies: what a project made
//! from the old template, or an older copy of Workbench, must delete when it adopts the package.

use std::fs;
use std::path::Path;

use schemars::JsonSchema;
use serde::Serialize;

use super::package_templates::package_template;
use super::setup::Link;
use crate::package::{RULES, SKILLS};

/// One vendored file or directory.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct VendoredFile {
    /// Relative to the project root; a directory has no trailing slash.
    pub path: String,
    pub kind: VendoredKind,
    /// For a template: `identical` to the package's, so removable, or `different`, an override
    /// the project keeps. Null for every other kind.
    pub template: Option<TemplateMatch>,
    /// What the file is and what to do with it.
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum VendoredKind {
    /// A project skill named like a Workbench skill.
    Skill,
    /// A project rule named like a Workbench rule.
    Rule,
    Template,
    /// One of the Oh My Pi extensions Workbench no longer uses.
    OmpExtension,
    /// The Oh My Pi workflow tool.
    WorkflowTool,
    /// A TypeScript tool the `workbench` CLI replaces, or its test.
    Tool,
    ChangesetsConfig,
    Vitepress,
    Fnox,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateMatch {
    Identical,
    Different,
}

const FIXED: &[(&str, VendoredKind, &str)] = &[
    (
        ".omp/extensions/check-before-stop.ts",
        VendoredKind::OmpExtension,
        "is an Oh My Pi extension Workbench no longer uses; delete it",
    ),
    (
        ".omp/extensions/format-after-edit.ts",
        VendoredKind::OmpExtension,
        "is an Oh My Pi extension Workbench no longer uses; delete it",
    ),
    (
        ".omp/extensions/prime-beads.ts",
        VendoredKind::OmpExtension,
        "is an Oh My Pi extension Workbench no longer uses; delete it",
    ),
    (
        ".omp/tools/dynamic-workflows",
        VendoredKind::WorkflowTool,
        "is the old workflow tool, which Oh My Pi's native helpers replace; delete it",
    ),
    (
        ".changeset/config.json",
        VendoredKind::ChangesetsConfig,
        "configures Changesets, which change files and Knope replace; delete it",
    ),
    (
        "docs/.vitepress",
        VendoredKind::Vitepress,
        "is the old VitePress site, which the package's Hugo theme replaces; delete it",
    ),
    (
        "fnox.toml",
        VendoredKind::Fnox,
        "configures Fnox, which Varlock replaces; delete it",
    ),
];

/// The TypeScript tools `workbench` replaces, by the part of the file name before its first dot.
const TOOLS: &[&str] = &[
    "validate",
    "status",
    "check-scoped-commit",
    "check-pr-commits",
    "require-changeset",
    "tag-release",
    "project-version",
    "bd",
    "fake-bd",
];

/// Every Workbench file the project vendors: Workbench-named skills and rules, templates (both
/// identical copies and overrides), then the fixed list, then the replaced tools.
pub fn find(root: &Path, link: &Link) -> Vec<VendoredFile> {
    let mut found = Vec::new();
    for name in entries(&root.join(".agents/skills"), |path| path.is_dir()) {
        if SKILLS.contains(&name.as_str()) {
            let path = format!(".agents/skills/{name}");
            found.push(collision(path, VendoredKind::Skill, "skill", &name));
        }
    }
    for file in entries(&root.join(".agents/rules"), Path::is_file) {
        if let Some(name) = file.strip_suffix(".md")
            && RULES.contains(&name)
        {
            let path = format!(".agents/rules/{file}");
            found.push(collision(path, VendoredKind::Rule, "rule", name));
        }
    }
    found.extend(templates(root, link));
    for (path, kind, message) in FIXED {
        if root.join(path).exists() {
            found.push(vendored(path.to_string(), *kind, None, message.to_string()));
        }
    }
    for file in entries(&root.join("tools"), Path::is_file) {
        if is_replaced_tool(&file) {
            found.push(vendored(
                format!("tools/{file}"),
                VendoredKind::Tool,
                None,
                "is a TypeScript tool the `workbench` CLI replaces; delete it".into(),
            ));
        }
    }
    found
}

fn collision(path: String, kind: VendoredKind, noun: &str, name: &str) -> VendoredFile {
    let message = format!(
        "`{path}` reuses the name of the Workbench {noun} `{name}`, which harnesses resolve \
         differently; remove it if it copies Workbench's, or rename it"
    );
    vendored(path, kind, None, message)
}

fn templates(root: &Path, link: &Link) -> Vec<VendoredFile> {
    let mut found = Vec::new();
    for name in entries(&root.join(".agents/templates"), Path::is_file) {
        let Some(package) = package_template(link, &name) else {
            continue;
        };
        let path = format!(".agents/templates/{name}");
        let identical = fs::read(root.join(&path)).is_ok_and(|ours| ours == package.as_bytes());
        let (template, message) = if identical {
            (
                TemplateMatch::Identical,
                format!("is identical to the package's `templates/{name}`; delete it"),
            )
        } else {
            (
                TemplateMatch::Different,
                format!(
                    "differs from the package's `templates/{name}`, so it overrides it; keep it if \
                     the difference is intended"
                ),
            )
        };
        found.push(vendored(
            path,
            VendoredKind::Template,
            Some(template),
            message,
        ));
    }
    found
}

pub fn is_replaced_tool(file: &str) -> bool {
    let Some(stem) = file.strip_suffix(".ts") else {
        return false;
    };
    let name = stem.split('.').next().unwrap_or(stem);
    TOOLS.contains(&name) || name.starts_with("beads")
}

fn vendored(
    path: String,
    kind: VendoredKind,
    template: Option<TemplateMatch>,
    message: String,
) -> VendoredFile {
    VendoredFile {
        path,
        kind,
        template,
        message,
    }
}

/// The names of the entries in `directory` that `keep` accepts, sorted; none when it is absent.
fn entries(directory: &Path, keep: impl Fn(&Path) -> bool) -> Vec<String> {
    let Ok(listing) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut names: Vec<String> = listing
        .filter_map(|entry| entry.ok())
        .filter(|entry| keep(&entry.path()))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaced_tools_include_their_tests_and_every_beads_module() {
        for file in [
            "validate.ts",
            "validate.test.ts",
            "status.collection.test.ts",
            "beads-cli.ts",
            "beads.test.ts",
            "fake-bd.ts",
            "bd.ts",
        ] {
            assert!(is_replaced_tool(file), "{file}");
        }
        for file in ["release.ts", "validate.js", "current-records.test.ts", "bd"] {
            assert!(!is_replaced_tool(file), "{file}");
        }
    }
}
