use thiserror::Error;

/// Errors originating from domain validation or invariants.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("todo title cannot be empty")]
    EmptyTitle,
}
