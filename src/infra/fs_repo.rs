use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    app::repository::TodoRepository,
    domain::todo::{Todo, TodoId},
};

/// Current schema version for the on-disk JSON file.
const SCHEMA_VERSION: u32 = 1;

/// On-disk schema wrapper.
/// This lets us migrate formats later without guessing.
#[derive(Debug, Serialize, Deserialize)]
struct DbFile {
    schema_version: u32,
    todos: Vec<Todo>,
}

impl DbFile {
    fn empty() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            todos: Vec::new(),
        }
    }
}

/// JSON repository backed by as single file.
pub struct JsonFileTodoRepository {
    path: PathBuf,
    todos: Vec<Todo>,
}

impl JsonFileTodoRepository {
    pub fn load_or_init(path: PathBuf) -> Result<Self> {
        if path.exists() {
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("failed reading db file: {}", path.display()))?;

            let db: DbFile =
                serde_json::from_str(&text).with_context(|| "failed parsing db JSON")?;

            // For now, only version 1 is supported.
            // We'll add migration in the next persistence steps.
            if db.schema_version != SCHEMA_VERSION {
                anyhow::bail!(
                    "unsupported schema_version {} (expected {})",
                    db.schema_version,
                    SCHEMA_VERSION
                );
            }

            Ok(Self {
                path,
                todos: db.todos,
            })
        } else {
            // Ensure parent dir exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("failed creating db parent dir: {}", parent.display())
                })?;
            }

            let mut repo = Self {
                path,
                todos: Vec::new(),
            };
            repo.save_atomic()?;
            Ok(repo)
        }
    }

    pub fn save_atomic(&self) -> Result<()> {
        let db = DbFile {
            schema_version: SCHEMA_VERSION,
            todos: self.todos.clone(),
        };

        let json = serde_json::to_string_pretty(&db).with_context(|| "failed serializing db")?;

        let tmp_path = tmp_path_for(&self.path);

        std::fs::write(&tmp_path, json)
            .with_context(|| format!("failed writing temp db file: {}", tmp_path.display()))?;

        // Atomic replace on most platforms when temp is in same directory.
        std::fs::rename(&tmp_path, &self.path).with_context(|| {
            format!(
                "failed remaining temp db file {} -> {}",
                tmp_path.display(),
                self.path.display()
            )
        });

        Ok(())
    }
}

fn tmp_path_for(path: &PathBuf) -> PathBuf {
    let mut p = path.to_path_buf();
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "db.json".to_string());
    p.set_file_name(format!("{file_name}.tmp"));

    p
}

impl TodoRepository for JsonFileTodoRepository {
    fn add(&mut self, todo: Todo) {
        self.todos.push(todo);
    }

    fn list(&self) -> Vec<Todo> {
        self.todos.clone()
    }

    fn replace(&mut self, todo: Todo) -> bool {
        if let Some(slot) = self.todos.iter_mut().find(|t| t.id == todo.id) {
            *slot = todo;
            true
        } else {
            false
        }
    }

    fn get(&self, id: TodoId) -> Option<Todo> {
        self.todos.iter().find(|t| t.id == id).cloned()
    }
}
