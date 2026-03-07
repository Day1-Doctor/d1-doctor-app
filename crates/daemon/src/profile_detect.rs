//! System profile auto-detection.
//!
//! Detects the user's environment: OS, architecture, default shell,
//! and commonly installed developer tools. The results are stored as
//! `ProfileFact` entries that feed into agent memory.

use serde::{Deserialize, Serialize};
use std::process::Command;

/// A single detected fact about the user's system profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileFact {
    /// Fact key (e.g., "os", "arch", "shell", "tool:git")
    pub key: String,
    /// Fact value (e.g., "macOS", "aarch64", "/bin/zsh", "2.43.0")
    pub value: String,
    /// How this fact was obtained (e.g., "system", "env", "which")
    pub source: String,
}

impl ProfileFact {
    fn new(key: impl Into<String>, value: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            source: source.into(),
        }
    }
}

/// Detect the system profile, returning a list of facts.
///
/// Collects OS name/version, CPU architecture, default shell,
/// and checks for commonly installed developer tools.
pub fn detect_system_profile() -> Vec<ProfileFact> {
    let mut facts = Vec::new();

    // OS detection
    detect_os(&mut facts);

    // Architecture
    detect_arch(&mut facts);

    // Shell
    detect_shell(&mut facts);

    // Hostname
    detect_hostname(&mut facts);

    // Hardware info via sysinfo
    detect_hardware(&mut facts);

    // Installed developer tools
    detect_tools(&mut facts);

    facts
}

fn detect_os(facts: &mut Vec<ProfileFact>) {
    let os_name = std::env::consts::OS;
    let os_family = std::env::consts::FAMILY;

    facts.push(ProfileFact::new("os", os_name, "system"));
    facts.push(ProfileFact::new("os_family", os_family, "system"));

    // Try to get a more detailed version string
    if let Some(version) = get_os_version() {
        facts.push(ProfileFact::new("os_version", version, "system"));
    }
}

fn get_os_version() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
    }

    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|l| l.starts_with("PRETTY_NAME="))
                    .map(|l| {
                        l.trim_start_matches("PRETTY_NAME=")
                            .trim_matches('"')
                            .to_string()
                    })
            })
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

fn detect_arch(facts: &mut Vec<ProfileFact>) {
    facts.push(ProfileFact::new("arch", std::env::consts::ARCH, "system"));
}

fn detect_shell(facts: &mut Vec<ProfileFact>) {
    if let Ok(shell) = std::env::var("SHELL") {
        facts.push(ProfileFact::new("shell", shell, "env"));
    }
}

fn detect_hostname(facts: &mut Vec<ProfileFact>) {
    if let Ok(output) = Command::new("hostname").output() {
        if output.status.success() {
            if let Ok(name) = String::from_utf8(output.stdout) {
                facts.push(ProfileFact::new("hostname", name.trim(), "system"));
            }
        }
    }
}

fn detect_hardware(facts: &mut Vec<ProfileFact>) {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_count = sys.cpus().len();
    facts.push(ProfileFact::new(
        "cpu_count",
        cpu_count.to_string(),
        "system",
    ));

    let total_memory_mb = sys.total_memory() / (1024 * 1024);
    facts.push(ProfileFact::new(
        "memory_total_mb",
        total_memory_mb.to_string(),
        "system",
    ));
}

fn detect_tools(facts: &mut Vec<ProfileFact>) {
    let tools = [
        ("git", "git"),
        ("node", "node"),
        ("npm", "npm"),
        ("python3", "python3"),
        ("pip3", "pip3"),
        ("cargo", "cargo"),
        ("rustc", "rustc"),
        ("docker", "docker"),
        ("brew", "brew"),
        ("code", "code"),
        ("vim", "vim"),
        ("curl", "curl"),
        ("wget", "wget"),
    ];

    for (name, cmd) in &tools {
        if let Some(version) = get_tool_version(cmd) {
            facts.push(ProfileFact::new(format!("tool:{}", name), version, "which"));
        }
    }
}

/// Try to get a tool's version string by running `<cmd> --version`.
fn get_tool_version(cmd: &str) -> Option<String> {
    Command::new(cmd)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| {
                        // Take only the first line for brevity
                        s.lines().next().unwrap_or("").trim().to_string()
                    })
                    .filter(|s| !s.is_empty())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_system_profile_not_empty() {
        let facts = detect_system_profile();
        assert!(!facts.is_empty(), "Should detect at least OS and arch");
    }

    #[test]
    fn test_os_fact_present() {
        let facts = detect_system_profile();
        let os_fact = facts.iter().find(|f| f.key == "os");
        assert!(os_fact.is_some(), "Should have an 'os' fact");
        assert!(!os_fact.unwrap().value.is_empty());
    }

    #[test]
    fn test_arch_fact_present() {
        let facts = detect_system_profile();
        let arch_fact = facts.iter().find(|f| f.key == "arch");
        assert!(arch_fact.is_some(), "Should have an 'arch' fact");
    }

    #[test]
    fn test_profile_fact_creation() {
        let fact = ProfileFact::new("test_key", "test_value", "test_source");
        assert_eq!(fact.key, "test_key");
        assert_eq!(fact.value, "test_value");
        assert_eq!(fact.source, "test_source");
    }

    #[test]
    fn test_profile_fact_serialization() {
        let fact = ProfileFact::new("os", "macOS", "system");
        let json = serde_json::to_string(&fact).unwrap();
        assert!(json.contains("\"key\":\"os\""));
        assert!(json.contains("\"value\":\"macOS\""));
    }
}
