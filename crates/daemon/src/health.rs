//! System health monitoring using the sysinfo crate.

use sysinfo::{Disks, System};

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub disk_percent: f32,
    pub is_healthy: bool,
}

/// Collect current system health metrics.
/// Refreshes sysinfo counters synchronously (takes ~200ms on first call).
pub fn collect_health() -> HealthStatus {
    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU: global usage across all cores (sysinfo 0.30 API)
    let cpu_percent = sys.global_cpu_info().cpu_usage();

    let memory_percent = if sys.total_memory() > 0 {
        (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0
    } else {
        0.0
    };

    // Disk: use primary disk
    let disks = Disks::new_with_refreshed_list();
    let disk_percent = disks
        .list()
        .first()
        .map(|d| {
            let total = d.total_space();
            let available = d.available_space();
            if total > 0 {
                ((total - available) as f32 / total as f32) * 100.0
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);

    let is_healthy = cpu_percent < 90.0 && memory_percent < 90.0 && disk_percent < 95.0;

    HealthStatus {
        cpu_percent,
        memory_percent,
        disk_percent,
        is_healthy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_health_returns_valid_ranges() {
        let h = collect_health();
        assert!(h.cpu_percent >= 0.0 && h.cpu_percent <= 100.0,
            "cpu_percent {} should be in [0, 100]", h.cpu_percent);
        assert!(h.memory_percent >= 0.0 && h.memory_percent <= 100.0,
            "memory_percent {} should be in [0, 100]", h.memory_percent);
        assert!(h.disk_percent >= 0.0 && h.disk_percent <= 100.0,
            "disk_percent {} should be in [0, 100]", h.disk_percent);
    }

    #[test]
    fn test_is_healthy_thresholds() {
        let healthy = HealthStatus {
            cpu_percent: 50.0,
            memory_percent: 60.0,
            disk_percent: 70.0,
            is_healthy: true,
        };
        assert!(healthy.is_healthy);

        let unhealthy_cpu = HealthStatus {
            cpu_percent: 95.0,
            memory_percent: 50.0,
            disk_percent: 50.0,
            is_healthy: false,
        };
        assert!(!unhealthy_cpu.is_healthy);

        let unhealthy_disk = HealthStatus {
            cpu_percent: 10.0,
            memory_percent: 10.0,
            disk_percent: 96.0,
            is_healthy: false,
        };
        assert!(!unhealthy_disk.is_healthy);
    }
}
