//! The plugin content every harness loads from `.workbench/`.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use support::plugin_dir;

fn read(relative: &str) -> String {
    fs::read_to_string(plugin_dir().join(relative))
        .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
}

/// The frontmatter lines and the body after them.
fn split_frontmatter(source: &str) -> (Vec<String>, String) {
    let rest = source
        .strip_prefix("---\n")
        .expect("frontmatter opens the file");
    let end = rest.find("\n---\n").expect("frontmatter closes");
    (
        rest[..end].lines().map(str::to_string).collect(),
        rest[end + 5..].to_string(),
    )
}

#[test]
fn the_claude_output_style_carries_the_same_contract_as_the_rule() {
    let (_, rule) = split_frontmatter(&read("rules/workbench.md"));
    let (style_frontmatter, style) = split_frontmatter(&read("output-styles/workbench.md"));
    assert!(
        rule == style,
        "the bodies of rules/workbench.md and output-styles/workbench.md differ"
    );
    assert!(
        style_frontmatter
            .iter()
            .any(|line| line == "force-for-plugin: true")
    );
    assert!(
        style_frontmatter
            .iter()
            .any(|line| line == "keep-coding-instructions: true")
    );
}

#[test]
fn the_contract_is_always_applied_and_every_other_rule_is_described() {
    let (contract, _) = split_frontmatter(&read("rules/workbench.md"));
    assert!(contract.iter().any(|line| line == "alwaysApply: true"));
    for entry in fs::read_dir(plugin_dir().join("rules")).unwrap() {
        let path = entry.unwrap().path();
        let (frontmatter, _) = split_frontmatter(&fs::read_to_string(&path).unwrap());
        let description = frontmatter
            .iter()
            .find_map(|line| line.strip_prefix("description: "));
        assert!(
            description.is_some_and(|text| !text.contains(": ")),
            "{} needs a one-line `description` without `: `, or OMP never shows it",
            path.display()
        );
    }
}

#[test]
fn every_skill_declares_its_name_and_when_to_use_it() {
    for entry in fs::read_dir(plugin_dir().join("skills")).unwrap() {
        let directory = entry.unwrap().path();
        let name = directory
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let (frontmatter, _) =
            split_frontmatter(&fs::read_to_string(directory.join("SKILL.md")).unwrap());
        assert!(
            frontmatter.contains(&format!("name: {name}")),
            "{name}: name does not match its directory"
        );
        assert!(
            frontmatter
                .iter()
                .any(|line| line.starts_with("description: ")),
            "{name} lacks a description"
        );
    }
}

#[test]
fn the_manifests_name_the_plugin_at_the_release_version() {
    let plugin: serde_json::Value =
        serde_json::from_str(&read(".claude-plugin/plugin.json")).unwrap();
    assert_eq!(plugin["name"], "workbench");
    assert_eq!(plugin["version"], env!("CARGO_PKG_VERSION"));
    let marketplace: serde_json::Value =
        serde_json::from_str(&read(".claude-plugin/marketplace.json")).unwrap();
    assert_eq!(marketplace["name"], "workbench");
    assert_eq!(marketplace["plugins"][0]["name"], "workbench");
    assert_eq!(marketplace["plugins"][0]["source"], "./");
}

fn markdown_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            markdown_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "md") {
            files.push(path);
        }
    }
}

/// Package files ship without the repository around them, so a relative link must stay inside
/// `plugin/` and point at a file that exists.
#[test]
fn every_relative_link_in_the_package_resolves_inside_the_package() {
    let link = regex::Regex::new(r"\]\(([^)#\s]+)(#[^)]*)?\)").unwrap();
    let root = plugin_dir().canonicalize().unwrap();
    let mut files = Vec::new();
    markdown_files(&root, &mut files);
    let mut broken = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file).unwrap();
        let prose = workbench::frontmatter::without_code(&source);
        for captures in link.captures_iter(&prose) {
            let target = &captures[1];
            let external = target.contains("://") || target.starts_with("mailto:");
            // A template's `[the proposal](proposal-url)` is a placeholder for the author to fill.
            let placeholder = !target.contains('/') && !target.contains('.');
            if external || placeholder {
                continue;
            }
            let resolved = file.parent().unwrap().join(target);
            let inside = resolved
                .canonicalize()
                .map(|path| path.starts_with(&root))
                .unwrap_or(false);
            if !inside {
                broken.push(format!(
                    "{} → {target}",
                    file.strip_prefix(&root).unwrap().display()
                ));
            }
        }
    }
    assert!(
        broken.is_empty(),
        "links that leave the package or do not resolve:\n{}",
        broken.join("\n")
    );
}
