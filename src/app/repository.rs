//! Repository trait (port).
//!
//! The UI and application logic depend on this trait,
//! not on any concrete storage implementation.

use crate::domain::todo::Todo;

/// Abstraction over todo storage.
pub trait TodoRepository {
    fn add(&mut self, todo: Todo);
    fn list(&self) -> Vec<Todo>;
}
