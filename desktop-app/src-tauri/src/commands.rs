use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};
use toole_core::{Peer, UI};

struct TauriUI {
    app: AppHandle,
}

impl UI for TauriUI {
    fn log(&self, msg: &str) {
        let _ = self.app.emit("log", msg);
    }
    fn peer_found(&self, peer: &Peer) {
        let _ = self.app.emit("peer-found", peer);
    }
    fn peer_lost(&self, hostname: &str) {
        let _ = self
            .app
            .emit("peer-lost", serde_json::json!({"hostname":hostname}));
    }
}

pub struct DiscoveryState {
    pub stop_flag: Mutex<Arc<AtomicBool>>,
}

#[tauri::command]
pub async fn start_discovery(
    app: AppHandle,
    state: State<'_, DiscoveryState>,
) -> Result<(), String> {
    let old = state.stop_flag.lock().unwrap();
    old.store(true, Ordering::Relaxed);
    drop(old);

    let stop = Arc::new(AtomicBool::new(false));
    *state.stop_flag.lock().unwrap() = stop.clone();

    let local_ip = toole_core::utils::local_ip();
    let ui = Arc::new(TauriUI { app });

    tokio::spawn(async move {
        if let Err(e) = toole_core::discovery::start_discovery(local_ip, stop, ui).await {
            eprintln!("Discovery task error: {e}");
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_discovery(state: State<'_, DiscoveryState>) -> Result<(), String> {
    let flag = state.stop_flag.lock().unwrap();
    flag.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub fn ping() -> String {
    "pong".to_string()
}