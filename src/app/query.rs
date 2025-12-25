//! App-layer query logic (filtering/sorting).
//!
//! Keeps UI thin and reusable for TUI later.

use crate::domain::todo::{Priority, Todo};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusFilter {
    Open,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Due,
    Priority,
    Created,
}

#[derive(Debug, Clone)]
pub struct ListQuery {
    pub status: Option<StatusFilter>,
    pub project: Option<String>,
    pub tag: Option<String>,
    pub search: Option<String>,
    pub overdue: bool,
    pub priority: Option<Priority>,
    pub sort: SortKey,
    pub desc: bool,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            status: None,
            project: None,
            tag: None,
            search: None,
            overdue: false,
            priority: None,
            sort: SortKey::Due,
            desc: false,
        }
    }
}

pub fn apply_list_query(mut todos: Vec<Todo>, q: &ListQuery, now: OffsetDateTime) -> Vec<Todo> {
    // Filter
    todos.retain(|t| {
        // status
        if let Some(sf) = q.status {
            let is_done = t.status.is_done();
            match sf {
                StatusFilter::Open if is_done => return false,
                StatusFilter::Done if !is_done => return false,
                _ => {}
            }
        }

        // project (case-insensitive match)
        if let Some(p) = &q.project {
            if !t.project.as_str().eq_ignore_ascii_case(p.trim()) {
                return false;
            }
        }

        // tag (normalized tags are lowercase)
        if let Some(tag) = &q.tag {
            let needle = tag.trim().to_ascii_lowercase();
            if !t.tags.iter().any(|x| x.as_str() == needle) {
                return false;
            }
        }

        // priority
        if let Some(pr) = q.priority {
            if t.priority != pr {
                return false;
            }
        }

        // overdue
        if q.overdue && !t.is_overdue(now) {
            return false;
        }

        // search (title + notes)
        if let Some(s) = &q.search {
            let needle = s.trim().to_ascii_lowercase();
            if needle.is_empty() {
                // ignore empty search
            } else {
                let title = t.title.as_str().to_ascii_lowercase();
                let notes = t
                    .notes
                    .as_ref()
                    .map(|n| n.as_str().to_ascii_lowercase())
                    .unwrap_or_default();

                if !title.contains(&needle) && !notes.contains(&needle) {
                    return false;
                }
            }
        }

        true
    });

    // Sort
    todos.sort_by(|a, b| match q.sort {
        SortKey::Due => {
            // None due sorts after Some due by default (strong default UX).
            // Within due: earlier first.
            a.due.cmp(&b.due)
        }
        SortKey::Priority => a.priority.cmp(&b.priority), // P1 < P4
        SortKey::Created => a.created_at.cmp(&b.created_at),
    });

    if q.desc {
        todos.reverse();
    }

    todos
}
