use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use toole_core::{discovery, utils, Peer, ToolError, UI};

struct ConsoleUI;

impl UI for ConsoleUI {
    fn log(&self, msg: &str) {
        println!("[LOG] {}", msg);
    }
    fn peer_found(&self, peer: &Peer) {
        println!("[PEER] Trouvé: {} @ {}", peer.id, peer.addr);
    }
    fn peer_lost(&self, hostname: &str) {
        println!("[PEER] Perdu: {}", hostname);
    }
    fn show_progress_bar(&self, _transfer_id: &str) {}
    fn update_progress_bar(&self, _transfer_id: &str, _bytes_sent: u64, _total_bytes: u64) {}
    fn file_progress_bar(
        &self,
        _transfer_id: &str,
        _file_name: &str,
        _file_bytes_sent: u64,
        _file_total_bytes: u64,
    ) {}
    fn transfert_cancel(&self, _transfer_id: &str) {}
    fn transfert_completed(&self, _transfer_id: &str) {}
    fn transfert_received(&self, transfer_id: &str, peer: &str, bytes: u64, files: Vec<String>) {
        println!("[RECU] {} depuis {} ({} octets, {:?})", transfer_id, peer, bytes, files);
    }
    fn tranfert_error(&self, _transfer_id: &str, _error: &ToolError) {}
}

#[tokio::main]
async fn main() -> Result<(), ToolError> {
    let local_ip = utils::local_ip();
    println!("Démarrage discovery sur {}", local_ip);

    let stop = Arc::new(AtomicBool::new(false));
    let ui: Arc<dyn UI> = Arc::new(ConsoleUI);

    discovery::start_discovery(local_ip, stop, ui).await
}
