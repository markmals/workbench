//! What this release of the Workbench package ships, recorded when the binary is built.

// `SKILLS` and `RULES`: the names of the skills under `plugin/skills/` and the rules under
// `plugin/rules/`, so checks that depend on them do not depend on `.workbench` being present.
include!(concat!(env!("OUT_DIR"), "/package_names.rs"));
