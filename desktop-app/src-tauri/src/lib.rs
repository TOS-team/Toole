pub mod commands;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .manage(commands::DiscoveryState {
            stop_flag: Mutex::new(Arc::new(AtomicBool::new(false))),
            handle: Mutex::new(None),
            peers: Arc::new(Mutex::new(Vec::new())),
        })
        .manage(commands::TransferState {
            active: Arc::new(Mutex::new(std::collections::HashMap::new())),
            decisions: Arc::new(toole_core::transfer::DecisionBoard::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_discovery,
            commands::stop_discovery,
            commands::get_device_id,
            commands::get_peers,
            commands::send_files,
            commands::cancel_transfer,
            commands::respond_transfer,
            commands::read_clipboard,
            commands::get_file_infos,
        ])
        .setup(|app| {
            // j'enregistre le plugin de mise à jour auto (desktop only) :
            // il interroge latest.json sur GitHub et installe les updates signées
            #[cfg(desktop)]
            let _ = app
                .handle()
                .plugin(tauri_plugin_updater::Builder::new().build());

            let window = match app.get_webview_window("main") {
                Some(w) => w,
                None => {
                    eprintln!("Main window not found; aborting startup");
                    return Ok(());
                }
            };
            let stop = Arc::new(AtomicBool::new(false));

            let dest_dir = dirs::download_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("Toolé");
            let _ = std::fs::create_dir_all(&dest_dir);

            let peers = Arc::new(Mutex::new(Vec::new()));
            let ui: Arc<dyn toole_core::UI> = Arc::new(commands::AppUI { peers, window });

            let transfer_state: tauri::State<'_, commands::TransferState> = app.state();
            let decisions = transfer_state.decisions.clone();
            let registry: Arc<dyn toole_core::TransferRegistry> =
                Arc::new(commands::TransferRegistryHandle {
                    active: transfer_state.active.clone(),
                });

            tauri::async_runtime::spawn(async move {
                if let Err(e) = toole_core::receiver::start_receiver(
                    ui,
                    dest_dir,
                    stop,
                    decisions,
                    registry,
                )
                .await
                {
                    eprintln!("Receiver error: {e}");
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| { eprintln!("error while running tauri application: {e}"); });
}
