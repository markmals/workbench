//! `workbench project init` in new repositories, and its structured-file additions.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use serde_json::json;

use crate::fixtures::{init, json, only_added_lines, paths, pinned_repository, snapshot, toml};
use crate::init_files::{
    assert_harness_declarations, assert_hooks_and_checks, assert_mise_entries, assert_records,
    task_run,
};
use crate::support::{Fixture, package_version};

#[test]
fn init_requires_a_workbench_pin() {
    let fixture = Fixture::git();
    fixture.write("mise.toml", "[tools]\nhk = \"2.8.1\"\n");
    let outcome = fixture.run(&["project", "init"]);
    assert_eq!(outcome.status, 3, "{}", outcome.stderr);
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
    let suggestions = outcome.error()["suggestions"].to_string();
    assert!(
        suggestions.contains("mise use github:markmals/workbench@"),
        "{suggestions}"
    );
    assert_eq!(fixture.read("mise.toml"), "[tools]\nhk = \"2.8.1\"\n");
}

#[test]
fn init_rejects_an_unknown_site_host() {
    let outcome = pinned_repository().run(&["project", "init", "--site", "netlify"]);
    assert_eq!(outcome.status, 2);
    assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
}

#[test]
fn init_sets_up_an_empty_repository_that_validate_and_context_read_back() {
    let fixture = pinned_repository();
    let receipt = init(&fixture, &[]);

    assert_eq!(receipt["status"], "initialized");
    assert_eq!(
        receipt["project"],
        fixture.root().canonicalize().unwrap().display().to_string()
    );
    assert_eq!(
        paths(&receipt["created"]),
        [
            ".gitignore",
            ".claude/settings.json",
            ".omp/config.yml",
            "hk.pkl",
            ".github/workflows/workbench.yml",
            "AGENTS.md",
            "VISION.md",
            "proposals/README.md",
            "policies/README.md",
            "decisions/README.md",
            "guides/README.md",
            ".agents/commit-scopes",
            "docs/hugo.toml",
        ]
    );
    assert_eq!(
        receipt["addedTo"],
        json!([{
            "path": "mise.toml",
            "entries": [
                "hooks.postinstall",
                "tools.hk",
                "tools.gh",
                "tools.\"github:gastownhall/beads\"",
                "tasks.status",
                "tasks.validate",
                "tasks.\"beads:sync\"",
                "tasks.\"beads:claim\"",
                "tasks.\"beads:close\"",
            ]
        }])
    );
    assert_eq!(receipt["vendored"], json!([]));
    assert_eq!(receipt["attention"], json!([]));
    assert_eq!(receipt["failed"], json!([]));
    assert_eq!(receipt["cannotPredict"], json!([]));
    let missing: Vec<&str> = receipt["missing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|piece| piece["piece"].as_str().unwrap())
        .collect();
    assert_eq!(missing, ["setup", "check", "fmt", "release"]);

    assert_mise_entries(&fixture);
    assert_harness_declarations(&fixture);
    assert_hooks_and_checks(&fixture);
    assert_records(&fixture);

    // `mise install` runs the postinstall hook, which links the pinned package.
    fixture.link_package();
    let validation = fixture.run(&["project", "validate"]);
    assert_eq!(validation.status, 0, "{}", validation.stdout);
    assert_eq!(validation.json(), json!({ "violations": [], "count": 0 }));

    let context = fixture.run(&["context", "show"]);
    assert_eq!(context.status, 0, "{}", context.stderr);
    let context = context.json();
    assert_eq!(
        context["workbench"]["pin"],
        json!({ "status": "pinned", "version": package_version() })
    );
    assert_eq!(context["workbench"]["link"]["state"], "linked");
    assert_eq!(context["workbench"]["link"]["version"], package_version());
    assert_eq!(context["harnesses"]["claudeCode"]["state"], "declared");
    assert_eq!(context["harnesses"]["ohMyPi"]["state"], "declared");
    assert_eq!(context["site"]["config"], "docs/hugo.toml");
}

#[test]
fn init_dry_run_returns_the_same_receipt_and_writes_nothing() {
    let fixture = pinned_repository();
    let before = snapshot(&fixture);
    let mut preview = init(&fixture, &["--dry-run"]);
    assert_eq!(snapshot(&fixture), before, "a dry run wrote to the project");
    assert_eq!(preview["status"], "dry-run");
    assert!(!preview["cannotPredict"].as_array().unwrap().is_empty());

    let mut receipt = init(&fixture, &[]);
    for field in ["status", "cannotPredict"] {
        preview.as_object_mut().unwrap().remove(field);
        receipt.as_object_mut().unwrap().remove(field);
    }
    assert_eq!(preview, receipt);
}

#[test]
fn init_is_idempotent() {
    let fixture = pinned_repository();
    init(&fixture, &["--site", "github-pages"]);
    let after_first = snapshot(&fixture);
    let receipt = init(&fixture, &["--site", "github-pages"]);
    assert_eq!(receipt["status"], "unchanged");
    assert_eq!(receipt["created"], json!([]));
    assert_eq!(receipt["addedTo"], json!([]));
    assert!(paths(&receipt["unchanged"]).contains(&"mise.toml".to_string()));
    assert_eq!(snapshot(&fixture), after_first);
}

#[test]
fn init_adds_only_missing_entries_to_structured_files() {
    let fixture = Fixture::git();
    let mise = "# Our tools.\n[tools]\n\"github:markmals/workbench\" = \"3.0.0\"\nhk = \"2.0.0\" # older, on purpose\n\n[tasks.status]\nrun = \"echo ours\"\n";
    let settings = "{\n  \"permissions\": { \"allow\": [\"Bash(ls)\"] },\n  \"enabledPlugins\": {\n    \"other@market\": true\n  }\n}\n";
    let omp = "theme: dark\nextensions:\n  - ./other\n";
    fixture
        .write("mise.toml", mise)
        .write(".gitignore", "node_modules")
        .write(".claude/settings.json", settings)
        .write(".omp/config.yml", omp);

    let receipt = init(&fixture, &[]);

    let after = fixture.read("mise.toml");
    assert!(only_added_lines(mise, &after), "{after}");
    let parsed = toml(&fixture, "mise.toml");
    assert_eq!(parsed["tools"]["hk"].as_str(), Some("2.0.0"));
    assert_eq!(parsed["tools"]["gh"].as_str(), Some("2.103.2"));
    assert_eq!(task_run(&parsed, "status").as_str(), Some("echo ours"));
    let mise_entries = &receipt["addedTo"][0];
    assert_eq!(mise_entries["path"], "mise.toml");
    let entries = mise_entries["entries"].to_string();
    assert!(!entries.contains("tools.hk") && !entries.contains("tasks.status"));
    let attention = receipt["attention"].to_string();
    assert!(attention.contains("tasks.status"), "{attention}");

    assert_eq!(fixture.read(".gitignore"), "node_modules\n.workbench\n");

    let parsed = json(&fixture, ".claude/settings.json");
    assert_eq!(parsed["permissions"], json!({ "allow": ["Bash(ls)"] }));
    assert_eq!(
        parsed["enabledPlugins"],
        json!({ "other@market": true, "workbench@workbench": true })
    );
    assert_eq!(
        parsed["extraKnownMarketplaces"]["workbench"]["source"]["path"],
        "./.workbench"
    );
    assert!(
        fixture
            .read(".claude/settings.json")
            .starts_with("{\n  \"permissions\"")
    );

    assert_eq!(
        fixture.read(".omp/config.yml"),
        "theme: dark\nextensions:\n  - ./other\n  - ./.workbench\n"
    );
    assert_eq!(
        paths(&receipt["addedTo"]),
        [
            "mise.toml",
            ".gitignore",
            ".claude/settings.json",
            ".omp/config.yml"
        ]
    );
}

#[test]
fn init_leaves_conflicting_entries_and_says_so() {
    let fixture = pinned_repository();
    let mise = format!(
        "{}\n[hooks]\npostinstall = \"echo installed\"\n",
        fixture.read("mise.toml")
    );
    let settings = "{ \"enabledPlugins\": { \"workbench@workbench\": false } }\n";
    fixture
        .write("mise.toml", &mise)
        .write(".claude/settings.json", settings)
        .write("AGENTS.md", "# Our project\n");

    let receipt = init(&fixture, &[]);

    assert_eq!(
        toml(&fixture, "mise.toml")["hooks"]["postinstall"].as_str(),
        Some("echo installed")
    );
    let settings_after = json(&fixture, ".claude/settings.json");
    assert_eq!(
        settings_after["enabledPlugins"]["workbench@workbench"],
        false
    );
    assert_eq!(fixture.read("AGENTS.md"), "# Our project\n");
    assert_eq!(
        paths(&receipt["attention"]),
        ["mise.toml", ".claude/settings.json", "AGENTS.md"]
    );
    let attention = receipt["attention"].to_string();
    assert!(attention.contains("postinstall"), "{attention}");
    assert!(attention.contains("workbench@workbench"), "{attention}");
    assert!(attention.contains(".workbench"), "{attention}");
}

#[test]
fn init_links_claude_skills_and_rules_to_existing_agents_directories() {
    let fixture = pinned_repository();
    fixture.write(
        ".agents/skills/our-skill/SKILL.md",
        "---\nname: our-skill\n---\n",
    );
    let receipt = init(&fixture, &[]);
    let created = paths(&receipt["created"]);
    assert!(
        created.contains(&".claude/skills".to_string()),
        "{created:?}"
    );
    assert!(!created.contains(&".claude/rules".to_string()));
    assert_eq!(
        fs::read_link(fixture.path(".claude/skills")).unwrap(),
        std::path::Path::new("../.agents/skills")
    );
    assert!(fixture.path(".claude/skills/our-skill/SKILL.md").exists());
    assert!(fs::symlink_metadata(fixture.path(".claude/rules")).is_err());
}

#[test]
fn init_creates_the_vision_from_a_project_template_override() {
    let fixture = pinned_repository();
    fixture.write(".agents/templates/VISION.md", "# Our vision template\n");
    let receipt = init(&fixture, &[]);
    assert_eq!(fixture.read("VISION.md"), "# Our vision template\n");
    assert_eq!(
        receipt["vendored"][0]["path"],
        ".agents/templates/VISION.md"
    );
    assert_eq!(receipt["vendored"][0]["template"], "different");
}

#[test]
fn init_reports_a_file_it_could_not_write() {
    let fixture = pinned_repository();
    fs::create_dir(fixture.path(".github")).unwrap();
    fs::set_permissions(fixture.path(".github"), fs::Permissions::from_mode(0o555)).unwrap();

    let outcome = fixture.run(&["project", "init"]);
    fs::set_permissions(fixture.path(".github"), fs::Permissions::from_mode(0o755)).unwrap();

    assert_eq!(outcome.status, 3, "{}", outcome.stderr);
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
    let receipt = outcome.json();
    assert_eq!(
        paths(&receipt["failed"]),
        [".github/workflows/workbench.yml"]
    );
    assert!(!paths(&receipt["created"]).contains(&".github/workflows/workbench.yml".to_string()));
    assert!(fixture.path("hk.pkl").exists());
    assert!(
        outcome.error()["message"]
            .as_str()
            .unwrap()
            .contains(".github/workflows/workbench.yml")
    );
}
