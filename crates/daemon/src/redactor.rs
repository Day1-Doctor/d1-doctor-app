//! Sensitive data redaction for cloud-bound messages.
//!
//! The [`Redactor`] inspects text destined for the cloud orchestrator and
//! replaces sensitive patterns (API keys, passwords, tokens, file paths,
//! .env contents, connection strings, private key blocks) with safe
//! placeholders like `[REDACTED:api_key]`.
//!
//! Redaction categories are individually configurable via
//! [`d1_common::config::RedactionConfig`].

use d1_common::config::RedactionConfig;
use regex::Regex;
use tracing::debug;

// ---------------------------------------------------------------------------
// Placeholder constants
// ---------------------------------------------------------------------------

const PH_API_KEY: &str = "[REDACTED:api_key]";
const PH_PASSWORD: &str = "[REDACTED:credential]";
const PH_TOKEN: &str = "[REDACTED:token]";
const PH_ENV_LINE: &str = "[REDACTED:env]";
const PH_FILE_PATH: &str = "[REDACTED:path]";
const PH_CONN_STRING: &str = "[REDACTED:connection_string]";
const PH_PRIVATE_KEY: &str = "[REDACTED:private_key]";
const PH_CUSTOM: &str = "[REDACTED:custom]";

// ---------------------------------------------------------------------------
// Redactor
// ---------------------------------------------------------------------------

/// Redacts sensitive data from text before it is sent to the cloud.
///
/// Construct via [`Redactor::from_config`] or [`Redactor::new`] (all rules on).
pub struct Redactor {
    config: RedactionConfig,
    /// Pre-compiled regexes grouped by category.
    api_key_patterns: Vec<Regex>,
    password_patterns: Vec<Regex>,
    token_patterns: Vec<Regex>,
    env_file_patterns: Vec<Regex>,
    file_path_patterns: Vec<Regex>,
    connection_string_patterns: Vec<Regex>,
    private_key_pattern: Regex,
    custom_patterns: Vec<Regex>,
}

impl Redactor {
    /// Create a redactor with all rules enabled (default config).
    pub fn new() -> Self {
        Self::from_config(&RedactionConfig::default())
    }

    /// Create a redactor driven by the given configuration.
    pub fn from_config(config: &RedactionConfig) -> Self {
        let custom_patterns = config
            .custom_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        Self {
            config: config.clone(),
            api_key_patterns: compile_api_key_patterns(),
            password_patterns: compile_password_patterns(),
            token_patterns: compile_token_patterns(),
            env_file_patterns: compile_env_file_patterns(),
            file_path_patterns: compile_file_path_patterns(),
            connection_string_patterns: compile_connection_string_patterns(),
            private_key_pattern: Regex::new(
                r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----",
            )
            .unwrap(),
            custom_patterns,
        }
    }

    /// Redact all enabled categories from `text`.
    pub fn redact(&self, text: &str) -> String {
        if !self.config.enabled {
            return text.to_string();
        }

        let mut result = text.to_string();

        // Order matters: longer/more-specific patterns first to avoid partial
        // matches interfering with broader patterns.

        if self.config.redact_private_keys {
            result = self
                .private_key_pattern
                .replace_all(&result, PH_PRIVATE_KEY)
                .to_string();
        }

        if self.config.redact_connection_strings {
            for re in &self.connection_string_patterns {
                result = re.replace_all(&result, PH_CONN_STRING).to_string();
            }
        }

        if self.config.redact_api_keys {
            for re in &self.api_key_patterns {
                result = re.replace_all(&result, PH_API_KEY).to_string();
            }
        }

        if self.config.redact_passwords {
            for re in &self.password_patterns {
                result = re.replace_all(&result, PH_PASSWORD).to_string();
            }
        }

        if self.config.redact_tokens {
            for re in &self.token_patterns {
                result = re.replace_all(&result, PH_TOKEN).to_string();
            }
        }

        if self.config.redact_env_file {
            for re in &self.env_file_patterns {
                result = re.replace_all(&result, PH_ENV_LINE).to_string();
            }
        }

        if self.config.redact_file_paths {
            for re in &self.file_path_patterns {
                result = re.replace_all(&result, PH_FILE_PATH).to_string();
            }
        }

        for re in &self.custom_patterns {
            result = re.replace_all(&result, PH_CUSTOM).to_string();
        }

        if result != text {
            debug!("Redactor: redacted sensitive data from cloud-bound message");
        }

        result
    }

