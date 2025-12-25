//! Command-line interface (non-interactive).
//!
//! This will coexist with the TUI later.

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{debug, info};

use crate::{
    app::{context::AppContext, store::Store},
    domain::todo::{Priority, Title},
    infra::memory_repo::MemoryTodoRepository,
};

/// Top-level CLI definition.
#[derive(Parser)]
#[command(name = "rustlytodo")]
#[command(about = "A fast, keyboard-first rustly todo app")]
struct Cli {
    /// Enable debug logging (useful for troubleshooting)
    #[arg(long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the interactive TUI (default if no subcommand is provided)
    Tui,
    /// Add a new todo
    Add {
        /// Title of the todo
        title: String,

        /// Project/context name (default: Inbox)
        #[arg(long)]
        project: Option<String>,

        /// Tags (repeatable): --tag work --tag urgent
        #[arg(long = "tag")]
        tags: Vec<String>,

        /// Optional notes
        #[arg(long)]
        notes: Option<String>,

        /// Priority: P1 (high) .. P4 (low)
        #[arg(long)]
        priority: Option<String>,

        /// Due datetime in RFC3339, e.g. 2026-01-02T09:00:00Z
        #[arg(long)]
        due: Option<String>,
    },

    /// List todos
    List,

    /// Edit an existing todo by short ID (from `list`)
    Edit {
        /// Short ID (first 8 chars shown in list)
        id: String,

        #[arg(long)]
        title: Option<String>,

        #[arg(long)]
        notes: Option<String>,

        #[arg(long)]
        clear_notes: bool,

        #[arg(long)]
        project: Option<String>,

        #[arg(long)]
        priority: Option<String>,

        #[arg(long)]
        due: Option<String>,

        #[arg(long)]
        clear_due: bool,

        /// Replace tags entirely (repeatable): --tag work --tag urgent
        #[arg(long = "tag")]
        tags: Vec<String>,

        #[arg(long)]
        clear_tags: bool,
    },
}

/// Peek `--debug` from args without fully running the CLI.
///
/// This lets `main` initialize logging at the correct level before we do real work.
pub fn peek_debug_flag() -> bool {
    std::env::args().any(|a| a == "--debug")
}

pub fn run(ctx: AppContext) -> Result<()> {
    let cli = Cli::parse();

    debug!(?ctx.paths, "detected application paths");
    debug!(?ctx.config, "loaded configuration");

    // In Milestone 3 this will be loaded from disk.
    let repo = MemoryTodoRepository::new();
    let mut store = Store::new(repo);

    if store.is_empty() {
        let defaults = crate::app::seed::default_todos();
        store.insert_many(defaults);
    }

    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => {
            // Placeholder until Milestone 5 (ratatui foundation).
            println!("TUI not implemented yet (coming in Milestone 5).");
            println!("For now try: todo add \"Buy milk\"   or   todo list");
        }
        Commands::Add {
            title,
            project,
            tags,
            notes,
            priority,
            due,
        } => {
            use crate::domain::todo::{DueAt, Notes, Priority, ProjectName, Tag, Todo};
            use std::collections::BTreeSet;

            let title = Title::parse(title)?;
            let mut todo = Todo::new(title);

            if let Some(p) = project {
                todo.project = ProjectName::parse(p)?;
            }

            if let Some(n) = notes {
                todo.notes = Some(Notes::parse(n)?)
            }

            if !tags.is_empty() {
                let mut set = BTreeSet::new();
                for t in tags {
                    set.insert(Tag::parse(t)?);
                }
                todo.tags = set;
            }

            if let Some(p) = priority {
                todo.priority = Priority::parse(p)?;
            }

            if let Some(d) = due {
                todo.due = Some(DueAt::parse_rfc3339(d)?);
            }

            // For now we insert the constructed todo directly.
            // Later, add/edit will be proper use-cases with validation + events.
            let id = todo.id;
            store.insert_todo(todo);
            info!("Todo added");
            println!("Added {}", id.short());
        }
        Commands::List => {
            let todos = store.list_todos();
            if todos.is_empty() {
                println!("No todos yet 🎉");
            } else {
                println!(
                    "{:<10} {:<2} {:<3} {:<8} {:<10} {:<18} {:<25} TITLE",
                    "ID", "S", "P", "!", "PROJECT", "TAGS", "DUE",
                );
                for todo in todos {
                    let now = time::OffsetDateTime::now_utc();
                    let due = todo
                        .due
                        .map(|d| d.format_rfc3339())
                        .unwrap_or_else(|| "-".to_string());
                    let overdue = if todo.is_overdue(now) { "OVERDUE" } else { "" };

                    let tags = if todo.tags.is_empty() {
                        "-".to_string()
                    } else {
                        todo.tags
                            .iter()
                            .map(|t| format!("#{}", t.as_str()))
                            .collect::<Vec<_>>()
                            .join(",")
                    };

                    println!(
                        "{:<10} {:<2} {:<3} {:<8} {:<10} {:<18} {:<25} {}",
                        todo.id.short(),
                        todo.status_symbol(),
                        todo.priority.label(),
                        todo.project.as_str(),
                        overdue,
                        tags,
                        due,
                        todo.title.as_str()
                    );
                }
            }
        }
        Commands::Edit {
            id,
            title,
            notes,
            clear_notes,
            project,
            priority,
            due,
            clear_due,
            tags,
            clear_tags,
        } => {
            use crate::domain::todo::{DueAt, Notes, Priority, ProjectName, Tag, Title, TodoPatch};
            use std::collections::BTreeSet;

            let todos = store.list_todos();
            let Some(todo_id) = resolve_short_id(&todos, &id) else {
                println!("No todo found with id prefix: {}", id);
                return Ok(());
            };

            let mut patch = TodoPatch::default();

            if let Some(t) = title {
                patch.title = Some(Title::parse(t)?)
            }

            if clear_notes {
                patch.notes = Some(None);
            } else if let Some(n) = notes {
                patch.notes = Some(Some(Notes::parse(n)?));
            }

            if let Some(p) = project {
                patch.project = Some(ProjectName::parse(p)?);
            }
            if let Some(p) = priority {
                patch.priority = Some(Priority::parse(p)?);
            }

            if clear_due {
                patch.due = Some(None);
            } else if let Some(d) = due {
                patch.due = Some(Some(DueAt::parse_rfc3339(d)?));
            }

            if clear_tags {
                patch.tags = Some(BTreeSet::new());
            } else if !tags.is_empty() {
                let mut set = BTreeSet::new();
                for t in tags {
                    set.insert(Tag::parse(t)?);
                }
                patch.tags = Some(set);
            }

            let changed = store.edit_todo(todo_id, patch)?;
            if changed {
                println!("Edited {}", id);
            } else {
                println!("Failed to edit {}", id);
            }
        }
    }

    Ok(())
}

fn resolve_short_id(
    todos: &[crate::domain::todo::Todo],
    short: &str,
) -> Option<crate::domain::todo::TodoId> {
    todos.iter().find(|t| t.id.short() == short).map(|t| t.id)
}
