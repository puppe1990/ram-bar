use serde::Serialize;
use sysinfo::{
    MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind,
};

use crate::app_spend::{app_label, rank_app_spenders, AppSpend};

/// SI gigabyte (10^9). Matches Activity Monitor, not binary GiB.
const SI_GB: f64 = 1_000_000_000.0;
const SI_MB: f64 = 1_000_000.0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySnapshot {
    pub total_bytes: u64,
    pub used_bytes: u64,
    /// sysinfo available; may be 0 on Apple. Menu Livre uses total−used.
    pub available_bytes: u64,
    pub apps: Vec<AppSpend>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MenuCopy {
    pub used: String,
    pub free: String,
    pub total: String,
    pub usage: String,
    pub details: String,
    pub quit: String,
}

/// Format bytes as Brazilian SI GB (one decimal, comma).
/// Mirrored in `web/js/format.mjs` — keep goldens in both suites.
///
/// ```
/// assert_eq!(ram_bar::memory::format_gb(12_400_000_000), "12,4 GB");
/// ```
pub fn format_gb(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / SI_GB).replace('.', ",")
}

/// Per-app RAM for the details list. GB from 1e9 up; MB below.
///
/// ```
/// assert_eq!(ram_bar::memory::format_spend(320_000_000), "320 MB");
/// ```
pub fn format_spend(bytes: u64) -> String {
    if bytes as f64 >= SI_GB {
        return format_gb(bytes);
    }
    let mb = bytes as f64 / SI_MB;
    if mb >= 10.0 {
        return format!("{} MB", mb.round() as u64);
    }
    format!("{:.1} MB", mb).replace('.', ",")
}

impl MemorySnapshot {
    /// Used/total as a clamped integer percent (`8 GB of 16 GB → 50`).
    pub fn usage_percent(&self) -> u8 {
        if self.total_bytes == 0 {
            return 0;
        }
        let pct = ((self.used_bytes as f64 / self.total_bytes as f64) * 100.0).round();
        pct.clamp(0.0, 100.0) as u8
    }

    /// Menu-bar title: used GB, two spaces, integer percent (`"8,0 GB  50%"`).
    pub fn tray_title(&self) -> String {
        format!("{}  {}%", format_gb(self.used_bytes), self.usage_percent())
    }

    /// Portuguese tray copy. Livre is total−used: sysinfo `available_memory`
    /// can be 0 on Apple when compressor pages exceed free+inactive+purgeable.
    pub fn menu_copy(&self) -> MenuCopy {
        let free_bytes = self.total_bytes.saturating_sub(self.used_bytes);
        MenuCopy {
            used: format!("Usado {}", format_gb(self.used_bytes)),
            free: format!("Livre {}", format_gb(free_bytes)),
            total: format!("Total {}", format_gb(self.total_bytes)),
            usage: format!("{}% em uso", self.usage_percent()),
            details: "Detalhes".to_string(),
            quit: "Sair".to_string(),
        }
    }
}

fn process_refresh() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_memory()
        .with_exe(UpdateKind::OnlyIfNotSet)
}

/// Wraps sysinfo. Apple used = active+wired+compressor+speculative.
pub fn read_snapshot() -> MemorySnapshot {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(process_refresh()),
    );
    sys.refresh_memory();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, process_refresh());
    MemorySnapshot {
        total_bytes: sys.total_memory(),
        used_bytes: sys.used_memory(),
        available_bytes: sys.available_memory(),
        apps: app_spenders_from(&sys),
    }
}

