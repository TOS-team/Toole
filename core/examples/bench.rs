// benchmark de la voie de données : transfert loopback du plus gros fichier
// possible, et mesure du débit d'envoi côté start_sender.
//
// le loopback ne borne pas le débit (il dépasse largement le gigabit câblé) :
// la mesure reflète donc le plafond du pipeline QUIC + TLS sur cette machine,
// un bon ordre de grandeur pour le LAN.
//
// usage :
//   cargo run -p toole_core --example bench -- [taille_mo]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use toole_core::receiver::start_receiver;
use toole_core::sender::start_sender;
use toole_core::transfer::DecisionBoard;
use toole_core::{Peer, ToolError, TransferRegistry, UI};

/// registre no-op : le bench ne lance pas l'app, aucune annulation UI n'est
/// attendue pendant le transfert loopback
struct NoopRegistry;

impl TransferRegistry for NoopRegistry {
    fn register(&self, _transfer_id: &str, _stop: Arc<std::sync::atomic::AtomicBool>) {}
    fn unregister(&self, _transfer_id: &str) {}
}

/// UI muette qui signale quand le récepteur écoute (via son log de démarrage)
struct BenchUI {
    receiver_ready: Arc<AtomicBool>,
}

impl UI for BenchUI {
    fn log(&self, msg: &str) {
        if msg.contains("ecoute") {
            self.receiver_ready.store(true, Ordering::Relaxed);
        }
    }
    fn peer_found(&self, _peer: &Peer) {}
    fn peer_lost(&self, _hostname: &str) {}
    fn show_progress_bar(&self, _transfer_id: &str) {}
    fn update_progress_bar(&self, _transfer_id: &str, _bytes_sent: u64, _total_bytes: u64) {}
    fn file_progress_bar(
        &self,
        _transfer_id: &str,
        _file_name: &str,
        _file_bytes_sent: u64,
        _file_total_bytes: u64,
    ) {
    }
    fn transfert_incoming(
        &self,
        _transfer_id: &str,
        _sender: &str,
        _total_bytes: u64,
        _files: Vec<String>,
    ) {
    }
    fn transfert_refused(&self, _transfer_id: &str) {}
    fn transfert_cancel(&self, _transfer_id: &str) {}
    fn transfert_completed(&self, _transfer_id: &str) {}
    fn transfert_received(&self, _transfer_id: &str, _peer: &str, _bytes: u64, _files: Vec<String>) {
    }
    fn transfert_error(&self, _transfer_id: &str, _error: &ToolError) {}
}

fn parse_arg() -> u64 {
    let default = 1024;
    std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(default)
}

async fn wait_ready(receiver_ready: &AtomicBool) {
    let mut waited = 0u64;
    while !receiver_ready.load(Ordering::Relaxed) {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        waited += 1;
        if waited > 250 {
            panic!("le récepteur n'a pas démarré");
        }
    }
}

/// écrit `size` octets (motif déterministe, non compressible) dans `path`
fn write_file(path: &PathBuf, size: u64) {
    use std::io::Write;
    let mut x: u32 = 0x5a5a_5a5a;
    // je génère d'abord un bloc de 1 Mo pseudo-aléatoire, puis je le répète :
    // écrire 1 Mo de données réelles puis les réutiliser est bien plus rapide
    // que de calculer le LCG octet par octet, sans changer la mesure (seul
    // l'envoi est chronométré)
    let mut block = vec![0u8; 1024 * 1024];
    for b in block.iter_mut() {
        x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        *b = (x >> 24) as u8;
    }
    let mut f = std::fs::File::create(path).unwrap();
    let mut remaining = size;
    while remaining > 0 {
        let n = block.len().min(remaining as usize);
        f.write_all(&block[..n]).unwrap();
        remaining -= n as u64;
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), ToolError> {
    let size_mib = parse_arg();
    let size = size_mib * 1024 * 1024;
    if size == 0 {
        eprintln!("taille invalide");
        std::process::exit(2);
    }

    let root = std::env::temp_dir().join(format!("toole_bench_{}", std::process::id()));
    let src = root.join("bench.bin");
    let dest = root.join("dest");
    std::fs::create_dir_all(&dest).unwrap();
    write_file(&src, size);

    let stop = Arc::new(AtomicBool::new(false));
    let ready = Arc::new(AtomicBool::new(false));

    let recv_ui = BenchUI {
        receiver_ready: ready.clone(),
    };
    let recv_stop = stop.clone();
    let recv_dest = dest.clone();
    // le bench accepte automatiquement la demande de validation (auto_accept)
    let decisions = Arc::new(DecisionBoard::new());
    decisions.set_auto_accept(true);
    let recv_task = tokio::spawn(async move {
        let _ = start_receiver(
            Arc::new(recv_ui),
            recv_dest,
            recv_stop,
            decisions,
            Arc::new(NoopRegistry),
        )
        .await;
    });
    wait_ready(&ready).await;

    let send_ui = BenchUI {
        receiver_ready: ready.clone(),
    };
    let start = Instant::now();
    start_sender(
        Arc::new(send_ui),
        "bench".to_string(),
        vec![PathBuf::from(&src)],
        "127.0.0.1:58200".parse().unwrap(),
        "bench-peer".to_string(),
        stop.clone(),
    )
    .await?;
    let elapsed = start.elapsed();

    stop.store(true, Ordering::Relaxed);
    let _ = recv_task.await;
    let _ = std::fs::remove_dir_all(&root);

    let secs = elapsed.as_secs_f64();
    let mib_s = (size as f64) / secs / (1024.0 * 1024.0);
    println!(
        "{size_mib} Mo en {secs:.2}s → {mib_s:.1} Mo/s ({:.0} Mbit/s)",
        mib_s * 8.0
    );
    Ok(())
}