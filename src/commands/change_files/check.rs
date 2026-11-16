//! `workbench change-files check`: a range describes its change for release notes, or says
//! it has none.

use std::path::Path;

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::json;

use super::change_file;
use super::knope::KnopeConfig;
use crate::commands::commits::range::Range;
use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::response::{RenderText, Response};

const CHANGESET_DIRECTORY: &str = ".changeset";
const TRAILER_KEY: &str = "Release-Note";
const TRAILER_VALUE: &str = "none";

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangeFilesCheck {
    /// The project root.
    pub project: String,
    pub base: String,
    pub head: String,
    /// Whether the project has a knope.toml, so packages and change types were checked
    /// against it. Without one, any well-formed frontmatter counts.
    pub knope_toml: bool,
    /// The Markdown files the range adds directly under `.changeset/`, as committed at head.
    pub change_files: Vec<AddedChangeFile>,
    /// The commits in the range, oldest first, that carry the trailer `Release-Note: none`.
    pub release_note_none: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct AddedChangeFile {
    /// Relative to the project root: `.changeset/<name>.md`.
    pub path: String,
    /// Whether Knope accepts its frontmatter.
    pub valid: bool,
    /// Why it is not a valid change file; empty when it is one.
    pub problems: Vec<String>,
}

impl ChangeFilesCheck {
    fn satisfied(&self) -> bool {
        self.change_files.iter().any(|file| file.valid) || !self.release_note_none.is_empty()
    }

    fn invalid_files(&self) -> impl Iterator<Item = &AddedChangeFile> {
        self.change_files.iter().filter(|file| !file.valid)
    }
}

impl RenderText for ChangeFilesCheck {
    fn render_text(&self) -> String {
        let mut text = String::new();
        for file in &self.change_files {
            let verdict = if file.valid { "valid" } else { "invalid" };
            text.push_str(&format!("{verdict:<8} {}\n", file.path));
            for problem in &file.problems {
                text.push_str(&format!("         {problem}\n"));
            }
        }
        for commit in &self.release_note_none {
            text.push_str(&format!("{TRAILER_KEY}: {TRAILER_VALUE} on {commit}\n"));
        }
        text.push_str(if self.satisfied() {
            "The range describes its change for release notes."
        } else {
            "The range adds no valid change file and no commit carries `Release-Note: none`."
        });
        text
    }
}

pub fn check(invocation: &Invocation) -> Result<Response, Failure> {
    let project = invocation.project()?;
    let base = invocation.string("base").expect("base is required");
    let head = invocation.string("head").expect("head is required");
    let range = Range::resolve(&project.root, &base, &head)?;
    let config = KnopeConfig::load(&project.root)?;

    let change_files = range
        .added_files(CHANGESET_DIRECTORY)?
        .into_iter()
        .filter(|path| is_change_file_path(path))
        .map(|path| {
            let problems = change_file::problems(&range.read_at_head(&path)?, config.as_ref());
            Ok(AddedChangeFile {
                valid: problems.is_empty(),
                path,
                problems,
            })
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    let result = ChangeFilesCheck {
        project: project.root.display().to_string(),
        base,
        head,
        knope_toml: config.is_some(),
        change_files,
        release_note_none: range.commits_with_trailer(TRAILER_KEY, TRAILER_VALUE)?,
    };
    Ok(respond(&result))
}

/// Knope reads only the Markdown files directly in `.changeset/`.
fn is_change_file_path(path: &str) -> bool {
    path.strip_prefix(".changeset/").is_some_and(|name| {
        !name.contains('/')
            && Path::new(name)
                .extension()
                .is_some_and(|extension| extension == "md")
    })
}

fn respond(result: &ChangeFilesCheck) -> Response {
    let mut response = Response::new(result);
    if !result.satisfied() {
        return response.failing(missing_release_note(result));
    }
    for file in result.invalid_files() {
        response = response.warn(format!(
            "{} is not a valid change file, and Knope fails on it: {}",
            file.path,
            file.problems.join("; ")
        ));
    }
    response
}

fn missing_release_note(result: &ChangeFilesCheck) -> Failure {
    let invalid: Vec<&str> = result
        .invalid_files()
        .map(|file| file.path.as_str())
        .collect();
    Failure::new(
        ErrorCode::CheckFailed,
        "the range adds no valid change file under .changeset/, and no commit in it carries the \
         trailer `Release-Note: none`",
    )
    .with_details(json!({ "invalidChangeFiles": invalid }))
    .suggest("Describe the change for release notes: `workbench change-files create --type <type> --summary <text>`, then commit the file")
    .suggest("If nothing user-visible changed, carry the trailer on a commit in the range: `git commit --trailer \"Release-Note: none\"`, or `git commit --amend --trailer \"Release-Note: none\"` for the last one")
}
