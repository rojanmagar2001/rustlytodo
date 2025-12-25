//! Command-line interface (non-interactive).
//!
//! This will coexist with the TUI later.

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{debug, info};

use crate::{
    app::{context::AppContext, store::Store},
    domain::todo::Title,
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
    },

    /// List todos
    List,
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

    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => {
            // Placeholder until Milestone 5 (ratatui foundation).
            println!("TUI not implemented yet (coming in Milestone 5).");
            println!("For now try: todo add \"Buy milk\"   or   todo list");
        }
        Commands::Add { title } => {
            let title = Title::parse(title)?;
            let id = store.add_todo(title)?;
            info!("Todo added");
            println!("Added {}", id.short());
        }
        Commands::List => {
            let todos = store.list_todos();
            if todos.is_empty() {
                println!("No todos yet 🎉");
            } else {
                // Column-ish output (simple, no extra deps)
                println!("{:<10} {}", "ID", "TITLE");
                for todo in todos {
                    println!("{:<10} {}", todo.id.short(), todo.title.as_str());
                }
            }
        }
    }

    Ok(())
}
