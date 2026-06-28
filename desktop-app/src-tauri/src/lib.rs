// point d'entrée de l'application Tauri
pub mod commands;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // j'initialise l'état partagé du discovery
        .manage(commands::DiscoveryState {
            stop_flag: std::sync::Mutex::new(std::sync::Arc::new(
                std::sync::atomic::AtomicBool::new(false),
            )),
            peers: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        })
        // j'ajoute le plugin de log en mode debug
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        // quand la fenetre se ferme j'arrete le discovery
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let state: tauri::State<'_, commands::DiscoveryState> = window.state();
                let flag = state.stop_flag.lock().unwrap();
                flag.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        })
        // j'enregistre toutes les commandes Tauri
        .invoke_handler(tauri::generate_handler![
            commands::start_discovery,
            commands::stop_discovery,
            commands::get_hostname,
            commands::get_peers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
