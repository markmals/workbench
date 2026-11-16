//! The structured error every failing command reports on stderr.

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, json};

/// A stable error code. Codes, exit statuses, and retryability are part of the CLI contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// A check ran and found violations.
    CheckFailed,
    /// Input is missing or malformed.
    InvalidArgument,
    /// A named record, item, commit, or file does not exist.
    NotFound,
    /// The project is not ready for the operation.
    PreconditionFailed,
    /// A required program is absent.
    DependencyMissing,
    /// `gh` or Git credentials are missing or rejected.
    Unauthenticated,
    /// Another claimant holds the item, a claim was lost, or a sync stopped on a conflict.
    Conflict,
    /// An operation exceeded its time limit.
    Timeout,
    /// The remote or the network was unreachable.
    Unavailable,
    /// A defect in `workbench` itself.
    Internal,
}

impl ErrorCode {
    pub const ALL: [ErrorCode; 10] = [
        ErrorCode::CheckFailed,
        ErrorCode::InvalidArgument,
        ErrorCode::NotFound,
        ErrorCode::PreconditionFailed,
        ErrorCode::DependencyMissing,
        ErrorCode::Unauthenticated,
        ErrorCode::Conflict,
        ErrorCode::Timeout,
        ErrorCode::Unavailable,
        ErrorCode::Internal,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::CheckFailed => "CHECK_FAILED",
            ErrorCode::InvalidArgument => "INVALID_ARGUMENT",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::PreconditionFailed => "PRECONDITION_FAILED",
            ErrorCode::DependencyMissing => "DEPENDENCY_MISSING",
            ErrorCode::Unauthenticated => "UNAUTHENTICATED",
            ErrorCode::Conflict => "CONFLICT",
            ErrorCode::Timeout => "TIMEOUT",
            ErrorCode::Unavailable => "UNAVAILABLE",
            ErrorCode::Internal => "INTERNAL",
        }
    }

    pub fn exit_status(self) -> i32 {
        match self {
            ErrorCode::CheckFailed => 1,
            ErrorCode::InvalidArgument => 2,
            ErrorCode::NotFound
            | ErrorCode::PreconditionFailed
            | ErrorCode::DependencyMissing
            | ErrorCode::Unauthenticated => 3,
            ErrorCode::Conflict => 4,
            ErrorCode::Timeout | ErrorCode::Unavailable => 5,
            ErrorCode::Internal => 70,
        }
    }

    pub fn retryable(self) -> bool {
        matches!(self, ErrorCode::Timeout | ErrorCode::Unavailable)
    }

    pub fn meaning(self) -> &'static str {
        match self {
            ErrorCode::CheckFailed => "A check ran and found violations.",
            ErrorCode::InvalidArgument => "Input is missing or malformed.",
            ErrorCode::NotFound => "A named record, item, commit, or file does not exist.",
            ErrorCode::PreconditionFailed => "The project is not ready for the operation.",
            ErrorCode::DependencyMissing => "A required program (git, gh, bd) is absent.",
            ErrorCode::Unauthenticated => "gh or Git credentials are missing or rejected.",
            ErrorCode::Conflict => {
                "Another claimant holds the item, a claim was lost, or a Beads sync stopped on a conflict."
            }
            ErrorCode::Timeout => "An operation exceeded its time limit.",
            ErrorCode::Unavailable => "The remote or the network was unreachable.",
            ErrorCode::Internal => "A defect in workbench itself.",
        }
    }
}

/// A failed operation: what the agent needs to decide whether and how to recover.
#[derive(Clone, Debug, PartialEq)]
pub struct Failure {
    pub code: ErrorCode,
    pub message: String,
    pub details: Value,
    pub suggestions: Vec<String>,
}

impl Failure {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Failure {
        Failure {
            code,
            message: message.into(),
            details: json!({}),
            suggestions: Vec::new(),
        }
    }

    pub fn with_details(mut self, details: Value) -> Failure {
        self.details = details;
        self
    }

    pub fn suggest(mut self, suggestion: impl Into<String>) -> Failure {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn invalid_argument(message: impl Into<String>) -> Failure {
        Failure::new(ErrorCode::InvalidArgument, message)
    }

    pub fn precondition(message: impl Into<String>) -> Failure {
        Failure::new(ErrorCode::PreconditionFailed, message)
    }

    pub fn not_found(message: impl Into<String>) -> Failure {
        Failure::new(ErrorCode::NotFound, message)
    }

    pub fn internal(message: impl Into<String>) -> Failure {
        Failure::new(ErrorCode::Internal, message)
    }

    pub fn dependency_missing(program: &str) -> Failure {
        Failure::new(
            ErrorCode::DependencyMissing,
            format!("`{program}` is not installed or not on PATH"),
        )
        .with_details(json!({ "program": program }))
        .suggest(format!(
            "Install `{program}`; in a Workbench project, `mise install` provisions it"
        ))
    }

    pub fn to_json(&self) -> Value {
        json!({
            "error": {
                "code": self.code.as_str(),
                "message": self.message,
                "retryable": self.code.retryable(),
                "details": self.details,
                "suggestions": self.suggestions,
            }
        })
    }

    pub fn to_text(&self) -> String {
        let mut text = format!("error[{}]: {}", self.code.as_str(), self.message);
        for suggestion in &self.suggestions {
            text.push_str("\n  try: ");
            text.push_str(suggestion);
        }
        text
    }
}
