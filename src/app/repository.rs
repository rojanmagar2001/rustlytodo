//! Repository trait (port).
//!
//! The UI and application logic depend on this trait,
//! not on any concrete storage implementation.

use crate::domain::todo::{Todo, TodoId};

/// Abstraction over todo storage.
pub trait TodoRepository {
    fn add(&mut self, todo: Todo);
    fn list(&self) -> Vec<Todo>;
    /// Replace an existing todo with the same ID.
    fn replace(&mut self, todo: Todo) -> bool;

    /// Get a todo by full ID (exact match).
    fn get(&self, id: TodoId) -> Option<Todo>;

    /// Replace the entire dataset (used for import/migrations).
    fn set_all(&mut self, todos: Vec<Todo>);
}
