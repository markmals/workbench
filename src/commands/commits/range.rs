//! A commit range, `<base>..<head>`, read through Git.

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::failure::Failure;
use crate::process::{Program, Run, RunError};

/// Separates the fields of one `git log` record; records are separated by NUL (`-z`).
const FIELD: char = '\u{1f}';

/// The commits reachable from `head` and not from `base`, with both revisions resolved.
pub struct Range {
    root: PathBuf,
    base: String,
    head: String,
}

impl Range {
    /// Resolves both revisions to commits; an unknown one is NOT_FOUND.
    pub fn resolve(root: &Path, base: &str, head: &str) -> Result<Range, Failure> {
        Ok(Range {
            root: root.to_path_buf(),
            base: resolve_commit(root, base)?,
            head: resolve_commit(root, head)?,
        })
    }

    /// Each commit's full hash and subject, oldest first.
    pub fn subjects(&self) -> Result<Vec<(String, String)>, Failure> {
        let log = self.log("%H%x1f%s")?;
        Ok(records(&log)
            .filter_map(|record| record.split_once(FIELD))
            .map(|(commit, subject)| (commit.to_string(), subject.to_string()))
            .collect())
    }

    /// The commits, oldest first, carrying the trailer `key: value`. Git parses the trailers
    /// as `git interpret-trailers --parse` does, matching `key` case-insensitively; the value
    /// must match exactly.
    pub fn commits_with_trailer(&self, key: &str, value: &str) -> Result<Vec<String>, Failure> {
        let format = format!("%H%x1f%(trailers:key={key},valueonly,unfold,separator=%x1f)");
        let log = self.log(&format)?;
        Ok(records(&log)
            .filter_map(|record| {
                let mut fields = record.split(FIELD);
                let commit = fields.next()?;
                fields
                    .any(|found| found == value)
                    .then(|| commit.to_string())
            })
            .collect())
    }

    /// The paths under `directory` that `head` adds since its merge base with `base`, so files
    /// `base` gained or lost after the branch point do not count.
    pub fn added_files(&self, directory: &str) -> Result<Vec<String>, Failure> {
        let merge_base = git(&self.root, &["merge-base", &self.base, &self.head])?;
        match merge_base.status {
            Some(0) => {}
            // `git merge-base` exits 1, saying nothing, when the commits share no history.
            Some(1) => {
                return Err(Failure::invalid_argument(
                    "the base and head revisions share no history",
                )
                .with_details(json!({ "base": self.base, "head": self.head }))
                .suggest("Pass a base the head branched from, such as the pull request's target"));
            }
            _ => {
                return Err(Failure::internal(format!(
                    "git merge-base failed: {}",
                    merge_base.stderr.trim()
                )));
            }
        }
        let diff = git_stdout(
            &self.root,
            &[
                "diff",
                "--name-only",
                "-z",
                "--diff-filter=A",
                merge_base.stdout.trim(),
                &self.head,
                "--",
                directory,
            ],
        )?;
        Ok(records(&diff).map(str::to_string).collect())
    }

    /// The file at `path` as committed at `head`.
    pub fn read_at_head(&self, path: &str) -> Result<String, Failure> {
        git_stdout(
            &self.root,
            &["cat-file", "blob", &format!("{}:{path}", self.head)],
        )
    }

    fn log(&self, format: &str) -> Result<String, Failure> {
        git_stdout(
            &self.root,
            &[
                "log",
                "-z",
                "--reverse",
                &format!("--format={format}"),
                &format!("{}..{}", self.base, self.head),
            ],
        )
    }
}

fn records(output: &str) -> impl Iterator<Item = &str> {
    output.split('\0').filter(|record| !record.is_empty())
}

fn resolve_commit(root: &Path, revision: &str) -> Result<String, Failure> {
    let run = git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            "--end-of-options",
            &format!("{revision}^{{commit}}"),
        ],
    )?;
    if !run.success() {
        return Err(
            Failure::not_found(format!("no commit `{revision}` in this repository"))
                .with_details(json!({ "revision": revision }))
                .suggest("Name an existing branch, tag, or commit; fetch it first if it lives on a remote"),
        );
    }
    Ok(run.stdout.trim().to_string())
}

fn git(root: &Path, args: &[&str]) -> Result<Run, Failure> {
    Program::new("git", root)
        .args(args.iter().copied())
        .run()
        .map_err(|error| match error {
            RunError::Missing => Failure::dependency_missing("git"),
            RunError::TimedOut => Failure::internal(format!("git {} timed out", args[0])),
            RunError::Spawn(reason) => Failure::internal(format!("cannot run git: {reason}")),
        })
}

/// The output of a Git command that cannot fail once the range resolved.
fn git_stdout(root: &Path, args: &[&str]) -> Result<String, Failure> {
    let run = git(root, args)?;
    if !run.success() {
        return Err(Failure::internal(format!(
            "git {} failed: {}",
            args[0],
            run.stderr.trim()
        )));
    }
    Ok(run.stdout)
}
