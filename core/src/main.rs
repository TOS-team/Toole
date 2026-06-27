use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use toole_core::{discovery, utils};

#[tokio::main]
async fn main() {
    let local_ip = utils::local_ip();
    let stop = Arc::new(AtomicBool::new(false));

    println!("Démarrage de Toolé Discovery...");

    if let Err(e) = discovery::start_discovery(local_ip, stop).await {
        eprintln!("Erreur : {}", e);
    }
}
