//! Internationalization support for the CLI.
//!
//! Provides a simple message catalog compiled into the binary via [`include_str!`].
//! Locale is detected from `LANG` / `LC_ALL` environment variables with
//! automatic fallback to English.

use std::collections::HashMap;
use std::sync::OnceLock;

/// English message catalog (compiled into binary).
const EN_CATALOG: &str = include_str!("../locales/en.toml");

/// Simplified-Chinese message catalog (compiled into binary).
const ZH_CN_CATALOG: &str = include_str!("../locales/zh-CN.toml");

/// Parsed message catalog: nested TOML sections flattened to dotted keys.
/// e.g. `[auth] opening_browser = "..."` becomes `"auth.opening_browser"`.
type Catalog = HashMap<String, String>;

/// Global catalog singleton.
static CATALOG: OnceLock<Catalog> = OnceLock::new();

/// Detect the user's locale from environment variables.
///
/// Checks `LC_ALL` first, then `LANG`. Returns a normalised locale tag
/// such as `"en"` or `"zh-CN"`. Unknown or unset values fall back to `"en"`.
pub fn detect_locale() -> String {
    let raw = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();

    normalise_locale(&raw)
}

/// Normalise a raw locale string (e.g. `"zh_CN.UTF-8"`) to a catalog key.
fn normalise_locale(raw: &str) -> String {
    // Strip encoding suffix (e.g. ".UTF-8")
    let base = raw.split('.').next().unwrap_or("");

    // Normalise separator: `zh_CN` -> `zh-CN`
    let base = base.replace('_', "-");

    match base.as_str() {
        "zh-CN" | "zh-Hans" | "zh" => "zh-CN".to_string(),
        _ => "en".to_string(),
    }
}

/// Parse a TOML catalog string into a flat key map.
fn parse_catalog(toml_str: &str) -> Catalog {
    let table: toml::Table = toml::from_str(toml_str).expect("invalid built-in catalog TOML");
    let mut map = HashMap::new();
    flatten("", &toml::Value::Table(table), &mut map);
    map
}

/// Recursively flatten a TOML value into dotted keys.
fn flatten(prefix: &str, value: &toml::Value, out: &mut Catalog) {
    match value {
        toml::Value::Table(table) => {
            for (k, v) in table {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten(&key, v, out);
            }
        }
        toml::Value::String(s) => {
            out.insert(prefix.to_string(), s.clone());
        }
        _ => {
            // Ignore non-string, non-table entries
        }
    }
}

/// Initialise the global catalog for the given locale.
/// Call this once at startup (from `main`).
pub fn init(locale: &str) {
    let toml_str = match locale {
        "zh-CN" => ZH_CN_CATALOG,
        _ => EN_CATALOG,
    };
    let catalog = parse_catalog(toml_str);
    let _ = CATALOG.set(catalog);
}

/// Initialise with auto-detected locale.
pub fn init_auto() {
    let locale = detect_locale();
    init(&locale);
}

/// Look up a message by dotted key (e.g. `"auth.opening_browser"`).
///
/// Returns the message string or the key itself if not found.
pub fn t(key: &str) -> String {
    let catalog = CATALOG
        .get()
        .expect("i18n not initialised -- call i18n::init() first");
    catalog.get(key).cloned().unwrap_or_else(|| key.to_string())
}

