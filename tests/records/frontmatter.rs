//! The frontmatter records use: scalar values and inline lists.

use workbench::frontmatter::{Frontmatter, parse};

use super::fixtures::{list, scalar};

#[test]
fn frontmatter_skips_blank_and_comment_lines_but_keeps_literal_hashes() {
    let source = [
        "---",
        "",
        "  # A YAML comment",
        "\t",
        "id: proposal.0001",
        "title: A title # with a literal hash",
        "authors: [Ada, Grace]",
        "---",
        "Body",
    ]
    .join("\n");

    let expected: Frontmatter = [
        ("id".to_string(), scalar("proposal.0001")),
        ("title".to_string(), scalar("A title # with a literal hash")),
        ("authors".to_string(), list(&["Ada", "Grace"])),
    ]
    .into_iter()
    .collect();
    assert_eq!(parse(&source), Ok(expected));
}

#[test]
fn frontmatter_rejects_malformed_input_and_duplicate_keys() {
    assert_eq!(
        parse("title: Missing delimiters"),
        Err("missing YAML frontmatter".to_string())
    );
    assert_eq!(
        parse("---\ntitle: First\ntitle: Second\n---"),
        Err(r#"duplicate frontmatter key "title""#.to_string())
    );
    assert_eq!(
        parse("---\nnot valid\n---"),
        Err(r#"cannot parse frontmatter line "not valid""#.to_string())
    );
}
