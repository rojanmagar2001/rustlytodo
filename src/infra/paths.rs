//! Platform-correct config and data paths.

use anyhow::{Result, anyhow};
use directories::ProjectDirs;

pub struct AppPaths {
    pub config_dir: std::path::PathBuf,
    pub data_dir: std::path::PathBuf,
}

impl AppPaths {
    pub fn detect() -> Result<Self> {
        let proj = ProjectDirs::from("com", "example", "rustlytodo")
            .ok_or_else(|| anyhow!("could not determine project directories"))?;

        Ok(Self {
            config_dir: proj.config_dir().to_path_buf(),
            data_dir: proj.data_dir().to_path_buf(),
        })
    }
}
