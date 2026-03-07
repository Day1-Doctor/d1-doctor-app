//! CLI version check and update nudge.
//!
//! On `d1 run`, performs an async, non-blocking check against the local daemon
//! (`GET /api/health`) to compare version numbers. If a newer version is
//! available, prints a one-line nudge to stderr. A stronger warning is shown
//! when the current version falls below `min_supported`.
//!
//! Checks are throttled to at most once per 24 hours via a persisted timestamp
//! file at `~/.d1doctor/version_check`.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// How often we bother checking (24 hours).
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Current CLI version (from Cargo.toml at compile time).
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------------------------------------------------------------------------
// Persisted state
// ---------------------------------------------------------------------------

/// On-disk record of the last version check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionCheckState {
    /// Unix timestamp (seconds) of the last successful check.
    pub last_check_epoch: u64,
    /// The latest version string returned by the daemon.
    pub latest_version: Option<String>,
    /// The minimum supported version string returned by the daemon (if any).
    pub min_supported: Option<String>,
}

impl Default for VersionCheckState {
    fn default() -> Self {
        Self {
            last_check_epoch: 0,
            latest_version: None,
            min_supported: None,
        }
    }
}

/// Path to the persisted state file.
pub fn state_path() -> PathBuf {
    d1_common::config_dir().join("version_check")
}

/// Load persisted state (returns default if missing or corrupt).
pub fn load_state() -> VersionCheckState {
    load_state_from(&state_path())
}

/// Load from a specific path (test-friendly).
pub fn load_state_from(path: &std::path::Path) -> VersionCheckState {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Persist state to disk.
pub fn save_state(state: &VersionCheckState) -> Result<()> {
    save_state_to(state, &state_path())
}

/// Persist state to a specific path (test-friendly).
pub fn save_state_to(state: &VersionCheckState, path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string(state)?;
    fs::write(path, json)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Throttle logic
// ---------------------------------------------------------------------------

/// Return the current epoch in seconds.
fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Returns `true` when enough time has passed since the last check.
pub fn should_check(state: &VersionCheckState) -> bool {
    should_check_with_now(state, now_epoch())
}

/// Testable variant that accepts an explicit "now" timestamp.
pub fn should_check_with_now(state: &VersionCheckState, now: u64) -> bool {
    now.saturating_sub(state.last_check_epoch) >= CHECK_INTERVAL.as_secs()
}

// ---------------------------------------------------------------------------
// Version comparison
// ---------------------------------------------------------------------------

/// Parse a semver-ish version string into (major, minor, patch).
/// Returns `None` if parsing fails.
pub fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let v = v.strip_prefix('v').unwrap_or(v);
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

/// Returns `true` when `current` is strictly older than `latest`.
pub fn is_update_available(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some(c), Some(l)) => c < l,
        _ => false,
    }
}

