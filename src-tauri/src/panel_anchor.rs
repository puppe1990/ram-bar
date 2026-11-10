/// Gap in logical pixels between the extra and the hanging panel.
pub const PANEL_GAP: f64 = 4.0;
/// Ignore a tray click that is the same gesture as the panel blur.
pub const TRAY_REOPEN_MS: u64 = 400;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ScreenRect {
    pub fn work(width: f64, height: f64) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Origin {
    pub x: f64,
    pub y: f64,
}

/// Physical bounds + scale of one display. Used to convert the extra
/// rect with the extra's monitor, not the panel window's.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorFrame {
    pub scale: f64,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Top-left origin of a panel hung under a menu-bar extra (y down).
/// Centers on the extra, then clamps into the work area.
///
/// ```
/// let tray = ram_bar::panel_anchor::ScreenRect { x: 1000.0, y: 0.0, width: 80.0, height: 24.0 };
/// let origin = ram_bar::panel_anchor::panel_origin(tray, 360.0, 520.0, ram_bar::panel_anchor::ScreenRect::work(1440.0, 900.0));
/// assert_eq!(origin.x, 860.0);
/// ```
pub fn panel_origin(tray: ScreenRect, panel_w: f64, panel_h: f64, work: ScreenRect) -> Origin {
    let max_x = work.x + work.width - panel_w;
    let max_y = work.y + work.height - panel_h;
    let centered = tray.x + tray.width / 2.0 - panel_w / 2.0;
    let below = tray.y + tray.height + PANEL_GAP;
    Origin {
        x: clamp(centered, work.x, max_x),
        y: clamp(below, work.y, max_y),
    }
}

/// Left-click on the extra opens the panel unless it is already open
/// or this click is what just dismissed it (blur).
pub fn tray_click_opens(panel_visible: bool, hidden_ago_ms: Option<u64>) -> bool {
    if panel_visible {
        return false;
    }
    match hidden_ago_ms {
        Some(ago) if ago < TRAY_REOPEN_MS => false,
        _ => true,
    }
}

/// The mouseUp of the opening click lands on the extra, outside the panel,
/// and would otherwise blur-dismiss immediately.
pub fn blur_hides(shown_ago_ms: Option<u64>) -> bool {
    match shown_ago_ms {
        Some(ago) if ago < TRAY_REOPEN_MS => false,
        _ => true,
    }
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if max < min {
        return min;
    }
    value.max(min).min(max)
}

impl MonitorFrame {
    fn contains(self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// Scale of the monitor that contains the extra. The panel window may
/// sit on another display; using that window's scale misplaces origin.
///
/// ```
/// let tray = ram_bar::panel_anchor::ScreenRect { x: 2238.0, y: 0.0, width: 222.0, height: 76.0 };
/// let laptop = ram_bar::panel_anchor::MonitorFrame { scale: 2.0, x: 0.0, y: 0.0, width: 3420.0, height: 2214.0 };
/// assert_eq!(ram_bar::panel_anchor::scale_for_tray(tray, &[laptop]), 2.0);
/// ```
pub fn scale_for_tray(tray: ScreenRect, monitors: &[MonitorFrame]) -> f64 {
    monitor_for_tray(tray, monitors)
        .map(|frame| frame.scale)
        .unwrap_or(1.0)
}

pub fn monitor_for_tray(tray: ScreenRect, monitors: &[MonitorFrame]) -> Option<MonitorFrame> {
    let cx = tray.x + tray.width / 2.0;
    let cy = tray.y + tray.height / 2.0;
    monitors
        .iter()
        .copied()
        .find(|frame| frame.contains(cx, cy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    fn fake_tray() -> ScreenRect {
        ScreenRect {
            x: (0.0..800.0).fake(),
            y: (0.0..40.0).fake(),
            width: (40.0..160.0).fake(),
            height: (22.0..32.0).fake(),
        }
    }

    #[test]
    fn panel_centers_on_extra_with_gap() {
        let tray = ScreenRect {
            x: 1000.0,
            y: 0.0,
            width: 80.0,
            height: 24.0,
        };
        let origin = panel_origin(tray, 360.0, 520.0, ScreenRect::work(1440.0, 900.0));
        assert_eq!(origin.x, 860.0);
        assert_eq!(origin.y, 24.0 + PANEL_GAP);
    }

    #[test]
    fn panel_clamps_to_right_edge() {
        let tray = ScreenRect {
            x: 1400.0,
            y: 0.0,
            width: 80.0,
            height: 24.0,
        };
        let origin = panel_origin(tray, 360.0, 520.0, ScreenRect::work(1440.0, 900.0));
        assert_eq!(origin.x, 1080.0);
    }

    #[test]
    fn panel_clamps_to_left_edge() {
        let tray = ScreenRect {
            x: 0.0,
            y: 0.0,
            width: 40.0,
            height: 24.0,
        };
        let origin = panel_origin(tray, 360.0, 520.0, ScreenRect::work(1440.0, 900.0));
        assert_eq!(origin.x, 0.0);
    }

    #[test]
    fn random_origin_fits_work_area() {
        let tray = fake_tray();
        let panel_w: f64 = (280.0..400.0).fake();
        let panel_h: f64 = (400.0..600.0).fake();
        let work = ScreenRect::work(1440.0, 900.0);
        let origin = panel_origin(tray, panel_w, panel_h, work);
        assert!(origin.x >= work.x);
        assert!(origin.y >= work.y);
        assert!(origin.x + panel_w <= work.x + work.width + 1e-9);
        assert!(origin.y + panel_h <= work.y + work.height + 1e-9);
    }

    #[test]
    fn visible_panel_click_closes() {
        assert!(!tray_click_opens(true, None));
    }

    #[test]
    fn click_right_after_blur_does_not_reopen() {
        let ago = (0u64..TRAY_REOPEN_MS).fake();
        assert!(!tray_click_opens(false, Some(ago)));
    }

    #[test]
    fn click_after_threshold_opens() {
        assert!(tray_click_opens(false, Some(TRAY_REOPEN_MS)));
        assert!(tray_click_opens(false, None));
    }

    #[test]
    fn opening_click_blur_does_not_hide() {
        let ago = (0u64..TRAY_REOPEN_MS).fake();
        assert!(!blur_hides(Some(ago)));
    }

    #[test]
    fn later_blur_hides() {
        assert!(blur_hides(Some(TRAY_REOPEN_MS)));
        assert!(blur_hides(None));
    }

    fn laptop_2x() -> MonitorFrame {
        MonitorFrame {
            scale: 2.0,
            x: 0.0,
            y: 0.0,
            width: 3420.0,
            height: 2214.0,
        }
    }

    fn dell_1x() -> MonitorFrame {
        MonitorFrame {
            scale: 1.0,
            x: 3420.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        }
    }

    #[test]
    fn tray_scale_follows_2x_monitor_not_panel_window() {
        let tray = ScreenRect {
            x: 2238.0,
            y: 0.0,
            width: 222.0,
            height: 76.0,
        };
        assert_eq!(scale_for_tray(tray, &[dell_1x(), laptop_2x()]), 2.0);
    }

    #[test]
    fn tray_scale_on_1x_monitor_is_1() {
        let tray = ScreenRect {
            x: 4747.0,
            y: 0.0,
            width: 111.0,
            height: 24.0,
        };
        assert_eq!(scale_for_tray(tray, &[laptop_2x(), dell_1x()]), 1.0);
    }

    #[test]
    fn tray_scale_without_monitors_is_1() {
        let tray = fake_tray();
        assert_eq!(scale_for_tray(tray, &[]), 1.0);
    }
}
