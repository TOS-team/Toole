use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State, WebviewWindow};
use toole_core::sender::start_sender;
use toole_core::{Peer, ToolError, UI};

// ───────────────────────────────────────────────
// UI unique
// ───────────────────────────────────────────────

pub struct AppUI {
    pub peers: Arc<Mutex<Vec<Peer>>>,
    pub window: WebviewWindow,
}

impl UI for AppUI {
    fn log(&self, msg: &str) {
        let _ = self.window.emit("tool://log", msg);
    }

    fn peer_found(&self, peer: &Peer) {
        let mut peers = self.peers.lock().unwrap();
        if !peers.iter().any(|p| p.id == peer.id) {
            peers.push(peer.clone());
            let _ = self.window.emit("tool://peer_found", peer);
        }
    }

    fn peer_lost(&self, id: &str) {
        let mut peers = self.peers.lock().unwrap();
        peers.retain(|p| p.id != id);
        let _ = self.window.emit("tool://peer_lost", id);
    }

    fn show_progress_bar(&self, transfer_id: &str) {
        let _ = self.window.emit("tool://transfer/start", transfer_id);
    }

    fn update_progress_bar(&self, transfer_id: &str, bytes_sent: u64, total_bytes: u64) {
        let percent = if total_bytes > 0 {
            (bytes_sent as f64 / total_bytes as f64 * 100.0).min(100.0) as u8
        } else {
            0
        };
        let payload = serde_json::json!({
            "transfer_id": transfer_id,
            "bytes_sent": bytes_sent,
            "total_bytes": total_bytes,
            "percent": percent
        });
        let _ = self.window.emit("tool://transfer/progress", payload);
    }

    fn transfert_cancel(&self, transfer_id: &str) {
        let _ = self.window.emit("tool://transfer/cancel", transfer_id);
    }

    fn transfert_completed(&self, transfer_id: &str) {
        let _ = self.window.emit("tool://transfer/done", transfer_id);
    }

    fn transfert_received(&self, transfer_id: &str, peer: &str, bytes: u64, files: Vec<String>) {
        let payload = serde_json::json!({
            "transfer_id": transfer_id,
            "peer": peer,
            "bytes": bytes,
            "files": files
        });
        let _ = self.window.emit("tool://transfer/received", payload);
    }

    fn tranfert_error(&self, transfer_id: &str, error: &ToolError) {
        let payload = serde_json::json!({
            "transfer_id": transfer_id,
            "error": error.to_string()
        });
        let _ = self.window.emit("tool://transfer/error", payload);
    }
}

// ───────────────────────────────────────────────
// États
// ───────────────────────────────────────────────

pub struct DiscoveryState {
    pub stop_flag: Mutex<Arc<AtomicBool>>,
    pub peers: Arc<Mutex<Vec<Peer>>>,
}

pub struct TransferState {
    pub active: Mutex<HashMap<String, (Arc<AtomicBool>, tokio::task::AbortHandle)>>,
}

// ───────────────────────────────────────────────
// Découverte
// ───────────────────────────────────────────────

#[tauri::command]
pub async fn start_discovery(
    state: State<'_, DiscoveryState>,
    window: WebviewWindow,
) -> Result<(), String> {
    let old = state.stop_flag.lock().unwrap();
    old.store(true, Ordering::Relaxed);
    drop(old);

    state.peers.lock().unwrap().clear();

    let stop = Arc::new(AtomicBool::new(false));
    *state.stop_flag.lock().unwrap() = stop.clone();

    let local_ip = toole_core::utils::local_ip();
    let peers = state.peers.clone();
    let ui: Arc<dyn UI> = Arc::new(AppUI { peers, window: window.clone() });

    tokio::spawn(async move {
        if let Err(e) = toole_core::discovery::start_discovery(local_ip, stop, ui).await {
            eprintln!("Discovery error: {e}");
            let _ = window.emit("tool://discovery/error", e.to_string());
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

// ───────────────────────────────────────────────
// Transfert
// ───────────────────────────────────────────────

#[tauri::command]
pub async fn send_files(
    paths: Vec<String>,
    peer_addr: String,
    state: State<'_, TransferState>,
    window: WebviewWindow,
) -> Result<String, String> {
    let transfer_id = uuid::Uuid::new_v4().to_string();
    let stop = Arc::new(AtomicBool::new(false));

    let path_bufs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let addr: SocketAddr = peer_addr
        .parse()
        .map_err(|e| format!("Adresse invalide: {e}"))?;

    let peers = Arc::new(Mutex::new(Vec::new()));
    let ui: Arc<dyn UI> = Arc::new(AppUI { peers, window });
    let transfer_id_clone = transfer_id.clone();
    let stop_clone = stop.clone();

    let handle = tokio::spawn(async move {
        if let Err(e) = start_sender(ui, transfer_id_clone, path_bufs, addr, stop_clone).await {
            eprintln!("Sender error: {e}");
        }
    });

    state
        .active
        .lock()
        .unwrap()
        .insert(transfer_id.clone(), (stop, handle.abort_handle()));

    Ok(transfer_id)
}

#[tauri::command]
pub async fn cancel_transfer(
    transfer_id: String,
    state: State<'_, TransferState>,
) -> Result<(), String> {
    let mut active = state.active.lock().unwrap();
    if let Some((stop, handle)) = active.remove(&transfer_id) {
        stop.store(true, Ordering::Relaxed);
        handle.abort();
    }
    Ok(())
}

// ───────────────────────────────────────────────
// Utilitaires
// ───────────────────────────────────────────────

#[tauri::command]
pub fn get_hostname() -> String {
    toole_core::utils::current_hostname()
}

#[tauri::command]
pub fn get_device_id() -> String {
    toole_core::utils::device_id()
}

#[tauri::command]
pub fn get_peers(state: State<'_, DiscoveryState>) -> Result<Vec<Peer>, String> {
    let peers = state.peers.lock().unwrap();
    Ok(peers.clone())
}

#[tauri::command]
pub fn read_clipboard() -> Result<String, String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| format!("Clipboard error: {e}"))?;
    cb.get_text()
        .map_err(|e| format!("Clipboard read error: {e}"))
}

#[tauri::command]
pub fn close_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_file_sizes(paths: Vec<String>) -> Result<Vec<u64>, String> {
    paths
        .iter()
        .map(|p| {
            std::fs::metadata(p)
                .map(|m| m.len())
                .map_err(|e| format!("Erreur {p}: {e}"))
        })
        .collect()
}
