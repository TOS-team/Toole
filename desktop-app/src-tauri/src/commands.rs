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

    fn file_progress_bar(
        &self,
        transfer_id: &str,
        file_name: &str,
        file_bytes_sent: u64,
        file_total_bytes: u64,
    ) {
        let percent = if file_total_bytes > 0 {
            (file_bytes_sent as f64 / file_total_bytes as f64 * 100.0).min(100.0) as u8
        } else {
            0
        };
        let payload = serde_json::json!({
            "transfer_id": transfer_id,
            "file_name": file_name,
            "file_bytes_sent": file_bytes_sent,
            "file_total_bytes": file_total_bytes,
            "percent": percent
        });
        let _ = self.window.emit("tool://transfer/file_progress", payload);
    }

    fn transfert_incoming(
        &self,
        transfer_id: &str,
        sender: &str,
        total_bytes: u64,
        files: Vec<String>,
    ) {
        let payload = serde_json::json!({
            "transfer_id": transfer_id,
            "sender": sender,
            "total_bytes": total_bytes,
            "files": files
        });
        let _ = self.window.emit("tool://transfer/incoming", payload);
    }

    fn transfert_refused(&self, transfer_id: &str) {
        let _ = self.window.emit("tool://transfer/refused", transfer_id);
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

    fn transfert_error(&self, transfer_id: &str, error: &ToolError) {
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
    /// handle de la tâche de découverte en cours : je m'en sers pour attendre
    /// qu'elle ait libéré la socket UDP avant d'en relancer une nouvelle
    /// (sinon le bind du refresh échoue en « port déjà utilisé »)
    pub handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    pub peers: Arc<Mutex<Vec<Peer>>>,
}

pub struct TransferState {
    /// transferts actifs (envois et réceptions) : drapeau d'arrêt + handle
    /// d'abandon. Le handle est None pour les réceptions (l'annulation passe
    /// par le drapeau, la connexion se ferme gracieusement).
    pub active: Arc<Mutex<HashMap<String, (Arc<AtomicBool>, Option<tokio::task::AbortHandle>)>>>,
    /// registre des demandes d'acceptation en attente : respond_transfer y
    /// résout la décision de l'utilisateur pour le récepteur
    pub decisions: Arc<toole_core::transfer::DecisionBoard>,
}

/// implémentation du registre de transferts côté Tauri : il se branche sur la
/// même map que TransferState pour que cancel_transfer gère les réceptions
pub struct TransferRegistryHandle {
    pub active: Arc<Mutex<HashMap<String, (Arc<AtomicBool>, Option<tokio::task::AbortHandle>)>>>,
}

impl toole_core::TransferRegistry for TransferRegistryHandle {
    fn register(&self, transfer_id: &str, stop: Arc<AtomicBool>) {
        self.active
            .lock()
            .unwrap()
            .insert(transfer_id.to_string(), (stop, None));
    }

    fn unregister(&self, transfer_id: &str) {
        self.active.lock().unwrap().remove(transfer_id);
    }
}

// ───────────────────────────────────────────────
// Découverte
// ───────────────────────────────────────────────

#[tauri::command]
pub async fn start_discovery(
    state: State<'_, DiscoveryState>,
    window: WebviewWindow,
) -> Result<(), String> {
    // je stoppe l'ancienne découverte si elle tourne encore et j'attends
    // qu'elle ait libéré la socket 58199 : sinon le bind ci-dessous échoue
    // (AddressInUse) et la découverte ne redémarre jamais après un refresh
    state.stop_flag.lock().unwrap().store(true, Ordering::Relaxed);
    let old = state.handle.lock().unwrap().take();
    if let Some(old) = old {
        let _ = old.await;
    }

    state.peers.lock().unwrap().clear();

    let stop = Arc::new(AtomicBool::new(false));
    *state.stop_flag.lock().unwrap() = stop.clone();

    let local_ip = toole_core::utils::local_ip();
    let peers = state.peers.clone();
    let ui: Arc<dyn UI> = Arc::new(AppUI {
        peers,
        window: window.clone(),
    });

    let handle = tokio::spawn(async move {
        if let Err(e) = toole_core::discovery::start_discovery(local_ip, stop, ui).await {
            eprintln!("Discovery error: {e}");
            let _ = window.emit("tool://discovery/error", e.to_string());
        }
    });
    *state.handle.lock().unwrap() = Some(handle);

    Ok(())
}

#[tauri::command]
pub async fn stop_discovery(state: State<'_, DiscoveryState>) -> Result<(), String> {
    state.stop_flag.lock().unwrap().store(true, Ordering::Relaxed);
    // j'attends la fin de la tâche pour que la socket soit libérée avant que
    // la commande ne rende la main (le refresh enchaîne sur start_discovery)
    let h = state.handle.lock().unwrap().take();
    if let Some(h) = h {
        let _ = h.await;
    }
    Ok(())
}

// ───────────────────────────────────────────────
// Transfert
// ───────────────────────────────────────────────

#[tauri::command]
pub async fn send_files(
    paths: Vec<String>,
    peer_addr: String,
    peer_id: String,
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
    let active = state.active.clone();

    let handle = tokio::spawn(async move {
        let tid = transfer_id_clone.clone();
        if let Err(e) = start_sender(ui, transfer_id_clone, path_bufs, addr, peer_id, stop_clone)
            .await
        {
            eprintln!("Sender error: {e}");
        }
        // l'envoi est terminé (succès, erreur ou annulation) : je retire
        // l'entrée du registre, sinon la map grossit à chaque transfert
        active.lock().unwrap().remove(&tid);
    });

    state
        .active
        .lock()
        .unwrap()
        .insert(transfer_id.clone(), (stop, Some(handle.abort_handle())));

    Ok(transfer_id)
}

#[tauri::command]
pub async fn cancel_transfer(
    transfer_id: String,
    state: State<'_, TransferState>,
    window: WebviewWindow,
) -> Result<(), String> {
    let mut active = state.active.lock().unwrap();
    if let Some((stop, handle)) = active.remove(&transfer_id) {
        stop.store(true, Ordering::Relaxed);
        // si l'envoi est bloqué (backpressure QUIC), le drapeau d'arrêt ne
        // suffit pas : j'interromps la tâche, et comme elle ne pourra pas
        // émettre son événement terminal, je le fais ici pour que la carte
        // frontend passe bien en « annulé »
        if let Some(h) = handle {
            if !h.is_finished() {
                h.abort();
                let _ = window.emit("tool://transfer/cancel", &transfer_id);
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn respond_transfer(
    transfer_id: String,
    accepted: bool,
    state: State<'_, TransferState>,
) -> Result<(), String> {
    if state.decisions.resolve(&transfer_id, accepted) {
        Ok(())
    } else {
        Err(format!("transfert {transfer_id} inconnu ou deja traite"))
    }
}

// ───────────────────────────────────────────────
// Utilitaires
// ───────────────────────────────────────────────

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
pub fn get_file_infos(paths: Vec<String>) -> Result<Vec<FileInfo>, String> {
    paths
        .iter()
        .map(|p| {
            std::fs::metadata(p)
                .map(|m| FileInfo {
                    size: if m.is_file() { m.len() } else { 0 },
                    is_dir: m.is_dir(),
                })
                .map_err(|e| format!("Erreur {p}: {e}"))
        })
        .collect()
}

#[derive(serde::Serialize)]
pub struct FileInfo {
    pub size: u64,
    pub is_dir: bool,
}
