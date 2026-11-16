//! Records the names of the skills and rules this release ships, for `workbench project validate`.

use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=plugin/skills");
    println!("cargo:rerun-if-changed=plugin/rules");

    let skills = names(
        Path::new("plugin/skills"),
        |entry| entry.path().is_dir(),
        |name| name.to_string(),
    );
    let rules = names(
        Path::new("plugin/rules"),
        |entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "md")
        },
        |name| name.trim_end_matches(".md").to_string(),
    );

    let source = format!(
        "/// The skills this release ships, by directory name.\npub const SKILLS: &[&str] = &{skills:?};\n\
         /// The rules this release ships, by file name without `.md`.\npub const RULES: &[&str] = &{rules:?};\n"
    );
    let out =
        Path::new(&std::env::var("OUT_DIR").expect("cargo sets OUT_DIR")).join("package_names.rs");
    fs::write(out, source).expect("OUT_DIR is writable");
}

fn names(
    directory: &Path,
    keep: impl Fn(&fs::DirEntry) -> bool,
    name: impl Fn(&str) -> String,
) -> Vec<String> {
    let entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));
    let mut names: Vec<String> = entries
        .map(|entry| entry.expect("a readable directory entry"))
        .filter(|entry| keep(entry))
        .map(|entry| name(&entry.file_name().to_string_lossy()))
        .collect();
    names.sort();
    names
}
