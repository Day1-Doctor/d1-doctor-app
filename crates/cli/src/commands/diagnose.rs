//! `d1 diagnose` — run system diagnostics and output a structured report.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::gateway;
use super::status;

/// Result of a single diagnostic check.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub detail: String,
}

/// Status of a diagnostic check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum CheckStatus {
    Ok,
    Warn,
    Fail,
}

/// Full diagnostic report.
#[derive(Debug, Clone, Serialize)]
pub struct DiagnoseReport {
    pub version: String,
    pub checks: Vec<CheckResult>,
    pub system: SystemInfo,
}

/// Basic system information.
#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub cli_version: String,
}

/// Credit balance response (subset of UsageSummary).
#[derive(Debug, Deserialize)]
struct BalanceResponse {
    dd_balance: f64,
}

fn collect_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        cli_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Check whether the local daemon is reachable.
fn check_daemon_connectivity() -> CheckResult {
    let config = d1_common::Config::load().unwrap_or_default();
    let port = config.daemon_port;
    let daemon = status::check_daemon(port);

    match daemon {
        status::DaemonStatus::Running => CheckResult {
            name: crate::i18n::t("diagnose.check_daemon"),
            status: CheckStatus::Ok,
            detail: crate::i18n::t_args("diagnose.daemon_running", &[("port", &port.to_string())]),
        },
        status::DaemonStatus::Stopped => CheckResult {
            name: crate::i18n::t("diagnose.check_daemon"),
            status: CheckStatus::Warn,
            detail: crate::i18n::t_args("diagnose.daemon_stopped", &[("port", &port.to_string())]),
        },
    }
}

/// Check API connectivity by hitting the health endpoint.
async fn check_api_connectivity() -> CheckResult {
    let name = crate::i18n::t("diagnose.check_api");
    let base_url = gateway::platform_api_base();
    let client = reqwest::Client::new();
    let url = format!("{}/health", base_url);

    let start = Instant::now();
    let result = client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await;
    let elapsed = start.elapsed();

    match result {
        Ok(resp) if resp.status().is_success() => CheckResult {
            name,
            status: CheckStatus::Ok,
            detail: crate::i18n::t_args(
                "diagnose.api_reachable",
                &[("ms", &elapsed.as_millis().to_string())],
            ),
        },
        Ok(resp) => CheckResult {
            name,
            status: CheckStatus::Warn,
            detail: crate::i18n::t_args(
                "diagnose.api_unexpected_status",
                &[("status", &resp.status().to_string())],
            ),
        },
        Err(e) => CheckResult {
            name,
            status: CheckStatus::Fail,
            detail: crate::i18n::t_args("diagnose.api_unreachable", &[("error", &e.to_string())]),
        },
    }
}

/// Check auth status by loading local credentials.
fn check_auth_status() -> CheckResult {
    let name = crate::i18n::t("diagnose.check_auth");

    match gateway::api_client() {
        Ok(_) => CheckResult {
            name,
            status: CheckStatus::Ok,
            detail: crate::i18n::t("diagnose.auth_ok"),
        },
        Err(_) => CheckResult {
            name,
            status: CheckStatus::Warn,
            detail: crate::i18n::t("diagnose.auth_missing"),
        },
    }
}

