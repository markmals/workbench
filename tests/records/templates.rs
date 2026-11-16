//! The record templates the package ships must produce records the validator accepts.

use std::fs;
use std::path::PathBuf;

use workbench::commands::records::validate::{Artifact, validate};
use workbench::frontmatter::parse;

fn templates_dir() -> PathBuf {
    crate::support::plugin_dir().join("templates")
}

fn read_template(name: &str) -> String {
    fs::read_to_string(templates_dir().join(name))
        .unwrap_or_else(|error| panic!("cannot read template {name}: {error}"))
}

#[test]
fn every_record_template_has_parsable_frontmatter() {
    let mut with_frontmatter: Vec<(String, String)> = fs::read_dir(templates_dir())
        .expect("the package has templates")
        .map(|entry| entry.expect("a readable template entry"))
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let source = read_template(&name);
            (name, source)
        })
        .filter(|(_, source)| source.starts_with("---"))
        .collect();
    with_frontmatter.sort();

    let names: Vec<&str> = with_frontmatter
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "DECISION.md",
            "GUIDE.md",
            "POLICY.md",
            "PROPOSAL.md",
            "VISION.md"
        ]
    );
    for (name, source) in &with_frontmatter {
        if let Err(error) = parse(source) {
            panic!("{name}: {error}");
        }
    }
}

#[test]
fn a_filled_in_proposal_template_validates() {
    let source = read_template("PROPOSAL.md")
        .replace("id: proposal.<NNNN>", "id: proposal.0001")
        .replace("title: <Title Case>", "title: Template Validation")
        .replace("authors: [<name>]", "authors: [Ada]")
        .replace(
            "pull-request: <url or number, empty while unopened>",
            "pull-request: 42",
        );
    let frontmatter = parse(&source).expect("the filled-in template parses");

    let proposal = Artifact {
        path: "proposals/0001-template.md".into(),
        frontmatter,
        content: source,
    };
    assert_eq!(validate(&[proposal]), vec![]);
}
