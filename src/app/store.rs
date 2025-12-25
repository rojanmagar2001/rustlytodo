//! Store: central application state holder.
//!
//! For now it just owns a repository, but later it will also own:
//! - loaded configuration
//! - undo/redo stacks
//! - dirty tracking for persistence

use anyhow::Result;

use crate::{
    app::{errors::AppError, repository::TodoRepository, service::TodoService},
    domain::{
        errors::DomainError,
        todo::{Title, Todo, TodoId, TodoPatch},
    },
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

    pub fn edit_todo(&mut self, id: TodoId, patch: TodoPatch) -> Result<bool> {
        self.service.edit_todo(id, patch)
    }

    /// Escape hatch for infra-specific operations (like saving).
    ///
    /// We'll replace this with a cleaner "Unit of Work" abstraction later,
    /// but it keeps our steps incremental.
    pub fn repo_mut(&mut self) -> &mut R {
        self.service.repo_mut()
    }

    pub fn set_all(&mut self, todos: Vec<Todo>) {
        self.repo_mut().set_all(todos);
    }

    pub fn mark_done(&mut self, id: TodoId) -> Result<(), AppError> {
        let Some(mut todo) = self.repo_mut().get(id) else {
            return Err(AppError::TodoNotFound);
        };

        match todo.mark_done() {
            Ok(()) => {}
            Err(DomainError::AlreadyDone) => return Err(AppError::AlreadyDone),
            Err(_) => return Err(AppError::TodoNotFound),
        }

        if self.repo_mut().replace(todo) {
            Ok(())
        } else {
            Err(AppError::TodoNotFound)
        }
    }

    pub fn mark_open(&mut self, id: TodoId) -> Result<(), AppError> {
        let Some(mut todo) = self.repo_mut().get(id) else {
            return Err(AppError::TodoNotFound);
        };

        match todo.mark_open() {
            Ok(()) => {}
            Err(DomainError::AlreadyOpen) => return Err(AppError::AlreadyOpen),
            Err(_) => return Err(AppError::TodoNotFound),
        }

        if self.repo_mut().replace(todo) {
            Ok(())
        } else {
            Err(AppError::TodoNotFound)
        }
    }

    pub fn delete(&mut self, id: TodoId) -> Result<(), AppError> {
        if self.repo_mut().remove(id) {
            Ok(())
        } else {
            Err(AppError::TodoNotFound)
        }
    }
}
