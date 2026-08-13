// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // contourne le fond blanc de WebKitGTK sur NVIDIA (renderer DMABUF), see tauri-apps/tauri#9394
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    // sous AppImage/AppDir (APPIMAGE/APPDIR posés par le runtime AppRun), l'app
    // tourne avec les libs embarquées (built sur Ubuntu 22.04, GLib 2.72) : les
    // modules GIO système (gvfs…) exigent des symboles de GLib >= 2.76 et font
    // crasher WebKitWebProcess → la fenêtre ne s'ouvre jamais. Je neutralise ces
    // modules dans ce cas uniquement ; le .deb/.rpm utilise les libs système.
    #[cfg(target_os = "linux")]
    if std::env::var_os("APPIMAGE").is_some() || std::env::var_os("APPDIR").is_some() {
        std::env::set_var("GIO_MODULE_DIR", "/dev/null");
    }
    app_lib::run();
}
