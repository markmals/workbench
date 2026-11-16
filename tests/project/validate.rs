//! `workbench project validate`.

use std::fs;

use crate::fixtures::complete_setup;
use crate::support::{Fixture, assert_matches_output_schema, package_version, plugin_dir};

/// Runs `project validate` and returns its violations as `(path, message)`, checking that a
/// non-empty list fails with CHECK_FAILED and an empty one passes.
fn violations(fixture: &Fixture) -> Vec<(String, String)> {
    let outcome = fixture.run(&["project", "validate"]);
    let result = outcome.json();
    assert_matches_output_schema(&["project", "validate"], &result);
    let violations: Vec<(String, String)> = result["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| {
            (
                violation["path"].as_str().unwrap().to_string(),
                violation["message"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(result["count"], violations.len());
    if violations.is_empty() {
        assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    } else {
        assert_eq!(outcome.status, 1, "{}", outcome.stderr);
        assert_eq!(outcome.error_code(), "CHECK_FAILED");
    }
    violations
}

fn violation_paths(fixture: &Fixture) -> Vec<String> {
    violations(fixture)
        .into_iter()
        .map(|(path, _)| path)
        .collect()
}

fn message_for(fixture: &Fixture, path: &str) -> String {
    violations(fixture)
        .into_iter()
        .find(|(found, _)| found == path)
        .unwrap_or_else(|| panic!("no violation for {path}"))
        .1
}

#[test]
fn validate_passes_a_complete_setup() {
    assert_eq!(violations(&complete_setup()), []);
}

#[test]
fn validate_reports_a_missing_pin_with_the_fix() {
    let fixture = complete_setup();
    fixture.write("mise.toml", "[tools]\nhk = \"2.8.1\"\n");
    assert_eq!(violation_paths(&fixture), ["mise.toml"]);
    let message = message_for(&fixture, "mise.toml");
    assert!(
        message.contains("mise use github:markmals/workbench@"),
        "{message}"
    );
}

#[test]
fn validate_reports_a_link_whose_version_differs_from_the_pin() {
    let fixture = complete_setup();
    fixture.write(
        "mise.toml",
        "[tools]\n\"github:markmals/workbench\" = \"0.0.1\"\n",
    );
    assert_eq!(violation_paths(&fixture), [".workbench"]);
    let message = message_for(&fixture, ".workbench");
    assert!(message.contains("0.0.1"), "{message}");
    assert!(message.contains(&package_version()), "{message}");
    assert!(message.contains("mise install"), "{message}");
}

#[test]
fn validate_reads_a_pin_written_as_a_table() {
    let fixture = complete_setup();
    fixture.write(
        "mise.toml",
        &format!(
            "[tools]\n\"github:markmals/workbench\" = {{ version = \"{}\" }}\n",
            package_version()
        ),
    );
    assert_eq!(violations(&fixture), []);
}

#[test]
fn validate_reports_a_missing_or_dangling_link() {
    let fixture = Fixture::git();
    crate::support::declare_workbench(&fixture);
    assert_eq!(violation_paths(&fixture), [".workbench"]);
    assert!(message_for(&fixture, ".workbench").contains("mise install"));

    std::os::unix::fs::symlink("/nonexistent/workbench", fixture.path(".workbench")).unwrap();
    assert_eq!(violation_paths(&fixture), [".workbench"]);
    assert!(message_for(&fixture, ".workbench").contains("dangling"));
}

#[test]
fn validate_exempts_workbench_s_own_repository_from_the_pin_check() {
    let fixture = Fixture::git();
    crate::support::declare_workbench(&fixture);
    fixture
        .write("mise.toml", "[tools]\nrust = \"1.100.0\"\n")
        .write(
            "Cargo.toml",
            "[package]\nname = \"workbench\"\nversion = \"9.9.9\"\n",
        )
        .write(
            "plugin/.claude-plugin/plugin.json",
            "{ \"name\": \"workbench\", \"version\": \"9.9.9\" }\n",
        );
    std::os::unix::fs::symlink("plugin", fixture.path(".workbench")).unwrap();
    assert_eq!(violations(&fixture), []);

    // Another crate linking its own plugin/ is not Workbench.
    fixture.write("Cargo.toml", "[package]\nname = \"other\"\n");
    assert_eq!(violation_paths(&fixture), ["mise.toml"]);
}

#[test]
fn validate_reports_missing_and_incomplete_harness_declarations() {
    let fixture = complete_setup();
    fs::remove_file(fixture.path(".claude/settings.json")).unwrap();
    fixture.write(".omp/config.yml", "extensions:\n    - ./other\n");
    assert_eq!(
        violation_paths(&fixture),
        [".claude/settings.json", ".omp/config.yml"]
    );
    assert!(message_for(&fixture, ".omp/config.yml").contains("./.workbench"));

    fixture.write(".claude/settings.json", "{ \"enabledPlugins\": {} }\n");
    let message = message_for(&fixture, ".claude/settings.json");
    assert!(message.contains("workbench@workbench"), "{message}");
    assert!(message.contains("extraKnownMarketplaces"), "{message}");

    fixture.write(".claude/settings.json", "{ not json");
    assert!(message_for(&fixture, ".claude/settings.json").contains("JSON"));
}

#[test]
fn validate_reports_skill_and_rule_name_collisions() {
    let fixture = complete_setup();
    fixture
        .write(
            ".agents/skills/writing-a-proposal/SKILL.md",
            "---\nname: writing-a-proposal\n---\n",
        )
        .write(
            ".agents/skills/our-skill/SKILL.md",
            "---\nname: our-skill\n---\n",
        )
        .write(".agents/rules/verification.md", "# Ours\n")
        .write(".agents/rules/our-rule.md", "# Ours\n");
    assert_eq!(
        violation_paths(&fixture),
        [
            ".agents/skills/writing-a-proposal",
            ".agents/rules/verification.md"
        ]
    );
    for path in [
        ".agents/skills/writing-a-proposal",
        ".agents/rules/verification.md",
    ] {
        let message = message_for(&fixture, path);
        assert!(message.contains(path), "{message}");
        assert!(
            message.contains("remove") && message.contains("rename"),
            "{message}"
        );
    }
}

#[test]
fn validate_reports_vendored_files_and_allows_template_overrides() {
    let fixture = complete_setup();
    let proposal = fs::read_to_string(plugin_dir().join("templates/PROPOSAL.md")).unwrap();
    fixture
        .write(".agents/templates/PROPOSAL.md", &proposal)
        .write(".agents/templates/GUIDE.md", "# Our guide template\n")
        .write(".agents/templates/OURS.md", "# Only ours\n")
        .write(".omp/extensions/check-before-stop.ts", "export {};\n")
        .write(".omp/extensions/format-after-edit.ts", "export {};\n")
        .write(".omp/extensions/our-extension.ts", "export {};\n")
        .write(".omp/tools/dynamic-workflows/index.ts", "export {};\n")
        .write("tools/check-pr-commits.ts", "export {};\n")
        .write("tools/status.collection.test.ts", "export {};\n")
        .write("tools/our-tool.ts", "export {};\n")
        .write(".changeset/config.json", "{}\n")
        .write("docs/.vitepress/config.ts", "export default {};\n")
        .write("fnox.toml", "\n");
    assert_eq!(
        violation_paths(&fixture),
        [
            ".agents/templates/PROPOSAL.md",
            ".omp/extensions/check-before-stop.ts",
            ".omp/extensions/format-after-edit.ts",
            ".omp/tools/dynamic-workflows",
            ".changeset/config.json",
            "docs/.vitepress",
            "fnox.toml",
            "tools/check-pr-commits.ts",
            "tools/status.collection.test.ts",
        ]
    );
    assert!(message_for(&fixture, ".agents/templates/PROPOSAL.md").contains("identical"));
}

#[test]
fn validate_compares_templates_with_built_in_copies_without_a_link() {
    let fixture = Fixture::git();
    crate::support::declare_workbench(&fixture);
    let vision = fs::read_to_string(plugin_dir().join("templates/VISION.md")).unwrap();
    fixture.write(".agents/templates/VISION.md", &vision);
    assert_eq!(
        violation_paths(&fixture),
        [".workbench", ".agents/templates/VISION.md"]
    );
}

#[test]
fn validate_renders_violations_as_text() {
    let fixture = complete_setup();
    fixture.write("fnox.toml", "\n");
    let outcome = fixture.run(&["project", "validate", "--output", "text"]);
    assert_eq!(outcome.status, 1);
    let lines: Vec<&str> = outcome.stdout.lines().collect();
    assert_eq!(lines.len(), 2, "{}", outcome.stdout);
    assert!(lines[0].starts_with("fnox.toml: "));
    assert_eq!(lines[1], "1 violation");
}
