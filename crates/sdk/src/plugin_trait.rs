//! Plugin trait definition — implement this to create a Day 1 Doctor plugin.

use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin name (must be unique)
    fn name(&self) -> &str;
    
    /// Plugin version (SemVer)
    fn version(&self) -> &str;
    
    /// Plugin description
    fn description(&self) -> &str;
    
    /// Execute the plugin's main logic
    async fn execute(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}
