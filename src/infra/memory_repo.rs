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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::todo::{Title, Todo};

    #[test]
    fn repo_add_then_list_returns_items() {
        let mut repo = MemoryTodoRepository::new();
        repo.add(Todo::new(Title::parse("One").unwrap()));
        repo.add(Todo::new(Title::parse("Two").unwrap()));

        let items = repo.list();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title.as_str(), "One");
        assert_eq!(items[1].title.as_str(), "Two");
    }
}
