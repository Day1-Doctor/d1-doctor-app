//! Security layer for command classification, path sandboxing, and permission flow
//!
//! This module enforces security policies on commands before they are executed
//! by the daemon. It classifies commands by risk level, validates that file paths
//! stay within the sandbox, and determines whether approval is required.

use std::path::{Path, PathBuf};

use d1_common::D1Error;
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Risk classification
// ---------------------------------------------------------------------------

/// Risk level assigned to a command by the security layer.
///
/// Note: This is distinct from `d1_common::proto::RiskLevel` which describes
/// plan-level risk. This enum describes *individual command* risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// Read-only, side-effect-free commands
    Low,
    /// Write / install commands that mutate the filesystem
    Medium,
    /// Destructive or privileged commands
    High,
    /// Catastrophic patterns that must never run
    Blocked,
}

/// Result of classifying a command string.
#[derive(Debug, Clone)]
pub struct RiskClassification {
    pub risk_level: RiskLevel,
    pub reason: String,
    pub requires_approval: bool,
}

// ---------------------------------------------------------------------------
// Permission decisions
// ---------------------------------------------------------------------------

/// Decision returned by [`SecurityLayer::check_permission`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    /// Command may proceed without any special handling.
    Allow,
    /// Command may proceed but should be logged for auditing.
    AllowWithLogging,
    /// Command requires explicit user / admin approval before execution.
    RequireApproval { reason: String },
    /// Command is denied outright.
    Deny { reason: String },
}

// ---------------------------------------------------------------------------
// Blocked patterns (catastrophic)
// ---------------------------------------------------------------------------

/// Patterns that are *always* blocked regardless of context.
const BLOCKED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "dd if=/dev",
    "mkfs",
    "format c:",
    ":(){ :|:& };:",
];

// ---------------------------------------------------------------------------
// Command lists by risk tier
// ---------------------------------------------------------------------------

/// Commands considered LOW risk (read-only / informational).
const LOW_COMMANDS: &[&str] = &[
    "ls", "cat", "head", "tail", "echo", "env", "whoami", "pwd", "which", "file", "wc", "du", "df",
    "uname", "date", "hostname",
];

/// Commands considered MEDIUM risk (write / install / build).
const MEDIUM_COMMANDS: &[&str] = &[
    "cp",
    "mv",
    "mkdir",
    "touch",
    "chmod",
    "chown",
    "brew install",
    "apt install",
    "pip install",
    "npm install",
    "cargo build",
];

/// Commands considered HIGH risk (destructive / privileged).
const HIGH_COMMANDS: &[&str] = &[
    "rm",
    "sudo",
    "kill",
    "pkill",
    "systemctl",
    "launchctl",
    "chroot",
    "mount",
    "umount",
    "iptables",
    "route",
];

// ---------------------------------------------------------------------------
// SecurityLayer
// ---------------------------------------------------------------------------

/// Core security enforcement structure.
///
/// Holds configurable sandbox settings and exposes methods for classifying
/// commands, validating paths, and determining permission decisions.
pub struct SecurityLayer {
    /// Root directory of the sandbox. Commands may only touch paths within
    /// this directory (or within `allowed_system_paths`).
    pub sandbox_root: PathBuf,

    /// Additional system paths that are permitted even though they live
    /// outside `sandbox_root` (e.g. `/usr/local/bin`).
    pub allowed_system_paths: Vec<PathBuf>,

    /// Hardcoded list of dangerous command patterns that are always blocked.
    pub blocked_commands: Vec<String>,
}

impl SecurityLayer {
    /// Create a new `SecurityLayer` with the user's home directory as the
    /// sandbox root and sensible defaults.
    pub fn new() -> Self {
        let sandbox_root = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        let blocked_commands: Vec<String> =
            BLOCKED_PATTERNS.iter().map(|s| s.to_string()).collect();

        Self {
            sandbox_root,
            allowed_system_paths: Vec::new(),
            blocked_commands,
        }
    }

    /// Create a `SecurityLayer` with a custom sandbox root (useful for tests).
    pub fn with_sandbox_root(sandbox_root: PathBuf) -> Self {
        let blocked_commands: Vec<String> =
            BLOCKED_PATTERNS.iter().map(|s| s.to_string()).collect();

        Self {
            sandbox_root,
            allowed_system_paths: Vec::new(),
            blocked_commands,
        }
    }

    // ----- Classification ------------------------------------------------

