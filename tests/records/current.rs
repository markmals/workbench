//! Policies, decisions, and guides state what holds now: they change in place or are retired.

use workbench::commands::records::validate::{Artifact, validate};

use super::fixtures::{artifact, list, scalar, set, violation};

const CURRENT_KINDS: [(&str, &str, [&str; 2]); 2] = [
    ("policy", "policies", ["draft", "active"]),
    ("decision", "decisions", ["proposed", "accepted"]),
];

fn current_record(kind: &str, directory: &str, status: &str) -> Artifact {
    artifact(
        &format!("{directory}/0001-current.md"),
        &[
            ("id", scalar(&format!("{kind}.0001"))),
            ("title", scalar("Current contract")),
            ("status", scalar(status)),
        ],
        "The current rule or choice.",
    )
}

#[test]
fn a_current_record_can_be_revised_in_place_without_provenance() {
    for (kind, directory, statuses) in CURRENT_KINDS {
        for status in statuses {
            let mut current = current_record(kind, directory, status);
            assert_eq!(validate(&[current.clone()]), vec![], "{kind} {status}");
            current.content = "The revised rule or choice, under the same identity.".into();
            assert_eq!(validate(&[current]), vec![], "revised {kind} {status}");
        }
    }
}

#[test]
fn supersedes_on_a_current_record_gives_migration_guidance() {
    for (kind, directory, [_, settled]) in CURRENT_KINDS {
        let mut current = current_record(kind, directory, settled);
        set(&mut current, "supersedes", list(&[]));
        let violations = validate(&[current.clone()]);
        assert_eq!(violations.len(), 1, "{kind}: {violations:?}");
        assert_eq!(violations[0].path, current.path);
        let message = &violations[0].message;
        assert!(message.contains("supersedes"), "{kind}: {message}");
        assert!(
            ["revise", "retire", "remove"]
                .iter()
                .any(|word| message.to_lowercase().contains(word)),
            "{kind}: {message}"
        );
    }
}

#[test]
fn superseded_status_on_a_current_record_explains_retirement() {
    for (kind, directory, _) in CURRENT_KINDS {
        let violations = validate(&[current_record(kind, directory, "superseded")]);
        assert_eq!(violations.len(), 1, "{kind}: {violations:?}");
        let message = &violations[0].message;
        assert!(message.contains("superseded"), "{kind}: {message}");
        assert!(
            ["retire", "delete", "remove"]
                .iter()
                .any(|word| message.to_lowercase().contains(word)),
            "{kind}: {message}"
        );
    }
}

#[test]
fn current_record_provenance_must_be_a_scalar_reference() {
    for (kind, directory, [_, settled]) in CURRENT_KINDS {
        let mut current = current_record(kind, directory, settled);
        set(&mut current, "established-by", list(&["proposal.0001"]));
        assert_eq!(
            validate(&[current.clone()]),
            vec![violation(
                &current.path,
                r#"key "established-by" must be a scalar"#
            )]
        );

        set(&mut current, "established-by", scalar("proposal.9999"));
        assert_eq!(
            validate(&[current.clone()]),
            vec![violation(
                &current.path,
                r#"reference "proposal.9999" in established-by does not resolve"#
            )]
        );
    }
}

#[test]
fn current_record_provenance_must_name_a_proposal() {
    for (kind, directory, [_, settled]) in CURRENT_KINDS {
        let mut current = current_record(kind, directory, settled);
        set(
            &mut current,
            "established-by",
            scalar(&format!("{kind}.0001")),
        );
        assert_eq!(
            validate(&[current.clone()]),
            vec![violation(
                &current.path,
                &format!(r#"reference "{kind}.0001" in established-by must name a proposal"#)
            )]
        );
    }
}

#[test]
fn a_guide_rejects_describes_in_favor_of_links() {
    let mut guide = artifact(
        "guides/current-contract.md",
        &[
            ("id", scalar("guide.current-contract")),
            ("title", scalar("Current contract")),
        ],
        "See [the current rule](../policies/0001-current.md).",
    );
    assert_eq!(validate(&[guide.clone()]), vec![]);

    set(&mut guide, "describes", list(&["proposal.0001"]));
    let violations = validate(&[guide.clone()]);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].path, guide.path);
    let message = &violations[0].message;
    assert!(message.contains("describes"), "{message}");
    assert!(message.to_lowercase().contains("links"), "{message}");
}
