//! Reading a project's Workbench setup: the pin, the `.workbench` link, and the harness
//! declarations. `context show` reports these, `project validate` judges them, and `project init`
//! completes them.

use std::fs;
use std::path::Path;

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, json};

use super::omp_config;
use crate::process::Program;

/// The mise tool that installs the Workbench package.
pub const PIN_TOOL: &str = "github:markmals/workbench";

/// The `.claude/settings.json` marketplace and plugin names that load the package.
pub const MARKETPLACE: &str = "workbench";
pub const PLUGIN: &str = "workbench@workbench";
/// How harness declarations name the package link: the marketplace path and the `.omp/config.yml`
/// extension entry.
pub const PACKAGE_PATH: &str = "./.workbench";

pub const CLAUDE_SETTINGS: &str = ".claude/settings.json";
pub const OMP_CONFIG: &str = ".omp/config.yml";

/// The URL of the checkout's `origin` remote, or `None` when it has none.
pub fn origin(root: &Path) -> Option<String> {
    let run = Program::new("git", root)
        .args(["remote", "get-url", "origin"])
        .run()
        .ok()?;
    run.success().then(|| run.stdout.trim().to_string())
}

/// What `mise.toml` pins for Workbench.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum Pin {
    /// `mise.toml` pins this version of `github:markmals/workbench`.
    Pinned { version: String },
    /// `mise.toml` is absent or pins no Workbench version.
    Unpinned,
    /// `mise.toml` exists but is not valid TOML.
    Unreadable { reason: String },
}

/// The `.workbench` link to the installed package.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    /// Always `.workbench`.
    pub path: String,
    pub state: LinkState,
    /// The directory the link resolves to; for a dangling link, its target as written.
    pub target: Option<String>,
    /// The package version at the target: `version` in its `.claude-plugin/plugin.json`.
    pub version: Option<String>,
    /// True in Workbench's own repository, where `.workbench` links to the working tree's
    /// `plugin/` and `mise.toml` pins no release.
    pub working_tree: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LinkState {
    /// It resolves to a directory.
    Linked,
    /// There is no `.workbench`.
    Missing,
    /// It points at nothing.
    Dangling,
}

/// Whether one harness's project configuration loads the package.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Declaration {
    /// The harness's project configuration file.
    pub path: String,
    pub state: DeclarationState,
    /// What the file lacks, or why it cannot be read; null when declared.
    pub detail: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum DeclarationState {
    Declared,
    /// The file does not exist.
    Missing,
    /// The file exists without the declaration.
    Incomplete,
    /// The file cannot be read or parsed.
    Unreadable,
}

pub fn read_pin(root: &Path) -> Pin {
    let Ok(source) = fs::read_to_string(root.join("mise.toml")) else {
        return Pin::Unpinned;
    };
    match source.parse::<toml::Table>() {
        Ok(table) => {
            pinned_version(&table).map_or(Pin::Unpinned, |version| Pin::Pinned { version })
        }
        Err(error) => Pin::Unreadable {
            reason: format!("mise.toml is not valid TOML: {}", error.message()),
        },
    }
}

/// The Workbench version a parsed `mise.toml` pins: `"3.0.0"`, `{ version = "3.0.0" }`, or the
/// first of a list of versions.
pub fn pinned_version(mise: &toml::Table) -> Option<String> {
    let pin = mise.get("tools")?.get(PIN_TOOL)?;
    let version = match pin {
        toml::Value::Table(options) => options.get("version")?,
        toml::Value::Array(versions) => versions.first()?,
        other => other,
    };
    version.as_str().map(str::to_string)
}

pub fn read_link(root: &Path) -> Link {
    let path = root.join(".workbench");
    let mut link = Link {
        path: ".workbench".into(),
        state: LinkState::Missing,
        target: None,
        version: None,
        working_tree: false,
    };
    if fs::symlink_metadata(&path).is_err() {
        return link;
    }
    match path.canonicalize() {
        Ok(target) if target.is_dir() => {
            link.state = LinkState::Linked;
            link.version = package_version(&target);
            link.working_tree = is_package_repository(root, &target);
            link.target = Some(target.display().to_string());
        }
        _ => {
            link.state = LinkState::Dangling;
            link.target = fs::read_link(&path)
                .ok()
                .map(|target| target.display().to_string());
        }
    }
    link
}

