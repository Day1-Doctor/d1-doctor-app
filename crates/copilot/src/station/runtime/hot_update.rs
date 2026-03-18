use serde::{Deserialize, Serialize};

/// Manifest describing an available runtime configuration update.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateManifest {
    /// Semantic version of the new config (e.g. "1.2.0").
    pub version: String,
    /// URL to download the updated configuration bundle.
    pub config_url: String,
    /// Human-readable changelog for this update.
    pub changelog: String,
}

/// Checks for and applies hot-updates to the local runtime configuration.
///
/// The updater polls `api.day1.doctor/v1/runtime/latest` for new config
/// manifests and writes updates to `~/.day1copilot/`.
///
/// **Stub implementation** — no real HTTP calls are made.
pub struct HotUpdater {
    /// The base URL for the update API.
    api_url: String,
    /// The currently installed config version.
    current_version: String,
}

impl HotUpdater {
    /// Create a new updater targeting the production API.
    pub fn new(current_version: &str) -> Self {
        Self {
            api_url: "https://api.day1.doctor/v1/runtime/latest".to_string(),
            current_version: current_version.to_string(),
        }
    }

    /// Create an updater with a custom API URL (for testing).
    pub fn with_api_url(current_version: &str, api_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            current_version: current_version.to_string(),
        }
    }

    /// Check for a newer runtime configuration.
    ///
    /// Stub — always returns `None` (no update available).
    pub fn check_update(&self) -> Option<UpdateManifest> {
        // In the real implementation this would:
        // 1. GET {api_url} with current_version as a query param
        // 2. Parse the response into an UpdateManifest
        // 3. Return Some(manifest) if manifest.version > current_version
        let _ = &self.api_url;
        let _ = &self.current_version;
        None
    }

    /// Apply an update by downloading and writing the new config.
    ///
    /// Stub — validates the manifest and returns Ok without doing real I/O.
    pub fn apply_update(&self, manifest: &UpdateManifest) -> Result<(), String> {
        // In the real implementation this would:
        // 1. Download manifest.config_url
        // 2. Validate the downloaded config
        // 3. Write to ~/.day1copilot/ (atomic rename)
        // 4. Update self.current_version
        if manifest.version.is_empty() {
            return Err("invalid manifest: empty version".to_string());
        }
        if manifest.config_url.is_empty() {
            return Err("invalid manifest: empty config_url".to_string());
        }
        Ok(())
    }

    /// Get the base API URL.
    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    /// Get the current installed version.
    pub fn current_version(&self) -> &str {
        &self.current_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_update_returns_none() {
        let updater = HotUpdater::new("1.0.0");
        assert!(updater.check_update().is_none());
    }

    #[test]
    fn test_apply_update_validates_manifest() {
        let updater = HotUpdater::new("1.0.0");

        let good = UpdateManifest {
            version: "1.1.0".to_string(),
            config_url: "https://api.day1.doctor/v1/runtime/config/1.1.0".to_string(),
            changelog: "Bug fixes and performance improvements.".to_string(),
        };
        assert!(updater.apply_update(&good).is_ok());

        let bad_version = UpdateManifest {
            version: "".to_string(),
            config_url: "https://api.day1.doctor/v1/runtime/config/bad".to_string(),
            changelog: "N/A".to_string(),
        };
        assert!(updater.apply_update(&bad_version).is_err());

        let bad_url = UpdateManifest {
            version: "1.1.0".to_string(),
            config_url: "".to_string(),
            changelog: "N/A".to_string(),
        };
        assert!(updater.apply_update(&bad_url).is_err());
    }

    #[test]
    fn test_custom_api_url() {
        let updater = HotUpdater::with_api_url("1.0.0", "http://localhost:8080/runtime/latest");
        assert_eq!(updater.api_url(), "http://localhost:8080/runtime/latest");
        assert_eq!(updater.current_version(), "1.0.0");
    }
}
