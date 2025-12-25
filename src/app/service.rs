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
