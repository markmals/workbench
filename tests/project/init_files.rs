//! What `project init` writes into a repository that held only the Workbench pin.

use std::fs;

use serde_json::json;

use crate::fixtures::{json, toml};
use crate::support::{Fixture, plugin_dir};

const POSTINSTALL: &str =
    r#"ln -sfn "$(mise where github:markmals/workbench)" "{{config_root}}/.workbench""#;

pub fn task_run(mise: &toml::Table, task: &str) -> toml::Value {
    mise["tasks"][task]["run"].clone()
}

pub fn assert_mise_entries(fixture: &Fixture) {
    let mise = toml(fixture, "mise.toml");
    assert_eq!(mise["hooks"]["postinstall"].as_str(), Some(POSTINSTALL));
    assert_eq!(mise["tools"]["hk"].as_str(), Some("2.8.1"));
    assert_eq!(mise["tools"]["gh"].as_str(), Some("2.103.2"));
    assert_eq!(
        mise["tools"]["github:gastownhall/beads"].as_str(),
        Some("1.3.2")
    );
    for (task, run) in [
        ("status", "workbench workflow status --output text"),
        ("beads:sync", "workbench beads sync --output text"),
        ("beads:claim", "workbench beads claim --output text"),
        ("beads:close", "workbench beads close --output text"),
    ] {
        assert_eq!(task_run(&mise, task).as_str(), Some(run), "{task}");
    }
    assert_eq!(
        task_run(&mise, "validate"),
        toml::Value::Array(vec![
            "workbench records validate --output text".into(),
            "workbench project validate --output text".into(),
        ])
    );
    assert!(mise["tools"].get("hugo").is_none());
    assert!(mise["tasks"].get("docs:build").is_none());
}

pub fn assert_harness_declarations(fixture: &Fixture) {
    assert_eq!(fixture.read(".gitignore"), ".workbench\n");
    let settings = json(fixture, ".claude/settings.json");
    assert_eq!(
        settings["extraKnownMarketplaces"]["workbench"]["source"],
        json!({ "source": "directory", "path": "./.workbench" })
    );
    assert_eq!(settings["enabledPlugins"]["workbench@workbench"], true);
    assert_eq!(
        fixture.read(".omp/config.yml"),
        "extensions:\n    - ./.workbench\n"
    );
}

pub fn assert_hooks_and_checks(fixture: &Fixture) {
    let hooks = fixture.read("hk.pkl");
    assert!(hooks.contains(
        r#"amends "package://github.com/jdx/hk/releases/download/v2.8.1/hk@2.8.1#/Config.pkl""#
    ));
    assert!(hooks.contains("workbench commits check --message-file {{commit_msg_file}}"));
    assert!(hooks.contains("bd hooks run pre-commit"));
    for hook in ["pre-push", "post-merge", "post-rewrite"] {
        assert!(hooks.contains(&format!("[\"{hook}\"]")), "{hook}");
    }
    assert!(hooks.contains("mise run beads:sync"));
    assert!(!hooks.contains(r#"check = "mise run check""#));

    let workflow = fixture.read(".github/workflows/workbench.yml");
    for expected in [
        "pull_request",
        "github.head_ref != 'release'",
        "actions/checkout@v7",
        "fetch-depth: 0",
        "jdx/mise-action@v5",
        r#"workbench commits check "$BASE_SHA" "$HEAD_SHA""#,
        r#"workbench change-files check "$BASE_SHA" "$HEAD_SHA""#,
    ] {
        assert!(
            workflow.contains(expected),
            "workbench.yml lacks {expected}"
        );
    }
}

pub fn assert_records(fixture: &Fixture) {
    let agents = fixture.read("AGENTS.md");
    assert!(agents.contains("`.workbench`") && agents.contains("mise install"));
    assert_dated_vision(fixture);
    for (readme, heading) in [
        ("proposals/README.md", "# Proposals"),
        ("policies/README.md", "# Policies"),
        ("decisions/README.md", "# Decisions"),
        ("guides/README.md", "# Guides"),
    ] {
        assert!(fixture.read(readme).starts_with(heading), "{readme}");
    }
    assert!(fixture.path(".agents/commit-scopes").is_file());
    let hugo = toml(fixture, "docs/hugo.toml");
    assert_eq!(hugo["themesDir"].as_str(), Some("../.workbench"));
    assert_eq!(hugo["theme"].as_str(), Some("site"));
}

/// `VISION.md` is the package's template with today's date for `<YYYY-MM-DD>`.
fn assert_dated_vision(fixture: &Fixture) {
    let vision = fixture.read("VISION.md");
    let template = fs::read_to_string(plugin_dir().join("templates/VISION.md")).unwrap();
    let updated = vision
        .lines()
        .find_map(|line| line.strip_prefix("updated: "))
        .expect("VISION.md has an updated date");
    let is_date = updated.len() == 10
        && updated
            .char_indices()
            .all(|(index, character)| match index {
                4 | 7 => character == '-',
                _ => character.is_ascii_digit(),
            });
    assert!(is_date, "updated: {updated}");
    assert_eq!(vision, template.replace("<YYYY-MM-DD>", updated));
}
