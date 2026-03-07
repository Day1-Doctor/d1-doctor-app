//! Session history storage using a local JSON file.
//!
//! Each session is stored as a JSON file in ~/.d1doctor/sessions/.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// A single entry in the chat history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub role: Role,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Agent,
}

/// Session metadata and message history.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub session_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub messages: Vec<HistoryEntry>,
}

/// Manages session history persistence.
pub struct SessionHistory {
    session_id: String,
    data: SessionData,
    path: PathBuf,
}

impl SessionHistory {
    /// Create a new session history, writing to ~/.d1doctor/sessions/<id>.json.
    pub fn new(session_id: &str) -> Result<Self> {
        let dir = sessions_dir();
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create sessions dir: {}", dir.display()))?;

        let path = dir.join(format!("{}.json", session_id));
        let now = Utc::now().to_rfc3339();

        let data = SessionData {
            session_id: session_id.to_string(),
            started_at: now,
            ended_at: None,
            messages: Vec::new(),
        };

        let history = Self {
            session_id: session_id.to_string(),
            data,
            path,
        };
        history.save()?;

        Ok(history)
    }

    /// Create a session history with a custom path (for testing).
    pub fn with_path(session_id: &str, path: PathBuf) -> Result<Self> {
        let now = Utc::now().to_rfc3339();

        let data = SessionData {
            session_id: session_id.to_string(),
            started_at: now,
            ended_at: None,
            messages: Vec::new(),
        };

        let history = Self {
            session_id: session_id.to_string(),
            data,
            path,
        };
        history.save()?;

        Ok(history)
    }

    /// Add a user message to the history.
    pub fn add_user_message(&mut self, content: &str) -> Result<()> {
        self.data.messages.push(HistoryEntry {
            role: Role::User,
            content: content.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        });
        self.save()
    }

    /// Add an agent response to the history.
    pub fn add_agent_response(&mut self, content: &str) -> Result<()> {
        self.data.messages.push(HistoryEntry {
            role: Role::Agent,
            content: content.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        });
        self.save()
    }

    /// Mark the session as ended and flush.
    pub fn finalize(&mut self) -> Result<()> {
        self.data.ended_at = Some(Utc::now().to_rfc3339());
        self.save()
    }

    /// Get the number of messages in this session.
    pub fn message_count(&self) -> usize {
        self.data.messages.len()
    }

    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.path, json)
            .with_context(|| format!("Failed to write session file: {}", self.path.display()))?;
        Ok(())
    }
}

fn sessions_dir() -> PathBuf {
    d1_common::config_dir().join("sessions")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_history_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test-session.json");

        let mut history = SessionHistory::with_path("test-123", path.clone()).unwrap();
        assert_eq!(history.message_count(), 0);
        assert_eq!(history.session_id(), "test-123");

        history.add_user_message("Hello doctor").unwrap();
        history
            .add_agent_response("Hello! How can I help?")
            .unwrap();
        assert_eq!(history.message_count(), 2);

        history.finalize().unwrap();

        // Verify the file was written correctly
        let contents = fs::read_to_string(&path).unwrap();
        let data: SessionData = serde_json::from_str(&contents).unwrap();
        assert_eq!(data.session_id, "test-123");
        assert_eq!(data.messages.len(), 2);
        assert_eq!(data.messages[0].role, Role::User);
        assert_eq!(data.messages[0].content, "Hello doctor");
        assert_eq!(data.messages[1].role, Role::Agent);
        assert!(data.ended_at.is_some());
    }

    #[test]
    fn test_empty_session() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty-session.json");

        let mut history = SessionHistory::with_path("empty", path.clone()).unwrap();
        history.finalize().unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        let data: SessionData = serde_json::from_str(&contents).unwrap();
        assert!(data.messages.is_empty());
        assert!(data.ended_at.is_some());
    }

    #[test]
    fn test_history_entry_serialization() {
        let entry = HistoryEntry {
            role: Role::User,
            content: "test message".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: HistoryEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.role, Role::User);
        assert_eq!(deserialized.content, "test message");
    }
}
