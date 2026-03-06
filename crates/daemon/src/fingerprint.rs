//! Device fingerprint generation for device registration and cloud auth.
//!
//! Generates a stable, deterministic fingerprint from machine ID + hostname + OS.
//! The fingerprint is a SHA-256 hash that remains consistent across daemon restarts.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Metadata about the device, sent alongside the fingerprint during AUTH handshake.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceMetadata {
    pub os_name: String,
    pub arch: String,
    pub hostname: String,
    pub daemon_version: String,
}

/// A device fingerprint consisting of a stable hash and associated metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceFingerprint {
    /// SHA-256 hex digest derived from machine_id + hostname + os_name.
    pub fingerprint: String,
    pub metadata: DeviceMetadata,
}

impl DeviceFingerprint {
    /// Generate the device fingerprint for the current machine.
    pub fn generate() -> Result<Self> {
        let machine_id = read_machine_id().context("failed to read machine ID")?;
        let hostname = get_hostname();
        let os_name = get_os_name();
        let arch = get_arch();
        let daemon_version = env!("CARGO_PKG_VERSION").to_string();

        let fingerprint = compute_fingerprint(&machine_id, &hostname, &os_name);

        Ok(Self {
            fingerprint,
            metadata: DeviceMetadata {
                os_name,
                arch,
                hostname,
                daemon_version,
            },
        })
    }

    /// Generate a fingerprint from explicit inputs (useful for testing).
    pub fn from_parts(machine_id: &str, hostname: &str, os_name: &str) -> Self {
        let fingerprint = compute_fingerprint(machine_id, hostname, os_name);

        Self {
            fingerprint,
            metadata: DeviceMetadata {
                os_name: os_name.to_string(),
                arch: get_arch(),
                hostname: hostname.to_string(),
                daemon_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}

/// Compute a deterministic SHA-256 fingerprint from the three input components.
fn compute_fingerprint(machine_id: &str, hostname: &str, os_name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(machine_id.trim().as_bytes());
    hasher.update(b":");
    hasher.update(hostname.trim().as_bytes());
    hasher.update(b":");
    hasher.update(os_name.trim().as_bytes());
    hex::encode(hasher.finalize())
}

/// Read the machine ID from the OS.
///
/// - **macOS**: `IOPlatformUUID` via `ioreg`
/// - **Linux**: `/etc/machine-id` (fallback: `/var/lib/dbus/machine-id`)
/// - **Windows**: `MachineGuid` from the registry via `reg query`
fn read_machine_id() -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        read_machine_id_macos()
    }

    #[cfg(target_os = "linux")]
    {
        read_machine_id_linux()
    }

    #[cfg(target_os = "windows")]
    {
        read_machine_id_windows()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        anyhow::bail!("unsupported platform for machine ID")
    }
}

#[cfg(target_os = "macos")]
fn read_machine_id_macos() -> Result<String> {
    let output = std::process::Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .context("failed to run ioreg")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("IOPlatformUUID") {
            // Line format: "IOPlatformUUID" = "XXXXXXXX-XXXX-..."
            if let Some(uuid) = line.split('"').nth(3) {
                return Ok(uuid.to_string());
            }
        }
    }
    anyhow::bail!("IOPlatformUUID not found in ioreg output")
}

#[cfg(target_os = "linux")]
fn read_machine_id_linux() -> Result<String> {
    // Primary: /etc/machine-id
    if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
        let trimmed = id.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }
    // Fallback: /var/lib/dbus/machine-id
    if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
        let trimmed = id.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }
    anyhow::bail!("could not read machine-id from /etc/machine-id or /var/lib/dbus/machine-id")
}

#[cfg(target_os = "windows")]
fn read_machine_id_windows() -> Result<String> {
    let output = std::process::Command::new("reg")
        .args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\Cryptography",
            "/v",
            "MachineGuid",
        ])
        .output()
        .context("failed to query Windows registry for MachineGuid")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Output format: "    MachineGuid    REG_SZ    xxxxxxxx-xxxx-..."
    for line in stdout.lines() {
        if line.contains("MachineGuid") {
            if let Some(guid) = line.split_whitespace().last() {
                return Ok(guid.to_string());
            }
        }
    }
    anyhow::bail!("MachineGuid not found in registry output")
}