fn package_version(package: &Path) -> Option<String> {
    let manifest = fs::read_to_string(package.join(".claude-plugin/plugin.json")).ok()?;
    let manifest: Value = serde_json::from_str(&manifest).ok()?;
    manifest["version"].as_str().map(str::to_string)
}

/// Workbench's own repository: its crate is `workbench` and `.workbench` links to its `plugin/`.
fn is_package_repository(root: &Path, target: &Path) -> bool {
    let Ok(plugin) = root.join("plugin").canonicalize() else {
        return false;
    };
    if plugin != target {
        return false;
    }
    fs::read_to_string(root.join("Cargo.toml"))
        .ok()
        .and_then(|source| source.parse::<toml::Table>().ok())
        .and_then(|manifest| {
            manifest
                .get("package")?
                .get("name")?
                .as_str()
                .map(|name| name == "workbench")
        })
        .unwrap_or(false)
}

pub fn claude_declaration(root: &Path) -> Declaration {
    let Some(source) = read_declaration_file(root, CLAUDE_SETTINGS) else {
        return declaration(CLAUDE_SETTINGS, DeclarationState::Missing, None);
    };
    let settings: Value = match source.and_then(|source| {
        serde_json::from_str(&source).map_err(|error| format!("it is not valid JSON: {error}"))
    }) {
        Ok(settings) => settings,
        Err(reason) => {
            return declaration(CLAUDE_SETTINGS, DeclarationState::Unreadable, Some(reason));
        }
    };
    let mut lacks = Vec::new();
    if !is_package_marketplace(&settings["extraKnownMarketplaces"][MARKETPLACE]) {
        lacks.push(format!(
            "the `{MARKETPLACE}` marketplace in `extraKnownMarketplaces` with the directory source `./.workbench`"
        ));
    }
    if settings["enabledPlugins"][PLUGIN] != Value::Bool(true) {
        lacks.push(format!("`\"{PLUGIN}\": true` in `enabledPlugins`"));
    }
    if lacks.is_empty() {
        return declaration(CLAUDE_SETTINGS, DeclarationState::Declared, None);
    }
    declaration(
        CLAUDE_SETTINGS,
        DeclarationState::Incomplete,
        Some(format!("it lacks {}", lacks.join(" and "))),
    )
}

/// The `extraKnownMarketplaces` entry that loads the package from `.workbench`.
pub fn package_marketplace() -> Value {
    json!({ "source": { "source": "directory", "path": PACKAGE_PATH } })
}

/// Whether an `extraKnownMarketplaces` entry loads the package from `.workbench`.
pub fn is_package_marketplace(entry: &Value) -> bool {
    let source = &entry["source"];
    source["source"] == "directory" && source["path"].as_str().is_some_and(is_package_link)
}

/// Whether a path written in a harness declaration names the project's `.workbench` link.
pub fn is_package_link(path: &str) -> bool {
    path.trim_start_matches("./").trim_end_matches('/') == ".workbench"
}

pub fn omp_declaration(root: &Path) -> Declaration {
    let source = match read_declaration_file(root, OMP_CONFIG) {
        None => return declaration(OMP_CONFIG, DeclarationState::Missing, None),
        Some(Err(reason)) => {
            return declaration(OMP_CONFIG, DeclarationState::Unreadable, Some(reason));
        }
        Some(Ok(source)) => source,
    };
    let declared = omp_config::extensions(&source)
        .unwrap_or_default()
        .iter()
        .any(|entry| is_package_link(entry));
    if declared {
        return declaration(OMP_CONFIG, DeclarationState::Declared, None);
    }
    declaration(
        OMP_CONFIG,
        DeclarationState::Incomplete,
        Some(format!("its `extensions:` list lacks `{PACKAGE_PATH}`")),
    )
}

/// The file's text, an error when it exists but cannot be read, or `None` when it is absent.
fn read_declaration_file(root: &Path, relative: &str) -> Option<Result<String, String>> {
    let path = root.join(relative);
    if !path.exists() {
        return None;
    }
    Some(fs::read_to_string(&path).map_err(|error| format!("it cannot be read: {error}")))
}

fn declaration(path: &str, state: DeclarationState, detail: Option<String>) -> Declaration {
    Declaration {
        path: path.to_string(),
        state,
        detail,
    }
}
