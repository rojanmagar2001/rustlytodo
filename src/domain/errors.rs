use thiserror::Error;

/// Errors originating from domain validation or invariants.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("todo title cannot be empty")]
    EmptyTitle,

    #[error("notes are too long (max {max} chars)")]
    NotesTooLong { max: usize },

    #[error("project name cannot be empty")]
    EmptyProjectName,

    #[error("tag cannot be empty")]
    EmptyTag,

    #[error("tag contains invalid characters (allowed: a-z, 0-9, '-', '_')")]
    InvalidTag,

    #[error("priority must be one of P1, P2, P3, P4")]
    InvalidPriority,

    #[error("due datetime must be RFC3339, e.g. 2026-01-02T09:00:00Z")]
    InvalidDueAt,

    #[error("cannot mark as dome: already done")]
    AlreadyDone,

    #[error("cannot mark as open: already open")]
    AlreadyOpen,
}
