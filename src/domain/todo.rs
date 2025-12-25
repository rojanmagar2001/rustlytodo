//! Core Todo entity.
//!
//! This is intentionally minimal for now.

use uuid::Uuid;

use crate::domain::errors::DomainError;

/// Strongly-typed identifier for a Todo.
///
/// Newtype pattern prevents mixing IDs accidentally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TodoId(Uuid);

impl TodoId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Avalidated todo title.
#[derive(Debug, Clone)]
pub struct Title(String);

impl Title {
    // Parse and validate a title
    pub fn parse(input: impl AsRef<str>) -> Result<Self, DomainError> {
        let trimmed = input.as_ref().trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyTitle);
        }

        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Core Todo entity
#[derive(Debug, Clone)]
pub struct Todo {
    pub id: TodoId,
    pub title: Title,
}

impl Todo {
    pub fn new(title: Title) -> Self {
        Self {
            id: TodoId::new(),
            title,
        }
    }
}
