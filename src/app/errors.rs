use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("todo not found")]
    TodoNotFound,

    #[error("refusing destructive action without confirmation (use --yes)")]
    ConfirmationRequired,
}
