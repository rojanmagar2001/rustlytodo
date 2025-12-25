//! Library entry point for the todo application.
//!
//! This allows:
//! - Integration tests (`tests/`) to import and exercise the app
//! - A clean separation between the binary (`main.rs`) and the core logic
//! - Future reuse (e.g. TUI-only binary, benchmarks, etc.)

pub mod app;
pub mod domain;
pub mod infra;
pub mod ui;