/// Returns `true` when `current` is strictly below `min_supported`.
pub fn is_unsupported(current: &str, min_supported: &str) -> bool {
    match (parse_version(current), parse_version(min_supported)) {
        (Some(c), Some(m)) => c < m,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Nudge messages
// ---------------------------------------------------------------------------

/// Build the one-line update nudge.
pub fn update_nudge(current: &str, latest: &str) -> String {
    format!(
        "Update available: v{latest} (current: v{current}). Run: brew upgrade d1"
    )
}

/// Build the stronger unsupported-version warning.
pub fn unsupported_warning() -> String {
    "WARNING: Your version is no longer supported. Please update.".to_string()
}

// ---------------------------------------------------------------------------
// Daemon response
// ---------------------------------------------------------------------------

/// Subset of the daemon `/api/health` JSON response we care about.
#[derive(Debug, Deserialize)]
pub struct DaemonHealthResponse {
    pub version: String,
    #[serde(default)]
    pub min_supported: Option<String>,
}

/// Fetch the daemon's health endpoint. Returns `None` on any failure (timeout,
/// connection refused, bad JSON, etc.) — this must never block the CLI.
pub async fn fetch_daemon_version(daemon_port: u16) -> Option<DaemonHealthResponse> {
    let url = format!("http://127.0.0.1:{}/api/health", daemon_port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .ok()?;
    let resp = client.get(&url).send().await.ok()?;
    resp.json::<DaemonHealthResponse>().await.ok()
}

// ---------------------------------------------------------------------------
// Orchestration — called from the `d1 run` startup path
// ---------------------------------------------------------------------------

/// Perform the version check and print a nudge if appropriate.
///
/// This function is intentionally fire-and-forget: it swallows all errors so
/// that a failed check never prevents the user from starting a session.
pub async fn maybe_nudge(daemon_port: u16) {
    maybe_nudge_inner(daemon_port, &state_path()).await;
}

/// Inner implementation that accepts a state path for testability.
async fn maybe_nudge_inner(daemon_port: u16, path: &std::path::Path) {
    let state = load_state_from(path);

    if !should_check(&state) {
        // Still within the throttle window — but we may have a cached result
        // from the last successful check that we already displayed.
        return;
    }

    let health = match fetch_daemon_version(daemon_port).await {
        Some(h) => h,
        None => return, // daemon unreachable — silently skip
    };

    // Persist the new check timestamp + response.
    let new_state = VersionCheckState {
        last_check_epoch: now_epoch(),
        latest_version: Some(health.version.clone()),
        min_supported: health.min_supported.clone(),
    };
    let _ = save_state_to(&new_state, path);

    let current = CURRENT_VERSION;

    // Stronger warning takes priority.
    if let Some(ref min) = health.min_supported {
        if is_unsupported(current, min) {
            eprintln!("{}", unsupported_warning());
            return;
        }
    }

    if is_update_available(current, &health.version) {
        eprintln!("{}", update_nudge(current, &health.version));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    // -- parse_version -------------------------------------------------------

    #[test]
    fn test_parse_version_basic() {
        assert_eq!(parse_version("0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("1.23.456"), Some((1, 23, 456)));
    }

    #[test]
    fn test_parse_version_with_v_prefix() {
        assert_eq!(parse_version("v2.3.4"), Some((2, 3, 4)));
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(parse_version("abc"), None);
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version("1.2.3.4"), None);
        assert_eq!(parse_version(""), None);
    }

    // -- is_update_available --------------------------------------------------

    #[test]
    fn test_update_available_newer() {
        assert!(is_update_available("0.1.0", "0.2.0"));
        assert!(is_update_available("0.1.0", "0.1.1"));
        assert!(is_update_available("0.1.0", "1.0.0"));
    }

    #[test]
    fn test_update_available_same() {
        assert!(!is_update_available("0.1.0", "0.1.0"));
    }

    #[test]
    fn test_update_available_older() {
        assert!(!is_update_available("0.2.0", "0.1.0"));
    }

    #[test]
    fn test_update_available_invalid() {
        assert!(!is_update_available("bad", "0.1.0"));
        assert!(!is_update_available("0.1.0", "bad"));
    }

    // -- is_unsupported -------------------------------------------------------

    #[test]
    fn test_unsupported_below_min() {
        assert!(is_unsupported("0.1.0", "0.2.0"));
    }

    #[test]
    fn test_unsupported_at_min() {
        assert!(!is_unsupported("0.2.0", "0.2.0"));
    }

    #[test]
    fn test_unsupported_above_min() {
        assert!(!is_unsupported("0.3.0", "0.2.0"));
    }

    // -- throttle logic -------------------------------------------------------

    #[test]
    fn test_should_check_first_time() {
        let state = VersionCheckState::default();
        // epoch 0 was a long time ago — should check
        assert!(should_check_with_now(&state, 1_000_000));
    }

    #[test]
    fn test_should_check_within_window() {
        let state = VersionCheckState {
            last_check_epoch: 1_000_000,
            ..Default::default()
        };
        // 1 hour later — should NOT check
        assert!(!should_check_with_now(&state, 1_000_000 + 3600));
    }

    #[test]
    fn test_should_check_after_window() {
        let state = VersionCheckState {
            last_check_epoch: 1_000_000,
            ..Default::default()
        };
        // 25 hours later — should check
        assert!(should_check_with_now(&state, 1_000_000 + 25 * 3600));
    }

    #[test]
    fn test_should_check_exactly_at_boundary() {
        let state = VersionCheckState {
            last_check_epoch: 1_000_000,
            ..Default::default()
        };
        // Exactly 24 hours — should check (>= comparison)
        assert!(should_check_with_now(&state, 1_000_000 + 24 * 3600));
    }

    // -- nudge messages -------------------------------------------------------

    #[test]
    fn test_update_nudge_message() {
        let msg = update_nudge("0.1.0", "0.2.0");
        assert!(msg.contains("v0.2.0"));
        assert!(msg.contains("v0.1.0"));
        assert!(msg.contains("brew upgrade d1"));
    }

    #[test]
    fn test_unsupported_warning_message() {
        let msg = unsupported_warning();
        assert!(msg.contains("WARNING"));
        assert!(msg.contains("no longer supported"));
    }

    // -- state persistence ----------------------------------------------------

    #[test]
    fn test_state_roundtrip() {
        let dir = std::env::temp_dir().join(format!("d1_version_check_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("version_check");

        let state = VersionCheckState {
            last_check_epoch: 12345,
            latest_version: Some("0.2.0".to_string()),
            min_supported: Some("0.1.0".to_string()),
        };

        save_state_to(&state, &path).unwrap();
        let loaded = load_state_from(&path);

        assert_eq!(loaded.last_check_epoch, 12345);
        assert_eq!(loaded.latest_version.as_deref(), Some("0.2.0"));
        assert_eq!(loaded.min_supported.as_deref(), Some("0.1.0"));

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_state_missing_file() {
        let path = std::path::Path::new("/tmp/d1_nonexistent_version_check_file");
        let state = load_state_from(path);
        assert_eq!(state.last_check_epoch, 0);
        assert!(state.latest_version.is_none());
    }

    #[test]
    fn test_load_state_corrupt_file() {
        let dir = std::env::temp_dir().join(format!("d1_version_check_corrupt_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("version_check");

        let mut f = fs::File::create(&path).unwrap();
        f.write_all(b"not json at all").unwrap();

        let state = load_state_from(&path);
        assert_eq!(state.last_check_epoch, 0);

        let _ = fs::remove_dir_all(&dir);
    }
}
