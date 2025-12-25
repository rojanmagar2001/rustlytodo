//! Command-line interface (non-interactive).
//!
//! This will coexist with the TUI later.

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

use crate::{
    app::service::{self, TodoService},
    domain::todo::Title,
    infra::memory_repo::MemoryTodoRepository,
};

/// Top-level CLI definition.
#[derive(Parser)]
#[command(name = "rustlytodo")]
#[command(about = "A fast, keyboard-first rustly todo app")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new todo
    Add {
        /// Title of the todo
        title: String,
    },

    /// List todos
    List,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    // In Milestone 3 this will be loaded from disk.
    let repo = MemoryTodoRepository::new();
    let mut service = TodoService::new(repo);

    match cli.command {
        Commands::Add { title } => {
            let title = Title::parse(title)?;
            service.add_todo(title)?;
            info!("Todo added");
        }
        Commands::List => {
            let todos = service.list_todos();
            if todos.is_empty() {
                println!("No todos yet 🎉");
            } else {
                for todo in todos {
                    println!("- {}", todo.title.as_str());
                }
            }
        }
    }

    Ok(())
}
