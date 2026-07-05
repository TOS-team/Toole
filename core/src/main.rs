// binaire standalone pour tester le discovery sans Tauri
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use toole_core::{discovery, utils, Peer, UI};

// implementation factice de UI qui affiche tout dans la console
struct NullUI;

impl UI for NullUI {
    // j'affiche le message dans la console
    fn log(&self, msg: &str) {
        println!("{msg}");
    }
    // j'affiche les infos du pair trouve
    fn peer_found(&self, peer: &Peer) {
        println!("Peer: {} ({})", peer.hostname, peer.addr);
    }
    // j'affiche que le pair est perdu
    fn peer_lost(&self, hostname: &str) {
        println!("Peer lost: {hostname}");
    }
}

#[tokio::main]
async fn main() {
    // je recupere l'ip locale et je prepare le flag d'arret
    let local_ip = utils::local_ip();
    let stop = Arc::new(AtomicBool::new(false));
    let ui = Arc::new(NullUI);

    println!("Démarrage de Toolé Discovery...");

    // je lance la boucle de discovery
    if let Err(e) = discovery::start_discovery(local_ip, stop, ui).await {
        eprintln!("Erreur : {}", e);
    }
}
