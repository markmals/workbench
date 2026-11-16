//! IDs, the references between records, and proposal supersession.

use workbench::commands::records::validate::validate;

use super::fixtures::{
    artifact, list, scalar, set, valid_artifacts, violation, violations_mentioning,
};

#[test]
fn ids_must_match_filenames_and_be_unique() {
    let mut records = valid_artifacts();
    set(&mut records[0], "id", scalar("proposal.9999"));
    set(&mut records[1], "id", scalar("policy.0001"));

    assert_eq!(
        violations_mentioning(&records, "ID"),
        vec![
            violation(
                "proposals/0001-records.md",
                r#"ID "proposal.9999" must be "proposal.0001" for this filename"#
            ),
            violation(
                "proposals/0002-replaces.md",
                r#"ID "policy.0001" must be "proposal.0002" for this filename"#
            ),
            violation(
                "proposals/0002-replaces.md",
                r#"duplicate ID "policy.0001"; first declared in policies/0001-records.md"#
            ),
        ]
    );
}

#[test]
fn a_guide_filename_must_be_kebab_case() {
    let mut records = valid_artifacts();
    records[4] = artifact(
        "guides/My Guide (DRAFT)!.md",
        &[
            ("id", scalar("guide.My Guide (DRAFT)!")),
            ("title", scalar("Record Validation Guide")),
        ],
        "",
    );

    let guide_violations: Vec<_> = validate(&records)
        .into_iter()
        .filter(|violation| violation.path.starts_with("guides/"))
        .collect();
    assert_eq!(
        guide_violations,
        vec![violation(
            "guides/My Guide (DRAFT)!.md",
            "guide filename must be kebab-case"
        )]
    );
}

#[test]
fn an_unresolved_reference_is_a_violation() {
    let mut records = valid_artifacts();
    set(&mut records[2], "established-by", scalar("proposal.9999"));
    assert_eq!(
        violations_mentioning(&records, "does not resolve"),
        vec![violation(
            "policies/0001-records.md",
            r#"reference "proposal.9999" in established-by does not resolve"#
        )]
    );
}

#[test]
fn a_superseded_proposal_must_have_status_superseded() {
    let mut records = valid_artifacts();
    set(&mut records[0], "status", scalar("implemented"));
    assert_eq!(
        violations_mentioning(&records, "must have status"),
        vec![violation(
            "proposals/0001-records.md",
            r#"record superseded by proposal.0002 must have status "superseded""#
        )]
    );
}

#[test]
fn a_proposal_cannot_supersede_itself() {
    let mut records = valid_artifacts();
    set(&mut records[1], "status", scalar("superseded"));
    set(&mut records[1], "supersedes", list(&["proposal.0002"]));
    assert_eq!(
        violations_mentioning(&records, "supersession cycle"),
        vec![violation(
            "proposals/0002-replaces.md",
            "supersession cycle: proposal.0002 -> proposal.0002"
        )]
    );
}

#[test]
fn supersession_cannot_form_a_cycle() {
    let mut records = valid_artifacts();
    set(&mut records[0], "supersedes", list(&["proposal.0002"]));
    set(&mut records[1], "status", scalar("superseded"));
    assert_eq!(
        violations_mentioning(&records, "supersession cycle"),
        vec![violation(
            "proposals/0001-records.md",
            "supersession cycle: proposal.0001 -> proposal.0002 -> proposal.0001"
        )]
    );
}
