use std::sync::{Arc, Mutex};

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::station::db::DbHandle;

/// A single entry in the memory store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub category: String,
    pub content: String,
    pub created_at: String,
}

/// A user profile assembled from profile_memory entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: Option<String>,
    pub preferences: Vec<MemoryEntry>,
}

/// SQL to create v2.x memory tables if they do not already exist.
///
/// These tables are simple key-value stores with a category column for
/// namespacing different types of memory (profile, session, task, agent).
const CREATE_MEMORY_TABLES: &str = r#"
CREATE TABLE IF NOT EXISTS profile_memory (
    id          TEXT PRIMARY KEY,
    category    TEXT NOT NULL,
    content     TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS session_memory (
    id          TEXT PRIMARY KEY,
    category    TEXT NOT NULL,
    content     TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_memory (
    id          TEXT PRIMARY KEY,
    category    TEXT NOT NULL,
    content     TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_memory (
    id          TEXT PRIMARY KEY,
    category    TEXT NOT NULL,
    content     TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_profile_memory_category ON profile_memory(category);
CREATE INDEX IF NOT EXISTS idx_session_memory_category ON session_memory(category);
CREATE INDEX IF NOT EXISTS idx_task_memory_category ON task_memory(category);
CREATE INDEX IF NOT EXISTS idx_agent_memory_category ON agent_memory(category);
"#;

/// Persistent memory store backed by SQLite.
///
/// Uses the session_memory table for general store/recall, and
/// profile_memory for user profile data. Tables are created on first
/// use if they do not already exist from v2.x migrations.
pub struct MemoryStore {
    db: DbHandle,
}

impl MemoryStore {
    /// Open a memory store backed by the given database handle.
    ///
    /// Creates the v2.x memory tables if they are missing.
    pub fn new(db: DbHandle) -> Result<Self, String> {
        {
            let conn = db
                .lock()
                .map_err(|e| format!("failed to lock db: {e}"))?;
            conn.execute_batch(CREATE_MEMORY_TABLES)
                .map_err(|e| format!("failed to create memory tables: {e}"))?;
        }
        Ok(Self { db })
    }

    /// Store a memory entry. Returns the generated ID.
    pub fn store(&self, category: &str, content: &str) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let conn = self
            .db
            .lock()
            .map_err(|e| format!("failed to lock db: {e}"))?;
        conn.execute(
            "INSERT INTO session_memory (id, category, content, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, category, content, now],
        )
        .map_err(|e| format!("failed to store memory: {e}"))?;
        Ok(id)
    }

    /// Recall the most recent `limit` memory entries (across all categories),
    /// optionally matching a substring query in content.
    pub fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, String> {
        let conn = self
            .db
            .lock()
            .map_err(|e| format!("failed to lock db: {e}"))?;
        if query.is_empty() {
            let mut stmt = conn
                .prepare(
                    "SELECT id, category, content, created_at FROM session_memory \
                     ORDER BY created_at DESC LIMIT ?1",
                )
                .map_err(|e| format!("failed to prepare recall query: {e}"))?;
            let rows = stmt
                .query_map(params![limit as i64], |row| {
                    Ok(MemoryEntry {
                        id: row.get(0)?,
                        category: row.get(1)?,
                        content: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                })
                .map_err(|e| format!("failed to execute recall query: {e}"))?;
            let mut entries: Vec<MemoryEntry> = rows
                .filter_map(|r| r.ok())
                .collect();
            entries.reverse(); // chronological order
            Ok(entries)
        } else {
            let pattern = format!("%{}%", query);
            let mut stmt = conn
                .prepare(
                    "SELECT id, category, content, created_at FROM session_memory \
                     WHERE content LIKE ?1 ORDER BY created_at DESC LIMIT ?2",
                )
                .map_err(|e| format!("failed to prepare recall query: {e}"))?;
            let rows = stmt
                .query_map(params![pattern, limit as i64], |row| {
                    Ok(MemoryEntry {
                        id: row.get(0)?,
                        category: row.get(1)?,
                        content: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                })
                .map_err(|e| format!("failed to execute recall query: {e}"))?;
            let mut entries: Vec<MemoryEntry> = rows
                .filter_map(|r| r.ok())
                .collect();
            entries.reverse();
            Ok(entries)
        }
    }

    /// Search for memory entries by category, optionally filtering by a
    /// content substring query.
    pub fn search(
        &self,
        category: &str,
        query: &str,
    ) -> Result<Vec<MemoryEntry>, String> {
        let conn = self
            .db
            .lock()
            .map_err(|e| format!("failed to lock db: {e}"))?;
        if query.is_empty() {
            let mut stmt = conn
                .prepare(
                    "SELECT id, category, content, created_at FROM session_memory \
                     WHERE category = ?1 ORDER BY created_at DESC",
                )
                .map_err(|e| format!("failed to prepare search query: {e}"))?;
            let rows = stmt
                .query_map(params![category], |row| {
                    Ok(MemoryEntry {
                        id: row.get(0)?,
                        category: row.get(1)?,
                        content: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                })
                .map_err(|e| format!("failed to execute search query: {e}"))?;
            let mut entries: Vec<MemoryEntry> = rows
                .filter_map(|r| r.ok())
                .collect();
            entries.reverse();
            Ok(entries)
        } else {
            let pattern = format!("%{}%", query);
            let mut stmt = conn
                .prepare(
                    "SELECT id, category, content, created_at FROM session_memory \
                     WHERE category = ?1 AND content LIKE ?2 ORDER BY created_at DESC",
                )
                .map_err(|e| format!("failed to prepare search query: {e}"))?;
            let rows = stmt
                .query_map(params![category, pattern], |row| {
                    Ok(MemoryEntry {
                        id: row.get(0)?,
                        category: row.get(1)?,
                        content: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                })
                .map_err(|e| format!("failed to execute search query: {e}"))?;
            let mut entries: Vec<MemoryEntry> = rows
                .filter_map(|r| r.ok())
                .collect();
            entries.reverse();
            Ok(entries)
        }
    }

    /// Retrieve the user profile from profile_memory entries.
    ///
    /// Returns a `UserProfile` with the name extracted from a "name"
    /// category entry, and all profile entries as preferences.
    pub fn profile(&self) -> Result<UserProfile, String> {
        let conn = self
            .db
            .lock()
            .map_err(|e| format!("failed to lock db: {e}"))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, category, content, created_at FROM profile_memory \
                 ORDER BY created_at DESC",
            )
            .map_err(|e| format!("failed to prepare profile query: {e}"))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    content: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| format!("failed to execute profile query: {e}"))?;
        let entries: Vec<MemoryEntry> = rows.filter_map(|r| r.ok()).collect();

        let name = entries
            .iter()
            .find(|e| e.category == "name")
            .map(|e| e.content.clone());

        Ok(UserProfile {
            name,
            preferences: entries,
        })
    }

    /// Store a profile memory entry. Returns the generated ID.
    pub fn store_profile(&self, category: &str, content: &str) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let conn = self
            .db
            .lock()
            .map_err(|e| format!("failed to lock db: {e}"))?;
        conn.execute(
            "INSERT INTO profile_memory (id, category, content, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, category, content, now],
        )
        .map_err(|e| format!("failed to store profile memory: {e}"))?;
        Ok(id)
    }
}

/// Create an in-memory database handle with memory tables for testing.
#[cfg(test)]
fn test_db() -> DbHandle {
    let conn = Connection::open_in_memory().expect("failed to open in-memory db");
    conn.execute_batch(CREATE_MEMORY_TABLES)
        .expect("failed to create memory tables");
    Arc::new(Mutex::new(conn))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_recall_roundtrip() {
        let db = test_db();
        let store = MemoryStore::new(db).unwrap();

        let id = store.store("fact", "The user prefers dark mode").unwrap();
        assert!(!id.is_empty());

        let entries = store.recall("", 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, id);
        assert_eq!(entries[0].category, "fact");
        assert_eq!(entries[0].content, "The user prefers dark mode");
    }

    #[test]
    fn test_recall_with_query() {
        let db = test_db();
        let store = MemoryStore::new(db).unwrap();

        store.store("fact", "User prefers dark mode").unwrap();
        store.store("fact", "User likes Rust").unwrap();
        store.store("note", "Meeting at 3pm").unwrap();

        let entries = store.recall("Rust", 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "User likes Rust");
    }

    #[test]
    fn test_recall_with_limit() {
        let db = test_db();
        let store = MemoryStore::new(db).unwrap();

        for i in 0..5 {
            store
                .store("fact", &format!("Memory entry {}", i))
                .unwrap();
        }

        let entries = store.recall("", 3).unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_search_by_category() {
        let db = test_db();
        let store = MemoryStore::new(db).unwrap();

        store.store("fact", "User prefers dark mode").unwrap();
        store.store("note", "Meeting at 3pm").unwrap();
        store.store("fact", "User likes Rust").unwrap();
        store.store("note", "Deadline Friday").unwrap();

        let facts = store.search("fact", "").unwrap();
        assert_eq!(facts.len(), 2);
        for entry in &facts {
            assert_eq!(entry.category, "fact");
        }

        let notes = store.search("note", "").unwrap();
        assert_eq!(notes.len(), 2);
        for entry in &notes {
            assert_eq!(entry.category, "note");
        }
    }

    #[test]
    fn test_search_by_category_and_query() {
        let db = test_db();
        let store = MemoryStore::new(db).unwrap();

        store.store("fact", "User prefers dark mode").unwrap();
        store.store("fact", "User likes Rust").unwrap();
        store.store("note", "User meeting at 3pm").unwrap();

        let results = store.search("fact", "dark").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "User prefers dark mode");
    }

    #[test]
    fn test_profile_returns_defaults() {
        let db = test_db();
        let store = MemoryStore::new(db).unwrap();

        // No profile entries stored — should return defaults.
        let profile = store.profile().unwrap();
        assert!(profile.name.is_none());
        assert!(profile.preferences.is_empty());
    }

    #[test]
    fn test_profile_with_data() {
        let db = test_db();
        let store = MemoryStore::new(db).unwrap();

        store.store_profile("name", "Alice").unwrap();
        store.store_profile("preference", "dark mode").unwrap();
        store.store_profile("preference", "verbose output").unwrap();

        let profile = store.profile().unwrap();
        assert_eq!(profile.name, Some("Alice".to_string()));
        assert_eq!(profile.preferences.len(), 3);
    }

    #[test]
    fn test_empty_recall() {
        let db = test_db();
        let store = MemoryStore::new(db).unwrap();

        let entries = store.recall("", 10).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_search_empty_category() {
        let db = test_db();
        let store = MemoryStore::new(db).unwrap();

        store.store("fact", "something").unwrap();

        let results = store.search("nonexistent", "").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_tables_created_idempotently() {
        let db = test_db();
        // Creating MemoryStore twice should not error (tables already exist).
        let _store1 = MemoryStore::new(db.clone()).unwrap();
        let _store2 = MemoryStore::new(db).unwrap();
    }
}
