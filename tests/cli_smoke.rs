use anyhow::Result;
use tempfile::tempdir;

use rustytodo::app::context::AppContext;
use rustytodo::infra::config::{AppConfig, Theme};
use rustytodo::infra::paths::AppPaths;

#[test]
fn done_and_delete_flow() -> Result<()> {
    let dir = tempdir()?;

    let paths = AppPaths {
        config_dir: dir.path().join("cfg"),
        data_dir: dir.path().join("data"),
    };

    let mut cfg = AppConfig::default();
    cfg.theme = Theme::Dark;
    cfg.storage_path = Some(dir.path().join("db.json"));

    let ctx = AppContext::new(paths, cfg);

    // List as JSON and parse output
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

    let rustytodos: Vec<rustytodo::domain::todo::Todo> = serde_json::from_slice(&buf)?;
    let first_id = rustytodos[0].id.short();

    // Mark done
    let mut out = Vec::new();
    rustytodo::ui::cli::run_with_args_to_writer(
        ctx.clone(),
        vec!["rustytodo".into(), "done".into(), first_id.clone()],
        &mut out,
    )?;

    // Delete
    let mut out2 = Vec::new();
    rustytodo::ui::cli::run_with_args_to_writer(
        ctx,
        vec![
            "rustytodo".into(),
            "delete".into(),
            first_id,
            "--yes".into(),
        ],
        &mut out2,
    )?;

    Ok(())
}
