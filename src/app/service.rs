//! Application services (use-cases).
//!
//! This is where orchestration logic lives.

use anyhow::Result;

use crate::{
    app::repository::TodoRepository,
    domain::todo::{Title, Todo, TodoId, TodoPatch},
};

/// High-level application service.
pub struct TodoService<R> {
    pub repo: R,
}

impl<R> TodoService<R>
where
    R: TodoRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub fn add_todo(&mut self, title: Title) -> Result<TodoId> {
        let todo = Todo::new(title);
        let id = todo.id;
        self.repo.add(todo);
        Ok(id)
    }

    pub fn list_todos(&self) -> Vec<Todo> {
        self.repo.list()
    }

    /// Insert a fully-constructed Todo (used for seeding / imports later).
    ///
    /// This avoids UI or seed logic needing access to repository internals.
    pub fn insert_todo(&mut self, todo: Todo) {
        self.repo.add(todo);
    }

    pub fn edit_todo(&mut self, id: TodoId, patch: TodoPatch) -> Result<bool> {
        if let Some(mut todo) = self.repo.get(id) {
            todo.apply_patch(patch);
            Ok(self.repo.replace(todo))
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::memory_repo::MemoryTodoRepository;

    #[test]
    fn service_add_then_list() {
        let repo = MemoryTodoRepository::new();
        let mut svc = TodoService::new(repo);

        svc.add_todo(Title::parse("Hello").unwrap()).unwrap();
        svc.add_todo(Title::parse("World").unwrap()).unwrap();

        let todos = svc.list_todos();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].title.as_str(), "Hello");
        assert_eq!(todos[1].title.as_str(), "World");
    }
}
