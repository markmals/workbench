//! Making the writes a plan lists. A file to create is opened with `create_new`, so a file that
//! appeared since the plan was made is never overwritten.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::Notice;
use super::plan::Operation;

/// Performs every operation, continuing past failures, and returns the ones that failed.
pub fn apply(root: &Path, operations: &[Operation]) -> Vec<Notice> {
    operations
        .iter()
        .filter_map(|operation| {
            perform(root, operation).err().map(|error| Notice {
                path: operation.path().to_string(),
                message: format!("it could not be written: {error}"),
            })
        })
        .collect()
}

fn perform(root: &Path, operation: &Operation) -> io::Result<()> {
    let path = root.join(operation.path());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    match operation {
        Operation::Create { contents, .. } => OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?
            .write_all(contents.as_bytes()),
        Operation::AddTo { contents, .. } => fs::write(&path, contents),
        Operation::Link { target, .. } => std::os::unix::fs::symlink(target, &path),
    }
}
