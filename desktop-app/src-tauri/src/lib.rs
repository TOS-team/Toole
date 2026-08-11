pub mod commands;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::DiscoveryState {
            stop_flag: Mutex::new(Arc::new(AtomicBool::new(false))),
            peers: Arc::new(Mutex::new(Vec::new())),
        })
        .manage(commands::TransferState {
            active: Mutex::new(std::collections::HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_discovery,
            commands::stop_discovery,
            commands::get_hostname,
            commands::get_device_id,
            commands::get_peers,
            commands::send_files,
            commands::cancel_transfer,
            commands::read_clipboard,
            commands::close_window,
            commands::get_file_infos,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let stop = Arc::new(AtomicBool::new(false));

            let dest_dir = dirs::download_dir()
                .unwrap_or_else(|| std::env::temp_dir())
                .join("Toolé");
            let _ = std::fs::create_dir_all(&dest_dir);

            let peers = Arc::new(Mutex::new(Vec::new()));
            let ui: Arc<dyn toole_core::UI> = Arc::new(commands::AppUI { peers, window });

            tauri::async_runtime::spawn(async move {
                if let Err(e) = toole_core::recever::start_receiver(ui, dest_dir, stop).await {
                    eprintln!("Receiver error: {e}");
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
