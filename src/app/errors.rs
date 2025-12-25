use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("todo not found")]
    TodoNotFound,

    #[error("todo is already done")]
    AlreadyDone,

    #[error("todo is already open")]
    AlreadyOpen,

    #[error("refusing destructive action without confirmation (use --yes)")]
    ConfirmationRequired,
}
