use serde::{Deserialize, Serialize};

/// Risk level classification for tool calls.
///
/// Used by the permission engine to decide whether a tool invocation
/// can be auto-approved or requires explicit user consent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Classify the risk level of a tool call based on the tool name and its parameters.
///
/// # Rules
/// - **Low**: read-only / passive tools (`memory`, `clipboard`, `web-search`, `web-fetch`)
///   as well as read-only filesystem operations (`read`, `glob`, `grep`).
/// - **Medium**: tools that mutate local data (`filesystem` writes, `data`, `document`).
/// - **High**: tools with broad system access (`shell`, `system`, `browser`).
/// - **Critical**: any unknown / unrecognised tool name.
pub fn classify_risk(tool_name: &str, params: &serde_json::Value) -> RiskLevel {
    match tool_name {
        // LOW risk — passive / read-only tools
        "memory" | "clipboard" | "web-search" | "web-fetch" => RiskLevel::Low,

        // MEDIUM risk — data-mutating tools (with read-operation downgrade)
        "filesystem" | "data" | "document" => {
            if let Some(op) = params.get("operation").and_then(|v| v.as_str()) {
                if op == "read" || op == "glob" || op == "grep" {
                    RiskLevel::Low
                } else {
                    RiskLevel::Medium
                }
            } else {
                RiskLevel::Medium
            }
        }

        // HIGH risk — system-level tools
        "shell" | "system" | "browser" => RiskLevel::High,

        // CRITICAL — unknown tools
        _ => RiskLevel::Critical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_classify_low_risk() {
        let empty = json!({});
        assert_eq!(classify_risk("memory", &empty), RiskLevel::Low);
        assert_eq!(classify_risk("clipboard", &empty), RiskLevel::Low);
        assert_eq!(classify_risk("web-search", &empty), RiskLevel::Low);
        assert_eq!(classify_risk("web-fetch", &empty), RiskLevel::Low);
    }

    #[test]
    fn test_classify_filesystem_read_is_low() {
        let params = json!({ "operation": "read" });
        assert_eq!(classify_risk("filesystem", &params), RiskLevel::Low);

        let params = json!({ "operation": "glob" });
        assert_eq!(classify_risk("filesystem", &params), RiskLevel::Low);

        let params = json!({ "operation": "grep" });
        assert_eq!(classify_risk("filesystem", &params), RiskLevel::Low);
    }

    #[test]
    fn test_classify_medium_risk() {
        let params = json!({ "operation": "write" });
        assert_eq!(classify_risk("filesystem", &params), RiskLevel::Medium);

        let empty = json!({});
        assert_eq!(classify_risk("data", &empty), RiskLevel::Medium);
        assert_eq!(classify_risk("document", &empty), RiskLevel::Medium);
        // filesystem with no operation defaults to medium
        assert_eq!(classify_risk("filesystem", &empty), RiskLevel::Medium);
    }

    #[test]
    fn test_classify_high_risk() {
        let empty = json!({});
        assert_eq!(classify_risk("shell", &empty), RiskLevel::High);
        assert_eq!(classify_risk("system", &empty), RiskLevel::High);
        assert_eq!(classify_risk("browser", &empty), RiskLevel::High);
    }

    #[test]
    fn test_classify_critical_risk() {
        let empty = json!({});
        assert_eq!(classify_risk("unknown-tool", &empty), RiskLevel::Critical);
        assert_eq!(
            classify_risk("something-random", &empty),
            RiskLevel::Critical
        );
    }
}
