//! App configuration (loaded from config.toml).
//!
//! This is intentionally small for now. We'll expand it as features land:
//! - theme selection
//! - keybind customization
//! - default filters/sorting
//! - storage path override
//!
//! Design goal: typed config with sane defaults and helpful errors.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::infra::paths::AppPaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Optional override for where the main database file lives.
    /// If None, we'll use paths.data_dir in later milestones.
    pub storage_path: Option<PathBuf>,

    /// UI theme preference (we'll implement in the TUI milestones).
    pub theme: Theme,

    /// If true, we may show extra UI hints / debug info later.
    pub show_hints: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    HighContrast,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            storage_path: None,
            theme: Theme::Dark,
            show_hints: true,
        }
    }
}

impl AppConfig {
    pub fn config_file_path(paths: &AppPaths) -> PathBuf {
        paths.config_dir.join("config.toml")
    }

    /// Load config.toml if it exists; otherwise create it with defaults.
    pub fn load_or_create(paths: &AppPaths) -> Result<Self> {
        let path = Self::config_file_path(paths);

        if path.exists() {
            let s = std::fs::read_to_string(&path)
                .with_context(|| format!("failed reading config file: {}", path.display()))?;

            let cfg: AppConfig =
                toml::from_str(&s).with_context(|| "failed parsing config.toml")?;
            Ok(cfg)
        } else {
            let cfg = AppConfig::default();
            cfg.save_to(&path)?;
            Ok(cfg)
        }
    }

    fn save_to(&self, path: &PathBuf) -> Result<()> {
        let toml_str =
            toml::to_string(self).with_context(|| "failed serializing config to TOML")?;

        // Ensure parent directory exists.
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed creating config directory: {}", parent.display())
            })?;
        }

        std::fs::write(path, toml_str)
            .with_context(|| format!("failed writing config file: {}", path.display()))?;

        Ok(())
    }

    /// Resolve the database path, using config override if present.
    pub fn resolve_db_path(&self, paths: &AppPaths) -> PathBuf {
        self.storage_path
            .clone()
            .unwrap_or_else(|| paths.data_dir.join("db.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn config_roundtrip_toml() {
        let cfg = AppConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let parsed: AppConfig = toml::from_str(&s).unwrap();
        assert!(matches!(parsed.theme, Theme::Dark));
        assert!(parsed.show_hints);
        assert!(parsed.storage_path.is_none());
    }

    #[test]
    fn load_or_create_creates_file_when_missing() {
        let dir = tempdir().unwrap();
        let paths = AppPaths {
            config_dir: dir.path().join("cfg"),
            data_dir: dir.path().join("data"),
        };

        let cfg = AppConfig::load_or_create(&paths).unwrap();
        assert!(matches!(cfg.theme, Theme::Dark));

        let config_path = AppConfig::config_file_path(&paths);
        assert!(config_path.exists());
    }
}