    /// Classify a command string and return a [`RiskClassification`].
    pub fn classify_command(&self, command: &str) -> RiskClassification {
        let trimmed = command.trim();
        let lower = trimmed.to_lowercase();

        // 1. Check blocked patterns first (highest priority)
        for pattern in &self.blocked_commands {
            if lower.contains(&pattern.to_lowercase()) {
                return RiskClassification {
                    risk_level: RiskLevel::Blocked,
                    reason: format!("Matches blocked pattern: {}", pattern),
                    requires_approval: false, // denied outright
                };
            }
        }

        // 2. Check HIGH risk commands
        for cmd in HIGH_COMMANDS {
            if Self::command_matches(&lower, cmd) {
                return RiskClassification {
                    risk_level: RiskLevel::High,
                    reason: format!("High-risk command: {}", cmd),
                    requires_approval: true,
                };
            }
        }

        // 3. Check MEDIUM risk commands
        for cmd in MEDIUM_COMMANDS {
            if Self::command_matches(&lower, cmd) {
                return RiskClassification {
                    risk_level: RiskLevel::Medium,
                    reason: format!("Medium-risk command: {}", cmd),
                    requires_approval: false,
                };
            }
        }

        // 4. Check LOW risk commands
        for cmd in LOW_COMMANDS {
            if Self::command_matches(&lower, cmd) {
                return RiskClassification {
                    risk_level: RiskLevel::Low,
                    reason: format!("Low-risk read-only command: {}", cmd),
                    requires_approval: false,
                };
            }
        }

        // 5. Unknown commands default to MEDIUM
        RiskClassification {
            risk_level: RiskLevel::Medium,
            reason: "Unknown command — defaulting to medium risk".to_string(),
            requires_approval: false,
        }
    }

    // ----- Path validation -----------------------------------------------

    /// Validate that `path` is within the sandbox or an allowed system path.
    ///
    /// Returns the canonicalized path on success, or a `PermissionDenied`
    /// error if the path escapes the sandbox.
    pub fn validate_path(&self, path: &str) -> d1_common::Result<PathBuf> {
        let raw = Path::new(path);

        // Resolve to an absolute path (relative to sandbox_root if needed)
        let absolute = if raw.is_absolute() {
            raw.to_path_buf()
        } else {
            self.sandbox_root.join(raw)
        };

        // Canonicalize to resolve symlinks and `..` components
        let canonical = absolute.canonicalize().map_err(|e| {
            D1Error::permission_denied(format!("Cannot resolve path '{}': {}", path, e))
        })?;

        // Canonicalize sandbox_root too (handles macOS /var -> /private/var etc.)
        let canonical_sandbox = self
            .sandbox_root
            .canonicalize()
            .unwrap_or_else(|_| self.sandbox_root.clone());

        // Check if within sandbox_root
        if canonical.starts_with(&canonical_sandbox) {
            debug!(?canonical, "Path validated within sandbox");
            return Ok(canonical);
        }

        // Check allowed system paths (also canonicalized)
        for allowed in &self.allowed_system_paths {
            let canonical_allowed = allowed.canonicalize().unwrap_or_else(|_| allowed.clone());
            if canonical.starts_with(&canonical_allowed) {
                debug!(
                    ?canonical,
                    ?allowed,
                    "Path validated via allowed system path"
                );
                return Ok(canonical);
            }
        }

        warn!(
            ?canonical,
            sandbox_root = ?canonical_sandbox,
            "Path escapes sandbox"
        );
        Err(D1Error::permission_denied(format!(
            "Path '{}' (resolved to '{}') is outside the sandbox root '{}'",
            path,
            canonical.display(),
            canonical_sandbox.display()
        )))
    }

    // ----- Permission decision -------------------------------------------

    /// Determine the permission decision for a command string.
    pub fn check_permission(&self, command: &str) -> PermissionDecision {
        let classification = self.classify_command(command);

        match classification.risk_level {
            RiskLevel::Low => PermissionDecision::Allow,
            RiskLevel::Medium => PermissionDecision::AllowWithLogging,
            RiskLevel::High => PermissionDecision::RequireApproval {
                reason: classification.reason,
            },
            RiskLevel::Blocked => PermissionDecision::Deny {
                reason: classification.reason,
            },
        }
    }

    // ----- Sudo detection ------------------------------------------------

    /// Returns `true` if the command invokes `sudo`.
    pub fn is_sudo_command(command: &str) -> bool {
        let trimmed = command.trim().to_lowercase();
        // Matches "sudo" at the start or preceded by shell operators
        trimmed == "sudo"
            || trimmed.starts_with("sudo ")
            || trimmed.contains("| sudo ")
            || trimmed.contains("&& sudo ")
            || trimmed.contains("; sudo ")
    }