fn get_hostname() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn get_os_name() -> String {
    std::env::consts::OS.to_string()
}

fn get_arch() -> String {
    std::env::consts::ARCH.to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_deterministic() {
        let fp1 = DeviceFingerprint::from_parts("machine-123", "myhost", "macos");
        let fp2 = DeviceFingerprint::from_parts("machine-123", "myhost", "macos");
        assert_eq!(fp1.fingerprint, fp2.fingerprint);
    }

    #[test]
    fn test_fingerprint_changes_with_machine_id() {
        let fp1 = DeviceFingerprint::from_parts("machine-aaa", "myhost", "macos");
        let fp2 = DeviceFingerprint::from_parts("machine-bbb", "myhost", "macos");
        assert_ne!(fp1.fingerprint, fp2.fingerprint);
    }

    #[test]
    fn test_fingerprint_changes_with_hostname() {
        let fp1 = DeviceFingerprint::from_parts("machine-123", "host-a", "macos");
        let fp2 = DeviceFingerprint::from_parts("machine-123", "host-b", "macos");
        assert_ne!(fp1.fingerprint, fp2.fingerprint);
    }

    #[test]
    fn test_fingerprint_changes_with_os() {
        let fp1 = DeviceFingerprint::from_parts("machine-123", "myhost", "macos");
        let fp2 = DeviceFingerprint::from_parts("machine-123", "myhost", "linux");
        assert_ne!(fp1.fingerprint, fp2.fingerprint);
    }

    #[test]
    fn test_fingerprint_is_valid_sha256_hex() {
        let fp = DeviceFingerprint::from_parts("id", "host", "os");
        assert_eq!(fp.fingerprint.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
        assert!(fp.fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_metadata_populated() {
        let fp = DeviceFingerprint::from_parts("id", "myhost", "linux");
        assert_eq!(fp.metadata.hostname, "myhost");
        assert_eq!(fp.metadata.os_name, "linux");
        assert!(!fp.metadata.arch.is_empty());
        assert!(!fp.metadata.daemon_version.is_empty());
    }

    #[test]
    fn test_fingerprint_stability_known_vector() {
        // Pin a known input→output pair to catch accidental algorithm changes
        let fp = DeviceFingerprint::from_parts(
            "550e8400-e29b-41d4-a716-446655440000",
            "dev-laptop",
            "macos",
        );
        // Compute expected: SHA256("550e8400-e29b-41d4-a716-446655440000:dev-laptop:macos")
        let expected = {
            let mut h = Sha256::new();
            h.update(b"550e8400-e29b-41d4-a716-446655440000");
            h.update(b":");
            h.update(b"dev-laptop");
            h.update(b":");
            h.update(b"macos");
            hex::encode(h.finalize())
        };
        assert_eq!(fp.fingerprint, expected);
    }

    #[test]
    fn test_generate_on_current_machine() {
        // Integration test: generate() should succeed on any supported platform
        let fp = DeviceFingerprint::generate().expect("generate() failed on this platform");
        assert_eq!(fp.fingerprint.len(), 64);
        assert!(!fp.metadata.hostname.is_empty());
        assert!(!fp.metadata.os_name.is_empty());
        assert!(!fp.metadata.arch.is_empty());
        assert!(!fp.metadata.daemon_version.is_empty());
    }

    #[test]
    fn test_generate_is_stable_across_calls() {
        let fp1 = DeviceFingerprint::generate().unwrap();
        let fp2 = DeviceFingerprint::generate().unwrap();
        assert_eq!(fp1.fingerprint, fp2.fingerprint);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let fp = DeviceFingerprint::from_parts("id-42", "testhost", "linux");
        let json = serde_json::to_string(&fp).unwrap();
        let deserialized: DeviceFingerprint = serde_json::from_str(&json).unwrap();
        assert_eq!(fp, deserialized);
    }
}
