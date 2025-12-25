//! Default data seeding (first-run / dev UX).

use std::collections::BTreeSet;

use time::Duration;
use time::OffsetDateTime;

use crate::domain::todo::{DueAt, Notes, Priority, ProjectName, Tag, Title, Todo};

pub fn default_todos() -> Vec<Todo> {
    let now = OffsetDateTime::now_utc();

    let inbox = ProjectName::inbox();
    let work = ProjectName::parse("Work").unwrap();

    let mut t1 = Todo::new(Title::parse("Welcome to rustlytodo").unwrap());
    t1.project = inbox.clone();
    t1.priority = Priority::P2;
    t1.notes = Some(Notes::parse("Tip: use `todo add \"...\" --tag work`").unwrap());

    let mut t2 = Todo::new(Title::parse("Press ? to view keybindings (TUI later)").unwrap());
    t2.project = inbox.clone();
    t2.priority = Priority::P4;

    let mut t3 = Todo::new(Title::parse("Fix CI flaky test").unwrap());
    t3.project = work;
    t3.priority = Priority::P1;
    t3.due = Some(DueAt::from_dt(now + Duration::days(3)));

    let mut tags = BTreeSet::new();
    tags.insert(Tag::parse("rust").unwrap());
    tags.insert(Tag::parse("build").unwrap());
    t3.tags = tags;

    vec![t1, t2, t3]
}
