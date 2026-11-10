fn main() {
    println!("cargo:rerun-if-changed=../web");
    // Ubuntu CI builds the lib without `desktop`; tauri_build needs that feature.
    if std::env::var("CARGO_FEATURE_DESKTOP").is_ok() {
        tauri_build::build();
    }
}
