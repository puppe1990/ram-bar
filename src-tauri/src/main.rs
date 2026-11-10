#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use ram_bar::memory::{read_snapshot, MemorySnapshot};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager};

const TRAY_ID: &str = "ram";
const POLL: Duration = Duration::from_secs(2);

struct StatusItems {
    used: MenuItem<tauri::Wry>,
    free: MenuItem<tauri::Wry>,
    total: MenuItem<tauri::Wry>,
    usage: MenuItem<tauri::Wry>,
}

#[tauri::command]
fn memory_snapshot() -> MemorySnapshot {
    read_snapshot()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![memory_snapshot])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            install_tray(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("tauri");
}

fn install_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let snap = read_snapshot();
    let copy = snap.menu_copy();
    let items = StatusItems {
        used: MenuItem::with_id(app, "used", &copy.used, false, None::<&str>)?,
        free: MenuItem::with_id(app, "free", &copy.free, false, None::<&str>)?,
        total: MenuItem::with_id(app, "total", &copy.total, false, None::<&str>)?,
        usage: MenuItem::with_id(app, "usage", &copy.usage, false, None::<&str>)?,
    };
    let details = MenuItem::with_id(app, "details", &copy.details, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", &copy.quit, true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &items.used,
            &items.free,
            &items.total,
            &items.usage,
            &sep,
            &details,
            &quit,
        ],
    )?;
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .title(snap.tray_title())
        .tooltip(tooltip(&snap))
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()));
    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }
    tray.build(app)?;
    spawn_poll(app.clone(), items);
    Ok(())
}

fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "quit" => app.exit(0),
        "details" => show_panel(app),
        _ => {}
    }
}

fn show_panel(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("panel") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn spawn_poll(app: AppHandle, items: StatusItems) {
    std::thread::spawn(move || {
        let mut last_title = String::new();
        loop {
            let snap = read_snapshot();
            let title = snap.tray_title();
            if title != last_title {
                apply_snapshot(&app, &items, &snap);
                last_title = title;
            } else {
                let _ = app.emit("memory", &snap);
            }
            std::thread::sleep(POLL);
        }
    });
}

fn apply_snapshot(app: &AppHandle, items: &StatusItems, snap: &MemorySnapshot) {
    let copy = snap.menu_copy();
    let _ = items.used.set_text(&copy.used);
    let _ = items.free.set_text(&copy.free);
    let _ = items.total.set_text(&copy.total);
    let _ = items.usage.set_text(&copy.usage);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_title(Some(snap.tray_title()));
        let _ = tray.set_tooltip(Some(tooltip(&snap)));
    }
    let _ = app.emit("memory", &snap);
}

fn tooltip(snap: &MemorySnapshot) -> String {
    let copy = snap.menu_copy();
    format!("{} · {}", copy.used, copy.total)
}
