//! Application entry point.
//!
//! Responsibilities:
//! - Initialize logging
//! - Detect config/data paths
//! - Parse CLI arguments
//! - Wire UI → application services → repository

use anyhow::Result;
use tracing::Level;

mod app;
mod domain;
mod infra;
mod ui;

fn main() -> Result<()> {
    // Initialize structured logging.
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    // Delegate everything else to the CLI UI for now.
    ui::cli::run()
}
