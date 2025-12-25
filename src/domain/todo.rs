//! Core Todo entity.
//!
//! This is intentionally minimal for now.

use std::collections::BTreeSet;

use time::{OffsetDateTime, format_description::well_known::Rfc3339};
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

/// Notes (optional, validated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notes(String);

impl Notes {
    const MAX_LEN: usize = 10_000;

    pub fn parse(input: impl AsRef<str>) -> Result<Self, DomainError> {
        let s = input.as_ref().trim().to_string();
        if s.len() > Self::MAX_LEN {
            return Err(DomainError::NotesTooLong { max: Self::MAX_LEN });
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Project/context name (validated).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProjectName(String);

impl ProjectName {
    pub fn parse(input: impl AsRef<str>) -> Result<Self, DomainError> {
        let trimmed = input.as_ref().trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyProjectName);
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn inbox() -> Self {
        // Safe default project.
        Self("Inbox".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Tag (validated + normalized to lowercase).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag(String);

impl Tag {
    pub fn parse(input: impl AsRef<str>) -> Result<Self, DomainError> {
        let raw = input.as_ref().trim();
        if raw.is_empty() {
            return Err(DomainError::EmptyTag);
        }

        let normalized = raw.to_ascii_lowercase();

        let ok = normalized
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');

        if !ok {
            return Err(DomainError::InvalidTag);
        }

        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Priority level.
///
/// P1 is highest urgency; P4 is lowest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Priority {
    P1,
    P2,
    #[default]
    P3,
    P4,
}

impl Priority {
    /// Parse from user input like "p1", "P2", etc.
    pub fn parse(input: impl AsRef<str>) -> Result<Self, DomainError> {
        match input.as_ref().trim().to_ascii_uppercase().as_str() {
            "P1" => Ok(Priority::P1),
            "P2" => Ok(Priority::P2),
            "P3" => Ok(Priority::P3),
            "P4" => Ok(Priority::P4),
            _ => Err(DomainError::InvalidPriority),
        }
    }

    /// Display-friendly label.
    pub fn label(self) -> &'static str {
        match self {
            Priority::P1 => "P1",
            Priority::P2 => "P2",
            Priority::P3 => "P3",
            Priority::P4 => "P4",
        }
    }
}

/// Due datetime (UTC for now).
///
/// We store this as an `OffsetDateTime`. For now we treat input as RFC3339.
/// Later we can add "friendly" parsing (e.g. `tomorrow 9am`) at the app/UI layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DueAt(OffsetDateTime);

impl DueAt {
    pub fn parse_rfc3339(input: impl AsRef<str>) -> Result<Self, DomainError> {
        let s = input.as_ref().trim();
        let dt = OffsetDateTime::parse(s, &Rfc3339).map_err(|_| DomainError::InvalidDueAt)?;
        Ok(Self(dt))
    }

    /// Construct directly from a datetime (useful for programmatic creation / seeding).
    pub fn from_dt(dt: OffsetDateTime) -> Self {
        Self(dt)
    }

    pub fn as_dt(self) -> OffsetDateTime {
        self.0
    }

    /// Format for UI display (RFC3339). We keep formatting simple + predictable.
    pub fn format_rfc3339(self) -> String {
        self.0
            .format(&Rfc3339)
            .unwrap_or_else(|_| "<invalid-datetime>".to_string())
    }
}

/// Todo status.
///
/// If Done, we record when it was completed (UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Open,
    Done { completed_at: OffsetDateTime },
}

impl Status {
    pub fn is_done(self) -> bool {
        matches!(self, Status::Done { .. })
    }
}

/// Core Todo entity
#[derive(Debug, Clone)]
pub struct Todo {
    pub id: TodoId,
    pub title: Title,
    pub notes: Option<Notes>,
    pub project: ProjectName,
    pub tags: BTreeSet<Tag>,
    pub status: Status,
    pub priority: Priority,
    pub due: Option<DueAt>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Todo {
    /// Create a new Todo with defaults:
    /// - status = Open
    /// - priority = P3
    /// - due = None
    /// - project = Inbox
    /// - tags empty
    /// - notes None
    pub fn new(title: Title) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: TodoId::new(),
            title,
            notes: None,
            project: ProjectName::inbox(),
            tags: BTreeSet::new(),
            status: Status::Open,
            priority: Priority::default(),
            due: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark done, if currently open.
    pub fn mark_done(&mut self) -> Result<(), DomainError> {
        match self.status {
            Status::Open => {
                let now = OffsetDateTime::now_utc();
                self.status = Status::Done { completed_at: now };
                self.updated_at = now;
                Ok(())
            }
            Status::Done { .. } => Err(DomainError::AlreadyDone),
        }
    }

    /// Mark open/undone, if currently done.
    pub fn mark_open(&mut self) -> Result<(), DomainError> {
        match self.status {
            Status::Done { .. } => {
                let now = OffsetDateTime::now_utc();
                self.status = Status::Open;
                self.updated_at = now;
                Ok(())
            }
            Status::Open => Err(DomainError::AlreadyOpen),
        }
    }

    /// Convenience for UI rendering.
    pub fn status_symbol(&self) -> &'static str {
        match self.status {
            Status::Open => "☐",
            Status::Done { .. } => "☑",
        }
    }

    /// Returns true if the todo is open and its due date is before `now`.
    pub fn is_overdue(&self, now: OffsetDateTime) -> bool {
        if self.status.is_done() {
            return false;
        }

        match self.due {
            Some(due) => due.as_dt() < now,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use time::Duration;

    use super::*;
    use crate::domain::errors::DomainError;

    #[test]
    fn title_parse_rejects_empty() {
        let err = Title::parse("   ").unwrap_err();
        assert_eq!(err, DomainError::EmptyTitle);
    }

    #[test]
    fn title_parse_trims_and_accepts() {
        let title = Title::parse("  Buy milk  ").expect("valid title");
        assert_eq!(title.as_str(), "Buy milk");
    }

    #[test]
    fn tag_is_normalized_to_lowercase() {
        let t = Tag::parse("Work").unwrap();
        assert_eq!(t.as_str(), "work");
    }

    #[test]
    fn tag_rejects_invalid_chars() {
        assert!(Tag::parse("hello!").is_err());
        assert!(Tag::parse("space tag").is_err());
    }

    #[test]
    fn project_name_requires_non_empty() {
        assert!(ProjectName::parse("   ").is_err());
        assert_eq!(ProjectName::parse("Work").unwrap().as_str(), "Work");
    }

    #[test]
    fn notes_max_len_is_enforced() {
        let big = "a".repeat(10_001);
        assert!(Notes::parse(big).is_err());
    }

    #[test]
    fn todo_defaults_project_inbox() {
        let todo = Todo::new(Title::parse("A").unwrap());
        assert_eq!(todo.project.as_str(), "Inbox");
        assert!(todo.tags.is_empty());
        assert!(todo.notes.is_none());
    }

    #[test]
    fn overdue_only_when_open_and_due_in_past() {
        let now = OffsetDateTime::now_utc();
        let mut todo = Todo::new(Title::parse("A").unwrap());
        todo.due = Some(DueAt::from_dt(now - Duration::days(1)));
        assert!(todo.is_overdue(now));

        todo.mark_done().unwrap();
        assert!(!todo.is_overdue(now));
    }

    #[test]
    fn priority_parse_accepts_p1_to_p4_case_insensitive() {
        assert_eq!(Priority::parse("p1").unwrap(), Priority::P1);
        assert_eq!(Priority::parse("P2").unwrap(), Priority::P2);
        assert_eq!(Priority::parse(" P3 ").unwrap(), Priority::P3);
        assert_eq!(Priority::parse("p4").unwrap(), Priority::P4);
    }

    #[test]
    fn priority_parse_rejects_invalid() {
        let err = Priority::parse("p9").unwrap_err();
        assert_eq!(err, DomainError::InvalidPriority);
    }

    #[test]
    fn dueat_parse_rfc3339_works_for_zulu() {
        let due = DueAt::parse_rfc3339("2026-01-02T09:00:00Z").unwrap();
        assert_eq!(due.format_rfc3339(), "2026-01-02T09:00:00Z");
    }

    #[test]
    fn dueat_parse_rejects_garbage() {
        let err = DueAt::parse_rfc3339("tomorrow at 9").unwrap_err();
        assert_eq!(err, DomainError::InvalidDueAt);
    }

    #[test]
    fn mark_done_transitions_open_to_done() {
        let mut todo = Todo::new(Title::parse("A").unwrap());
        assert_eq!(todo.status, Status::Open);

        todo.mark_done().unwrap();
        assert!(todo.status.is_done());
        assert!(todo.updated_at >= todo.created_at);
    }

    #[test]
    fn mark_done_rejects_if_already_done() {
        let mut todo = Todo::new(Title::parse("A").unwrap());
        todo.mark_done().unwrap();
        let err = todo.mark_done().unwrap_err();
        assert_eq!(err, DomainError::AlreadyDone);
    }

    #[test]
    fn mark_open_transitions_done_to_open() {
        let mut todo = Todo::new(Title::parse("A").unwrap());
        todo.mark_done().unwrap();
        todo.mark_open().unwrap();
        assert_eq!(todo.status, Status::Open);
    }
}