    // ----- Helpers -------------------------------------------------------

    /// Check whether a lowercased command string matches a known command
    /// pattern. Handles both single-word commands (e.g. "ls") and multi-word
    /// patterns (e.g. "brew install").
    fn command_matches(lower_command: &str, pattern: &str) -> bool {
        // For multi-word patterns, check if the command contains the pattern
        if pattern.contains(' ') {
            return lower_command.contains(pattern);
        }

        // For single-word commands, match the first token or match after a
        // pipe / semicolon / && so that "echo hello | cat" matches "cat".
        let first_token = lower_command.split_whitespace().next().unwrap_or("");

        if first_token == pattern {
            return true;
        }

        // Also check after shell operators
        for segment in lower_command.split(&['|', ';'][..]) {
            let seg_first = segment.trim().split_whitespace().next().unwrap_or("");
            if seg_first == pattern {
                return true;
            }
        }

        // Check after &&
        for segment in lower_command.split("&&") {
            let seg_first = segment.trim().split_whitespace().next().unwrap_or("");
            if seg_first == pattern {
                return true;
            }
        }

        false
    }
}

impl Default for SecurityLayer {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_layer() -> SecurityLayer {
        SecurityLayer::with_sandbox_root(std::env::temp_dir())
    }

    // ---- LOW classification ----

    #[test]
    fn test_classify_low_commands() {
        let layer = test_layer();
        let low_cmds = [
            "ls -la",
            "cat /etc/hosts",
            "head -n 10 file.txt",
            "tail -f log.txt",
            "echo hello",
            "env",
            "whoami",
            "pwd",
            "which rustc",
            "file foo.bin",
            "wc -l a.txt",
            "du -sh .",
            "df -h",
            "uname -a",
            "date",
            "hostname",
        ];
        for cmd in &low_cmds {
            let result = layer.classify_command(cmd);
            assert_eq!(
                result.risk_level,
                RiskLevel::Low,
                "Expected LOW for '{}', got {:?}: {}",
                cmd,
                result.risk_level,
                result.reason
            );
            assert!(!result.requires_approval);
        }
    }

    // ---- MEDIUM classification ----

    #[test]
    fn test_classify_medium_commands() {
        let layer = test_layer();
        let medium_cmds = [
            "cp a.txt b.txt",
            "mv old new",
            "mkdir -p dir",
            "touch newfile",
            "chmod 755 script.sh",
            "chown user:group file",
            "brew install ripgrep",
            "apt install curl",
            "pip install flask",
            "npm install express",
            "cargo build --release",
        ];
        for cmd in &medium_cmds {
            let result = layer.classify_command(cmd);
            assert_eq!(
                result.risk_level,
                RiskLevel::Medium,
                "Expected MEDIUM for '{}', got {:?}: {}",
                cmd,
                result.risk_level,
                result.reason
            );
        }
    }

    // ---- HIGH classification ----

    #[test]
    fn test_classify_high_commands() {
        let layer = test_layer();
        let high_cmds = [
            "rm file.txt",
            "sudo apt update",
            "kill -9 1234",
            "pkill nginx",
            "systemctl restart sshd",
            "launchctl load plist",
            "chroot /newroot",
            "mount /dev/sda1 /mnt",
            "umount /mnt",
            "iptables -A INPUT -j DROP",
            "route add default gw 10.0.0.1",
        ];
        for cmd in &high_cmds {
            let result = layer.classify_command(cmd);
            assert_eq!(
                result.risk_level,
                RiskLevel::High,
                "Expected HIGH for '{}', got {:?}: {}",
                cmd,
                result.risk_level,
                result.reason
            );
            assert!(result.requires_approval);
        }
    }

    // ---- BLOCKED classification ----

    #[test]
    fn test_classify_blocked_commands() {
        let layer = test_layer();
        let blocked_cmds = [
            "rm -rf /",
            "rm -rf /*",
            "dd if=/dev/zero of=/dev/sda",
            "mkfs.ext4 /dev/sda1",
            ":(){ :|:& };:",
        ];
        for cmd in &blocked_cmds {
            let result = layer.classify_command(cmd);
            assert_eq!(
                result.risk_level,
                RiskLevel::Blocked,
                "Expected BLOCKED for '{}', got {:?}: {}",
                cmd,
                result.risk_level,
                result.reason
            );
        }
    }

    // ---- Path validation: valid path within sandbox ----

