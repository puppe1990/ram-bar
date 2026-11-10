use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;

pub const APP_SPEND_LIMIT: usize = 25;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppSpend {
    pub name: String,
    pub bytes: u64,
}

/// Prefer the `.app` bundle stem so Chrome Helper rolls into Google Chrome.
pub fn app_label(exe: Option<&Path>, process_name: &str) -> String {
    if let Some(path) = exe {
        if let Some(label) = bundle_stem(path) {
            return label;
        }
    }
    process_name.trim().to_string()
}

fn bundle_stem(path: &Path) -> Option<String> {
    for ancestor in path.ancestors() {
        if ancestor.extension().is_some_and(|ext| ext == "app") {
            let label = ancestor.file_stem()?.to_string_lossy();
            if !label.is_empty() {
                return Some(label.into_owned());
            }
        }
    }
    None
}

/// Group by name, drop empty/zero, sort biggest first, cap at 25.
pub fn rank_app_spenders(rows: impl IntoIterator<Item = (String, u64)>) -> Vec<AppSpend> {
    let mut totals: HashMap<String, u64> = HashMap::new();
    for (name, bytes) in rows {
        add_spend(&mut totals, name, bytes);
    }
    sort_spenders(totals)
}

fn add_spend(totals: &mut HashMap<String, u64>, name: String, bytes: u64) {
    let name = name.trim();
    if name.is_empty() || bytes == 0 {
        return;
    }
    *totals.entry(name.to_string()).or_insert(0) += bytes;
}

fn sort_spenders(totals: HashMap<String, u64>) -> Vec<AppSpend> {
    let mut ranked: Vec<AppSpend> = totals
        .into_iter()
        .map(|(name, bytes)| AppSpend { name, bytes })
        .collect();
    ranked.sort_by(|left, right| {
        right
            .bytes
            .cmp(&left.bytes)
            .then_with(|| left.name.cmp(&right.name))
    });
    ranked.truncate(APP_SPEND_LIMIT);
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use std::path::Path;

    #[test]
    fn app_label_uses_bundle_stem_over_helper_process_name() {
        let exe = Path::new(
            "/Applications/Google Chrome.app/Contents/Frameworks/Google Chrome Framework.framework/Versions/Current/Helpers/Google Chrome Helper",
        );
        assert_eq!(
            app_label(Some(exe), "Google Chrome Helper"),
            "Google Chrome"
        );
    }

    #[test]
    fn app_label_falls_back_to_process_name() {
        let name: String = format!("proc-{}", (1u32..9_000u32).fake::<u32>());
        assert_eq!(app_label(None, &name), name);
    }

    #[test]
    fn rank_groups_same_app_and_sorts_biggest_first() {
        let hog: String = format!("hog-{}", (1u32..9_000u32).fake::<u32>());
        let mid: String = format!("mid-{}", (1u32..9_000u32).fake::<u32>());
        let low: String = format!("low-{}", (1u32..9_000u32).fake::<u32>());
        let ranked = rank_app_spenders([
            (mid.clone(), 200_000_000),
            (hog.clone(), 800_000_000),
            (hog.clone(), 400_000_000),
            (low.clone(), 50_000_000),
        ]);
        assert_eq!(
            ranked
                .iter()
                .map(|row| row.name.as_str())
                .collect::<Vec<_>>(),
            vec![hog.as_str(), mid.as_str(), low.as_str()]
        );
        assert_eq!(ranked[0].bytes, 1_200_000_000);
    }

    #[test]
    fn rank_skips_blank_name_and_zero_bytes() {
        let keep: String = format!("keep-{}", (1u32..9_000u32).fake::<u32>());
        let ranked = rank_app_spenders([
            ("  ".to_string(), 500_000_000),
            (keep.clone(), 0),
            (keep.clone(), 80_000_000),
        ]);
        assert_eq!(
            ranked,
            vec![AppSpend {
                name: keep,
                bytes: 80_000_000,
            }]
        );
    }

    #[test]
    fn rank_caps_at_25() {
        let rows = (0..30u64).map(|i| (format!("app-{i}"), (30 - i) * 10_000_000));
        let ranked = rank_app_spenders(rows);
        assert_eq!(ranked.len(), 25);
        assert!(ranked[0].bytes > ranked[24].bytes);
    }

    #[test]
    fn rank_tie_breaks_by_name() {
        let ranked = rank_app_spenders([
            ("Safari".to_string(), 100_000_000),
            ("Chrome".to_string(), 100_000_000),
        ]);
        assert_eq!(
            ranked
                .iter()
                .map(|row| row.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Chrome", "Safari"]
        );
    }
}
