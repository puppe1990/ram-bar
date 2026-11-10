fn main() {
    println!("cargo:rerun-if-changed=../web");
    if std::env::var("CARGO_FEATURE_DESKTOP").is_ok() {
        tauri_build::build();
    }
}
