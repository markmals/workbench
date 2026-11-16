//! The shipped CLI reference is the registry, rendered. `mise run cli:reference` regenerates it.

use std::path::Path;

#[test]
fn the_shipped_cli_reference_matches_the_registry() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("plugin/reference/cli.md");
    let generated = workbench::reference::render();
    if std::env::var_os("WORKBENCH_UPDATE_REFERENCE").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &generated).unwrap();
        return;
    }
    let shipped = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        shipped == generated,
        "plugin/reference/cli.md is out of date with the command registry; run `mise run cli:reference`"
    );
}
