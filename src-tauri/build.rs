fn main() {
    println!("cargo:rerun-if-env-changed=APTABASE_APP_KEY");
    tauri_build::build()
}
