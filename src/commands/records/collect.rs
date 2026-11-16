//! Reading the records from the project's working tree.

use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

use serde_json::json;

use crate::failure::Failure;
use crate::frontmatter;

use super::kind::{DIRECTORIES, VISION_FILE};
use super::validate::{Artifact, Violation};

/// The records read from disk, and the files that could not become one.
#[derive(Default)]
pub struct Collected {
    pub artifacts: Vec<Artifact>,
    /// Nested record files and files whose frontmatter does not parse.
    pub violations: Vec<Violation>,
}

/// Reads `VISION.md` and the Markdown files directly inside each record directory under `root`,
/// skipping directory READMEs. Fails when a file or directory that exists cannot be read.
pub fn collect(root: &Path) -> Result<Collected, Failure> {
    let mut collected = Collected::default();
    let vision = root.join(VISION_FILE);
    if vision.exists() {
        collected.add(&vision, VISION_FILE.to_string())?;
    }
    for (directory, _) in DIRECTORIES {
        let absolute = root.join(directory);
        if absolute.exists() {
            collected.read_directory(&absolute, directory, directory)?;
        }
    }
    Ok(collected)
}

impl Collected {
    /// Reads the Markdown files in `absolute`, known to the user as `relative`, inside the record
    /// directory `top`. Any Markdown below `top` itself is a nested record file.
    fn read_directory(
        &mut self,
        absolute: &Path,
        top: &str,
        relative: &str,
    ) -> Result<(), Failure> {
        for entry in sorted_entries(absolute).map_err(|error| unreadable(relative, &error))? {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = format!("{relative}/{name}");
            let file_type = entry
                .file_type()
                .map_err(|error| unreadable(&path, &error))?;
            // A symlinked directory reports as a symlink, so it is never traversed.
            if file_type.is_dir() {
                self.read_directory(&entry.path(), top, &path)?;
            } else if name.ends_with(".md") && is_regular_file(&entry, file_type) {
                if relative != top {
                    self.violations.push(Violation {
                        message: format!("nested record file; move it to {top}/ or remove it"),
                        path,
                    });
                } else if name != "README.md" {
                    self.add(&entry.path(), path)?;
                }
            }
        }
        Ok(())
    }

    fn add(&mut self, absolute: &Path, path: String) -> Result<(), Failure> {
        let bytes = fs::read(absolute).map_err(|error| unreadable(&path, &error))?;
        let content = String::from_utf8_lossy(&bytes).into_owned();
        match frontmatter::parse(&content) {
            Ok(frontmatter) => self.artifacts.push(Artifact {
                path,
                frontmatter,
                content,
            }),
            Err(message) => self.violations.push(Violation { path, message }),
        }
        Ok(())
    }
}

fn sorted_entries(directory: &Path) -> io::Result<Vec<DirEntry>> {
    let mut entries = fs::read_dir(directory)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(DirEntry::file_name);
    Ok(entries)
}

/// A regular file, or a symlink to one. A dangling symlink is neither.
fn is_regular_file(entry: &DirEntry, file_type: fs::FileType) -> bool {
    file_type.is_file()
        || (file_type.is_symlink()
            && fs::metadata(entry.path()).is_ok_and(|metadata| metadata.is_file()))
}

fn unreadable(path: &str, error: &io::Error) -> Failure {
    Failure::precondition(format!("cannot read {path}: {error}"))
        .with_details(json!({ "path": path }))
        .suggest(format!(
            "Make {path} a readable file or directory, then run `workbench records validate` again"
        ))
}
