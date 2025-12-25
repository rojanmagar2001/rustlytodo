//! Versioned on-disk schema definitions.
//!
//! Keep these in infra so domain remains stable and pure.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::todo::Todo;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub mod v1 {

    use super::*;

    /// Schema version 1.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct DbFileV1 {
        pub schema_version: u32,
        pub todos: Vec<Todo>,
    }

    impl DbFileV1 {
        pub fn empty() -> Self {
            Self {
                schema_version: super::CURRENT_SCHEMA_VERSION,
                todos: Vec::new(),
            }
        }
    }
}

/// Load any supported schema version and convert into current in-memory representation.
///
/// Today, v1 == current, so conversion is trivial.
/// Tomorrow, v2/v3 can map old fields into new domain types safely.
pub fn load_any(json_text: &str) -> Result<Vec<Todo>> {
    let v: Value = serde_json::from_str(json_text).context("failed parsing db JSON")?;

    let schema_version = v
        .get("schema_version")
        .and_then(|x| x.as_u64())
        .map(|x| x as u32)
        .unwrap_or(0);

    match schema_version {
        1 => {
            let db: v1::DbFileV1 =
                serde_json::from_value(v).context("failed decoding schema v1 db")?;
            Ok(db.todos)
        }
        other => anyhow::bail!("unsupported schema_version {} (supported: 1)", other),
    }
}

/// Serialize current in-memory state to the current on-disk format.
pub fn write_current(todos: &[Todo]) -> Result<String> {
    let db = v1::DbFileV1 {
        schema_version: CURRENT_SCHEMA_VERSION,
        todos: todos.to_vec(),
    };
    let s = serde_json::to_string_pretty(&db).context("failed serializing db JSON")?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::todo::Title;

    #[test]
    fn load_any_reads_v1() {
        let todo = crate::domain::todo::Todo::new(Title::parse("A").unwrap());
        let json = write_current(&[todo]).unwrap();
        let todos = load_any(&json).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title.as_str(), "A");
    }
}
