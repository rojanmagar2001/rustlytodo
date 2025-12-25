//! CSV import/export helpers.
//!
//! CSV is intentionally "basic": it flattens a subset of fields for compatibility.

use std::{collections::BTreeSet, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::domain::todo::{DueAt, Notes, Priority, ProjectName, Tag, Title, Todo};

#[derive(Debug, Serialize, Deserialize)]
struct CsvTodoRow {
    // We export full UUID to be unambiguous.
    id: String,
    title: String,
    status: String,   // "open" or "done"
    priority: String, // "P1".."P4"
    project: String,
    due: Option<String>,   // RFC3339
    notes: Option<String>, // plain text
    tags: Option<String>,  // "tag1,tag2"
}

pub fn export_csv(path: &Path, todos: &[Todo]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed creating export dir: {}", parent.display()))?;
        }
    }

    let mut wtr = csv::Writer::from_path(path)
        .with_context(|| format!("failed creating csv file: {}", path.display()))?;

    for t in todos {
        let status = if t.status.is_done() { "done" } else { "open" }.to_string();

        let tags = if t.tags.is_empty() {
            None
        } else {
            Some(
                t.tags
                    .iter()
                    .map(|x| x.as_str().to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            )
        };

        let row = CsvTodoRow {
            id: t.id.as_uuid_str(),
            title: t.title.as_str().to_string(),
            status,
            priority: t.priority.label().to_string(),
            project: t.project.as_str().to_string(),
            due: t.due.map(|d| d.format_rfc3339()),
            notes: t.notes.as_ref().map(|n| n.as_str().to_string()),
            tags,
        };
        wtr.serialize(row).context("failed writing csv row")?;
    }

    wtr.flush().context("failed flushing csv writer")?;
    Ok(())
}

pub fn import_csv(path: &Path) -> Result<Vec<Todo>> {
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("failed opening csv file: {}", path.display()))?;

    let mut todos = Vec::new();

    for rec in rdr.deserialize::<CsvTodoRow>() {
        let row = rec.context("failed reading csv row")?;

        let title = Title::parse(row.title)?;
        let mut t = Todo::new(title);

        // Preserve ID if possible. (Since TodoId is a newtype, we set via parsing.)
        t.id = crate::domain::todo::TodoId::parse_uuid(row.id)?;

        // Status: basic
        if row.status.trim().eq_ignore_ascii_case("done") {
            // We don't store completed_at in CSV (basic), so we just mark done "now"
            // via domain transition.
            let _ = t.mark_done();
        }

        t.priority = Priority::parse(row.priority)?;
        t.project = ProjectName::parse(row.project)?;

        if let Some(due) = row.due {
            t.due = Some(DueAt::parse_rfc3339(due)?);
        }

        if let Some(notes) = row.notes {
            t.notes = Some(Notes::parse(notes)?);
        }

        if let Some(tags) = row.tags {
            let mut set = BTreeSet::new();
            for raw in tags.split(",").map(|s| s.trim()).filter(|s| !s.is_empty()) {
                set.insert(Tag::parse(raw)?);
            }
            t.tags = set;
        }

        todos.push(t);
    }

    Ok(todos)
}
