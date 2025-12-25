//! Command-line interface (non-interactive).
//!
//! This will coexist with the TUI later.

use std::io::{self, Write};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::{debug, info};

use crate::{
    app::repository::TodoRepository,
    app::{context::AppContext, store::Store},
    domain::todo::Title,
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
    List {
        /// Output format: table (default) or json
        #[arg(long, default_value = "table")]
        format: String,

        /// Filter by status: open|done
        #[arg(long)]
        status: Option<String>,

        /// Filter by project name
        #[arg(long)]
        project: Option<String>,

        /// Filter by tag (e.g. --tag rust)
        #[arg(long)]
        tag: Option<String>,

        /// Search text in title/notes
        #[arg(long)]
        search: Option<String>,

        /// Only show overdue (open + due in past)
        #[arg(long)]
        overdue: bool,

        /// Filter by priority: P1..P4
        #[arg(long)]
        priority: Option<String>,

        /// Sort by: due|priority|created
        #[arg(long, default_value = "due")]
        sort: String,

        /// Sort descending
        #[arg(long)]
        desc: bool,
    },

    /// Show a single todo
    Show {
        /// Todo ID (full UUID or unique prefix)
        id: String,

        /// Output format: table (default) or json
        #[arg(long, default_value = "table")]
        format: String,
    },

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

    /// Export todos to a JSON file (lossless).
    Export {
        /// Format: json (lossless) or csv (basic)
        #[arg(long, default_value = "json")]
        format: String,

        /// Output file path
        #[arg(long)]
        out: String,
    },

    /// Import todos from a JSON file (lossless). Replaces current DB.
    Import {
        /// Format: json (lossless) or csv (basic)
        #[arg(long, default_value = "json")]
        format: String,

        /// Input file path
        #[arg(long)]
        r#in: String,
    },

    /// Mark a todo as done
    Done {
        /// Todo ID (full UUID or unique prefix)
        id: String,
    },

    /// Mark a todo as open/undone
    Undone {
        /// Todo ID (full UUID or unique prefix)
        id: String,
    },

    /// Delete a todo (destructive)
    Delete {
        /// Todo ID (full UUID or unique prefix)
        id: String,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
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
    let mut out = io::stdout();
    run_inner(ctx, cli, &mut out)
}

fn run_inner(ctx: AppContext, cli: Cli, out: &mut dyn Write) -> Result<()> {
    debug!(?ctx.paths, "detected application paths");
    debug!(?ctx.config, "loaded configuration");

    let db_path = ctx.config.resolve_db_path(&ctx.paths);
    let mut store = {
        let repo = crate::infra::fs_repo::JsonFileTodoRepository::load_or_init(db_path)?;
        Store::new(repo)
    };

    // Seed defaults only if DB is empty/new.
    if store.is_empty() {
        let defaults = crate::app::seed::default_todos();
        store.insert_many(defaults);
        store.repo_mut().save_atomic()?;
    }

    handle_command(&mut store, cli.command.unwrap_or(Commands::Tui), out)
}

pub fn run_with_args(ctx: AppContext, args: impl IntoIterator<Item = String>) -> Result<()> {
    let cli = Cli::parse_from(args);
    let mut out = io::stdout();
    run_inner(ctx, cli, &mut out)
}

/// Same as run_with_args, but writes output into a provided writer (tests).
pub fn run_with_args_to_writer(
    ctx: AppContext,
    args: impl IntoIterator<Item = String>,
    out: &mut dyn Write,
) -> Result<()> {
    let cli = Cli::parse_from(args);
    run_inner(ctx, cli, out)
}

fn handle_command(
    store: &mut Store<crate::infra::fs_repo::JsonFileTodoRepository>,
    command: Commands,
    out: &mut dyn Write,
) -> Result<()> {
    match command {
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
            store.repo_mut().save_atomic()?;
            info!("Todo added");
            println!("Added {}", id.short());
        }

        Commands::List {
            format,
            status,
            project,
            tag,
            search,
            overdue,
            priority,
            sort,
            desc,
        } => {
            use crate::app::query::{ListQuery, SortKey, StatusFilter, apply_list_query};
            use crate::domain::todo::Priority;

            let now = time::OffsetDateTime::now_utc();

            // Parse status flag
            let status = match status.as_deref().map(|s| s.trim().to_ascii_lowercase()) {
                None => None,
                Some(s) if s == "open" => Some(StatusFilter::Open),
                Some(s) if s == "done" => Some(StatusFilter::Done),
                Some(other) => {
                    writeln!(out, "unknown --status {other} (use open|done)")?;
                    return Ok(());
                }
            };

            // Parse priority
            let priority = match priority {
                None => None,
                Some(p) => Some(Priority::parse(p).map_err(|e| anyhow::anyhow!(e))?),
            };

            // Parse sort key
            let sort_key = match sort.trim().to_ascii_lowercase().as_str() {
                "due" => SortKey::Due,
                "priority" => SortKey::Priority,
                "created" => SortKey::Created,
                other => {
                    writeln!(out, "unknown --sort {other} (use due|priority|created)")?;
                    return Ok(());
                }
            };

            let q = ListQuery {
                status,
                project,
                tag,
                search,
                overdue,
                priority,
                sort: sort_key,
                desc,
            };

            let todos = store.list_todos();
            let todos = apply_list_query(todos, &q, now);

            match format.trim().to_ascii_lowercase().as_str() {
                "json" => {
                    let s = serde_json::to_string_pretty(&todos)
                        .with_context(|| "failed serializing todos to json")?;
                    writeln!(out, "{s}")?;
                }
                "table" => {
                    if todos.is_empty() {
                        writeln!(out, "No matching todos.")?;
                    } else {
                        writeln!(
                            out,
                            "{:<10} {:<2} {:<3} {:<8} {:<10} {:<18} {:<25} {}",
                            "ID", "S", "P", "!", "PROJECT", "TAGS", "DUE", "TITLE"
                        )?;

                        for todo in todos {
                            let due = todo
                                .due
                                .map(|d| d.format_rfc3339())
                                .unwrap_or_else(|| "-".to_string());

                            let overdue_mark = if todo.is_overdue(now) { "OVERDUE" } else { "" };

                            let tags = if todo.tags.is_empty() {
                                "-".to_string()
                            } else {
                                todo.tags
                                    .iter()
                                    .map(|t| format!("#{}", t.as_str()))
                                    .collect::<Vec<_>>()
                                    .join(",")
                            };

                            writeln!(
                                out,
                                "{:<10} {:<2} {:<3} {:<8} {:<10} {:<18} {:<25} {}",
                                todo.id.short(),
                                todo.status_symbol(),
                                todo.priority.label(),
                                overdue_mark,
                                todo.project.as_str(),
                                tags,
                                due,
                                todo.title.as_str()
                            )?;
                        }
                    }
                }
                other => {
                    writeln!(out, "unknown list format: {other} (use table|json)")?;
                }
            }
        }

        Commands::Show { id, format } => {
            let todos = store.list_todos();
            let todo_id = match resolve_id_input(&todos, &id) {
                Ok(x) => x,
                Err(msg) => {
                    writeln!(out, "{msg}")?;
                    return Ok(());
                }
            };

            let Some(todo) = store.repo_mut().get(todo_id) else {
                writeln!(out, "todo not found")?;
                return Ok(());
            };

            match format.trim().to_ascii_lowercase().as_str() {
                "json" => {
                    let s = serde_json::to_string_pretty(&todo)
                        .with_context(|| "failed serializing todo to json")?;
                    writeln!(out, "{s}")?;
                }
                "table" => {
                    // Human friendly details
                    writeln!(out, "ID:       {}", todo.id.as_uuid_str())?;
                    writeln!(out, "Short:    {}", todo.id.short())?;
                    writeln!(
                        out,
                        "Status:   {}",
                        if todo.status.is_done() {
                            "Done"
                        } else {
                            "Open"
                        }
                    )?;
                    writeln!(out, "Priority: {}", todo.priority.label())?;
                    writeln!(out, "Project:  {}", todo.project.as_str())?;
                    writeln!(
                        out,
                        "Due:      {}",
                        todo.due
                            .map(|d| d.format_rfc3339())
                            .unwrap_or_else(|| "-".to_string())
                    )?;

                    let tags = if todo.tags.is_empty() {
                        "-".to_string()
                    } else {
                        todo.tags
                            .iter()
                            .map(|t| format!("#{}", t.as_str()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    writeln!(out, "Tags:     {tags}")?;

                    writeln!(out, "Title:    {}", todo.title.as_str())?;
                    if let Some(n) = &todo.notes {
                        writeln!(out, "Notes:\n{}\n", n.as_str())?;
                    }
                }
                other => {
                    writeln!(out, "unknown show format: {other} (use table|json)")?;
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
            let todo_id = match resolve_id_input(&todos, &id) {
                Ok(x) => x,
                Err(msg) => {
                    println!("{msg}");
                    return Ok(());
                }
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
                store.repo_mut().save_atomic()?;
                println!("Edited {}", id);
            } else {
                println!("Failed to edit {}", id);
            }
        }

        Commands::Done { id } => {
            let todos = store.list_todos();
            let todo_id = match resolve_id_input(&todos, &id) {
                Ok(x) => x,
                Err(msg) => {
                    println!("{msg}");
                    return Ok(());
                }
            };

            match store.mark_done(todo_id) {
                Ok(()) => {
                    store.repo_mut().save_atomic()?;
                    println!("Done {}", id);
                }
                Err(e) => {
                    println!("{e}");
                }
            }
        }

        Commands::Undone { id } => {
            let todos = store.list_todos();
            let todo_id = match resolve_id_input(&todos, &id) {
                Ok(x) => x,
                Err(msg) => {
                    println!("{msg}");
                    return Ok(());
                }
            };

            match store.mark_open(todo_id) {
                Ok(()) => {
                    store.repo_mut().save_atomic()?;
                    println!("Undone {}", id);
                }
                Err(e) => {
                    println!("{e}");
                }
            }
        }

        Commands::Delete { id, yes } => {
            if !yes {
                // Minimal confirmation: require explicit flag.
                // (Better interactive prompts later; this is safe & scriptable.)
                println!("Refusing to delete without confirmation. Re-run with --yes.");
                return Ok(());
            }

            let todos = store.list_todos();
            let todo_id = match resolve_id_input(&todos, &id) {
                Ok(x) => x,
                Err(msg) => {
                    println!("{msg}");
                    return Ok(());
                }
            };

            match store.delete(todo_id) {
                Ok(()) => {
                    store.repo_mut().save_atomic()?;
                    println!("Deleted {}", id);
                }
                Err(e) => {
                    println!("{e}");
                }
            }
        }

        Commands::Export { format, out } => {
            use std::path::PathBuf;

            let out_path = PathBuf::from(out);
            let todos = store.list_todos();

            match format.trim().to_ascii_lowercase().as_str() {
                "json" => {
                    let json = crate::infra::db_schema::write_current(&todos)?;

                    if let Some(parent) = out_path.parent() {
                        if !parent.as_os_str().is_empty() {
                            std::fs::create_dir_all(parent).with_context(|| {
                                format!("failed creating export directory: {}", parent.display())
                            })?;
                        }
                    }

                    std::fs::write(&out_path, json).with_context(|| {
                        format!("failed writing export file: {}", out_path.display())
                    })?;
                }
                "csv" => {
                    crate::infra::csv_io::export_csv(&out_path, &todos)?;
                }
                other => {
                    println!("unknown export format: {other} (use json|csv)");
                    return Ok(());
                }
            }

            println!("Exported {} todos to {}", todos.len(), out_path.display());
        }

        Commands::Import { format, r#in } => {
            use std::path::PathBuf;

            let in_path = PathBuf::from(r#in);

            let todos = match format.trim().to_ascii_lowercase().as_str() {
                "json" => {
                    let text = std::fs::read_to_string(&in_path).with_context(|| {
                        format!("failed reading import file: {}", in_path.display())
                    })?;
                    crate::infra::db_schema::load_any(&text)?
                }
                "csv" => crate::infra::csv_io::import_csv(&in_path)?,
                other => {
                    println!("unknown import format: {other} (use json|csv)");
                    return Ok(());
                }
            };

            let count = todos.len();

            store.set_all(todos);
            store.repo_mut().save_atomic()?; // persist immediately

            println!("Imported {} todos from {}", count, in_path.display());
        }
    }
    Ok(())
}

fn resolve_id_input(
    todos: &[crate::domain::todo::Todo],
    input: &str,
) -> Result<crate::domain::todo::TodoId, String> {
    let s = input.trim();

    // 1) If it's a full UUID, accept it directly.
    if let Ok(id) = crate::domain::todo::TodoId::parse_uuid(s) {
        return Ok(id);
    }

    // 2) Otherwise treat it as a prefix match on short() or full UUID.
    if s.len() < 4 {
        return Err("id prefix too short (use at least 4 chars, or full UUID)".to_string());
    }

    let mut matches = Vec::new();
    for t in todos {
        let full = t.id.as_uuid_str();
        if t.id.short() == s || full.starts_with(s) {
            matches.push((t.id, t.title.as_str().to_string()));
        }
    }

    match matches.len() {
        0 => Err(format!("no todo found matching id: {}", s)),
        1 => Ok(matches[0].0),
        _ => {
            let mut msg = format!("ambiguous id '{}'. Matches:\n", s);
            for (id, title) in matches.into_iter().take(10) {
                msg.push_str(&format!("  {}  {}\n", id.short(), title));
            }
            Err(msg)
        }
    }
}
