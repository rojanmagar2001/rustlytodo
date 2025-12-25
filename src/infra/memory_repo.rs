//! In-memory repository implementation.
//!
//! Used for early development and tests.

use crate::{
    app::repository::TodoRepository,
    domain::todo::{Todo, TodoId},
};

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

    fn replace(&mut self, todo: Todo) -> bool {
        if let Some(slot) = self.todos.iter_mut().find(|t| t.id == todo.id) {
            *slot = todo;
            true
        } else {
            false
        }
    }

    fn get(&self, id: TodoId) -> Option<Todo> {
        self.todos.iter().find(|t| t.id == id).cloned()
    }

    fn set_all(&mut self, todos: Vec<Todo>) {
        self.todos = todos;
    }

    fn remove(&mut self, id: TodoId) -> bool {
        let before = self.todos.len();
        self.todos.retain(|t| t.id != id);
        self.todos.len() != before
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

    #[test]
    fn repo_replace_updates_item() {
        let mut repo = MemoryTodoRepository::new();
        let mut t = Todo::new(Title::parse("One").unwrap());
        let id = t.id;
        repo.add(t.clone());

        // mutate and replace
        t.title = Title::parse("Updated").unwrap();
        assert!(repo.replace(t));

        let got = repo.get(id).unwrap();
        assert_eq!(got.title.as_str(), "Updated");
    }
}
