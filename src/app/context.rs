//! Application context (composition root-ish data).
//!
//! This is *not* domain logic. It's a convenient container for
//! environment/config paths and shared cross-cutting concerns.

use crate::infra::{config::AppConfig, paths::AppPaths};

#[derive(Debug, Clone)]
pub struct AppContext {
    pub paths: AppPaths,
    pub config: AppConfig,
}

impl AppContext {
    pub fn new(paths: AppPaths, config: AppConfig) -> Self {
        Self { paths, config }
    }
}
