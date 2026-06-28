use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use toole_core::{discovery, utils, Peer, UI};

struct NullUI;

impl UI for NullUI {
    fn log(&self, msg: &str) { println!("{msg}"); }
    fn peer_found(&self, peer: &Peer) { println!("Peer: {} ({})", peer.hostname, peer.addr); }
    fn peer_lost(&self, hostname: &str) { println!("Peer lost: {hostname}"); }
}

#[tokio::main]
async fn main() {
    let local_ip = utils::local_ip();
    let stop = Arc::new(AtomicBool::new(false));
    let ui = Arc::new(NullUI);

    println!("Démarrage de Toolé Discovery...");

    if let Err(e) = discovery::start_discovery(local_ip, stop, ui).await {
        eprintln!("Erreur : {}", e);
    }
}
