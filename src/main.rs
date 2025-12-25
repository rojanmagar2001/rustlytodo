//! Application entry point.
//!
//! Responsibilities:
//! - Initialize logging
//! - Detect config/data paths
//! - Parse CLI arguments
//! - Wire UI → application services → repository

use anyhow::{Context, Result};
use tracing::Level;

mod app;
mod domain;
mod infra;
mod ui;

fn main() -> Result<()> {
    // Parse only the global flags first (currently just --debug).
    // We do this before initializing logging.
    let debug_enabled = ui::cli::peek_debug_flag();

    let level = if debug_enabled {
        Level::DEBUG
    } else {
        Level::INFO
    };

    // Initialize structured logging.
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();

    // Detect app paths once and share via context.
    let paths = infra::paths::AppPaths::detect()?;

    // Ensure directories exist early (cross-platform friendly).
    std::fs::create_dir_all(&paths.config_dir)
        .with_context(|| format!("failed creating config dir: {}", paths.config_dir.display()))?;
    std::fs::create_dir_all(&paths.data_dir)
        .with_context(|| format!("failed creating data dir: {}", paths.data_dir.display()))?;

    let config = infra::config::AppConfig::load_or_create(&paths)?;
    let ctx = app::context::AppContext::new(paths, config);

    // Delegate everything else to the CLI UI for now.
    ui::cli::run(ctx)
}