/// Look up a message and perform `{placeholder}` substitutions.
///
/// ```ignore
/// t_args("auth.login_success", &[("email", "user@example.com")])
/// ```
pub fn t_args(key: &str, args: &[(&str, &str)]) -> String {
    let mut msg = t(key);
    for (name, value) in args {
        msg = msg.replace(&format!("{{{}}}", name), value);
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalise_locale_english_default() {
        assert_eq!(normalise_locale(""), "en");
        assert_eq!(normalise_locale("en_US.UTF-8"), "en");
        assert_eq!(normalise_locale("C"), "en");
        assert_eq!(normalise_locale("POSIX"), "en");
    }

    #[test]
    fn test_normalise_locale_chinese() {
        assert_eq!(normalise_locale("zh_CN.UTF-8"), "zh-CN");
        assert_eq!(normalise_locale("zh_CN"), "zh-CN");
        assert_eq!(normalise_locale("zh-CN"), "zh-CN");
        assert_eq!(normalise_locale("zh"), "zh-CN");
        assert_eq!(normalise_locale("zh-Hans"), "zh-CN");
    }

    #[test]
    fn test_normalise_locale_unknown_fallback() {
        assert_eq!(normalise_locale("fr_FR.UTF-8"), "en");
        assert_eq!(normalise_locale("ja_JP"), "en");
        assert_eq!(normalise_locale("de_DE.UTF-8"), "en");
    }

    #[test]
    fn test_parse_catalog_flat_keys() {
        let catalog = parse_catalog(EN_CATALOG);
        assert!(catalog.contains_key("app.name"));
        assert!(catalog.contains_key("auth.opening_browser"));
        assert!(catalog.contains_key("chat.goodbye"));
        assert!(catalog.contains_key("gateway.table.model"));
    }

    #[test]
    fn test_parse_catalog_zh_cn() {
        let catalog = parse_catalog(ZH_CN_CATALOG);
        assert_eq!(
            catalog.get("chat.goodbye").unwrap(),
            "\u{518D}\u{89C1}\u{FF01}"
        );
        assert!(catalog.contains_key("auth.login_success"));
    }

    #[test]
    fn test_catalogs_have_same_keys() {
        let en = parse_catalog(EN_CATALOG);
        let zh = parse_catalog(ZH_CN_CATALOG);

        for key in en.keys() {
            assert!(zh.contains_key(key), "zh-CN catalog missing key: {}", key);
        }
        for key in zh.keys() {
            assert!(en.contains_key(key), "en catalog missing key: {}", key);
        }
    }

    #[test]
    fn test_detect_locale_from_env() {
        // Save originals
        let orig_lc_all = std::env::var("LC_ALL").ok();
        let orig_lang = std::env::var("LANG").ok();

        // Test LC_ALL takes precedence
        std::env::set_var("LC_ALL", "zh_CN.UTF-8");
        std::env::set_var("LANG", "en_US.UTF-8");
        assert_eq!(detect_locale(), "zh-CN");

        // Test LANG fallback
        std::env::remove_var("LC_ALL");
        std::env::set_var("LANG", "zh_CN.UTF-8");
        assert_eq!(detect_locale(), "zh-CN");

        // Test default when both unset
        std::env::remove_var("LC_ALL");
        std::env::remove_var("LANG");
        assert_eq!(detect_locale(), "en");

        // Restore originals
        match orig_lc_all {
            Some(v) => std::env::set_var("LC_ALL", v),
            None => std::env::remove_var("LC_ALL"),
        }
        match orig_lang {
            Some(v) => std::env::set_var("LANG", v),
            None => std::env::remove_var("LANG"),
        }
    }

    // Note: t() and t_args() tests use init() which sets OnceLock,
    // so we test them via parse_catalog + direct substitution logic
    // to avoid contaminating the global state.

    #[test]
    fn test_t_args_substitution() {
        let mut msg = "Successfully logged in as {email}".to_string();
        for (name, value) in &[("email", "test@example.com")] {
            msg = msg.replace(&format!("{{{}}}", name), value);
        }
        assert_eq!(msg, "Successfully logged in as test@example.com");
    }

    #[test]
    fn test_t_args_multiple_placeholders() {
        let mut msg = "Token exchange failed ({status}): {body}".to_string();
        for (name, value) in &[("status", "401"), ("body", "unauthorized")] {
            msg = msg.replace(&format!("{{{}}}", name), value);
        }
        assert_eq!(msg, "Token exchange failed (401): unauthorized");
    }
}
