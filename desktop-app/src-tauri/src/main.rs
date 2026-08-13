// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // contourne le fond blanc de WebKitGTK sur NVIDIA (renderer DMABUF), see tauri-apps/tauri#9394
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    app_lib::run();
}
