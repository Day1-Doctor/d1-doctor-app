//! JWT token persistence for the CLI.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStore {
    pub access_token: String,
    pub user_id: String,
}

impl TokenStore {
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn try_load(path: &Path) -> Option<Self> {
        Self::load(path).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_token() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("token.json");

        let token = TokenStore {
            access_token: "jwt-abc".to_string(),
            user_id: "user-123".to_string(),
        };
        token.save(&path).unwrap();

        let loaded = TokenStore::load(&path).unwrap();
        assert_eq!(loaded.access_token, "jwt-abc");
        assert_eq!(loaded.user_id, "user-123");
    }

    #[test]
    fn test_load_missing_returns_none() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        let result = TokenStore::try_load(&path);
        assert!(result.is_none());
    }
}
