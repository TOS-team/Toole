// ici je gere les commandes Tauri qui relient le frontend au backend Rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::State;
use toole_core::{Peer, UI};

// implementation de UI pour Tauri : je stocke les pairs dans l'etat partage
struct TauriUI {
    peers: Arc<Mutex<Vec<Peer>>>,
}

impl UI for TauriUI {
    fn log(&self, _msg: &str) {
        // les logs sont desactives dans l'interface
    }
    // j'ajoute le pair a la liste partagee
    fn peer_found(&self, peer: &Peer) {
        let mut peers = self.peers.lock().unwrap();
        if !peers.iter().any(|p| p.hostname == peer.hostname) {
            peers.push(peer.clone());
        }
    }
    // je retire le pair de la liste partagee
    fn peer_lost(&self, hostname: &str) {
        let mut peers = self.peers.lock().unwrap();
        peers.retain(|p| p.hostname != hostname);
    }
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


