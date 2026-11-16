//! Every resource's commands. Each module declares its registry entries in `COMMANDS`.

use crate::registry::CommandSpec;

pub mod beads;
pub mod change_files;
pub mod cli;
pub mod commits;
pub mod context;
pub mod docs;
pub mod project;
pub mod records;
pub mod schema;
pub mod workflow;

/// The registry, by resource, in the order discovery lists them.
pub static GROUPS: &[&[CommandSpec]] = &[
    cli::COMMANDS,
    schema::COMMANDS,
    docs::COMMANDS,
    context::COMMANDS,
    workflow::COMMANDS,
    records::COMMANDS,
    project::COMMANDS,
    commits::COMMANDS,
    change_files::COMMANDS,
    beads::COMMANDS,
];
