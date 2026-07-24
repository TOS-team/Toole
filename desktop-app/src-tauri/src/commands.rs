// ici je gere les commandes Tauri qui relient le frontend au backend Rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{State, Window};
use toole_core::{Peer, UI,Transfer,ToolError};

// implementation de UI pour Tauri : je stocke les pairs dans l'etat partage
struct TauriUI {
    peers: Arc<Mutex<Vec<Peer>>>,
    
}
struct StranTauriUI{
    window:Window
}

impl StranTauriUI {
    fn new(window: Window) -> Self {
        StranTauriUI { window }
    }
}
struct Payload{
    message:String
}

#[derive(Clone, serde::Serialize)]
struct Progress {
    value: u8,
}

#[derive(Clone,serde::Serialize)]
struct Palyoad{
    message:String
}



// état partagé pour controler le demarrage/arret du discovery
pub struct DiscoveryState {
    pub stop_flag: Mutex<Arc<AtomicBool>>,
    pub peers: Arc<Mutex<Vec<Peer>>>,
}

// je demarre la decouverte de pairs sur le reseau local
#[tauri::command]
pub async fn start_discovery(
    state: State<'_, DiscoveryState>,
) -> Result<(), String> {
    // je stoppe l'ancienne tache si elle tourne encore
    let old = state.stop_flag.lock().unwrap();
    old.store(true, Ordering::Relaxed);
    drop(old);

    // je vide la liste des pairs
    state.peers.lock().unwrap().clear();

    // je cree un nouveau flag pour la nouvelle tache
    let stop = Arc::new(AtomicBool::new(false));
    *state.stop_flag.lock().unwrap() = stop.clone();

    let local_ip = toole_core::utils::local_ip();
    let peers = state.peers.clone();
    let ui = Arc::new(TauriUI { peers });

    // je lance le discovery dans une tache asynchrone
    tokio::spawn(async move {
        if let Err(e) = toole_core::discovery::start_discovery(local_ip, stop, ui).await {
            eprintln!("Discovery task error: {e}");
        }
    });

    Ok(())
}

// j'arrete la decouverte de pairs
#[tauri::command]
pub async fn stop_discovery(state: State<'_, DiscoveryState>) -> Result<(), String> {
    let flag = state.stop_flag.lock().unwrap();
    flag.store(true, Ordering::Relaxed);
    Ok(())
}

// je renvoie le nom de la machine au frontend
#[tauri::command]
pub fn get_hostname() -> String {
    toole_core::utils::current_hostname()
}

// je renvoie la liste des pairs decouverts
#[tauri::command]
pub fn get_peers(state: State<'_, DiscoveryState>) -> Result<Vec<Peer>, String> {
    let peers = state.peers.lock().unwrap();
    Ok(peers.clone())
}

// je lis le texte du presse-papier (pour le Ctrl+V)
#[tauri::command]
pub fn read_clipboard() -> Result<String, String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| format!("Clipboard error: {e}"))?;
    cb.get_text().map_err(|e| format!("Clipboard read error: {e}"))
}

// je ferme la fenetre
#[tauri::command]
pub fn close_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

// je retourne la taille des fichiers en octets
#[tauri::command]
pub fn get_file_sizes(paths: Vec<String>) -> Result<Vec<u64>, String> {
    paths
        .iter()
        .map(|p| {
            std::fs::metadata(p)
                .map(|m| m.len())
                .map_err(|e| format!("Erreur lecture {p}: {e}"))
        })
        .collect()
}

#[tauri::command]
fn cancel_transfer(transfer_id: String, state: tauri::State<Transfer>) {
    // remove() au lieu de get(), car on doit à la fois récupérer ET nettoyer
    if let Some(handle) = state.cancel_handle.lock().unwrap().remove(&transfer_id) {
        handle.abort();
    }
}

use tauri::{Emitter};
use serde::Serialize;

#[derive(Clone, Serialize)]
struct ProgressPayload {
    transfer_id: String,
    bytes_sent: u64,
    total_bytes: u64,
}

#[derive(Clone, Serialize)]
struct ErrorPayload {
    transfer_id: String,
    message: String,
}




impl UI for StranTauriUI {

    fn log(&self, msg: &str) {
        let _ = self.window.emit("tool://log", msg);
    }

    // pas pertinent ici, no-op
    fn peer_found(&self, _peer: &Peer) {}
    fn peer_lost(&self, _hostname: &str) {}

    fn show_progress_bar(&self, tranfert_id: &str) {
        let _ = self.window.emit("tool://show_progress_bar", tranfert_id);
    }

    fn update_progress_bar(&self, transfer_id: &str, bytes_sent: u64, total_bytes: u64) {
        let _ = self.window.emit("tool://progress", ProgressPayload {
            transfer_id: transfer_id.to_string(),
            bytes_sent,
            total_bytes,
        });
    }

    fn transfert_cancel(&self, tranfert_id: &str) {
        let _ = self.window.emit("tool://transfert_cancel", tranfert_id);
    }

    fn transfert_completed(&self, tranfert_id: &str) {
        let _ = self.window.emit("tool://transfert_completed", tranfert_id);
    }

    fn tranfert_error(&self, tranfert_id: &str, error: &ToolError) {
        let _ = self.window.emit("tool://tranfert_error", ErrorPayload {
            transfer_id: tranfert_id.to_string(),
            message: error.to_string(),
        });
    }
}
