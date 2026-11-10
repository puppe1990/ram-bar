use serde::Serialize;

const SI_GB: f64 = 1_000_000_000.0;
const SI_TB: f64 = 1_000_000_000_000.0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskVolume {
    pub name: String,
    pub mount: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSnapshot {
    pub name: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub volumes: Vec<DiskVolume>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiskCopy {
    pub used: String,
    pub free: String,
    pub total: String,
    pub usage: String,
}

const SKIP_MOUNTS: &[&str] = &[
    "/dev",
    "/System/Volumes/Preboot",
    "/System/Volumes/VM",
    "/System/Volumes/Update",
    "/System/Volumes/iSCPreboot",
    "/System/Volumes/Hardware",
    "/System/Volumes/Data",
];

/// Disk size: SI TB from 1e12, otherwise GB. Mirrored in `web/js/format.mjs`.
///
/// ```
/// assert_eq!(ram_bar::disk::format_disk(1_000_000_000_000), "1,0 TB");
/// ```
pub fn format_disk(bytes: u64) -> String {
    if bytes as f64 >= SI_TB {
        return format!("{:.1} TB", bytes as f64 / SI_TB).replace('.', ",");
    }
    format!("{:.1} GB", bytes as f64 / SI_GB).replace('.', ",")
}

/// Keep the boot volume and /Volumes. Drop macOS system slices (incl. Data).
///
/// ```
/// assert!(!ram_bar::disk::keep_disk_mount("/System/Volumes/Preboot"));
/// ```
pub fn keep_disk_mount(mount: &str) -> bool {
    let path = mount.trim();
    if path.is_empty() {
        return false;
    }
    !SKIP_MOUNTS
        .iter()
        .any(|skip| path == *skip || path.starts_with(&format!("{skip}/")))
}

/// Drop system slices and empty disks, add used, sort fullest first.
pub fn rank_disk_volumes(
    rows: impl IntoIterator<Item = (String, String, u64, u64)>,
) -> Vec<DiskVolume> {
    let mut ranked: Vec<DiskVolume> = rows.into_iter().filter_map(volume_from_row).collect();
    ranked.sort_by(|left, right| {
        right
            .used_bytes
            .cmp(&left.used_bytes)
            .then_with(|| left.name.cmp(&right.name))
    });
    collapse_disk_twins(ranked)
}

fn collapse_disk_twins(volumes: Vec<DiskVolume>) -> Vec<DiskVolume> {
    let mut kept: Vec<DiskVolume> = Vec::new();
    for volume in volumes {
        match kept.iter().position(|row| disk_twin(row, &volume)) {
            Some(index) if volume.mount == "/" => kept[index] = volume,
            Some(_) => {}
            None => kept.push(volume),
        }
    }
    kept
}

fn disk_twin(left: &DiskVolume, right: &DiskVolume) -> bool {
    left.name == right.name
        && left.total_bytes == right.total_bytes
        && left.used_bytes == right.used_bytes
}

fn volume_from_row(
    (name, mount, total, available): (String, String, u64, u64),
) -> Option<DiskVolume> {
    let name = name.trim();
    let mount = mount.trim();
    if !keep_disk_mount(mount) || total == 0 {
        return None;
    }
    let label = if name.is_empty() { mount } else { name };
    let available = available.min(total);
    Some(DiskVolume {
        name: label.to_string(),
        mount: mount.to_string(),
        total_bytes: total,
        available_bytes: available,
        used_bytes: total.saturating_sub(available),
    })
}

/// Boot volume (`/`) when present, otherwise the largest disk.
pub fn pick_primary_disk(volumes: &[DiskVolume]) -> DiskVolume {
    if let Some(root) = volumes.iter().find(|row| row.mount == "/") {
        return root.clone();
    }
    volumes
        .iter()
        .max_by_key(|row| row.total_bytes)
        .cloned()
        .unwrap_or_else(empty_volume)
}

fn empty_volume() -> DiskVolume {
    DiskVolume {
        name: String::new(),
        mount: String::new(),
        total_bytes: 0,
        available_bytes: 0,
        used_bytes: 0,
    }
}

fn usage_percent(used: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    let pct = ((used as f64 / total as f64) * 100.0).round();
    pct.clamp(0.0, 100.0) as u8
}

impl DiskSnapshot {
    pub fn disk_copy(&self) -> DiskCopy {
        DiskCopy {
            used: format!("Usado {}", format_disk(self.used_bytes)),
            free: format!("Livre {}", format_disk(self.available_bytes)),
            total: format!("Total {}", format_disk(self.total_bytes)),
            usage: format!(
                "{}% em uso",
                usage_percent(self.used_bytes, self.total_bytes)
            ),
        }
    }
}

/// Wraps sysinfo Disks. Primary is `/` when present.
pub fn read_disk_snapshot() -> DiskSnapshot {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    snapshot_from_rows(disks.list().iter().map(|disk| {
        (
            disk.name().to_string_lossy().into_owned(),
            disk.mount_point().to_string_lossy().into_owned(),
            disk.total_space(),
            disk.available_space(),
        )
    }))
}

fn snapshot_from_rows(rows: impl IntoIterator<Item = (String, String, u64, u64)>) -> DiskSnapshot {
    let volumes = rank_disk_volumes(rows);
    let primary = pick_primary_disk(&volumes);
    DiskSnapshot {
        name: primary.name,
        total_bytes: primary.total_bytes,
        used_bytes: primary.used_bytes,
        available_bytes: primary.available_bytes,
        volumes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    fn fake_bytes() -> u64 {
        (100_000_000u64..2_000_000_000_000u64).fake()
    }

    #[test]
    fn one_tb_is_1_0_tb() {
        assert_eq!(format_disk(1_000_000_000_000), "1,0 TB");
    }

    #[test]
    fn below_one_tb_stays_gb() {
        let bytes: u64 = (1_000_000_000u64..999_000_000_000u64).fake();
        let expected = format!("{:.1} GB", bytes as f64 / SI_GB).replace('.', ",");
        assert_eq!(format_disk(bytes), expected);
    }

    #[test]
    fn half_tb_is_gb() {
        assert_eq!(format_disk(500_000_000_000), "500,0 GB");
    }

    #[test]
    fn root_and_volumes_pass() {
        assert!(keep_disk_mount("/"));
        let volume = format!("/Volumes/{}", (1u32..9_000u32).fake::<u32>());
        assert!(keep_disk_mount(&volume));
    }

    #[test]
    fn macos_system_slices_are_dropped() {
        assert!(!keep_disk_mount("/System/Volumes/Preboot"));
        assert!(!keep_disk_mount("/System/Volumes/VM"));
        assert!(!keep_disk_mount("/System/Volumes/Update"));
        assert!(!keep_disk_mount("/System/Volumes/Data"));
        assert!(!keep_disk_mount("/dev"));
        assert!(!keep_disk_mount(""));
    }

    #[test]
    fn rank_skips_preboot_and_sorts_fullest_first() {
        let hog = format!("hog-{}", (1u32..9_000u32).fake::<u32>());
        let mid = format!("mid-{}", (1u32..9_000u32).fake::<u32>());
        let ranked = rank_disk_volumes([
            (
                mid.clone(),
                "/Volumes/Mid".into(),
                400_000_000_000,
                200_000_000_000,
            ),
            (hog.clone(), "/".into(), 1_000_000_000_000, 200_000_000_000),
            (
                "Preboot".into(),
                "/System/Volumes/Preboot".into(),
                1_000_000_000,
                100_000_000,
            ),
        ]);
        assert_eq!(ranked[0].name, hog);
        assert_eq!(ranked[0].used_bytes, 800_000_000_000);
        assert_eq!(ranked[1].name, mid);
        assert!(ranked
            .iter()
            .all(|row| row.mount != "/System/Volumes/Preboot"));
    }

    #[test]
    fn primary_prefers_root() {
        let root_name = format!("root-{}", (1u32..9_000u32).fake::<u32>());
        let volumes = rank_disk_volumes([
            (
                "extra".into(),
                "/Volumes/Extra".into(),
                4_000_000_000_000,
                1,
            ),
            (
                root_name.clone(),
                "/".into(),
                500_000_000_000,
                100_000_000_000,
            ),
        ]);
        let primary = pick_primary_disk(&volumes);
        assert_eq!(primary.mount, "/");
        assert_eq!(primary.name, root_name);
    }

    #[test]
    fn primary_without_root_is_largest_total() {
        let volumes = rank_disk_volumes([
            ("small".into(), "/Volumes/A".into(), 200_000_000_000, 10),
            ("big".into(), "/Volumes/B".into(), 800_000_000_000, 10),
        ]);
        assert_eq!(pick_primary_disk(&volumes).name, "big");
    }

    #[test]
    fn disk_copy_uses_available_as_livre() {
        let snap = DiskSnapshot {
            name: "Macintosh HD".into(),
            total_bytes: 1_000_000_000_000,
            used_bytes: 250_000_000_000,
            available_bytes: 750_000_000_000,
            volumes: Vec::new(),
        };
        let copy = snap.disk_copy();
        assert_eq!(copy.used, "Usado 250,0 GB");
        assert_eq!(copy.free, "Livre 750,0 GB");
        assert_eq!(copy.total, "Total 1,0 TB");
        assert_eq!(copy.usage, "25% em uso");
    }

    #[test]
    fn apfs_twins_with_same_name_and_size_collapse_to_root() {
        let name = format!("vol-{}", (1u32..9_000u32).fake::<u32>());
        let ranked = rank_disk_volumes([
            (
                name.clone(),
                "/System/Volumes/Data".into(),
                245_000_000_000,
                5_000_000_000,
            ),
            (name.clone(), "/".into(), 245_000_000_000, 5_000_000_000),
        ]);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].mount, "/");
        assert_eq!(ranked[0].name, name);
    }

    #[test]
    fn same_name_different_size_stays_two_rows() {
        let name = format!("vol-{}", (1u32..9_000u32).fake::<u32>());
        let ranked = rank_disk_volumes([
            (name.clone(), "/".into(), 245_000_000_000, 5_000_000_000),
            (
                name,
                "/Volumes/Extra".into(),
                1_000_000_000_000,
                100_000_000_000,
            ),
        ]);
        assert_eq!(ranked.len(), 2);
    }

    #[test]
    fn random_bytes_match_si_gold() {
        let bytes = fake_bytes();
        if bytes as f64 >= SI_TB {
            let expected = format!("{:.1} TB", bytes as f64 / SI_TB).replace('.', ",");
            assert_eq!(format_disk(bytes), expected);
        } else {
            let expected = format!("{:.1} GB", bytes as f64 / SI_GB).replace('.', ",");
            assert_eq!(format_disk(bytes), expected);
        }
    }

    #[test]
    fn live_disk_has_nonzero_total() {
        let snap = read_disk_snapshot();
        assert!(
            snap.total_bytes > 0,
            "disk total_bytes should be > 0, got {}",
            snap.total_bytes
        );
        assert!(
            snap.used_bytes <= snap.total_bytes,
            "used {} larger than total {}",
            snap.used_bytes,
            snap.total_bytes
        );
    }
}
