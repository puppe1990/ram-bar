#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use std::time::{Duration, Instant};

use ram_bar::disk::{read_disk_snapshot, DiskSnapshot};
use ram_bar::memory::{read_snapshot, MemorySnapshot, MenuCopy};
use ram_bar::panel_anchor::{
    blur_hides, monitor_for_tray, panel_origin, scale_for_tray, tray_click_opens, MonitorFrame,
    Origin, ScreenRect,
};
use ram_bar::tray_grid::{tray_grid_rgba, TRAY_GRID_SIZE};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Rect};

const TRAY_ID: &str = "ram";
const POLL: Duration = Duration::from_secs(2);
const PANEL_FALLBACK_W: f64 = 360.0;
const PANEL_FALLBACK_H: f64 = 540.0;

struct PanelGate {
    last_hidden: Mutex<Option<Instant>>,
    last_shown: Mutex<Option<Instant>>,
}

#[tauri::command]
fn memory_snapshot() -> MemorySnapshot {
    read_snapshot()
}

#[tauri::command]
fn disk_snapshot() -> DiskSnapshot {
    read_disk_snapshot()
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn hide_panel(app: AppHandle) {
    if let Some(window) = app.get_webview_window("panel") {
        let _ = window.hide();
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            memory_snapshot,
            disk_snapshot,
            quit_app,
            hide_panel
        ])
        .setup(|app| {
            // Accessory + LSUIElement: menu-bar extra, no Dock icon.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            app.manage(PanelGate {
                last_hidden: Mutex::new(None),
                last_shown: Mutex::new(None),
            });
            if let Some(window) = app.get_webview_window("panel") {
                let _ = window.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));
            }
            install_tray(app.handle())?;
            #[cfg(target_os = "macos")]
            enable_login_at_start();
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
                mark_hidden(window.app_handle());
            }
            tauri::WindowEvent::Focused(false) => {
                if blur_hides(shown_ago_ms(window.app_handle())) {
                    let _ = window.hide();
                    mark_hidden(window.app_handle());
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("tauri run failed (need src-tauri/tauri.conf.json and --features desktop)");
}

fn install_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let snap = read_snapshot();
    let copy = snap.menu_copy();
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(tray_mark_icon())
        .icon_as_template(true)
        .title(snap.tray_title())
        .tooltip(tray_tooltip(&copy))
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            // Open on mouseDown like a native extra. Up of the same click
            // lands on the extra (outside the panel) and must not toggle closed.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Down,
                rect,
                ..
            } = event
            {
                on_tray_left_click(tray.app_handle(), rect);
            }
        })
        .build(app)?;
    spawn_poll(app.clone());
    Ok(())
}

fn on_tray_left_click(app: &AppHandle, rect: Rect) {
    let Some(window) = app.get_webview_window("panel") else {
        return;
    };
    let visible = window.is_visible().unwrap_or(false);
    if !tray_click_opens(visible, hidden_ago_ms(app)) {
        let _ = window.hide();
        return;
    }
    show_panel_at_tray(&window, rect);
}

fn show_panel_at_tray(window: &tauri::WebviewWindow, tray: Rect) {
    let frames = monitor_frames(window);
    let tray_physical = physical_rect(&tray);
    let scale = scale_for_tray(tray_physical, &frames);
    let tray_rect = logical_rect(&tray, scale);
    let size = panel_logical_size(window);
    let work = work_for_tray(window, tray_physical, &frames);
    let Origin { x, y } = panel_origin(tray_rect, size.width, size.height, work);
    let _ = window.set_position(tauri::LogicalPosition::new(x, y));
    mark_shown(window.app_handle());
    let _ = window.show();
    let _ = window.set_focus();
}

fn panel_logical_size(window: &tauri::WebviewWindow) -> tauri::LogicalSize<f64> {
    let scale = window.scale_factor().unwrap_or(1.0);
    window
        .outer_size()
        .ok()
        .map(|physical| physical.to_logical::<f64>(scale))
        .unwrap_or(tauri::LogicalSize::new(PANEL_FALLBACK_W, PANEL_FALLBACK_H))
}

fn physical_rect(rect: &Rect) -> ScreenRect {
    // Tray events ship a Physical rect; scale 1 is ignored in that arm.
    let pos = rect.position.to_physical::<f64>(1.0);
    let size = rect.size.to_physical::<f64>(1.0);
    ScreenRect {
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
    }
}

fn monitor_frames(window: &tauri::WebviewWindow) -> Vec<MonitorFrame> {
    window
        .available_monitors()
        .ok()
        .into_iter()
        .flatten()
        .map(|monitor| frame_from_monitor(&monitor))
        .collect()
}

