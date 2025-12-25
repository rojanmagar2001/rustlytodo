use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    app::repository::TodoRepository,
    domain::todo::{Todo, TodoId},
    infra::db_schema,
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

            let todos = db_schema::load_any(&text)?;
            Ok(Self { path, todos: todos })
        } else {
            // Ensure parent dir exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("failed creating db parent dir: {}", parent.display())
                })?;
            }

            let repo = Self {
                path,
                todos: Vec::new(),
            };
            repo.save_atomic()?;
            Ok(repo)
        }
    }

    /// Save current in-memory state to disk using an atomic replace.
    ///
    /// Durability strategy (best-effort):
    /// 1) write temp file
    /// 2) fsync temp file
    /// 3) rename temp -> final
    /// 4) best-effort fsync parent dir
    pub fn save_atomic(&self) -> Result<()> {
        let json = db_schema::write_current(&self.todos)?;

        let tmp_path = tmp_path_for(&self.path);

        write_file_and_sync(&tmp_path, json.as_bytes())
            .with_context(|| format!("failed writing temp db file: {}", tmp_path.display()))?;

        // Atomic replace on most platforms when temp is in same directory.
        std::fs::rename(&tmp_path, &self.path).with_context(|| {
            format!(
                "failed remaining temp db file {} -> {}",
                tmp_path.display(),
                self.path.display()
            )
        })?;

        // Best-effort directory fsync (platform-dependent).
        if let Some(parent) = self.path.parent() {
            let _ = sync_dir_best_effort(parent);
        }

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

fn write_file_and_sync(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f =
        File::create(path).with_context(|| format!("failed creating file: {}", path.display()))?;
    f.write_all(bytes)
        .with_context(|| format!("failed writing file: {}", path.display()))?;
    f.sync_all()
        .with_context(|| format!("failed fsync file: {}", path.display()))?;

    Ok(())
}

/// Best-effort fsync of a directory.
/// On some platforms/filesystems this may fail; that's okay.
fn sync_dir_best_effort(dir: &Path) -> Result<()> {
    // On Unix-like systems (including macOS), opening a directory as a File is allowed.
    // On Windows it may fail depending on permissions/filesystem.
    let f = File::open(dir).with_context(|| format!("failed opening dir: {}", dir.display()))?;
    f.sync_all()
        .with_context(|| format!("failed fsync dir: {}", dir.display()))?;
    Ok(())
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

    fn set_all(&mut self, todos: Vec<Todo>) {
        self.todos = todos;
    }

    fn remove(&mut self, id: TodoId) -> bool {
        let before = self.todos.len();
        self.todos.retain(|t| t.id != id);
        self.todos.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::todo::Title;
    use tempfile::tempdir;

    #[test]
    fn fs_repo_roundtrip_persists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("db.json");

        let mut repo = JsonFileTodoRepository::load_or_init(path.clone()).unwrap();
        repo.add(Todo::new(Title::parse("A").unwrap()));
        repo.save_atomic().unwrap();

        let repo2 = JsonFileTodoRepository::load_or_init(path).unwrap();
        let items = repo2.list();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title.as_str(), "A");
    }
}
