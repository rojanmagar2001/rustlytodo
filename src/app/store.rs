//! Store: central application state holder.
//!
//! For now it just owns a repository, but later it will also own:
//! - loaded configuration
//! - undo/redo stacks
//! - dirty tracking for persistence

use anyhow::Result;

use crate::{
    app::{repository::TodoRepository, service::TodoService},
    domain::todo::{Title, Todo, TodoId},
};

/// App store that owns stateful dependencies.
pub struct Store<R> {
    service: TodoService<R>,
}

impl<R> Store<R>
where
    R: TodoRepository,
{
    pub fn new(repo: R) -> Self {
        Self {
            service: TodoService::new(repo),
        }
    }

    pub fn add_todo(&mut self, title: Title) -> Result<TodoId> {
        self.service.add_todo(title)
    }

    pub fn list_todos(&self) -> Vec<Todo> {
        self.service.list_todos()
    }
}
