// point d'entrée de l'application Tauri
pub mod commands;
use tauri::{Emitter, Manager};

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
            app.handle().plugin(tauri_plugin_dialog::init())?;
            Ok(())
        })
        // quand la fenetre se ferme j'arrete le discovery
        // quand on depose des fichiers je les envoie au frontend
        .on_window_event(|window, event| {
            match &event {
                tauri::WindowEvent::CloseRequested { .. } => {
                    let state: tauri::State<'_, commands::DiscoveryState> = window.state();
                    let flag = state.stop_flag.lock().unwrap();
                    flag.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                tauri::WindowEvent::DragDrop(drag) => {
                    if let tauri::DragDropEvent::Drop { paths, .. } = drag {
                        let file_paths: Vec<String> = paths
                            .iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();
                        let _ = window.emit("dropped-files", file_paths);
                    }
                }
                _ => {}
            }
        })
        // j'enregistre toutes les commandes Tauri
        .invoke_handler(tauri::generate_handler![
            commands::start_discovery,
            commands::stop_discovery,
            commands::get_hostname,
            commands::get_peers,
            commands::read_clipboard,
            commands::close_window,
            commands::get_file_sizes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
