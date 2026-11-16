//! Projects for the `project` tests, and ways to compare a project before and after a command.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::support::{Fixture, assert_matches_output_schema, declare_workbench, workbench_pin};

/// Runs `project init` with `args`, expecting success, and returns its receipt.
pub fn init(fixture: &Fixture, args: &[&str]) -> Value {
    let mut command = vec!["project", "init"];
    command.extend_from_slice(args);
    let outcome = fixture.run(&command);
    assert_eq!(outcome.status, 0, "{}\n{}", outcome.stdout, outcome.stderr);
    let receipt = outcome.json();
    assert_matches_output_schema(&["project", "init"], &receipt);
    receipt
}

/// A Git repository whose `mise.toml` holds only the Workbench pin, and no `.gitignore`.
pub fn pinned_repository() -> Fixture {
    let fixture = Fixture::git();
    fs::remove_file(fixture.path(".gitignore")).expect("the fixture's .gitignore");
    fixture.write("mise.toml", &workbench_pin());
    fixture.commit_all("chore: pin workbench");
    fixture
}

/// A project set up as `project init` and `mise install` leave it.
pub fn complete_setup() -> Fixture {
    let fixture = Fixture::git();
    declare_workbench(&fixture).link_package();
    fixture
}

/// Every file and link under the project, outside `.git` and the fixture's programs, with its
/// contents or, for a link, its target.
pub fn snapshot(fixture: &Fixture) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    collect(fixture.root(), fixture.root(), &mut files);
    files
}

fn collect(root: &Path, directory: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
    for entry in fs::read_dir(directory).expect("a readable directory") {
        let path = entry.expect("a directory entry").path();
        let relative = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        if relative == ".git" || relative == ".fixture-bin" {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).unwrap();
        if metadata.file_type().is_symlink() {
            let target = fs::read_link(&path).unwrap();
            files.insert(relative, format!("-> {}", target.display()).into_bytes());
        } else if metadata.is_dir() {
            collect(root, &path, files);
        } else {
            files.insert(relative, fs::read(&path).unwrap());
        }
    }
}

/// Whether every line of `before` appears in `after`, in order: entries were only added.
pub fn only_added_lines(before: &str, after: &str) -> bool {
    let mut remaining = after.lines();
    before
        .lines()
        .all(|line| remaining.by_ref().any(|candidate| candidate == line))
}

pub fn paths(items: &Value) -> Vec<String> {
    items
        .as_array()
        .expect("an array")
        .iter()
        .map(|item| match item {
            Value::String(path) => path.clone(),
            other => other["path"].as_str().expect("a path").to_string(),
        })
        .collect()
}

pub fn toml(fixture: &Fixture, relative: &str) -> toml::Table {
    fixture
        .read(relative)
        .parse()
        .unwrap_or_else(|error| panic!("{relative} is not TOML: {error}"))
}

pub fn json(fixture: &Fixture, relative: &str) -> Value {
    serde_json::from_str(&fixture.read(relative))
        .unwrap_or_else(|error| panic!("{relative} is not JSON: {error}"))
}
