//! In-memory repository implementation.
//!
//! Used for early development and tests.

use crate::{app::repository::TodoRepository, domain::todo::Todo};

/// Simple in-memory store.
#[derive(Default)]
pub struct MemoryTodoRepository {
    todos: Vec<Todo>,
}

impl MemoryTodoRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TodoRepository for MemoryTodoRepository {
    fn add(&mut self, todo: Todo) {
        self.todos.push(todo);
    }

    fn list(&self) -> Vec<Todo> {
        self.todos.clone()
    }
}
