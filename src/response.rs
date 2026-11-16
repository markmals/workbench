//! What a command hands back: its result, a text rendering for humans, and whether it failed.

use serde::Serialize;
use serde_json::Value;

use crate::failure::Failure;

/// A rendering of a result for `--output text`.
pub trait RenderText {
    fn render_text(&self) -> String;
}

/// A command's outcome. The result always goes to stdout. A `failure` here is a check that ran
/// and found problems, or a refusal whose result still describes the state: it goes to stderr
/// and sets the exit status, while the result stays on stdout.
#[derive(Debug)]
pub struct Response {
    pub result: Value,
    pub text: String,
    pub failure: Option<Failure>,
    /// Diagnostics for stderr, such as why a sync that completed locally was not published.
    pub warnings: Vec<String>,
}

impl Response {
    pub fn new<T: Serialize + RenderText>(result: &T) -> Response {
        Response {
            result: serde_json::to_value(result).expect("command results serialize to JSON"),
            text: result.render_text(),
            failure: None,
            warnings: Vec::new(),
        }
    }

    pub fn failing(mut self, failure: Failure) -> Response {
        self.failure = Some(failure);
        self
    }

    pub fn warn(mut self, warning: impl Into<String>) -> Response {
        self.warnings.push(warning.into());
        self
    }
}