    #[test]
    fn test_validate_path_within_sandbox() {
        let sandbox = std::env::temp_dir();
        let layer = SecurityLayer::with_sandbox_root(sandbox.clone());

        // Create a temp file inside the sandbox to canonicalize against
        let test_file = sandbox.join("d1_security_test_valid.txt");
        fs::write(&test_file, "test").expect("Failed to create test file");

        let result = layer.validate_path(test_file.to_str().unwrap());
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);

        // Clean up
        let _ = fs::remove_file(&test_file);
    }

    // ---- Path validation: path outside sandbox rejected ----

    #[test]
    fn test_validate_path_outside_sandbox() {
        // Use a narrow sandbox that definitely doesn't include /etc
        let sandbox = std::env::temp_dir().join("d1_sandbox_narrow");
        fs::create_dir_all(&sandbox).ok();
        let layer = SecurityLayer::with_sandbox_root(sandbox.clone());

        let result = layer.validate_path("/etc/hosts");
        assert!(result.is_err(), "Expected Err for path outside sandbox");
        let err = result.unwrap_err();
        assert!(err.is_permission_denied());

        // Clean up
        let _ = fs::remove_dir(&sandbox);
    }

    // ---- Path validation: traversal rejected ----

    #[test]
    fn test_validate_path_traversal_rejected() {
        let sandbox = std::env::temp_dir().join("d1_sandbox_traversal");
        fs::create_dir_all(&sandbox).ok();
        let layer = SecurityLayer::with_sandbox_root(sandbox.clone());

        // Attempt to escape via ../
        let result = layer.validate_path("../../../etc/hosts");
        assert!(result.is_err(), "Expected Err for path traversal");
        let err = result.unwrap_err();
        assert!(err.is_permission_denied());

        // Clean up
        let _ = fs::remove_dir(&sandbox);
    }

    // ---- Path validation: allowed system path ----

    #[test]
    fn test_validate_path_allowed_system_path() {
        let sandbox = std::env::temp_dir().join("d1_sandbox_sys");
        fs::create_dir_all(&sandbox).ok();
        let mut layer = SecurityLayer::with_sandbox_root(sandbox.clone());
        layer.allowed_system_paths.push(PathBuf::from("/etc"));

        let result = layer.validate_path("/etc/hosts");
        assert!(
            result.is_ok(),
            "Expected Ok for allowed system path, got: {:?}",
            result
        );

        // Clean up
        let _ = fs::remove_dir(&sandbox);
    }

    // ---- Sudo detection ----

    #[test]
    fn test_is_sudo_command() {
        assert!(SecurityLayer::is_sudo_command("sudo apt update"));
        assert!(SecurityLayer::is_sudo_command("  sudo rm -rf /tmp/foo"));
        assert!(SecurityLayer::is_sudo_command(
            "echo hello | sudo tee /etc/file"
        ));
        assert!(SecurityLayer::is_sudo_command("ls && sudo rm foo"));
        assert!(SecurityLayer::is_sudo_command("ls; sudo rm foo"));

        assert!(!SecurityLayer::is_sudo_command("ls -la"));
        assert!(!SecurityLayer::is_sudo_command("echo sudo is a word"));
        assert!(!SecurityLayer::is_sudo_command("cat sudoers"));
    }

    // ---- Permission decision mapping ----

    #[test]
    fn test_permission_decision_low() {
        let layer = test_layer();
        let decision = layer.check_permission("ls -la");
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_permission_decision_medium() {
        let layer = test_layer();
        let decision = layer.check_permission("cp a.txt b.txt");
        assert_eq!(decision, PermissionDecision::AllowWithLogging);
    }

    #[test]
    fn test_permission_decision_high() {
        let layer = test_layer();
        let decision = layer.check_permission("rm file.txt");
        assert!(matches!(
            decision,
            PermissionDecision::RequireApproval { .. }
        ));
    }

    #[test]
    fn test_permission_decision_blocked() {
        let layer = test_layer();
        let decision = layer.check_permission("rm -rf /");
        assert!(matches!(decision, PermissionDecision::Deny { .. }));
    }

    // ---- Unknown commands default to MEDIUM ----

    #[test]
    fn test_unknown_command_defaults_to_medium() {
        let layer = test_layer();
        let result = layer.classify_command("some_obscure_tool --flag");
        assert_eq!(result.risk_level, RiskLevel::Medium);
    }

    // ---- Default constructor ----

    #[test]
    fn test_default_constructor() {
        let layer = SecurityLayer::new();
        assert!(!layer.sandbox_root.as_os_str().is_empty());
        assert!(!layer.blocked_commands.is_empty());
    }
}
