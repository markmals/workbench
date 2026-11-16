//! The project a command acts on: the Git top level of the current directory.

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::failure::Failure;
use crate::process::{Program, RunError};

pub struct Project {
    pub root: PathBuf,
}

impl Project {
    /// Resolves the project from `cwd`, never from the location of the `workbench` binary.
    pub fn discover(cwd: &Path) -> Result<Project, Failure> {
        let run = Program::new("git", cwd)
            .args(["rev-parse", "--show-toplevel"])
            .run()
            .map_err(|error| match error {
                RunError::Missing => Failure::dependency_missing("git"),
                RunError::TimedOut => Failure::internal("git rev-parse timed out"),
                RunError::Spawn(reason) => Failure::internal(format!("cannot run git: {reason}")),
            })?;
        if !run.success() {
            return Err(Failure::precondition(format!(
                "{} is not inside a Git repository",
                cwd.display()
            ))
            .with_details(json!({ "directory": cwd.display().to_string() }))
            .suggest("Run workbench from inside the project's Git checkout"));
        }
        Ok(Project {
            root: PathBuf::from(run.stdout.trim_end()),
        })
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    /// The package link: `.workbench` at the project root.
    pub fn workbench_link(&self) -> PathBuf {
        self.root.join(".workbench")
    }

    /// The installed package the link points at, or a failure naming the fix.
    pub fn package_root(&self) -> Result<PathBuf, Failure> {
        let link = self.workbench_link();
        match link.canonicalize() {
            Ok(target) if target.is_dir() => Ok(target),
            _ => Err(Failure::precondition(
                "the Workbench package link .workbench is missing or dangling",
            )
            .with_details(json!({ "link": link.display().to_string() }))
            .suggest("Run `mise install`, which re-links .workbench to the pinned release")),
        }
    }
}