fn frame_from_monitor(monitor: &tauri::window::Monitor) -> MonitorFrame {
    let pos = monitor.position();
    let size = monitor.size();
    MonitorFrame {
        scale: monitor.scale_factor(),
        x: f64::from(pos.x),
        y: f64::from(pos.y),
        width: f64::from(size.width),
        height: f64::from(size.height),
    }
}

fn work_for_tray(
    window: &tauri::WebviewWindow,
    tray: ScreenRect,
    frames: &[MonitorFrame],
) -> ScreenRect {
    match monitor_for_tray(tray, frames) {
        Some(frame) => work_from_matching_monitor(window, frame),
        None => fallback_work(window),
    }
}

fn work_from_matching_monitor(window: &tauri::WebviewWindow, frame: MonitorFrame) -> ScreenRect {
    window
        .available_monitors()
        .ok()
        .into_iter()
        .flatten()
        .find(|monitor| same_frame(&frame_from_monitor(monitor), &frame))
        .map(|monitor| work_from_monitor(&monitor))
        .unwrap_or_else(|| fallback_work(window))
}

fn same_frame(left: &MonitorFrame, right: &MonitorFrame) -> bool {
    left.x == right.x && left.y == right.y && left.scale == right.scale
}

fn fallback_work(window: &tauri::WebviewWindow) -> ScreenRect {
    window
        .primary_monitor()
        .ok()
        .flatten()
        .map(|monitor| work_from_monitor(&monitor))
        .unwrap_or(ScreenRect::work(1440.0, 900.0))
}

fn logical_rect(rect: &Rect, scale: f64) -> ScreenRect {
    let pos = rect.position.to_logical::<f64>(scale);
    let size = rect.size.to_logical::<f64>(scale);
    ScreenRect {
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
    }
}

fn work_from_monitor(monitor: &tauri::window::Monitor) -> ScreenRect {
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let pos = area.position.to_logical::<f64>(scale);
    let size = area.size.to_logical::<f64>(scale);
    ScreenRect {
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
    }
}

fn hidden_ago_ms(app: &AppHandle) -> Option<u64> {
    let gate = app.try_state::<PanelGate>()?;
    let stamp = *gate.last_hidden.lock().ok()?;
    Some(stamp?.elapsed().as_millis() as u64)
}

fn shown_ago_ms(app: &AppHandle) -> Option<u64> {
    let gate = app.try_state::<PanelGate>()?;
    let stamp = *gate.last_shown.lock().ok()?;
    Some(stamp?.elapsed().as_millis() as u64)
}

fn mark_hidden(app: &AppHandle) {
    if let Some(gate) = app.try_state::<PanelGate>() {
        if let Ok(mut stamp) = gate.last_hidden.lock() {
            *stamp = Some(Instant::now());
        }
    }
}

fn mark_shown(app: &AppHandle) {
    if let Some(gate) = app.try_state::<PanelGate>() {
        if let Ok(mut stamp) = gate.last_shown.lock() {
            *stamp = Some(Instant::now());
        }
    }
}

fn spawn_poll(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last_title = String::new();
        loop {
            let snap = read_snapshot();
            let title = snap.tray_title();
            // set_title rebuilds NSStatusItem; skip when unchanged.
            if title != last_title {
                if let Some(tray) = app.tray_by_id(TRAY_ID) {
                    let _ = tray.set_title(Some(snap.tray_title()));
                    let _ = tray.set_tooltip(Some(tray_tooltip(&snap.menu_copy())));
                }
                last_title = title;
            }
            let _ = app.emit("memory", &snap);
            let _ = app.emit("disk", &read_disk_snapshot());
            std::thread::sleep(POLL);
        }
    });
}

fn tray_mark_icon() -> tauri::image::Image<'static> {
    tauri::image::Image::new_owned(tray_grid_rgba(), TRAY_GRID_SIZE, TRAY_GRID_SIZE)
}

fn tray_tooltip(copy: &MenuCopy) -> String {
    format!("{} · {}", copy.used, copy.total)
}

/// Writes ~/Library/LaunchAgents only when this process is the bundled .app.
/// `tauri dev` debug bins skip this so they do not point login at target/debug.
#[cfg(target_os = "macos")]
fn enable_login_at_start() {
    if let Err(err) = write_login_agent_for_bundle() {
        eprintln!(
            "login agent write failed (received {err}; expected writable ~/Library/LaunchAgents)"
        );
    }
}

#[cfg(target_os = "macos")]
fn write_login_agent_for_bundle() -> Result<(), Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()?;
    let Some(program) = ram_bar::login_agent::bundle_macos_executable(&exe) else {
        return Ok(());
    };
    let program = program.to_str().ok_or_else(|| {
        format!(
            "login program path is not utf-8: received {}, expected unicode",
            program.display()
        )
    })?;
    let dir = ram_bar::login_agent::default_agents_dir()?;
    ram_bar::login_agent::write_login_agent(&dir, program)?;
    Ok(())
}