    /// Redact a JSON value in-place (recursively walks strings).
    pub fn redact_json(&self, value: &serde_json::Value) -> serde_json::Value {
        if !self.config.enabled {
            return value.clone();
        }

        match value {
            serde_json::Value::String(s) => serde_json::Value::String(self.redact(s)),
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| self.redact_json(v)).collect())
            }
            serde_json::Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    new_map.insert(k.clone(), self.redact_json(v));
                }
                serde_json::Value::Object(new_map)
            }
            other => other.clone(),
        }
    }

    /// Strip a system info JSON object to only safe fields:
    /// `os`, `arch`, `shell`, `hostname`.
    pub fn sanitize_system_info(&self, info: &serde_json::Value) -> serde_json::Value {
        if !self.config.limit_system_info {
            return info.clone();
        }

        const SAFE_FIELDS: &[&str] = &["os", "arch", "shell", "hostname"];

        match info {
            serde_json::Value::Object(map) => {
                let mut safe = serde_json::Map::new();
                for &field in SAFE_FIELDS {
                    if let Some(val) = map.get(field) {
                        safe.insert(field.to_string(), val.clone());
                    }
                }
                serde_json::Value::Object(safe)
            }
            _ => info.clone(),
        }
    }
}

impl Default for Redactor {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Pattern compilation helpers
// ---------------------------------------------------------------------------

fn compile_api_key_patterns() -> Vec<Regex> {
    [
        // OpenAI / generic sk- keys (at least 20 chars after prefix)
        r"sk-[A-Za-z0-9_\-]{20,}",
        // AWS access key IDs
        r"AKIA[0-9A-Z]{16}",
        // GitHub personal access tokens (classic and fine-grained)
        r"ghp_[A-Za-z0-9]{36,}",
        // GitHub OAuth tokens
        r"gho_[A-Za-z0-9]{36,}",
        // GitHub user-to-server tokens
        r"ghu_[A-Za-z0-9]{36,}",
        // GitHub server-to-server tokens
        r"ghs_[A-Za-z0-9]{36,}",
        // GitHub refresh tokens
        r"ghr_[A-Za-z0-9]{36,}",
        // Slack bot tokens
        r"xoxb-[A-Za-z0-9\-]{24,}",
        // Slack user tokens
        r"xoxp-[A-Za-z0-9\-]{24,}",
        // Slack app-level tokens
        r"xapp-[A-Za-z0-9\-]{24,}",
        // Stripe secret keys
        r"sk_live_[A-Za-z0-9]{24,}",
        // Stripe test keys
        r"sk_test_[A-Za-z0-9]{24,}",
        // Anthropic API keys
        r"sk-ant-[A-Za-z0-9_\-]{20,}",
        // Google API keys
        r"AIza[A-Za-z0-9_\-]{35}",
        // Supabase service role / anon keys (long JWT-like base64)
        r"(?:service_role|anon)\s*[:=]\s*eyJ[A-Za-z0-9_\-]+\.eyJ[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+",
        // AWS secret access keys (40 char base64, preceded by common labels)
        r"(?i)(?:aws_secret_access_key|secret_key)\s*[:=]\s*[A-Za-z0-9/+=]{40}",
    ]
    .iter()
    .map(|p| Regex::new(p).expect("invalid api key regex"))
    .collect()
}

fn compile_password_patterns() -> Vec<Regex> {
    [
        // password= / passwd= / pwd= / secret= with value
        r"(?i)(?:password|passwd|pwd|secret|credentials?)\s*[:=]\s*\S+",
    ]
    .iter()
    .map(|p| Regex::new(p).expect("invalid password regex"))
    .collect()
}

fn compile_token_patterns() -> Vec<Regex> {
    [
        // Authorization: Bearer <token>
        r"(?i)(?:authorization|auth)\s*[:=]\s*bearer\s+\S+",
        // Generic token= patterns
        r"(?i)(?:api_?token|access_?token|auth_?token|token)\s*[:=]\s*[A-Za-z0-9_\-\.]{16,}",
    ]
    .iter()
    .map(|p| Regex::new(p).expect("invalid token regex"))
    .collect()
}

fn compile_env_file_patterns() -> Vec<Regex> {
    [
        // Lines that look like .env entries: KEY=value (uppercase key, no spaces before =)
        // Must start at line beginning or after whitespace
        r"(?m)^[A-Z][A-Z0-9_]{2,}=[^\n]+",
    ]
    .iter()
    .map(|p| Regex::new(p).expect("invalid env file regex"))
    .collect()
}

fn compile_file_path_patterns() -> Vec<Regex> {
    [
        // Unix absolute paths: /home/user/... /Users/name/...
        // Must have at least 2 segments to avoid matching lone /
        r#"/(?:home|Users|root|var|etc|opt|tmp|private)/[^\s:,;"'`\]})]+"#,
        // Windows paths: C:\Users\...
        r#"[A-Z]:\\(?:Users|Documents|AppData|Program Files)[^\s:;"'`\]})]+"#,
        // ~ home expansion
        r#"~/[^\s:,;"'`\]})]+"#,
    ]
    .iter()
    .map(|p| Regex::new(p).expect("invalid file path regex"))
    .collect()
}

fn compile_connection_string_patterns() -> Vec<Regex> {
    [
        // Database connection URLs with credentials
        r#"(?i)(?:postgres(?:ql)?|mysql|mongodb(?:\+srv)?|redis|amqp)://[^\s"'`]+"#,
    ]
    .iter()
    .map(|p| Regex::new(p).expect("invalid connection string regex"))
    .collect()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use d1_common::config::RedactionConfig;

    fn redactor() -> Redactor {
        Redactor::new()
    }

    // -- Master switch -------------------------------------------------------

    #[test]
    fn disabled_redactor_passes_through() {
        let config = RedactionConfig {
            enabled: false,
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let input = "sk-abc123456789012345678901234567890";
        assert_eq!(r.redact(input), input);
    }

    // -- API key patterns ----------------------------------------------------

    #[test]
    fn redact_openai_key() {
        let r = redactor();
        let input = "My key is sk-abcdefghijklmnopqrstuvwxyz1234567890ABCD";
        let out = r.redact(input);
        assert!(!out.contains("sk-abcdefghij"));
        assert!(out.contains(PH_API_KEY));
    }

    #[test]
    fn redact_anthropic_key() {
        let r = redactor();
        let input = "key=sk-ant-api03-aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789";
        let out = r.redact(input);
        assert!(!out.contains("sk-ant-"));
        assert!(out.contains(PH_API_KEY));
    }

    #[test]
    fn redact_aws_access_key_id() {
        let r = redactor();
        let input = "AKIAIOSFODNN7EXAMPLE";
        let out = r.redact(input);
        assert!(!out.contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(out.contains(PH_API_KEY));
    }

    #[test]
    fn redact_github_pat() {
        let r = redactor();
        let input = "token=ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij0123";
        let out = r.redact(input);
        assert!(!out.contains("ghp_ABCDE"));
        assert!(out.contains(PH_API_KEY));
    }

    #[test]
    fn redact_github_oauth_token() {
        let r = redactor();
        let input = "gho_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij0123";
        let out = r.redact(input);
        assert!(!out.contains("gho_"));
        assert!(out.contains(PH_API_KEY));
    }

    #[test]
    fn redact_slack_bot_token() {
        let r = redactor();
        // Build the test token at runtime to avoid GitHub push-protection false positive.
        let prefix = "xoxb-";
        let suffix = "0000000000-FAKETOKEN00000";
        let input = format!("{prefix}{suffix}");
        let out = r.redact(&input);
        assert!(!out.contains("xoxb-"));
        assert!(out.contains(PH_API_KEY));
    }

    #[test]
    fn redact_stripe_live_key() {
        let r = redactor();
        // Build at runtime to avoid GitHub push-protection false positive.
        let input = format!("{}_{}", "sk_live", "FAKE0aBcDeFgHiJkLmNoPqRsTuVwX");
        let out = r.redact(&input);
        assert!(!out.contains("sk_live_"));
        assert!(out.contains(PH_API_KEY));
    }

    #[test]
    fn redact_stripe_test_key() {
        let r = redactor();
        // Build at runtime to avoid GitHub push-protection false positive.
        let input = format!("{}_{}", "sk_test", "FAKE0aBcDeFgHiJkLmNoPqRsTuVwX");
        let out = r.redact(&input);
        assert!(!out.contains("sk_test_"));
        assert!(out.contains(PH_API_KEY));
    }

    #[test]
    fn redact_google_api_key() {
        let r = redactor();
        let input = "AIzaSyD-abcdefghijklmnopqrstuvwxyz123456";
        let out = r.redact(input);
        assert!(!out.contains("AIzaSyD"));
        assert!(out.contains(PH_API_KEY));
    }

    #[test]
    fn redact_aws_secret_access_key() {
        let r = redactor();
        let input = "aws_secret_access_key=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let out = r.redact(input);
        assert!(!out.contains("wJalrXUtnFEMI"));
        assert!(out.contains(PH_API_KEY));
    }

    #[test]
    fn does_not_redact_short_sk_prefix() {
        // "sk-short" is too short to be a real key — should not match
        let r = redactor();
        let input = "sk-short";
        let out = r.redact(input);
        assert_eq!(out, input);
    }

    // -- Password patterns ---------------------------------------------------

    #[test]
    fn redact_password_equals() {
        let r = redactor();
        let input = "password=SuperSecret123!";
        let out = r.redact(input);
        assert!(!out.contains("SuperSecret123!"));
        assert!(out.contains(PH_PASSWORD));
    }

    #[test]
    fn redact_passwd_colon() {
        let r = redactor();
        let input = "passwd: hunter2";
        let out = r.redact(input);
        assert!(!out.contains("hunter2"));
        assert!(out.contains(PH_PASSWORD));
    }

    #[test]
    fn redact_secret_key() {
        let r = redactor();
        let input = "SECRET=my_secret_value_here";
        let out = r.redact(input);
        assert!(!out.contains("my_secret_value_here"));
    }

    #[test]
    fn redact_credential_value() {
        let r = redactor();
        let input = "credential=abc123def456";
        let out = r.redact(input);
        assert!(!out.contains("abc123def456"));
        assert!(out.contains(PH_PASSWORD));
    }

    // -- Token patterns ------------------------------------------------------

    #[test]
    fn redact_bearer_token() {
        let r = redactor();
        let input = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let out = r.redact(input);
        assert!(!out.contains("eyJhbGci"));
        assert!(out.contains(PH_TOKEN));
    }

    #[test]
    fn redact_api_token_equals() {
        let r = redactor();
        let input = "api_token=abcdefghijklmnop12345678";
        let out = r.redact(input);
        assert!(!out.contains("abcdefghijklmnop12345678"));
        assert!(out.contains(PH_TOKEN));
    }

    #[test]
    fn redact_access_token() {
        let r = redactor();
        let input = "access_token=some_long_token_value_here";
        let out = r.redact(input);
        assert!(!out.contains("some_long_token_value_here"));
        assert!(out.contains(PH_TOKEN));
    }

    // -- .env file patterns --------------------------------------------------

    #[test]
    fn redact_env_line() {
        let r = redactor();
        let input =
            "DATABASE_URL=postgres://user:pass@host/db\nAPI_KEY=sk-12345\nNODE_ENV=production";
        let out = r.redact(input);
        // The env lines themselves should be redacted
        assert!(!out.contains("postgres://user:pass@host/db"));
        assert!(!out.contains("production"));
    }

    #[test]
    fn env_redaction_only_matches_uppercase_keys() {
        let r = redactor();
        // Lowercase keys should NOT be caught by the env pattern
        let input = "some_variable=hello";
        let out = r.redact(input);
        // This should not be redacted by env pattern (lowercase start)
        assert_eq!(out, input);
    }

    // -- File path patterns --------------------------------------------------

    #[test]
    fn redact_unix_home_path() {
        let r = redactor();
        let input = "File at /home/john/Documents/secret.txt";
        let out = r.redact(input);
        assert!(!out.contains("/home/john/Documents/secret.txt"));
        assert!(out.contains(PH_FILE_PATH));
    }

    #[test]
    fn redact_macos_users_path() {
        let r = redactor();
        let input = "Reading /Users/alice/projects/app/config.toml";
        let out = r.redact(input);
        assert!(!out.contains("/Users/alice/projects/app/config.toml"));
        assert!(out.contains(PH_FILE_PATH));
    }

    #[test]
    fn redact_windows_path() {
        let r = redactor();
        let input = r"Located at C:\Users\bob\Documents\project\file.rs";
        let out = r.redact(input);
        assert!(!out.contains(r"C:\Users\bob"));
        assert!(out.contains(PH_FILE_PATH));
    }

    #[test]
    fn redact_tilde_path() {
        let r = redactor();
        let input = "Config in ~/projects/my-app/.env";
        let out = r.redact(input);
        assert!(!out.contains("~/projects/my-app/.env"));
        assert!(out.contains(PH_FILE_PATH));
    }

    #[test]
    fn preserves_single_slash() {
        let r = redactor();
        let input = "Use / as root";
        let out = r.redact(input);
        assert!(out.contains("/"));
    }

    // -- Connection string patterns ------------------------------------------

    #[test]
    fn redact_postgres_url() {
        let r = redactor();
        // Use a sentence context (not a bare KEY=val env line) to test the
        // connection-string pattern in isolation.
        let input = "Connecting to postgres://admin:p4ss@db.example.com:5432/mydb now";
        let out = r.redact(input);
        assert!(!out.contains("admin:p4ss"));
        assert!(out.contains(PH_CONN_STRING));
    }

    #[test]
    fn redact_postgres_url_in_env_line() {
        let r = redactor();
        // When the whole line looks like an env var, the env pattern fires
        // and removes the sensitive data even though it's also a conn string.
        let input = "DATABASE_URL=postgres://admin:p4ss@db.example.com:5432/mydb";
        let out = r.redact(input);
        assert!(!out.contains("admin:p4ss"));
        // Redacted by either env or conn-string pattern — either is fine.
        assert!(out.contains("[REDACTED:"));
    }

    #[test]
    fn redact_mongodb_url() {
        let r = redactor();
        let input = "mongodb+srv://user:pass@cluster0.abc.mongodb.net/test";
        let out = r.redact(input);
        assert!(!out.contains("user:pass"));
        assert!(out.contains(PH_CONN_STRING));
    }

    #[test]
    fn redact_redis_url() {
        let r = redactor();
        let input = "redis://default:secret@redis.example.com:6379";
        let out = r.redact(input);
        assert!(!out.contains("default:secret"));
        assert!(out.contains(PH_CONN_STRING));
    }

    #[test]
    fn redact_mysql_url() {
        let r = redactor();
        let input = "mysql://root:password@localhost:3306/app";
        let out = r.redact(input);
        assert!(!out.contains("root:password"));
        assert!(out.contains(PH_CONN_STRING));
    }

    // -- Private key patterns ------------------------------------------------

    #[test]
    fn redact_rsa_private_key() {
        let r = redactor();
        let input =
            "Key:\n-----BEGIN RSA PRIVATE KEY-----\nMIIE...\n-----END RSA PRIVATE KEY-----\nDone";
        let out = r.redact(input);
        assert!(!out.contains("MIIE"));
        assert!(out.contains(PH_PRIVATE_KEY));
        // "Done" should survive
        assert!(out.contains("Done"));
    }

    #[test]
    fn redact_ec_private_key() {
        let r = redactor();
        let input = "-----BEGIN EC PRIVATE KEY-----\ndata\n-----END EC PRIVATE KEY-----";
        let out = r.redact(input);
        assert!(!out.contains("data"));
        assert!(out.contains(PH_PRIVATE_KEY));
    }

    // -- Custom patterns -----------------------------------------------------

    #[test]
    fn custom_pattern_redacts() {
        let config = RedactionConfig {
            custom_patterns: vec![r"SSN-\d{3}-\d{2}-\d{4}".to_string()],
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let input = "SSN-123-45-6789 is on file";
        let out = r.redact(input);
        assert!(!out.contains("123-45-6789"));
        assert!(out.contains(PH_CUSTOM));
    }

    #[test]
    fn invalid_custom_pattern_ignored() {
        let config = RedactionConfig {
            custom_patterns: vec![r"[invalid regex".to_string()],
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        // Should not panic, just no custom patterns applied
        let out = r.redact("hello");
        assert_eq!(out, "hello");
    }

    // -- Category toggles ----------------------------------------------------

    #[test]
    fn disabling_api_keys_preserves_them() {
        let config = RedactionConfig {
            redact_api_keys: false,
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let input = "sk-abcdefghijklmnopqrstuvwxyz1234567890";
        let out = r.redact(input);
        assert!(out.contains("sk-abcdefghijklmnop"));
    }

    #[test]
    fn disabling_passwords_preserves_them() {
        let config = RedactionConfig {
            redact_passwords: false,
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let input = "password=hunter2";
        let out = r.redact(input);
        assert!(out.contains("hunter2"));
    }

    #[test]
    fn disabling_tokens_preserves_them() {
        let config = RedactionConfig {
            redact_tokens: false,
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let input = "Authorization: Bearer mytoken1234567890ab";
        let out = r.redact(input);
        assert!(out.contains("Bearer mytoken1234567890ab"));
    }

    #[test]
    fn disabling_file_paths_preserves_them() {
        let config = RedactionConfig {
            redact_file_paths: false,
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let input = "/Users/alice/projects/app/main.rs";
        let out = r.redact(input);
        assert!(out.contains("/Users/alice/projects/app/main.rs"));
    }

    #[test]
    fn disabling_connection_strings_preserves_them() {
        let config = RedactionConfig {
            redact_connection_strings: false,
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let input = "postgres://user:pass@host/db";
        let out = r.redact(input);
        assert!(out.contains("user:pass"));
    }

    #[test]
    fn disabling_private_keys_preserves_them() {
        let config = RedactionConfig {
            redact_private_keys: false,
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let input = "-----BEGIN RSA PRIVATE KEY-----\ndata\n-----END RSA PRIVATE KEY-----";
        let out = r.redact(input);
        assert!(out.contains("data"));
    }

    // -- JSON redaction ------------------------------------------------------

    #[test]
    fn redact_json_string_values() {
        let r = redactor();
        let val = serde_json::json!({
            "message": "Check /Users/alice/project/.env",
            "count": 42,
            "nested": {
                "key": "sk-abcdefghijklmnopqrstuvwxyz1234567890"
            }
        });

        let out = r.redact_json(&val);
        assert!(out["message"].as_str().unwrap().contains(PH_FILE_PATH));
        assert!(out["nested"]["key"].as_str().unwrap().contains(PH_API_KEY));
        assert_eq!(out["count"], 42);
    }

    #[test]
    fn redact_json_array_values() {
        let r = redactor();
        let val = serde_json::json!(["safe text", "password=secret123", 42]);
        let out = r.redact_json(&val);
        let arr = out.as_array().unwrap();
        assert_eq!(arr[0].as_str().unwrap(), "safe text");
        assert!(arr[1].as_str().unwrap().contains(PH_PASSWORD));
        assert_eq!(arr[2], 42);
    }

    // -- System info sanitization --------------------------------------------

    #[test]
    fn sanitize_system_info_keeps_safe_fields() {
        let r = redactor();
        let info = serde_json::json!({
            "os": "macOS",
            "arch": "aarch64",
            "shell": "/bin/zsh",
            "hostname": "macbook",
            "cpu_count": 10,
            "memory_bytes": 17179869184u64,
            "disk_bytes": 500000000000u64,
            "os_version": "14.0",
            "username": "alice",
            "home_dir": "/Users/alice"
        });

        let out = r.sanitize_system_info(&info);
        let map = out.as_object().unwrap();

        assert_eq!(map.len(), 4);
        assert_eq!(map["os"], "macOS");
        assert_eq!(map["arch"], "aarch64");
        assert_eq!(map["shell"], "/bin/zsh");
        assert_eq!(map["hostname"], "macbook");
        assert!(!map.contains_key("cpu_count"));
        assert!(!map.contains_key("memory_bytes"));
        assert!(!map.contains_key("username"));
        assert!(!map.contains_key("home_dir"));
    }

    #[test]
    fn sanitize_system_info_disabled_passes_through() {
        let config = RedactionConfig {
            limit_system_info: false,
            ..Default::default()
        };
        let r = Redactor::from_config(&config);
        let info = serde_json::json!({
            "os": "Linux",
            "cpu_count": 8,
            "username": "root"
        });
        let out = r.sanitize_system_info(&info);
        assert_eq!(out, info);
    }

    // -- Mixed content -------------------------------------------------------

    #[test]
    fn redact_mixed_content() {
        let r = redactor();
        let input = concat!(
            "Error in /Users/bob/project/app.rs:\n",
            "  password=hunter2\n",
            "  API key: sk-abcdefghijklmnopqrstuvwxyz1234567890\n",
            "  DB: postgres://admin:secret@db.host:5432/app\n",
        );
        let out = r.redact(input);
        assert!(!out.contains("/Users/bob/project/app.rs"));
        assert!(!out.contains("hunter2"));
        assert!(!out.contains("sk-abcdef"));
        assert!(!out.contains("admin:secret"));
    }

    #[test]
    fn safe_text_unchanged() {
        let r = redactor();
        let input = "This is a normal message about installing packages.";
        let out = r.redact(input);
        assert_eq!(out, input);
    }

    #[test]
    fn empty_string_unchanged() {
        let r = redactor();
        assert_eq!(r.redact(""), "");
    }

    // -- Default config round-trip -------------------------------------------

    #[test]
    fn default_config_round_trips_toml() {
        let config = RedactionConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: RedactionConfig = toml::from_str(&toml_str).unwrap();
        assert!(parsed.enabled);
        assert!(parsed.redact_api_keys);
        assert!(parsed.redact_passwords);
        assert!(parsed.redact_tokens);
        assert!(parsed.redact_env_file);
        assert!(parsed.redact_file_paths);
        assert!(parsed.limit_system_info);
        assert!(parsed.redact_connection_strings);
        assert!(parsed.redact_private_keys);
        assert!(parsed.custom_patterns.is_empty());
    }
}
