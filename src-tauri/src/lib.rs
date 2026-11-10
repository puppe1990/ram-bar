//! Formatters live here so Ubuntu CI can `cargo test --lib --no-default-features`
//! without linking GTK/Tauri (`desktop` feature).
pub mod app_spend;
pub mod disk;
pub mod login_agent;
pub mod memory;
pub mod panel_anchor;
pub mod tray_grid;
