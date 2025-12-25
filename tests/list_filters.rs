use anyhow::Result;
use tempfile::tempdir;

use rustytodo::app::context::AppContext;
use rustytodo::infra::config::{AppConfig, Theme};
use rustytodo::infra::paths::AppPaths;

fn test_ctx() -> Result<AppContext> {
    let dir = tempdir()?;

    let paths = AppPaths {
        config_dir: dir.path().join("cfg"),
        data_dir: dir.path().join("data"),
    };

    let mut cfg = AppConfig::default();
    cfg.theme = Theme::Dark;
    cfg.storage_path = Some(dir.path().join("db.json"));

    Ok(AppContext::new(paths, cfg))
}

#[test]
fn list_status_open_returns_only_open() -> Result<()> {
    let ctx = test_ctx()?;

    // Seed is automatic. Mark first rustytodo done, then filter open should exclude it.
    let mut buf = Vec::new();
    rustytodo::ui::cli::run_with_args_to_writer(
        ctx.clone(),
        vec![
            "rustytodo".into(),
            "list".into(),
            "--format".into(),
            "json".into(),
        ],
        &mut buf,
    )?;
    let todos: Vec<rustytodo::domain::todo::Todo> = serde_json::from_slice(&buf)?;
    let first = todos[0].id.short();

    let mut out = Vec::new();
    rustytodo::ui::cli::run_with_args_to_writer(
        ctx.clone(),
        vec!["rustytodo".into(), "done".into(), first],
        &mut out,
    )?;

    // Now list only open
    let mut buf2 = Vec::new();
    rustytodo::ui::cli::run_with_args_to_writer(
        ctx,
        vec![
            "rustytodo".into(),
            "list".into(),
            "--format".into(),
            "json".into(),
            "--status".into(),
            "open".into(),
        ],
        &mut buf2,
    )?;
    let open: Vec<rustytodo::domain::todo::Todo> = serde_json::from_slice(&buf2)?;
    assert!(open.iter().all(|t| !t.status.is_done()));

    Ok(())
}

#[test]
fn list_sort_priority_orders_p1_first() -> Result<()> {
    let ctx = test_ctx()?;

    // list sorted by priority
    let mut buf = Vec::new();
    rustytodo::ui::cli::run_with_args_to_writer(
        ctx,
        vec![
            "rustytodo".into(),
            "list".into(),
            "--format".into(),
            "json".into(),
            "--sort".into(),
            "priority".into(),
        ],
        &mut buf,
    )?;

    let todos: Vec<rustytodo::domain::todo::Todo> = serde_json::from_slice(&buf)?;

    // Verify non-decreasing order (P1..P4)
    for w in todos.windows(2) {
        assert!(w[0].priority <= w[1].priority);
    }

    Ok(())
}
