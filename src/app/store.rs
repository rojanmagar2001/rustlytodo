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

    pub fn is_empty(&self) -> bool {
        self.list_todos().is_empty()
    }

    /// Insert an already-built Todo (for seeding / import).
    pub fn insert_todo(&mut self, todo: Todo) {
        self.service.insert_todo(todo);
    }

    pub fn insert_many(&mut self, todos: Vec<Todo>) {
        for todo in todos {
            self.insert_todo(todo);
        }
    }
}
