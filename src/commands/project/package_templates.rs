//! The package's artifact templates. This release's copies are built into the binary, so
//! `project init` can write `VISION.md`, and vendored copies can be recognized, before
//! `mise install` has linked `.workbench`.

use std::fs;
use std::path::Path;

use super::setup::{Link, LinkState};

/// Every file in `plugin/templates/`, by name.
const BUILT_IN: &[(&str, &str)] = &[
    (
        "DECISION.md",
        include_str!("../../../plugin/templates/DECISION.md"),
    ),
    (
        "FEATURE.feature",
        include_str!("../../../plugin/templates/FEATURE.feature"),
    ),
    (
        "GUIDE.md",
        include_str!("../../../plugin/templates/GUIDE.md"),
    ),
    (
        "POLICY.md",
        include_str!("../../../plugin/templates/POLICY.md"),
    ),
    (
        "PROPOSAL.md",
        include_str!("../../../plugin/templates/PROPOSAL.md"),
    ),
    (
        "READINESS_REPORT.md",
        include_str!("../../../plugin/templates/READINESS_REPORT.md"),
    ),
    (
        "README.md",
        include_str!("../../../plugin/templates/README.md"),
    ),
    (
        "VISION.md",
        include_str!("../../../plugin/templates/VISION.md"),
    ),
];

/// The package's template `name`: the one in `.workbench/templates/` when the link resolves,
/// otherwise this release's built-in copy. `None` when the package has no such template.
pub fn package_template(link: &Link, name: &str) -> Option<String> {
    match (link.state, &link.target) {
        (LinkState::Linked, Some(target)) => {
            fs::read_to_string(Path::new(target).join("templates").join(name)).ok()
        }
        _ => built_in(name).map(str::to_string),
    }
}

pub fn built_in(name: &str) -> Option<&'static str> {
    BUILT_IN
        .iter()
        .find(|(template, _)| *template == name)
        .map(|(_, contents)| *contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_built_in_templates_are_the_package_templates() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("plugin/templates");
        let mut shipped: Vec<String> = fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        shipped.sort();
        let built_in: Vec<String> = BUILT_IN.iter().map(|(name, _)| name.to_string()).collect();
        assert_eq!(built_in, shipped);
        for (name, contents) in BUILT_IN {
            assert_eq!(
                *contents,
                fs::read_to_string(directory.join(name)).unwrap(),
                "{name}"
            );
        }
    }
}