fn app_spenders_from(sys: &System) -> Vec<AppSpend> {
    rank_app_spenders(sys.processes().values().map(|proc| {
        let name = proc.name().to_string_lossy().into_owned();
        (app_label(proc.exe(), &name), proc.memory())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    fn fake_snapshot() -> MemorySnapshot {
        let total: u64 = (4_000_000_000u64..128_000_000_000u64).fake();
        let used: u64 = (0u64..=total).fake();
        MemorySnapshot {
            total_bytes: total,
            used_bytes: used,
            available_bytes: total.saturating_sub(used),
            apps: Vec::new(),
        }
    }

    #[test]
    fn format_gb_uses_si_and_comma() {
        let bytes: u64 = (100_000_000u64..128_000_000_000u64).fake();
        let expected = format!("{:.1} GB", bytes as f64 / 1_000_000_000.0).replace('.', ",");
        assert_eq!(format_gb(bytes), expected);
    }

    #[test]
    fn golden_12_4_gb() {
        assert_eq!(format_gb(12_400_000_000), "12,4 GB");
    }

    #[test]
    fn zero_is_0_0_gb() {
        assert_eq!(format_gb(0), "0,0 GB");
    }

    #[test]
    fn usage_percent_is_rounded_and_clamped() {
        let snap = fake_snapshot();
        let expected = ((snap.used_bytes as f64 / snap.total_bytes as f64) * 100.0).round() as i64;
        let expected = expected.clamp(0, 100) as u8;
        assert_eq!(snap.usage_percent(), expected);
    }

    #[test]
    fn half_ram_is_50_percent() {
        let total: u64 = (8u64..64u64).fake::<u64>() * 1_000_000_000;
        let snap = MemorySnapshot {
            total_bytes: total,
            used_bytes: total / 2,
            available_bytes: total / 2,
            apps: Vec::new(),
        };
        assert_eq!(snap.usage_percent(), 50);
    }

    #[test]
    fn total_zero_is_0_percent() {
        let used: u64 = (100_000_000u64..8_000_000_000u64).fake();
        let snap = MemorySnapshot {
            total_bytes: 0,
            used_bytes: used,
            available_bytes: 0,
            apps: Vec::new(),
        };
        assert_eq!(snap.usage_percent(), 0);
    }

    #[test]
    fn used_above_total_saturates_at_100() {
        let total: u64 = (1_000_000_000u64..8_000_000_000u64).fake();
        let extra: u64 = (1u64..4_000_000_000u64).fake();
        let snap = MemorySnapshot {
            total_bytes: total,
            used_bytes: total + extra,
            available_bytes: 0,
            apps: Vec::new(),
        };
        assert_eq!(snap.usage_percent(), 100);
    }

    #[test]
    fn tray_title_joins_used_and_percent() {
        let snap = fake_snapshot();
        assert_eq!(
            snap.tray_title(),
            format!("{}  {}%", format_gb(snap.used_bytes), snap.usage_percent())
        );
    }

    #[test]
    fn golden_tray_title() {
        let snap = MemorySnapshot {
            total_bytes: 16_000_000_000,
            used_bytes: 8_000_000_000,
            available_bytes: 8_000_000_000,
            apps: Vec::new(),
        };
        assert_eq!(snap.tray_title(), "8,0 GB  50%");
    }

    #[test]
    fn menu_copy_is_portuguese() {
        let snap = fake_snapshot();
        let copy = snap.menu_copy();
        assert_eq!(copy.used, format!("Usado {}", format_gb(snap.used_bytes)));
        assert_eq!(
            copy.free,
            format!(
                "Livre {}",
                format_gb(snap.total_bytes.saturating_sub(snap.used_bytes))
            )
        );
        assert_eq!(copy.total, format!("Total {}", format_gb(snap.total_bytes)));
        assert_eq!(copy.usage, format!("{}% em uso", snap.usage_percent()));
        assert_eq!(copy.details, "Detalhes");
        assert_eq!(copy.quit, "Sair");
    }

    #[test]
    fn livre_uses_total_minus_used_when_available_is_zero() {
        let snap = MemorySnapshot {
            total_bytes: 8_600_000_000,
            used_bytes: 7_200_000_000,
            available_bytes: 0,
            apps: Vec::new(),
        };
        assert_eq!(snap.menu_copy().free, "Livre 1,4 GB");
    }

    #[test]
    fn format_spend_uses_gb_from_one_billion() {
        assert_eq!(format_spend(2_400_000_000), "2,4 GB");
    }

    #[test]
    fn format_spend_uses_integer_mb_between_10mb_and_1gb() {
        assert_eq!(format_spend(320_000_000), "320 MB");
    }

    #[test]
    fn format_spend_uses_comma_mb_below_10mb() {
        assert_eq!(format_spend(5_400_000), "5,4 MB");
    }

    #[test]
    fn live_snapshot_has_nonzero_total() {
        let snap = read_snapshot();
        assert!(
            snap.total_bytes > 0,
            "total_bytes should be > 0, got {}",
            snap.total_bytes
        );
        assert!(
            snap.used_bytes <= snap.total_bytes.saturating_mul(2),
            "used_bytes {} is implausibly larger than total {}",
            snap.used_bytes,
            snap.total_bytes
        );
    }

    #[test]
    fn live_snapshot_lists_ranked_apps() {
        let snap = read_snapshot();
        assert!(
            !snap.apps.is_empty(),
            "expected at least one process with RAM"
        );
        for pair in snap.apps.windows(2) {
            assert!(
                pair[0].bytes >= pair[1].bytes,
                "apps must be sorted biggest first, got {} then {}",
                pair[0].bytes,
                pair[1].bytes
            );
        }
    }
}