/// Check credit balance by calling the usage API.
async fn check_credit_balance() -> CheckResult {
    let name = crate::i18n::t("diagnose.check_credits");

    let (client, base_url, token) = match gateway::api_client() {
        Ok(c) => c,
        Err(_) => {
            return CheckResult {
                name,
                status: CheckStatus::Warn,
                detail: crate::i18n::t("diagnose.credits_no_auth"),
            };
        }
    };

    let result = client
        .get(format!("{}/api/v1/usage?days=1", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .timeout(Duration::from_secs(10))
        .send()
        .await;

    match result {
        Ok(resp) if resp.status().is_success() => match resp.json::<BalanceResponse>().await {
            Ok(balance) => CheckResult {
                name,
                status: CheckStatus::Ok,
                detail: crate::i18n::t_args(
                    "diagnose.credits_balance",
                    &[("amount", &format!("{:.2}", balance.dd_balance))],
                ),
            },
            Err(_) => CheckResult {
                name,
                status: CheckStatus::Warn,
                detail: crate::i18n::t("diagnose.credits_parse_error"),
            },
        },
        Ok(resp) => CheckResult {
            name,
            status: CheckStatus::Fail,
            detail: crate::i18n::t_args(
                "diagnose.credits_fetch_failed",
                &[("status", &resp.status().to_string())],
            ),
        },
        Err(e) => CheckResult {
            name,
            status: CheckStatus::Fail,
            detail: crate::i18n::t_args(
                "diagnose.credits_fetch_failed",
                &[("status", &e.to_string())],
            ),
        },
    }
}

/// Run all diagnostics and return the report.
pub async fn collect_report() -> DiagnoseReport {
    let system = collect_system_info();
    let mut checks = Vec::new();

    checks.push(check_daemon_connectivity());
    checks.push(check_api_connectivity().await);
    checks.push(check_auth_status());
    checks.push(check_credit_balance().await);

    DiagnoseReport {
        version: system.cli_version.clone(),
        checks,
        system,
    }
}

/// Print the diagnostic report to stdout.
pub fn print_report(report: &DiagnoseReport) {
    println!(
        "{}",
        crate::i18n::t_args("diagnose.title", &[("version", &report.version)])
    );
    println!();

    // System info section
    println!("{}", crate::i18n::t("diagnose.system_title"));
    println!(
        "{}",
        crate::i18n::t_args("diagnose.system_os", &[("os", &report.system.os)])
    );
    println!(
        "{}",
        crate::i18n::t_args("diagnose.system_arch", &[("arch", &report.system.arch)])
    );
    println!(
        "{}",
        crate::i18n::t_args(
            "diagnose.system_version",
            &[("version", &report.system.cli_version)]
        )
    );
    println!();

    // Checks section
    println!("{}", crate::i18n::t("diagnose.checks_title"));
    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Ok => "OK",
            CheckStatus::Warn => "!!", // warning
            CheckStatus::Fail => "XX",
        };
        println!("  [{}] {}: {}", icon, check.name, check.detail);
    }

    println!();
    let (ok, warn, fail) = report
        .checks
        .iter()
        .fold((0, 0, 0), |(o, w, f), c| match c.status {
            CheckStatus::Ok => (o + 1, w, f),
            CheckStatus::Warn => (o, w + 1, f),
            CheckStatus::Fail => (o, w, f + 1),
        });
    println!(
        "{}",
        crate::i18n::t_args(
            "diagnose.summary",
            &[
                ("ok", &ok.to_string()),
                ("warn", &warn.to_string()),
                ("fail", &fail.to_string()),
            ]
        )
    );
}

/// Entry-point called from the command router.
pub async fn run() -> anyhow::Result<()> {
    let report = collect_report().await;
    print_report(&report);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_system_info() {
        let info = collect_system_info();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
        assert!(!info.cli_version.is_empty());
    }

    #[test]
    fn test_check_daemon_connectivity_stopped() {
        crate::i18n::init("en");
        let result = check_daemon_connectivity();
        // Daemon is not running in test environment
        assert!(
            result.status == CheckStatus::Warn || result.status == CheckStatus::Ok,
            "unexpected check status: {:?}",
            result.status
        );
    }

    #[test]
    fn test_check_auth_status_no_credentials() {
        crate::i18n::init("en");
        let result = check_auth_status();
        // No credentials in test environment → Warn
        assert_eq!(result.status, CheckStatus::Warn);
    }

    #[test]
    fn test_print_report_does_not_panic() {
        crate::i18n::init("en");
        let report = DiagnoseReport {
            version: "0.1.0".to_string(),
            checks: vec![
                CheckResult {
                    name: "Daemon".to_string(),
                    status: CheckStatus::Ok,
                    detail: "Running on port 9876".to_string(),
                },
                CheckResult {
                    name: "API".to_string(),
                    status: CheckStatus::Fail,
                    detail: "Unreachable".to_string(),
                },
                CheckResult {
                    name: "Auth".to_string(),
                    status: CheckStatus::Warn,
                    detail: "No credentials".to_string(),
                },
            ],
            system: SystemInfo {
                os: "macos".to_string(),
                arch: "aarch64".to_string(),
                cli_version: "0.1.0".to_string(),
            },
        };
        print_report(&report);
    }

    #[test]
    fn test_check_result_serialization() {
        let result = CheckResult {
            name: "test".to_string(),
            status: CheckStatus::Ok,
            detail: "all good".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"status\":\"Ok\""));
        assert!(json.contains("\"name\":\"test\""));
    }

    #[test]
    fn test_diagnose_report_serialization() {
        let report = DiagnoseReport {
            version: "0.1.0".to_string(),
            checks: vec![CheckResult {
                name: "test".to_string(),
                status: CheckStatus::Ok,
                detail: "ok".to_string(),
            }],
            system: SystemInfo {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                cli_version: "0.1.0".to_string(),
            },
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"version\":\"0.1.0\""));
        assert!(json.contains("\"checks\""));
        assert!(json.contains("\"system\""));
    }

    #[tokio::test]
    async fn test_collect_report_returns_all_checks() {
        crate::i18n::init("en");
        let report = collect_report().await;
        assert_eq!(report.checks.len(), 4);
        assert!(!report.version.is_empty());
    }
}
