//! `workbench project init` in a project made from the old Workbench template, which vendors
//! Workbench's files.

use std::fs;

use serde_json::Value;

use crate::fixtures::{only_added_lines, paths, snapshot};
use crate::support::{Fixture, assert_matches_output_schema, plugin_dir};

const MISE: &str = r#"# mise owns the task graph.
min_version = "2026.10.4"

[tools]
"github:markmals/workbench" = "3.0.0"
hk = "2.8.1"
node = "26.12.1"

[tasks.setup]
run = "pnpm install"

[tasks.status]
run = "node tools/status.ts"
"#;

/// A copy of a project created from the Workbench template before the package existed.
fn template_copy() -> Fixture {
    let fixture = Fixture::git();
    let template =
        |name: &str| fs::read_to_string(plugin_dir().join("templates").join(name)).unwrap();
    fixture
        .write("mise.toml", MISE)
        .write(".gitignore", ".fixture-bin/\nnode_modules\n")
        .write(
            "hk.pkl",
            "// commit-msg runs node tools/check-scoped-commit.ts\n",
        )
        .write("AGENTS.md", "# Our project\n\nThe contract.\n")
        .write("VISION.md", "# Our vision\n")
        .write("proposals/README.md", "# Proposals\n\nOurs.\n")
        .write(".agents/commit-scopes", "api\n")
        .write(
            ".agents/skills/writing-a-proposal/SKILL.md",
            "---\nname: writing-a-proposal\n---\n",
        )
        .write(
            ".agents/skills/our-skill/SKILL.md",
            "---\nname: our-skill\n---\n",
        )
        .write(".agents/rules/verification.md", "# Verification\n")
        .write(".agents/templates/PROPOSAL.md", &template("PROPOSAL.md"))
        .write(".agents/templates/GUIDE.md", "# Our guide template\n")
        .write(".omp/extensions/prime-beads.ts", "export {};\n")
        .write(".omp/tools/dynamic-workflows/index.ts", "export {};\n")
        .write("tools/validate.ts", "export {};\n")
        .write("tools/validate.test.ts", "export {};\n")
        .write("tools/beads-cli.ts", "export {};\n")
        .write("tools/our-script.ts", "export {};\n")
        .write(".changeset/config.json", "{}\n")
        .write("docs/.vitepress/config.ts", "export default {};\n")
        .write("fnox.toml", "[secrets]\n")
        .write(".github/workflows/ci.yml", "name: CI\n");
    fixture.commit_all("chore: start from the template");
    fixture
}

#[test]
fn init_changes_no_existing_file_in_a_copy_of_a_consumer() {
    let fixture = template_copy();
    let before = snapshot(&fixture);

    let outcome = fixture.run(&["project", "init"]);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_matches_output_schema(&["project", "init"], &receipt);

    let after = snapshot(&fixture);
    for (path, contents) in &before {
        let now = after
            .get(path)
            .unwrap_or_else(|| panic!("init deleted {path}"));
        if path == "mise.toml" || path == ".gitignore" {
            let (old, new) = (
                String::from_utf8_lossy(contents),
                String::from_utf8_lossy(now),
            );
            assert!(only_added_lines(&old, &new), "{path} lost lines:\n{new}");
        } else {
            assert_eq!(now, contents, "init changed {path}");
        }
    }
    assert_eq!(paths(&receipt["addedTo"]), ["mise.toml", ".gitignore"]);

    let vendored: Vec<(String, Value)> = receipt["vendored"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["path"].as_str().unwrap().to_string(),
                file["template"].clone(),
            )
        })
        .collect();
    let expected = [
        (".agents/skills/writing-a-proposal", Value::Null),
        (".agents/rules/verification.md", Value::Null),
        (".agents/templates/GUIDE.md", Value::from("different")),
        (".agents/templates/PROPOSAL.md", Value::from("identical")),
        (".omp/extensions/prime-beads.ts", Value::Null),
        (".omp/tools/dynamic-workflows", Value::Null),
        (".changeset/config.json", Value::Null),
        ("docs/.vitepress", Value::Null),
        ("fnox.toml", Value::Null),
        ("tools/beads-cli.ts", Value::Null),
        ("tools/validate.test.ts", Value::Null),
        ("tools/validate.ts", Value::Null),
    ];
    let expected: Vec<(String, Value)> = expected
        .into_iter()
        .map(|(path, template)| (path.to_string(), template))
        .collect();
    assert_eq!(vendored, expected);

    let missing: Vec<&str> = receipt["missing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|piece| piece["piece"].as_str().unwrap())
        .collect();
    assert_eq!(missing, ["check", "fmt", "release"]);

    let unchanged = paths(&receipt["unchanged"]);
    for path in [
        "hk.pkl",
        "AGENTS.md",
        "VISION.md",
        "proposals/README.md",
        ".agents/commit-scopes",
    ] {
        assert!(
            unchanged.contains(&path.to_string()),
            "{path} not unchanged: {unchanged:?}"
        );
    }
    let attention = receipt["attention"].to_string();
    assert!(attention.contains("AGENTS.md"), "{attention}");
    assert!(attention.contains("tasks.status"), "{attention}");

    let created = paths(&receipt["created"]);
    assert!(created.contains(&".claude/skills".to_string()));
    assert!(created.contains(&".claude/rules".to_string()));
    assert!(created.contains(&"policies/README.md".to_string()));
}
