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

    /// A short, human-friendly identifier (first 8 hex chars).
    ///
    /// Used for CLI/TUI display. Not guaranteed unique forever, but good enough
    /// for daily usage; later we’ll add collision handling when resolving IDs.
    pub fn short(&self) -> String {
        let s = self.0.to_string(); // UUID is hex with hyphens.
        // Example: "550e8400-e29b-41d4-a716-446655440000"
        // Take the first 8 chars for a concise prefix.
        s.chars().take(8).collect()
    }

    // Full UUID string(used for export/debugging).
    pub fn as_uuid_str(&self) -> String {
        self.0.to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::DomainError;

    #[test]
    fn title_parse_rejects_empty() {
        let err = Title::parse("   ").unwrap_err();
        match err {
            DomainError::EmptyTitle => {}
        }
    }

    #[test]
    fn title_parse_trims_and_accepts() {
        let title = Title::parse("  Buy milk  ").expect("valid title");
        assert_eq!(title.as_str(), "Buy milk");
    }

    #[test]
    fn todo_new_generates_id() {
        let t1 = Todo::new(Title::parse("A").unwrap());
        let t2 = Todo::new(Title::parse("B").unwrap());
        // Probabilistic but effectively guaranteed for v4 UUIDs.
        assert!(t1.id != t2.id);
    }
}
