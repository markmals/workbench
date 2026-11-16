//! Fixture projects for running the real `workbench` binary in tests.
#![allow(dead_code)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

pub fn plugin_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("plugin")
}

/// A temporary directory standing in for a project, with a private `bin/` for fake programs.
pub struct Fixture {
    dir: TempDir,
    env: Vec<(String, String)>,
}

/// What one run of `workbench` produced.
#[derive(Debug)]
pub struct Outcome {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Outcome {
    /// The result on stdout, which must be a single JSON document.
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.stdout).unwrap_or_else(|error| {
            panic!(
                "stdout is not JSON ({error}):\n{}\nstderr:\n{}",
                self.stdout, self.stderr
            )
        })
    }

    /// The `error` object on stderr.
    pub fn error(&self) -> Value {
        let document: Value = serde_json::from_str(self.stderr.trim()).unwrap_or_else(|error| {
            panic!(
                "stderr is not one JSON document ({error}):\n{}",
                self.stderr
            )
        });
        document["error"].clone()
    }

    pub fn error_code(&self) -> String {
        self.error()["code"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }
}

impl Fixture {
    /// An empty directory that is not a Git repository.
    pub fn empty() -> Fixture {
        let dir = tempfile::Builder::new()
            .prefix("workbench-test-")
            .tempdir()
            .expect("a temporary directory");
        fs::create_dir_all(dir.path().join(".fixture-bin")).expect("a bin directory");
        Fixture {
            dir,
            env: Vec::new(),
        }
    }

    /// A Git repository on `main` with one commit.
    pub fn git() -> Fixture {
        let fixture = Fixture::empty();
        fixture.git_command(&["init", "--quiet", "--initial-branch=main"]);
        fixture.write(".gitignore", ".fixture-bin/\n.workbench\n");
        fixture.commit_all("chore: start");
        fixture
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root().join(relative)
    }

    pub fn write(&self, relative: &str, contents: &str) -> &Fixture {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().expect("a file has a parent"))
            .expect("parent directories");
        fs::write(path, contents).expect("a writable fixture");
        self
    }

    pub fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative))
            .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
    }

    /// Links `.workbench` to this working tree's `plugin/`, as `mise install` links the release.
    pub fn link_package(&self) -> &Fixture {
        std::os::unix::fs::symlink(plugin_dir(), self.path(".workbench"))
            .expect("a .workbench link");
        self
    }

    /// Installs an executable shell script as `name` on the PATH `workbench` sees.
    pub fn fake_program(&self, name: &str, script: &str) -> &Fixture {
        let path = self.root().join(".fixture-bin").join(name);
        fs::write(&path, format!("#!/bin/sh\n{script}\n")).expect("a fake program");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("an executable fake");
        self
    }

    pub fn set_env(&mut self, key: &str, value: &str) -> &mut Fixture {
        self.env.push((key.to_string(), value.to_string()));
        self
    }

    pub fn git_command(&self, args: &[&str]) -> String {
        let output = isolated_git(self.root())
            .args(args)
            .output()
            .expect("git runs");
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    pub fn commit_all(&self, message: &str) {
        self.git_command(&["add", "--all"]);
        self.git_command(&["commit", "--quiet", "--allow-empty", "--message", message]);
    }

    pub fn run(&self, args: &[&str]) -> Outcome {
        self.run_in(".", args)
    }

    pub fn run_in(&self, directory: &str, args: &[&str]) -> Outcome {
        let path = format!(
            "{}:{}",
            self.root().join(".fixture-bin").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let mut command = Command::new(env!("CARGO_BIN_EXE_workbench"));
        command
            .args(args)
            .current_dir(self.path(directory))
            .env("PATH", path);
        isolate_git_environment(&mut command);
        for (key, value) in &self.env {
            command.env(key, value);
        }
        let output = command.output().expect("workbench runs");
        Outcome {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

fn isolated_git(directory: &Path) -> Command {
    let mut command = Command::new("git");
    command.current_dir(directory);
    isolate_git_environment(&mut command);
    command
}

/// Keeps the developer's Git configuration out of fixtures: no signing, hooks, or identity.
fn isolate_git_environment(command: &mut Command) {
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "Fixture Author")
        .env("GIT_AUTHOR_EMAIL", "author@example.com")
        .env("GIT_COMMITTER_NAME", "Fixture Author")
        .env("GIT_COMMITTER_EMAIL", "author@example.com");
}

/// Fails the test when `result` does not match the output schema `workbench schema` declares
/// for the command at `path`.
pub fn assert_matches_output_schema(path: &[&str], result: &Value) {
    let fixture = Fixture::empty();
    let mut args = vec!["schema"];
    args.extend_from_slice(path);
    let description = fixture.run(&args).json();
    let schema = &description["output"];
    let validator = jsonschema::validator_for(schema).unwrap_or_else(|error| {
        panic!(
            "{} declares an invalid output schema: {error}",
            path.join(" ")
        )
    });
    let problems: Vec<String> = validator
        .iter_errors(result)
        .map(|error| format!("{} at {}", error, error.instance_path()))
        .collect();
    assert!(
        problems.is_empty(),
        "output of `workbench {}` does not match its schema:\n{}\n{}",
        path.join(" "),
        problems.join("\n"),
        serde_json::to_string_pretty(result).unwrap_or_default()
    );
}

/// The project every registered example runs in: a Workbench project with records, a package
/// link, and fake `bd` and `gh` programs. Extend it when a new example needs more.
pub fn example_project() -> Fixture {
    let fixture = Fixture::git();
    fixture.link_package();
    fixture
        .write(
            "VISION.md",
            "---\ntitle: Example Vision\nupdated: 2026-11-15\n---\n\n# Example Vision\n",
        )
        .write("proposals/README.md", "# Proposals\n")
        .write("guides/README.md", "# Guides\n")
        .write(
            "guides/getting-started.md",
            "---\nid: guide.getting-started\ntitle: Getting Started\n---\n\n# Getting Started\n",
        );
    fixture.commit_all("docs: add the example records");
    fixture
}
