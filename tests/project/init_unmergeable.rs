//! Files `project init` must add to but cannot extend safely: it leaves each as it is and names
//! it under `attention`, so the receipt never claims a declaration it did not make.

use std::fs;

use serde_json::Value;

use crate::fixtures::{init, pinned_repository};

fn attention_for(receipt: &Value, path: &str) -> Vec<String> {
    receipt["attention"]
        .as_array()
        .expect("the receipt lists attention")
        .iter()
        .filter(|notice| notice["path"] == path)
        .map(|notice| notice["message"].as_str().unwrap_or_default().to_string())
        .collect()
}

fn added_to(receipt: &Value, path: &str) -> bool {
    receipt["addedTo"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item["path"] == path))
}

#[test]
fn a_symlinked_file_is_never_written_through() {
    let fixture = pinned_repository();
    let outside = tempfile::NamedTempFile::new().unwrap();
    fs::write(outside.path(), "node_modules\n").unwrap();
    std::os::unix::fs::symlink(outside.path(), fixture.path(".gitignore")).unwrap();

    let receipt = init(&fixture, &[]);

    assert_eq!(
        fs::read_to_string(outside.path()).unwrap(),
        "node_modules\n"
    );
    assert!(!added_to(&receipt, ".gitignore"), "{receipt}");
    assert!(
        !attention_for(&receipt, ".gitignore").is_empty(),
        "{receipt}"
    );
}

#[test]
fn a_scalar_omp_extensions_value_is_left_for_the_project_to_extend() {
    let fixture = pinned_repository();
    fixture.write(".omp/config.yml", "extensions: './other'\n");

    let receipt = init(&fixture, &[]);

    assert_eq!(fixture.read(".omp/config.yml"), "extensions: './other'\n");
    assert!(!added_to(&receipt, ".omp/config.yml"), "{receipt}");
    assert!(
        !attention_for(&receipt, ".omp/config.yml").is_empty(),
        "{receipt}"
    );
}

#[test]
fn claude_settings_with_a_duplicate_key_are_left_unchanged() {
    let fixture = pinned_repository();
    let settings = "{\"enabledPlugins\":{},\"enabledPlugins\":{}}\n";
    fixture.write(".claude/settings.json", settings);

    let receipt = init(&fixture, &[]);

    assert_eq!(fixture.read(".claude/settings.json"), settings);
    assert!(!added_to(&receipt, ".claude/settings.json"), "{receipt}");
    assert!(
        !attention_for(&receipt, ".claude/settings.json").is_empty(),
        "{receipt}"
    );
}

#[test]
fn a_postinstall_hook_that_only_mentions_the_link_does_not_count_as_linking_it() {
    let fixture = pinned_repository();
    let mise = format!(
        "{}\n[hooks]\npostinstall = \"echo .workbench\"\n",
        fixture.read("mise.toml")
    );
    fixture.write("mise.toml", &mise);

    let receipt = init(&fixture, &[]);

    let notices = attention_for(&receipt, "mise.toml");
    assert!(
        notices
            .iter()
            .any(|message| message.contains("hooks.postinstall")),
        "{receipt}"
    );
    assert!(
        fixture
            .read("mise.toml")
            .contains("postinstall = \"echo .workbench\"")
    );
}
