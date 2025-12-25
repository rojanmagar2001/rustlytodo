//! Application services (use-cases).
//!
//! This is where orchestration logic lives.

use anyhow::Result;

use crate::{
    app::repository::TodoRepository,
    domain::todo::{Title, Todo},
};

/// High-level application service.
pub struct TodoService<R> {
    repo: R,
}

impl<R> TodoService<R>
where
    R: TodoRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub fn add_todo(&mut self, title: Title) -> Result<()> {
        let todo = Todo::new(title);
        self.repo.add(todo);
        Ok(())
    }

    pub fn list_todos(&self) -> Vec<Todo> {
        self.repo.list()
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
