//! Default data seeding.
//!
//! This is used during early development and for first-run UX.
//! Once persistence is implemented, this will only run if the database is empty.

use time::OffsetDateTime;

use crate::domain::todo::{DueAt, Priority, Title, Todo};

/// Generate a small set of default todos.
///
/// This function is pure: it just returns data.
pub fn default_todos() -> Vec<Todo> {
    let now = OffsetDateTime::now_utc();

    vec![
        {
            let mut t = Todo::new(Title::parse("Welcome to rustlytodo").unwrap());
            t.priority = Priority::P2;
            t
        },
        {
            let mut t = Todo::new(Title::parse("Press ? to view keybindings").unwrap());
            t.priority = Priority::P4;
            t
        },
        {
            let mut t = Todo::new(Title::parse("Add your first real task").unwrap());
            t.priority = Priority::P3;
            t
        },
        {
            let mut t = Todo::new(Title::parse("Task with a due date").unwrap());
            t.priority = Priority::P1;
            t.due = Some(
                DueAt::parse_rfc3339(
                    now.replace_hour(17)
                        .unwrap()
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                )
                .unwrap(),
            );
            t
        },
    ]
}
