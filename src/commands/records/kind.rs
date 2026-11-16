//! The kinds of record, where each lives, and what its frontmatter must hold.

use std::sync::LazyLock;

use regex::Regex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordKind {
    Vision,
    Proposal,
    Policy,
    Decision,
    Guide,
}

/// The file holding the vision, at the project root.
pub const VISION_FILE: &str = "VISION.md";

/// The record directories at the project root, in the order the validator reads them.
pub const DIRECTORIES: [(&str, RecordKind); 4] = [
    ("proposals", RecordKind::Proposal),
    ("policies", RecordKind::Policy),
    ("decisions", RecordKind::Decision),
    ("guides", RecordKind::Guide),
];

/// The frontmatter one kind of record requires.
pub struct Schema {
    pub required: &'static [&'static str],
    /// Required keys whose value is an inline list rather than a scalar.
    pub lists: &'static [&'static str],
    /// Keys that may be absent but must be an inline list when present.
    pub optional_lists: &'static [&'static str],
    pub non_empty_lists: &'static [&'static str],
    /// The statuses the kind allows, or `None` when it has no status.
    pub statuses: Option<&'static [&'static str]>,
}

const PROPOSAL: Schema = Schema {
    required: &[
        "id",
        "title",
        "authors",
        "status",
        "pull-request",
        "supersedes",
    ],
    lists: &["authors", "supersedes"],
    optional_lists: &["issues"],
    non_empty_lists: &["authors"],
    statuses: Some(&[
        "draft",
        "awaiting-implementation",
        "active-review",
        "returned-for-revisions",
        "accepted",
        "implemented",
        "rejected",
        "withdrawn",
        "superseded",
    ]),
};

const POLICY: Schema = Schema {
    required: &["id", "title", "status"],
    lists: &[],
    optional_lists: &[],
    non_empty_lists: &[],
    statuses: Some(&["draft", "active"]),
};

const DECISION: Schema = Schema {
    required: &["id", "title", "status"],
    lists: &[],
    optional_lists: &[],
    non_empty_lists: &[],
    statuses: Some(&["proposed", "accepted"]),
};

const GUIDE: Schema = Schema {
    required: &["id", "title"],
    lists: &[],
    optional_lists: &[],
    non_empty_lists: &[],
    statuses: None,
};

const VISION: Schema = Schema {
    required: &["title", "updated"],
    lists: &[],
    optional_lists: &[],
    non_empty_lists: &[],
    statuses: None,
};

static KEBAB_CASE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("valid regex"));
static NUMBERED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([0-9]{4})-[a-z0-9]+(?:-[a-z0-9]+)*$").expect("valid regex"));

impl RecordKind {
    /// The kind of the record at `path`, relative to the project root with `/` separators, or
    /// `None` when no kind of record lives there.
    pub fn of_path(path: &str) -> Option<RecordKind> {
        if path == VISION_FILE {
            return Some(RecordKind::Vision);
        }
        let mut segments = path.rsplit('/');
        segments.next();
        let directory = segments.next()?;
        DIRECTORIES
            .iter()
            .find(|(name, _)| *name == directory)
            .map(|(_, kind)| *kind)
    }

    /// The word messages use for the kind, which is also its IDs' prefix.
    pub fn name(self) -> &'static str {
        match self {
            RecordKind::Vision => "vision",
            RecordKind::Proposal => "proposal",
            RecordKind::Policy => "policy",
            RecordKind::Decision => "decision",
            RecordKind::Guide => "guide",
        }
    }

    pub fn schema(self) -> &'static Schema {
        match self {
            RecordKind::Vision => &VISION,
            RecordKind::Proposal => &PROPOSAL,
            RecordKind::Policy => &POLICY,
            RecordKind::Decision => &DECISION,
            RecordKind::Guide => &GUIDE,
        }
    }

    /// Policies and decisions state what holds now: they change in place or are retired, never
    /// superseded.
    pub fn is_current_record(self) -> bool {
        matches!(self, RecordKind::Policy | RecordKind::Decision)
    }

    /// The ID the record at `path` must declare, or `None` when its filename breaks the kind's
    /// naming rule. The vision has no ID.
    pub fn expected_id(self, path: &str) -> Option<String> {
        let filename = path.rsplit('/').next().unwrap_or(path);
        let stem = filename.strip_suffix(".md").unwrap_or(filename);
        match self {
            RecordKind::Vision => None,
            RecordKind::Guide => KEBAB_CASE.is_match(stem).then(|| format!("guide.{stem}")),
            RecordKind::Proposal | RecordKind::Policy | RecordKind::Decision => NUMBERED
                .captures(stem)
                .map(|captures| format!("{}.{}", self.name(), &captures[1])),
        }
    }

    /// The naming rule `expected_id` enforces, as a violation message.
    pub fn filename_rule(self) -> &'static str {
        match self {
            RecordKind::Guide => "guide filename must be kebab-case",
            _ => "filename must begin with a four-digit record number and slug",
        }
    }
}
