//! Platform-correct config and data paths.

use anyhow::{Result, anyhow};
use directories::ProjectDirs;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_dir: std::path::PathBuf,
    pub data_dir: std::path::PathBuf,
}

impl AppPaths {
    pub fn detect() -> Result<Self> {
        // "com/example/todo" is a placeholder vendor/app id for now.
        // You can later change it to your real org/domain.
        let proj = ProjectDirs::from("com", "example", "rustlytodo")
            .ok_or_else(|| anyhow!("could not determine project directories"))?;

        Ok(Self {
            config_dir: proj.config_dir().to_path_buf(),
            data_dir: proj.data_dir().to_path_buf(),
        })
    }
}
